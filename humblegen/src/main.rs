//! Humblegen code application

mod cli;

use anyhow::{Context, Result};
use structopt::StructOpt;

fn main() -> Result<()> {
    let args = cli::CliArgs::from_args();

    let spec_file = std::fs::File::open(&args.input).context(format!(
        "unable to open specification file {:?}",
        &args.input
    ))?;
    let spec = humblegen::parse(spec_file).context(format!(
        "failed to parse specification file {:?}",
        &args.input
    ))?;

    args.code_generator()?.generate(&spec, &args.output)?;

    Ok(())
}
