//! Humblegen code application

mod cli;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let args: cli::CliArgs = argh::from_env();

    let spec_file = std::fs::File::open(&args.input).context(format!("unable to open specification file {:?}", &args.input))?;
    let spec = humblegen::parse(spec_file).context(format!("failed to parse specification file {:?}", &args.input))?;

    args.code_generator()?.generate(&spec, &args.output)?;

    Ok(())
}
