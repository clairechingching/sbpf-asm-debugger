pub mod vm;
pub mod program;
pub mod instruction;
pub mod log_buffer;

use helios_assembler::{Parser, Program};
use crate::vm::VM;
use helios_assembler::debuginfo::RegisterType;
use std::cell::RefCell;
use serde::Serialize;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct Register {
    name: String,
    value: String,
    register_type: String,
}

#[derive(Serialize)]
struct Rdata {
    label: String,
    address: usize,
    value: String,
}

#[derive(Serialize)]
struct Memory {
    label: String,
    value: String,
}

thread_local! {
    static VM_INSTANCE: RefCell<VM> = RefCell::new(VM::new());
}

#[wasm_bindgen]
pub fn get_registers() -> JsValue {
    let registers: Vec<Register> = VM_INSTANCE.with(|vm| {
        let vm = vm.borrow();
        let reg_values = vm.get_registers();
        reg_values.iter().enumerate()
            .map(|(i, reg)| Register {
                name: format!("r{}", i),
                value: match reg.register_type {
                    RegisterType::Addr => format!("0x{:016x}", reg.value),
                    RegisterType::Int => format!("{}", reg.value),
                    RegisterType::Null => format!("null"),
                },
                register_type: format!("{:?}", reg.register_type.to_string()),
            })
            .collect()
    });
    to_value(&registers).unwrap()
}

#[wasm_bindgen]
pub fn get_rodata() -> JsValue {
    let rodata: Vec<Rdata> = VM_INSTANCE.with(|vm| {
        let vm = vm.borrow();
        let rodata = vm.get_rodata();
        rodata.iter()
            .map(|(label, offset, val)| Rdata {
                label: label.to_string(),
                address: *offset + vm.get_entry_point(),
                value: val.to_string(),
            })
            .collect()
    });
    to_value(&rodata).unwrap()
}

#[wasm_bindgen]
pub fn get_memory() -> JsValue {
    let instruction_data = VM_INSTANCE.with(|vm| {
        let vm = vm.borrow();
        vm.get_instruction_data()
    });

    let memory = vec![
        Memory {
            label: "instruction_data".to_string(),
            value: format!("{:?}", instruction_data),
        }
    ];
    to_value(&memory).unwrap()
}

#[wasm_bindgen]
pub fn assemble(assembly: &str) -> Result<Vec<u8>, String> {
    let tokens = match helios_assembler::tokenize(assembly) {
        Ok(tokens) => tokens,
        Err(e) => return Err(format!("Tokenizer error: {}", e)),
    };

    let mut parser = Parser::new(tokens);
    let parse_result = match parser.parse() {
        Ok(program) => program,
        Err(e) => return Err(format!("Parser error: {}", e)),
    };

    let program = Program::from_parse_result(parse_result);
    let mut ro_data = Vec::new();
    if program.has_rodata() {
        ro_data = program.parse_rodata();
    }

    VM_INSTANCE.with(|vm| {
        let mut vm = vm.borrow_mut();
        vm.load_rodata(ro_data);
    });

    let bytecode = program.emit_bytecode();
    let line_map = program.get_line_map();
    let debug_map = program.get_debug_map();
    VM_INSTANCE.with(|vm| {
        let mut vm = vm.borrow_mut();
        vm.load_line_map(line_map);
        vm.load_debug_map(debug_map);
    });
    Ok(bytecode)
}

#[wasm_bindgen]
pub fn initialize(assembly: &str) -> Result<u64, String> {
    VM_INSTANCE.with(|vm| {
        let mut vm = vm.borrow_mut();
        vm.reset();
    });
    let bytecode = assemble(assembly)?;
    VM_INSTANCE.with(|vm| {
        let mut vm = vm.borrow_mut();
        vm.load_program(bytecode);
    });
    Ok(0)
}

#[wasm_bindgen]
pub fn load_input_data(account_number: u64, data: &[u8], data_type: &str) {
    VM_INSTANCE.with(|vm| {
        let mut vm = vm.borrow_mut();
        vm.load_input_data(account_number, data, data_type);
    })
}

#[wasm_bindgen]
pub fn run(assembly: &str) -> Result<u64, String> {
    let bytecode = assemble(assembly)?;
    VM_INSTANCE.with(|vm| {
        let mut vm = vm.borrow_mut();
        vm.load_program(bytecode)?;
        vm.run()
    })
}

#[wasm_bindgen]
pub fn step() -> usize {
    VM_INSTANCE.with(|vm| {
        let mut vm = vm.borrow_mut();
        vm.step_instruction();
        vm.get_line_number()
    })
}

#[wasm_bindgen]
pub fn get_line_number() -> usize {
    VM_INSTANCE.with(|vm| {
        let vm = vm.borrow();
        vm.get_line_number()
    })
}

#[wasm_bindgen]
pub fn is_exited() -> bool {
    VM_INSTANCE.with(|vm| {
        let vm = vm.borrow();
        vm.is_exited()
    })
}