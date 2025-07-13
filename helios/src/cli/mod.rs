mod assemble;
mod run;
mod tokenize;
use std::io::Write;

use clap::{CommandFactory, Parser, Subcommand};

use crate::{
    constant::PROGRAM_NAME,
    error::{Error, Result},
    shadow,
};

#[derive(Parser)]
#[command(
    name = PROGRAM_NAME,
    author,
    version,
    long_version = shadow::CLAP_LONG_VERSION,
    about,
    long_about = None
)]
pub struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Print the version information")]
    Version,

    #[command(about = "Output shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: clap_complete::Shell },

    #[command(about = "Tokenize a source file")]
    Tokenize {
        #[clap(flatten)]
        command: tokenize::Command,
    },

    #[command(visible_alias = "asm", about = "Translate sBPF Assembly to bytecode")]
    Assemble {
        #[clap(flatten)]
        command: assemble::Command,
    },

    #[command(about = "Run a program")]
    Run {
        #[clap(flatten)]
        command: run::Command,
    },
}

impl Default for Cli {
    fn default() -> Self { Self::parse() }
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.commands {
            Commands::Version => {
                std::io::stdout()
                    .write_all(Self::command().render_long_version().as_bytes())
                    .expect("Failed to write to stdout");
                Ok(())
            }
            Commands::Completions { shell } => {
                let mut app = Self::command();
                let bin_name = app.get_name().to_string();
                clap_complete::generate(shell, &mut app, bin_name, &mut std::io::stdout());
                Ok(())
            }
            Commands::Assemble { command } => command.run().map_err(Error::from),
            Commands::Run { command } => command.run().map_err(Error::from),
            Commands::Tokenize { command } => command.run().map_err(Error::from),
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::{Cli, Commands};

    #[test]
    fn test_command_simple() {
        if matches!(Cli::parse_from(["program_name", "version"]).commands, Commands::Version) {
            // everything is good.
        } else {
            panic!();
        }
    }
}
