pub struct Program {
    pub bytecode: Vec<u8>,
    pub entry_point: u64,
}

impl Program {
    pub fn new(bytecode: Vec<u8>) -> Result<Self, String> {
        if bytecode.len() < 64 { // Minimum size for ELF header
            return Err("Invalid bytecode: too short to be an ELF file".to_string());
        }

        // Verify ELF magic number
        if bytecode[0] != 0x7f || bytecode[1] != 0x45 || bytecode[2] != 0x4c || bytecode[3] != 0x46 {
            return Err("Invalid bytecode: not an ELF file".to_string());
        }

        let mut program = Program { 
            bytecode, 
            entry_point: 0 
        };
        
        program.parse_bytecode()?;
        Ok(program)
    }

    fn parse_bytecode(&mut self) -> Result<(), String> {
        // Parse entry point from ELF header (offset 24-31)
        self.entry_point = u64::from_le_bytes([
            self.bytecode[24], self.bytecode[25], self.bytecode[26], self.bytecode[27],
            self.bytecode[28], self.bytecode[29], self.bytecode[30], self.bytecode[31],
        ]);

        Ok(())
    }

    pub fn read(&self, address: u64, length: u64) -> Result<Vec<u8>, String> {
        let mut data = Vec::new();
        for i in 0..length {
            data.push(self.bytecode[address as usize + i as usize]);
        }
        Ok(data)
    }
}
