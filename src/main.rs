use std::process::{ExitCode, Termination};

mod game;

fn main() -> ExitCode {
    // maybe use clap later
    game::ballphys_client_main().report()
}
