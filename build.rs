use clap::{CommandFactory, ValueEnum};
use clap_complete as completion;
use std::{env, io};

include!("src/cli.rs");

fn main() -> Result<(), io::Error> {
    // The clap command.
    let mut command = Args::command();
    // The directory to place the shell completions.
    let out_dir = match env::var_os("OUT_DIR") {
        Some(out_dir) => out_dir,

        None => return Ok(()),
    };

    // Generate completion for each supported shell.
    for &shell in Shell::value_variants() {
        completion::generate_to(shell, &mut command, NAME, &out_dir)?;
    }

    Ok(())
}
