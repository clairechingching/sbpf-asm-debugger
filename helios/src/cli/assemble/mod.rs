use std::path::PathBuf;

use clap::Args;

use crate::error::CommandError;

#[derive(Args)]
pub struct Command {
    #[arg(name = "input-file-path")]
    source_file_path: PathBuf,

    #[arg(short = 'o', long, name = "output-file-path")]
    output_file_path: Option<PathBuf>,
}

impl Command {
    pub fn run(self) -> Result<(), Error> {
        let Self { source_file_path, output_file_path } = self;
        let source_code = std::fs::read_to_string(&source_file_path)
            .map_err(|e| Error::ReadFile { file_path: source_file_path.clone(), source: e })?;
        let output_file_path = output_file_path.unwrap_or_else(|| {
            let mut file_path = source_file_path.clone();
            let _ = file_path.set_extension("so");
            file_path
        });

        // Tokenize the source.
        let tokens = sbpf_assembler::tokenize(&source_code)
            .map_err(|e| Error::Tokenize { source: e })?;

        // Parse the tokens into an AST.
        let parse_result = sbpf_assembler::Parser::new(tokens)
            .parse()
            .map_err(|e| Error::Parse { source: e })?;

        // Construct program from ParseResult.
        let program = sbpf_assembler::Program::from_parse_result(parse_result);

        // Assemble the source code and emit the bytecode.
        let bytecode = program.emit_bytecode();
        std::fs::write(&output_file_path, bytecode)
            .map_err(|e| Error::WriteFile { file_path: output_file_path, source: e })
    }
}

#[derive(Debug)]
pub enum Error {
    ReadFile { file_path: PathBuf, source: std::io::Error },
    WriteFile { file_path: PathBuf, source: std::io::Error },
    Tokenize { source: String },
    Parse { source: String },
    ConstructProgram { source: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ReadFile { file_path, source } => {
                write!(f, "Failed to read file {}, error: {}", file_path.display(), source)
            }
            Error::WriteFile { file_path, source } => {
                write!(f, "Failed to write file {}, error: {}", file_path.display(), source)
            }
            Error::Tokenize { source } => {
                write!(f, "Failed to tokenize source: {}", source)
            }
            Error::Parse { source } => {
                write!(f, "Failed to parse source code: {}", source)
            }
            Error::ConstructProgram { source } => {
                write!(f, "Failed to construct program: {}", source)
            }
        }
    }
}

impl std::error::Error for Error {}

impl CommandError for Error {
    fn exit_code(&self) -> exitcode::ExitCode {
        match self {
            Self::Tokenize { .. } | Self::Parse { .. } | Self::ConstructProgram { .. } => {
                exitcode::DATAERR
            }
            Self::ReadFile { .. } | Self::WriteFile { .. } => exitcode::IOERR,
        }
    }
}
