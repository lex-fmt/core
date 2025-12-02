use clap::{Arg, Command, ValueHint};
use clap_complete::{generate_to, shells::*};
use std::env;
use std::io::Error;

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = Command::new("lexv")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Interactive terminal viewer for lex documents")
        .arg(
            Arg::new("path")
                .help("Path to the lex document to open")
                .required(true)
                .index(1)
                .value_hint(ValueHint::FilePath),
        );

    // Generate completions for bash
    generate_to(Bash, &mut cmd, "lexv", &outdir)?;

    // Generate completions for zsh
    generate_to(Zsh, &mut cmd, "lexv", &outdir)?;

    // Generate completions for fish
    generate_to(Fish, &mut cmd, "lexv", &outdir)?;

    println!("cargo:warning=Shell completions generated in {outdir:?}");

    Ok(())
}
