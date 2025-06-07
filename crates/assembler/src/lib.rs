// Tokenizer and parser
pub mod parser;
pub mod tokenizer;
pub mod opcode;
pub mod instruction_verifier;
pub mod utils;

// Intermediate Representation
pub mod astnode;
pub mod dynsym;

// ELF header, program, section
pub mod header;
pub mod program;
pub mod section;

// Debug info
pub mod debuginfo;

#[cfg(test)]
mod tests;

// Type aliases for error handling
pub type ParserError = String;
pub type ProgramError = String;
pub type TokenizerError = String;

pub use self::{
    parser::Parser,
    program::Program,
    tokenizer::tokenize,
};

