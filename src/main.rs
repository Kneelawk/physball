use std::process::{ExitCode, Termination};

mod game;
mod macros;

fn main() -> ExitCode {
    // maybe use clap later
    game::physball_client_main().report()
}
