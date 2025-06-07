use crate::opcode::Opcode;
use crate::tokenizer::{Token, ImmediateValue};
use crate::debuginfo::{DebugInfo, RegisterHint, RegisterType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ASTNode {
    Directive {
        name: String,
        args: Vec<Token>,
        line_number: usize,
    },
    Label {
        name: String,
        line_number: usize,
    },
    Instruction {
        opcode: Opcode,
        operands: Vec<Token>,
        offset: u64,
        line_number: usize,
    },
    RODataLabel {
        name: String,
        args: Vec<Token>,
        offset: u64,
        line_number: usize,
    },
}

impl ASTNode {
    pub fn get_offset(&self) -> Option<u64> {
        match self {
            ASTNode::Instruction { offset, .. } => Some(*offset),
            _ => None,
        }
    }

    pub fn get_opcode(&self) -> Option<&Opcode> {
        match self {
            ASTNode::Instruction { opcode, .. } => Some(opcode),
            _ => None,
        }
    }

    pub fn get_operands(&self) -> Option<&Vec<Token>> {
        match self {
            ASTNode::Instruction { operands, .. } => Some(operands),
            _ => None,
        }
    }

    pub fn get_label_name(&self) -> Option<&String> {
        match self {
            ASTNode::Label { name, .. } => Some(name),
            ASTNode::RODataLabel { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn get_directive_name(&self) -> Option<&String> {
        match self {
            ASTNode::Directive { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn get_directive_args(&self) -> Option<&Vec<Token>> {
        match self {
            ASTNode::Directive { args, .. } => Some(args),
            _ => None,
        }
    }

    pub fn get_rodata_args(&self) -> Option<&Vec<Token>> {
        match self {
            ASTNode::RODataLabel { args, .. } => Some(args),
            _ => None,
        }
    }

    pub fn get_line_number(&self) -> usize {
        match self {
            ASTNode::Directive { line_number, .. } => *line_number,
            ASTNode::Label { line_number, .. } => *line_number,
            ASTNode::Instruction { line_number, .. } => *line_number,
            ASTNode::RODataLabel { line_number, .. } => *line_number,
        }
    }

    pub fn bytecode_with_debug_map(&self) -> Option<(Vec<u8>, HashMap<u64, DebugInfo>)> {
        match self {
            ASTNode::Instruction { opcode, operands, offset, line_number } => {
                let mut bytes = Vec::new();
                let mut line_map = HashMap::new();
                let mut debug_map = HashMap::new();
                // Record the start of this instruction
                line_map.insert(*offset, *line_number);
                let mut debug_info = DebugInfo::new(*line_number);
                bytes.push(opcode.to_bytecode());  // 1 byte opcode
                
                if *opcode == Opcode::Call {
                    // currently hardcoded to call sol_log_
                    bytes.extend_from_slice(&[0x10, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF]);
                } else {
                    match &operands[..] {
                        [Token::ImmediateValue(imm, _)] => {
                            // 1 byte of zeros (no register)
                            bytes.push(0);
                            
                            if *opcode == Opcode::Ja {
                                // 2 bytes immediate value in little-endian for 'ja'
                                let imm16 = match imm {
                                    ImmediateValue::Int(val) => *val as i16,
                                    ImmediateValue::Addr(val) => *val as i16,
                                };
                                bytes.extend_from_slice(&imm16.to_le_bytes());
                            } else {
                                // 4 bytes immediate value in little-endian
                                let imm32 = match imm {
                                    ImmediateValue::Int(val) => *val as i32,
                                    ImmediateValue::Addr(val) => *val as i32,
                                };
                                bytes.extend_from_slice(&imm32.to_le_bytes());
                            }
                        },

                        [Token::Register(reg, _), Token::ImmediateValue(imm, _)] => {
                            // 1 byte register number (strip 'r' prefix)
                            bytes.push(reg.strip_prefix("r").unwrap().parse::<u8>().unwrap());
                            
                            // 2 bytes of zeros (offset/reserved)
                            bytes.extend_from_slice(&[0, 0]);
                            
                            // 4 bytes immediate value in little-endian
                            let imm32 = match imm {
                                ImmediateValue::Int(val) => *val as i32,
                                ImmediateValue::Addr(val) => {
                                    debug_info.register_hint = RegisterHint {
                                        register: reg.strip_prefix("r").unwrap().parse().unwrap(),
                                        register_type: RegisterType::Addr
                                    };
                                    *val as i32
                                }
                            };
                            bytes.extend_from_slice(&imm32.to_le_bytes());
                        },

                        [Token::Register(reg, _), Token::ImmediateValue(imm, _), Token::ImmediateValue(offset, _)] => {
                            // 1 byte register number (strip 'r' prefix)
                            bytes.push(reg.strip_prefix("r").unwrap().parse::<u8>().unwrap());
                            
                            // 2 bytes of offset in little-endian
                            let offset16 = match offset {
                                ImmediateValue::Int(val) => *val as u16,
                                ImmediateValue::Addr(val) => *val as u16,
                            };
                            bytes.extend_from_slice(&offset16.to_le_bytes());
                            
                            // 4 bytes immediate value in little-endianß
                            let imm32 = match imm {
                                ImmediateValue::Int(val) => *val as i32,
                                ImmediateValue::Addr(val) => {
                                    debug_info.register_hint = RegisterHint {
                                        register: reg.strip_prefix("r").unwrap().parse().unwrap(),
                                        register_type: RegisterType::Addr
                                    };
                                    *val as i32
                                }
                            };
                            bytes.extend_from_slice(&imm32.to_le_bytes());
                        },                    
                        
                        [Token::Register(dst, _), Token::Register(src, _)] => {
                            // Convert register strings to numbers
                            let dst_num = dst.strip_prefix("r").unwrap().parse::<u8>().unwrap();
                            let src_num = src.strip_prefix("r").unwrap().parse::<u8>().unwrap();
                            
                            // Combine src and dst into a single byte (src in high nibble, dst in low nibble)
                            let reg_byte = (src_num << 4) | dst_num;
                            bytes.push(reg_byte);
                        },
                        [Token::Register(dst, _), Token::Expression(expr, _)] => {
                            // Parse the expression to extract the base register and offset
                            if let Some((base_reg, offset)) = parse_expression(expr) {
                                // Convert register strings to numbers
                                let dst_num = dst.strip_prefix("r").unwrap().parse::<u8>().unwrap();
                                let base_reg_num = base_reg.strip_prefix("r").unwrap().parse::<u8>().unwrap();
                                
                                // Combine base register and destination register into a single byte
                                let reg_byte = (base_reg_num << 4) | dst_num;
                                bytes.push(reg_byte);
                                
                                // Add the offset as a 16-bit value in little-endian
                                let offset16 = offset as u16;
                                bytes.extend_from_slice(&offset16.to_le_bytes());
                            }
                        },
                        
                        _ => {}
                    }
                }

                // Add padding to make it 8 or 16 bytes depending on opcode
                let target_len = if *opcode == Opcode::Lddw { 16 } else { 8 };
                while bytes.len() < target_len {
                    bytes.push(0);
                }

                debug_map.insert(*offset, debug_info);
                
                Some((bytes, debug_map))
            },
            ASTNode::RODataLabel { name: _, args, offset, line_number } => {
                let mut bytes = Vec::new();
                let mut line_map = HashMap::<u64, usize>::new();
                let mut debug_map = HashMap::<u64, DebugInfo>::new();
                for arg in args {
                    if let Token::StringLiteral(s, _) = arg {
                        // Convert string to bytes and add null terminator
                        let str_bytes = s.as_bytes().to_vec();
                        bytes.extend(str_bytes);
                    }
                }
                Some((bytes, debug_map))
            },
            _ => None
        }
    }

    // Keep the old bytecode method for backward compatibility
    pub fn bytecode(&self) -> Option<Vec<u8>> {
        self.bytecode_with_debug_map().map(|(bytes, _)| bytes)
    }
}

fn parse_expression(expr: &String) -> Option<(String, i32)> {
    // Split the expression by '+' and trim whitespace
    let parts: Vec<&str> = expr.split('+').map(str::trim).collect();
    
    if parts.len() == 2 {
        // Assume the first part is the register and the second part is the offset
        let base_reg = parts[0].to_string();
        if let Ok(offset) = parts[1].parse::<i32>() {
            return Some((base_reg, offset));
        }
    }
    None
}
