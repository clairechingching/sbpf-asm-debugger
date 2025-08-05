use crate::program::Program;
use crate::instruction::{Instruction, decode_instruction};
use crate::log_buffer::{get_log, log_message};
use sbpf_assembler::debuginfo::DebugInfo;
use sbpf_assembler::debuginfo::RegisterType;
use std::collections::HashMap;

// Memory layout constants for map-based memory design
// pub const STACK_START: u64 = 0x200000000;
// pub const HEAP_START: usize = 0x300000000;

// writable memory region
pub const MEMORY_INPUT_DATA_START: u64 = 0x400000000;

#[derive(Debug, Clone)]
pub struct Register {
    pub name: String,
    pub value: u64,
    pub register_type: RegisterType,
}

pub struct VM {
    program: Option<Program>, // Loaded program
    entry_point: Option<usize>,
    rodata: Option<Vec<(String, usize, String)>>,
    line_map: Option<HashMap<u64, usize>>,
    debug_map: Option<HashMap<u64, DebugInfo>>,
    state: VMState,
}

#[derive(Debug)]
pub struct VMState {
    pub registers: [Register; 11],
    pub memory: Vec<u8>,
    // program pointer
    pub pc: usize,
    pub exited: bool,
}

impl VMState {
    pub fn exit(&mut self) {
        log_message(&format!("{}", self.registers[0].value));
        self.exited = true;
    }

    pub fn reset(&mut self) {
        self.registers = [
            // initialze r0 to 0 (true)
            Register { name: "r0".to_string(), value: 0, register_type: RegisterType::Int },
            // initialze r1 to the start of memory input data
            Register { name: "r1".to_string(), value: MEMORY_INPUT_DATA_START, register_type: RegisterType::Addr },
            // initialze the rest registers to null
            Register { name: "r2".to_string(), value: 0, register_type: RegisterType::Null },
            Register { name: "r3".to_string(), value: 0, register_type: RegisterType::Null },
            Register { name: "r4".to_string(), value: 0, register_type: RegisterType::Null },
            Register { name: "r5".to_string(), value: 0, register_type: RegisterType::Null },
            Register { name: "r6".to_string(), value: 0, register_type: RegisterType::Null },
            Register { name: "r7".to_string(), value: 0, register_type: RegisterType::Null },
            Register { name: "r8".to_string(), value: 0, register_type: RegisterType::Null },
            Register { name: "r9".to_string(), value: 0, register_type: RegisterType::Null },
            Register { name: "r10".to_string(), value: 0, register_type: RegisterType::Null },
        ];
        // figure out memory size
        self.memory = vec![0u8; 20000];
        self.pc = 0;
        self.exited = false;
    }

    pub fn update_register(&mut self, register: usize, value: u64, register_type: RegisterType) {
        self.registers[register].value = value;
        if self.registers[register].register_type == RegisterType::Null {
            self.registers[register].register_type = RegisterType::Int;
        } else {
            self.registers[register].register_type = register_type;
        }
    }
}

impl VM {
    pub fn new() -> Self {
        VM {
            // MISC TODO : self.reset()
            state: VMState {
                registers: [
                    Register { name: "r0".to_string(), value: 0, register_type: RegisterType::Int },
                    // r1 points to the start of the memory
                    Register { name: "r1".to_string(), value: MEMORY_INPUT_DATA_START, register_type: RegisterType::Addr },
                    Register { name: "r2".to_string(), value: 0, register_type: RegisterType::Null },
                    Register { name: "r3".to_string(), value: 0, register_type: RegisterType::Null },
                    Register { name: "r4".to_string(), value: 0, register_type: RegisterType::Null },
                    Register { name: "r5".to_string(), value: 0, register_type: RegisterType::Null },
                    Register { name: "r6".to_string(), value: 0, register_type: RegisterType::Null },
                    Register { name: "r7".to_string(), value: 0, register_type: RegisterType::Null },
                    Register { name: "r8".to_string(), value: 0, register_type: RegisterType::Null },
                    Register { name: "r9".to_string(), value: 0, register_type: RegisterType::Null },
                    Register { name: "r10".to_string(), value: 0, register_type: RegisterType::Null },
                ],
                memory: vec![0u8; 20000],
                pc: 0,
                exited: false,
            },
            program: None,
            entry_point: None,
            rodata: None,
            line_map: None,
            debug_map: None,
        }
    }

    pub fn reset(&mut self) {
        self.state.reset();
    }

    pub fn load_rodata(&mut self, rodata: Vec<(String, usize, String)>) {
        self.rodata = Some(rodata);
    }

    pub fn load_line_map(&mut self, line_map: HashMap<u64, usize>) {
        self.line_map = Some(line_map);
    }

    pub fn load_debug_map(&mut self, debug_map: HashMap<u64, DebugInfo>) {
        self.debug_map = Some(debug_map);
    }

    pub fn load_program(&mut self, bytecode: Vec<u8>) -> Result<(), String> {
        let program = Program::new(bytecode)?;
        self.program = Some(program);
        self.entry_point = Some(self.program.as_ref().unwrap().entry_point as usize);
        self.state.pc = self.entry_point.unwrap();
        Ok(())
    }

    pub fn load_input_data(&mut self, account_number: u64, data: &[u8], data_type: &str) {
        // hard code account number to 0
        let start_addr = 8;
        self.state.memory[8] = data.len() as u8;
        self.state.memory[start_addr + 8 .. start_addr + 8 + data.len()].copy_from_slice(data);
    }

    pub fn get_instruction_data(&self) -> Vec<u8> {
        let start_addr = 8;
        let len = self.state.memory[start_addr] as usize;
        self.state.memory[start_addr + 8 .. start_addr + 8 + len].to_vec()
    }

    pub fn is_exited(&self) -> bool {
        self.state.exited
    }

    pub fn run(&mut self) -> Result<u64, String> {
        let program = self.program.as_ref().ok_or("No program loaded")?;
        
        // BUG TODO : update it to call step_instruction()
        while !self.state.exited {
            // Get the current instruction bytes
            let current_bytes = &program.bytecode[self.state.pc..];
            
            // Get debug info for current instruction if available
            let debug_info = if let Some(debug_map) = &self.debug_map {
                let offset = self.state.pc as u64 - self.entry_point.unwrap() as u64;
                debug_map.get(&offset)
            } else {
                None
            };
            
            // Decode the instruction first
            let (instruction, size) = decode_instruction(current_bytes)?;
            
            // Execute the instruction with debug info
            instruction.execute(&mut self.state, program, debug_info)?;

            // Only increment PC if the instruction didn't modify it (e.g., jump)
            if self.state.pc == self.state.pc {
                self.state.pc += size;
            }
        }
        
        // Return the result from r0
        Ok(self.state.registers[0].value)
    }
    
    pub fn step_instruction(&mut self) -> Result<(), String> {
        let program = self.program.as_ref().ok_or("No program loaded")?;
        let current_bytes = &program.bytecode[self.state.pc..];
        
        // Get debug info for current instruction if available
        let debug_info = if let Some(debug_map) = &self.debug_map {
            let offset = self.state.pc as u64 - self.entry_point.unwrap() as u64;
            debug_map.get(&offset)
        } else {
            None
        };
        
        let (instruction, size) = decode_instruction(current_bytes)?;
        instruction.execute(&mut self.state, program, debug_info)?;
        self.state.pc += size;
        Ok(())
    }

    pub fn get_entry_point(&self) -> usize {
        self.entry_point.unwrap()
    }

    pub fn get_line_number(&self) -> usize {
        if let Some(debug_map) = &self.debug_map {
            debug_map.get(&(self.state.pc as u64 - self.entry_point.unwrap() as u64)).map(|debug_info| debug_info.line_number).unwrap_or(0)
        } else {
            0
        }
    }

    pub fn get_rodata(&self) -> Vec<(String, usize, String)> {
        self.rodata.as_ref().unwrap().clone()
    }

    pub fn get_registers(&self) -> Vec<Register> {
        self.state.registers.to_vec()
    }
}
