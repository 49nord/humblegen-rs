//! Humblegen code application

mod cli;

fn main() {
    let args: cli::CliArgs = argh::from_env();
    let spec = humblegen::parse_spec(args.input).expect("parse input file");

    println!("{}", args.lang.render(&spec));
}
