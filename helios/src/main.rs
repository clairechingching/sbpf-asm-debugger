mod cli;
mod constant;
mod error;
mod shadow {
    #![allow(clippy::needless_raw_string_hashes)]
    use shadow_rs::shadow;
    shadow!(build);

    pub use self::build::*;
}

use std::process;

use crate::cli::Cli;

fn main() -> process::ExitCode {
    let exit_code = if let Err(err) = Cli::default().run() {
        eprintln!("{err}");
        err.exit_code()
    } else {
        exitcode::OK
    };

    process::ExitCode::from(
        u8::try_from(exit_code).expect("`exit_code` should not be greater than 127"),
    )
}
