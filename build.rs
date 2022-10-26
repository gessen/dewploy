use clap::CommandFactory;
use clap_complete::{
    generate_to,
    shells::{Bash, Zsh},
};
use std::env;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let out_dir = match env::var_os("OUT_DIR") {
        Some(out_dir) => out_dir,
        None => return Ok(()),
    };

    let mut cmd = Args::command();
    generate_to(Bash, &mut cmd, "dewploy", out_dir.clone())?;
    generate_to(Zsh, &mut cmd, "dewploy", out_dir)?;

    Ok(())
}
