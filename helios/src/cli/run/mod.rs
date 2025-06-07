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
        let ret = helios_vm::run(&source_code).map_err(|e| Error::RunBytecode { source: e })?;
        println!("Return value: {}", ret);
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    ReadFile { file_path: PathBuf, source: std::io::Error },
    RunBytecode { source: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ReadFile { file_path, source } => {
                write!(f, "Failed to read file {}, error: {}", file_path.display(), source)
            }
            Error::RunBytecode { source } => {
                write!(f, "Failed to run bytecode: {}", source)
            }
        }
    }
}

impl std::error::Error for Error {}

impl CommandError for Error {
    fn exit_code(&self) -> exitcode::ExitCode {
        match self {
            Self::RunBytecode { .. } => {
                exitcode::DATAERR
            }
            Self::ReadFile { .. } => exitcode::IOERR,
        }
    }
}
