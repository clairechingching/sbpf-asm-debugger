use crate::vm::VMState;
use crate::program::Program;
use crate::log_buffer::log_message;
use helios_assembler::opcode::Opcode;
use helios_assembler::debuginfo::{RegisterType, RegisterHint, DebugInfo};

pub trait Instruction {
    fn execute(&self, vm: &mut VMState, program: &Program, debug_info: Option<&DebugInfo>) -> Result<(), String>;
}

#[derive(Debug)]
pub enum InstructionType {
    Lddw(Lddw),
    Ldxb(Ldxb),
    AddImm(AddImm),
    AddReg(AddReg),
    SubImm(SubImm),
    SubReg(SubReg),
    MoveImm(MoveImm),
    MoveReg(MoveReg),
    Jump(Jump),
    Call(Call),
    Exit(Exit),
}

impl Instruction for InstructionType {
    fn execute(&self, vm: &mut VMState, program: &Program, debug_info: Option<&DebugInfo>) -> Result<(), String> {
        match self {
            InstructionType::Lddw(instr) => instr.execute(vm, program, debug_info),
            InstructionType::Ldxb(instr) => instr.execute(vm, program, debug_info),
            InstructionType::AddImm(instr) => instr.execute(vm, program, debug_info),
            InstructionType::AddReg(instr) => instr.execute(vm, program, debug_info),
            InstructionType::SubImm(instr) => instr.execute(vm, program, debug_info),
            InstructionType::SubReg(instr) => instr.execute(vm, program, debug_info),
            InstructionType::MoveImm(instr) => instr.execute(vm, program, debug_info),
            InstructionType::MoveReg(instr) => instr.execute(vm, program, debug_info),
            InstructionType::Jump(instr) => instr.execute(vm, program, debug_info),
            InstructionType::Call(instr) => instr.execute(vm, program, debug_info),
            InstructionType::Exit(instr) => instr.execute(vm, program, debug_info),
        }
    }
}

#[derive(Debug)]
pub struct Lddw {
    pub register: usize,
    pub value: u64,
}

impl Lddw {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 16 {
            return Err("Not enough bytes for Lddw instruction".to_string());
        }
        Ok(Lddw {
            register: bytes[1] as usize,
            value: u64::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11]]),
        })
    }
}

impl Instruction for Lddw {
    fn execute(&self, vm: &mut VMState, _program: &Program, debug_info: Option<&DebugInfo>) -> Result<(), String> {
        if self.register >= vm.registers.len() {
            return Err("Invalid register index".to_string());
        }
        
        let register_type = debug_info
            .filter(|info| info.register_hint.register == self.register)
            .map(|info| info.register_hint.register_type)
            .unwrap_or(RegisterType::Int);
            
        vm.update_register(self.register, self.value, register_type);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Ldxb {
    pub register: usize,
    pub base_reg: usize,
    pub offset: u16,
}

impl Ldxb {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 16 {
            return Err("Not enough bytes for Ldxb instruction".to_string());
        }
        Ok(Ldxb {
            register: (bytes[1] & 0x0F) as usize,
            base_reg: (bytes[1] >> 4) as usize,
            offset: u16::from_le_bytes([bytes[2], bytes[3]]),
        })
    }
}

impl Instruction for Ldxb {
    fn execute(&self, vm: &mut VMState, _program: &Program, _debug_info: Option<&DebugInfo>) -> Result<(), String> {
        let base_addr = vm.registers[self.base_reg].value as usize;
        let offset = self.offset as usize;
        vm.update_register(self.register, vm.memory[base_addr + offset] as u64, RegisterType::Int);
        Ok(())
    }
}

#[derive(Debug)]
pub struct AddImm {    
    pub register: usize,
    pub value: u64,
}

impl AddImm {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err("Not enough bytes for AddImm instruction".to_string());
        }
        Ok(AddImm {
            register: bytes[1] as usize,
            value: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as u64,
        })
    }
}

impl Instruction for AddImm {
    fn execute(&self, vm: &mut VMState, _program: &Program, _debug_info: Option<&DebugInfo>) -> Result<(), String> {
        let addition = vm.registers[self.register].value + self.value;
        vm.update_register(self.register, addition, vm.registers[self.register].register_type);
        Ok(())
    }
}

#[derive(Debug)]
pub struct AddReg {
    pub src: usize,
    pub dest: usize,
}

impl AddReg {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err("Not enough bytes for AddReg instruction".to_string());
        }
        Ok(AddReg {
            src: (bytes[1] >> 4) as usize,  // high nibble
            dest: (bytes[1] & 0x0F) as usize,  // low nibble
        })
    }
}

impl Instruction for AddReg {
    fn execute(&self, vm: &mut VMState, _program: &Program, _debug_info: Option<&DebugInfo>) -> Result<(), String> {
        let addition = vm.registers[self.dest].value + vm.registers[self.src].value;
        vm.update_register(self.dest, addition, vm.registers[self.dest].register_type);
        Ok(())
    }
}

#[derive(Debug)]
pub struct SubImm {
    pub register: usize,
    pub value: u64,
}


impl SubImm {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err("Not enough bytes for SubImm instruction".to_string());
        }
        Ok(SubImm {
            register: bytes[1] as usize,
            value: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as u64,
        })
    }
}

impl Instruction for SubImm {
    fn execute(&self, vm: &mut VMState, _program: &Program, _debug_info: Option<&DebugInfo>) -> Result<(), String> {
        let subtraction = vm.registers[self.register].value - self.value;
        vm.update_register(self.register, subtraction, vm.registers[self.register].register_type);
        Ok(())
    }
}

#[derive(Debug)]
pub struct SubReg {
    pub src: usize,
    pub dest: usize,
}

impl SubReg {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err("Not enough bytes for SubReg instruction".to_string());
        }
        Ok(SubReg {
            src: (bytes[1] >> 4) as usize,  // high nibble
            dest: (bytes[1] & 0x0F) as usize,  // low nibble
        })
    }
}

impl Instruction for SubReg {
    fn execute(&self, vm: &mut VMState, _program: &Program, _debug_info: Option<&DebugInfo>) -> Result<(), String> {
        let subtraction = vm.registers[self.dest].value - vm.registers[self.src].value;
        vm.update_register(self.dest, subtraction, vm.registers[self.dest].register_type);
        Ok(())
    }
}

#[derive(Debug)]
pub struct MoveImm {
    pub register: usize,
    pub value: u64,
}

impl MoveImm {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err("Not enough bytes for MoveImm instruction".to_string());
        }
        Ok(MoveImm {
            register: bytes[1] as usize,
            value: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as u64,
        })
    }
}

impl Instruction for MoveImm {
    fn execute(&self, vm: &mut VMState, _program: &Program, _debug_info: Option<&DebugInfo>) -> Result<(), String> {
        if self.register >= vm.registers.len() {
            return Err("Invalid register index".to_string());
        }
        vm.update_register(self.register, self.value, RegisterType::Int);
        Ok(())
    }
}

#[derive(Debug)]
pub struct MoveReg {
    pub src: usize,
    pub dest: usize,
}

impl MoveReg {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err("Not enough bytes for MoveReg instruction".to_string());
        }
        Ok(MoveReg {
            src: (bytes[1] >> 4) as usize,  // high nibble
            dest: (bytes[1] & 0x0F) as usize,  // low nibble
        })
    }
}

impl Instruction for MoveReg {
    fn execute(&self, vm: &mut VMState, _program: &Program, _debug_info: Option<&DebugInfo>) -> Result<(), String> {
        vm.update_register(self.dest, vm.registers[self.src].value, RegisterType::Int);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Jump {
    pub register: usize,
    pub offset: i16,  // Offset in number of instructions
    pub value: u32,
    pub opcode: Opcode,
}

impl Jump {
    pub fn decode(bytes: &[u8], opcode: Opcode) -> Result<Self, String> {
        if bytes.len() < 16 {
            return Err("Not enough bytes for Jump instruction".to_string());
        }
        let register = bytes[1] as usize;
        let offset = i16::from_le_bytes([bytes[2], bytes[3]]);
        let value = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        Ok(Jump { register, value, offset, opcode })
    }
}

impl Instruction for Jump {
    fn execute(&self, vm: &mut VMState, _program: &Program, _debug_info: Option<&DebugInfo>) -> Result<(), String> {
        let condition_met = match self.opcode {
            Opcode::Ja => true,
            Opcode::JeqImm => vm.registers[self.register].value == self.value as u64,
            Opcode::JneImm => vm.registers[self.register].value != self.value as u64,
            Opcode::JgtImm => vm.registers[self.register].value > self.value as u64,
            Opcode::JgeImm => vm.registers[self.register].value >= self.value as u64,
            Opcode::JltImm => vm.registers[self.register].value < self.value as u64,
            Opcode::JleImm => vm.registers[self.register].value <= self.value as u64,
            _ => return Err("Invalid jump opcode".to_string()),
        };

        if condition_met {
            // Since each instruction is 8 bytes, multiply offset by 8 to get byte offset
            vm.pc = (vm.pc as i32 + (self.offset as i32 * 8)) as usize;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Call {
    pub function_id: u32,
}

impl Call {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err("Not enough bytes for Call instruction".to_string());
        }
        // Only read the 3 bytes for function ID (10 00 00), ignoring pAddImming
        let function_id = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], 0]);
        Ok(Call {
            function_id,
        })
    }
}

impl Instruction for Call {
    fn execute(&self, vm: &mut VMState, program: &Program, _debug_info: Option<&DebugInfo>) -> Result<(), String> {
        match self.function_id {
            0x10 => {
                // sol_log_ implementation
                let r1 = vm.registers[1].value; // pointer to buffer
                let r2 = vm.registers[2].value; // length of buffer
                
                if r2 > 0 {
                    // Read memory at r1 for r2 bytes
                    let buffer = program.read(r1 as u64, r2 as u64)
                        .map_err(|e| format!("Failed to read memory: {}", e))?;
                
                    // Convert buffer to string and print
                    let message = String::from_utf8_lossy(&buffer);
                    log_message(&format!("sol_log_: {}", message));
                } else {
                    log_message(&format!("sol_log_64_: {}", r1));
                }
                Ok(())
            }
            _ => Err(format!("Unsupported function ID: 0x{:x}", self.function_id)),
        }
    }
}

#[derive(Debug)]
pub struct Exit;

impl Instruction for Exit {
    fn execute(&self, vm: &mut VMState, _program: &Program, _debug_info: Option<&DebugInfo>) -> Result<(), String> {
        vm.exit();
        Ok(())
    }
}

// Function to decode a single instruction from bytecode
pub fn decode_instruction(bytes: &[u8]) -> Result<(InstructionType, usize), String> {
    if bytes.is_empty() {
        return Err("Empty bytecode".to_string());
    }

    let opcode = Opcode::from_u8(bytes[0]).ok_or_else(|| format!("Unknown opcode: 0x{:02x}", bytes[0]))?;
    let (instr, size) = match opcode {
        Opcode::Lddw => {
            let lddw = Lddw::decode(bytes)?;
            (InstructionType::Lddw(lddw), 16)
        }
        Opcode::Ldxb => {
            let ldxb = Ldxb::decode(bytes)?;
            (InstructionType::Ldxb(ldxb), 8)
        }
        Opcode::Add64Imm | Opcode::Add32Imm => {
            let AddImm = AddImm::decode(bytes)?;
            (InstructionType::AddImm(AddImm), 8)
        }
        Opcode::Add64Reg | Opcode::Add32Reg => {
            let AddReg = AddReg::decode(bytes)?;
            (InstructionType::AddReg(AddReg), 8)
        }
        Opcode::Sub64Imm | Opcode::Sub32Imm => {
            let SubImm = SubImm::decode(bytes)?;
            (InstructionType::SubImm(SubImm), 8)
        }
        Opcode::Sub64Reg | Opcode::Sub32Reg => {
            let SubReg = SubReg::decode(bytes)?;
            (InstructionType::SubReg(SubReg), 8)
        }
        Opcode::Mov64Imm | Opcode::Mov32Imm => {
            let mov = MoveImm::decode(bytes)?;
            (InstructionType::MoveImm(mov), 8)
        }
        Opcode::Mov64Reg | Opcode::Mov32Reg => {
            let mov = MoveReg::decode(bytes)?;
            (InstructionType::MoveReg(mov), 8)
        }
        Opcode::Ja | Opcode::JeqImm | Opcode::JneImm | Opcode::JgtImm | 
        Opcode::JgeImm | Opcode::JltImm | Opcode::JleImm => {
            let jump = Jump::decode(bytes, opcode)?;
            (InstructionType::Jump(jump), 8)
        }
        Opcode::Call => {
            let call = Call::decode(bytes)?;
            (InstructionType::Call(call), 8)
        }
        Opcode::Exit => {
            (InstructionType::Exit(Exit), 8)
        }
        _ => return Err(format!("Unsupported opcode: {}", opcode.to_str())),
    };

    Ok((instr, size))
} 