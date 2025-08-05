use std::path::PathBuf;

use clap::Args;

use crate::error::CommandError;

#[derive(Args)]
pub struct Command {
    #[arg(name = "input-file-path")]
    source_file_path: PathBuf,
}

impl Command {
    pub fn run(self) -> Result<(), Error> {
        let Self { source_file_path } = self;
        let source_code = std::fs::read_to_string(&source_file_path).map_err(|e| Error::ReadFile { file_path: source_file_path.clone(), source: e })?;
        let tokens = sbpf_assembler::tokenize(&source_code).map_err(|e| Error::Tokenize { source: e })?;
        //
        let mut prev_line = 0;
        for token in &tokens {
            let line = match token {
                sbpf_assembler::lexer::Token::Directive(_, line) |
sbpf_assembler::lexer::Token::Label(_, line) |
sbpf_assembler::lexer::Token::Identifier(_, line) |
sbpf_assembler::lexer::Token::Opcode(_, line) |
sbpf_assembler::lexer::Token::Register(_, line) |
sbpf_assembler::lexer::Token::ImmediateValue(_, line) |
sbpf_assembler::lexer::Token::BinaryOp(_, line) |
sbpf_assembler::lexer::Token::StringLiteral(_, line) |
sbpf_assembler::lexer::Token::LeftBracket(line) |
sbpf_assembler::lexer::Token::RightBracket(line) |
sbpf_assembler::lexer::Token::Comma(line) |
sbpf_assembler::lexer::Token::Colon(line) => *line
            };
            if line != prev_line {
                println!();
                prev_line = line;
            }
            print!("{:?} ", token);
        }
        println!();
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    ReadFile { file_path: PathBuf, source: std::io::Error },
    Tokenize { source: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ReadFile { file_path, source } => {
                write!(f, "Failed to read file {}, error: {}", file_path.display(), source)
            }
            Error::Tokenize { source } => {
                write!(f, "Failed to tokenize: {}", source)
            }
        }
    }
}

impl std::error::Error for Error {}

impl CommandError for Error {
    fn exit_code(&self) -> exitcode::ExitCode {
        match self {
            Self::Tokenize { .. } => {
                exitcode::DATAERR
            }
            Self::ReadFile { .. } => exitcode::IOERR,
        }
    }
}
