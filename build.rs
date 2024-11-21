use clap::CommandFactory;
use clap_complete::{generate_to, shells::Shell};
use std::env;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let out_dir = match env::var_os("OUT_DIR") {
        Some(out_dir) => out_dir,
        None => return Ok(()),
    };

    let mut cmd = Args::command();
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, env!("CARGO_PKG_NAME"), out_dir.clone())?;
    }

    Ok(())
}
