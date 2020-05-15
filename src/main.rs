//! Humblegen code application

mod cli;

fn main() {
    let args: cli::CliArgs = argh::from_env();
    let spec = humblegen::parse(std::fs::File::open(args.input).expect("open input file"))
        .expect("parse input file");

    println!("{}", args.lang.render(&spec));
}
