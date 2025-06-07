use crate::opcode::Opcode;
use crate::tokenizer::{Token, ImmediateValue};
use crate::section::CodeSection;
use crate::section::DataSection;
use num_traits::FromPrimitive;
use std::collections::HashMap;
// use crate::instruction_verifier::verify_instruction;
use crate::astnode::ASTNode;
use crate::dynsym::DynamicSymbolMap;
use crate::dynsym::RelDynMap;
use crate::dynsym::RelocationType;
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,

    pub m_prog_is_static: bool,
    pub m_accum_offset: u64,
    pub m_entry_offset: u64,

    m_entry_label: Option<String>,
    m_label_offsets: HashMap<String, u64>,
    m_dynamic_symbols: DynamicSymbolMap,
    m_rel_dyns: RelDynMap,

    m_rodata_phase: bool,
    m_rodata_size: u64,
}

pub struct ParseResult {
    pub code_section: CodeSection,

    pub data_section: DataSection,

    pub dynamic_symbols: DynamicSymbolMap,

    pub relocation_data: RelDynMap,

    pub prog_is_static: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0
            , m_prog_is_static: true
            , m_accum_offset: 0
            , m_entry_label: None
            , m_entry_offset: 0
            , m_label_offsets: HashMap::new()
            , m_rodata_phase: false
            , m_rodata_size: 0
            , m_dynamic_symbols: DynamicSymbolMap::new()
            , m_rel_dyns: RelDynMap::new()
        }
    }
    
    pub fn parse(&mut self) -> Result<ParseResult, String> {
        let mut nodes = Vec::new();
        let mut rodata_nodes = Vec::new();

        while self.current < self.tokens.len() {
            // clone the token upfront so we don't hold a reference
            let mut current_token = self.peek().cloned();
            match current_token {
                Some(Token::Directive(name, line_number)) => {
                    self.advance();
                    let mut args = Vec::new();
                    current_token = self.peek().cloned();
                    if let Some(token @(Token::StringLiteral(_, _) | Token::Label(_, _))) = current_token {
                        self.advance();
                        args.push(token.clone());
                    }
                    nodes.push(ASTNode::Directive { name: name.clone(), args, line_number });
                }
                Some(Token::Rodata(line_number)) => {
                    self.m_rodata_phase = true;
                    self.advance();
                    rodata_nodes.push(ASTNode::Directive { name: "rodata".to_string(), args: Vec::new(), line_number });
                }
                Some(Token::Global(line_number)) => {
                    self.advance();
                    let mut args = Vec::new();
                    current_token = self.peek().cloned();
                    if let Some(ref token @ Token::Label(ref name, _)) = current_token {
                        self.m_entry_label = Some(name.clone());
                        self.advance();
                        args.push(token.clone());
                    }
                    nodes.push(ASTNode::Directive { name: "global".to_string(), args, line_number });
                }
                Some(Token::Extern(line_number)) => {
                    self.advance();
                    let mut args = Vec::new();
                    while let Some(token) = self.peek() {
                        match token {
                            Token::Label(_, _) => {
                                args.push(token.clone());
                                self.advance();
                            }
                            _ => break,
                        }
                    }
                    nodes.push(ASTNode::Directive { name: "extern".to_string(), args, line_number });
                }
                Some(Token::Label(name, line_number)) => {
                    self.advance();
                    if self.m_rodata_phase {
                        let mut args = Vec::new();
                        while let Some(token) = self.peek() {
                            match token {
                                Token::StringLiteral(s, _) => {
                                    args.push(token.clone());
                                    self.m_rodata_size += s.len() as u64;
                                    self.advance();
                                }
                                Token::Directive(_, _) => {
                                    args.push(token.clone());
                                    self.advance();
                                }
                                _ => break,
                            }
                        }
                        rodata_nodes.push(ASTNode::RODataLabel { name: name.clone(), args, offset: self.m_accum_offset, line_number });
                    } else {
                        nodes.push(ASTNode::Label { name: name.clone(), line_number });
                    }
                    self.m_label_offsets.insert(name.clone(), self.m_accum_offset);
                }
                Some(Token::Opcode(mut opcode, line_number)) => {
                    self.advance();
                    let mut operands = Vec::new();
                    match opcode {
                        // 
                        Opcode::Add32 | Opcode::Sub32 | Opcode::Mul32 
                        | Opcode::Div32 | Opcode::Or32 | Opcode::And32 
                        | Opcode::Lsh32 | Opcode::Rsh32 | Opcode::Mod32 
                        | Opcode::Xor32 | Opcode::Mov32 | Opcode::Arsh32 
                        | Opcode::Lmul32 | Opcode::Udiv32 | Opcode::Urem32 
                        | Opcode::Sdiv32 | Opcode::Srem32 | Opcode::Neg32
                        | Opcode::Add64 | Opcode::Sub64 | Opcode::Mul64 
                        | Opcode::Div64 | Opcode::Or64 | Opcode::And64 
                        | Opcode::Lsh64 | Opcode::Rsh64 | Opcode::Mod64 
                        | Opcode::Xor64 | Opcode::Mov64 | Opcode::Arsh64 
                        | Opcode::Lmul64 | Opcode::Uhmul64 | Opcode::Udiv64 
                        | Opcode::Urem64 | Opcode::Sdiv64 | Opcode::Srem64
                        | Opcode::Jeq | Opcode::Jgt | Opcode::Jge
                        | Opcode::Jlt | Opcode::Jle | Opcode::Jset
                        | Opcode::Jne | Opcode::Jsgt | Opcode::Jsge
                        | Opcode::Jslt | Opcode::Jsle => {
                            let mut tokens = Vec::new();
                            // Get next 2 tokens
                            for _ in 0..2 {
                                if let Some(token) = self.peek() {
                                    match token {
                                        Token::Register(_, _) | Token::ImmediateValue(_, _) => {
                                            tokens.push(token.clone());
                                            self.advance();
                                        }
                                        Token::Comma(_) => {
                                            self.advance();
                                        }
                                        _ => break,
                                    }
                                }
                            }

                            // For jump instructions, get the label token
                            if matches!(opcode, 
                                Opcode::Jeq | Opcode::Jgt | Opcode::Jge |
                                Opcode::Jlt | Opcode::Jle | Opcode::Jset |
                                Opcode::Jne | Opcode::Jsgt | Opcode::Jsge |
                                Opcode::Jslt | Opcode::Jsle) {
                                
                                // Skip comma if present
                                if let Some(Token::Comma(_)) = self.peek() {
                                    self.advance();
                                }

                                // Get the label token
                                if let Some(token) = self.peek() {
                                    match token {
                                        Token::Label(_, _) => {
                                            tokens.push(token.clone());
                                            self.advance();
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            // Update opcode based on operand types
                            if tokens.len() == 2 {
                                match (&tokens[0], &tokens[1]) {
                                    (Token::Register(_, _), Token::ImmediateValue(_, _)) => {
                                        // Add32 + 1 = Add32Imm
                                        opcode = FromPrimitive::from_u8((opcode as u8) + 1)
                                            .expect("Invalid opcode conversion");
                                    }
                                    (Token::Register(_, _), Token::Register(_, _)) => {
                                        // Add32 + 2 = Add32Reg  
                                        opcode = FromPrimitive::from_u8((opcode as u8) + 2)
                                            .expect("Invalid opcode conversion");
                                    }
                                    _ => {}
                                }
                            }
                            if tokens.len() == 3 {
                                match (&tokens[0], &tokens[1], &tokens[2]) {
                                    (Token::Register(_, _), Token::ImmediateValue(_, _), Token::Label(_, _)) => {
                                        opcode = FromPrimitive::from_u8((opcode as u8) + 1)
                                            .expect("Invalid opcode conversion");
                                    }
                                    (Token::Register(_, _), Token::Register(_, _), Token::Label(_, _)) => {
                                        opcode = FromPrimitive::from_u8((opcode as u8) + 2)
                                            .expect("Invalid opcode conversion");
                                    }
                                    _ => {}
                                }
                            }
                            
                            operands.extend(tokens);
                        },
                        // jump to label
                        Opcode::Ja => {
                            if let Some(token) = self.peek() {
                                match token {
                                    Token::Label(_, _)
                                    | Token::ImmediateValue(_, _) => {
                                        operands.push(token.clone());
                                        self.advance();
                                    }
                                    _ => {}
                                }
                            }
                        },
                        // exit
                        Opcode::Exit => {
                            // exit doesn't take any argument
                        },
                        _ => {
                            while let Some(token) = self.peek() {
                                match token {
                                    Token::Register(_, _)          // 
                                    | Token::ImmediateValue(_, _)  //
                                    | Token::StringLiteral(_, _)   //
                                    | Token::Label(_, _)           //
                                    | Token::Expression(_, _) => {
                                        operands.push(token.clone());
                                        self.advance();
                                    }
                                    Token::Comma(_) => {
                                        self.advance();
                                    }
                                    _ => break,
                                }
                            }
                        }
                    }

                    // verify_instruction(&opcode, &operands)?;

                    // Check for Call instruction and handle dynamic symbols before moving operands
                    if opcode == Opcode::Call {
                        self.m_prog_is_static = false;
                        if let Some(Token::Label(name, _)) = operands.last() {
                            self.m_dynamic_symbols.add_call_target(name.clone(), self.m_accum_offset);
                            self.m_rel_dyns.add_rel_dyn(self.m_accum_offset, RelocationType::RSbfSyscall, name.clone());
                        }
                    }

                    if opcode == Opcode::Lddw {
                        if let Some(Token::Label(name, _)) = operands.last() {
                            self.m_rel_dyns.add_rel_dyn(self.m_accum_offset, RelocationType::RSbf64Relative, name.clone());
                        }
                    }

                    nodes.push(ASTNode::Instruction {
                        opcode: opcode,
                        operands,
                        offset: self.m_accum_offset,
                        line_number,
                    });

                    if opcode == Opcode::Lddw {
                        self.m_accum_offset += 16;
                    } else {
                        self.m_accum_offset += 8;
                    }
                }
                
                _ => return Err("Unexpected token".to_string()),
            }
        }

        // println!("m_entry_label: {:?}", self.m_entry_label);
        // println!("m_label_offsets: {:?}", self.m_label_offsets);

        // Second pass to resolve labels
        for node in &mut nodes {
            match node {
                ASTNode::Instruction { opcode, operands, offset, .. } => {
                    // For jump instructions, replace label operands with relative offsets
                    if *opcode == Opcode::Ja || *opcode == Opcode::JeqImm || *opcode == Opcode::JgtImm || *opcode == Opcode::JgeImm 
                    || *opcode == Opcode::JltImm || *opcode == Opcode::JleImm || *opcode == Opcode::JsetImm || *opcode == Opcode::JneImm     
                    || *opcode == Opcode::JsgtImm || *opcode == Opcode::JsgeImm || *opcode == Opcode::JsltImm || *opcode == Opcode::JsleImm
                    || *opcode == Opcode::JeqReg || *opcode == Opcode::JgtReg || *opcode == Opcode::JgeReg || *opcode == Opcode::JltReg 
                    || *opcode == Opcode::JleReg || *opcode == Opcode::JsetReg || *opcode == Opcode::JneReg || *opcode == Opcode::JsgtReg 
                    || *opcode == Opcode::JsgeReg || *opcode == Opcode::JsltReg || *opcode == Opcode::JsleReg {
                        if let Some(Token::Label(label, _)) = operands.last() {
                            let label = label.clone(); // Clone early to avoid borrow conflict
                            if let Some(target_offset) = self.m_label_offsets.get(&label) {
                                let rel_offset = (*target_offset as i64 - *offset as i64) / 8 - 1;
                                // Replace label with immediate value
                                let last_idx = operands.len() - 1;
                                operands[last_idx] = Token::ImmediateValue(ImmediateValue::Int(rel_offset), 0);
                            }
                        }
                    }
                    if *opcode == Opcode::Lddw {
                        if let Some(Token::Label(name, _)) = operands.last() {
                            let label = name.clone();
                            if let Some(target_offset) = self.m_label_offsets.get(&label) {
                                let ph_count = if self.m_prog_is_static { 1 } else { 3 };
                                let ph_offset = 64 + (ph_count as u64 * 56) as i64;
                                let abs_offset = *target_offset as i64 + ph_offset;
                                // Replace label with immediate value
                                let last_idx = operands.len() - 1;
                                operands[last_idx] = Token::ImmediateValue(ImmediateValue::Addr(abs_offset), 0);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Set entry point offset if an entry label was specified
        if let Some(entry_label) = &self.m_entry_label {
            if let Some(offset) = self.m_label_offsets.get(entry_label) {
                self.m_entry_offset = *offset;
                self.m_dynamic_symbols.add_entry_point(entry_label.clone(), *offset);
            }
        }
        
        Ok(ParseResult {
            code_section: CodeSection::new(nodes, self.m_accum_offset),
            data_section: DataSection::new(rodata_nodes, self.m_rodata_size),
            dynamic_symbols: DynamicSymbolMap::copy(&self.m_dynamic_symbols),
            relocation_data: RelDynMap::copy(&self.m_rel_dyns),
            prog_is_static: self.m_prog_is_static,
        })
    }
    
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
    }
}