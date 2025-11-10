use std::process::{ExitCode, Termination};

mod game;
mod macros;

fn main() -> ExitCode {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
    }

    // maybe use clap later
    game::physball_client_main().report()
}
