//! Humblegen code application

use std::fs;

mod ast;
mod cli;
mod elm;
mod parser;
mod rust;

fn main() {
    let args: cli::CliArgs = argh::from_env();

    let input = fs::read_to_string(args.input).expect("could not read input file");
    let spec = parser::parse(&input);

    match args.lang {
        cli::Language::Rust => println!("{}", rust::render(&spec)),
        cli::Language::Elm => println!("{}", elm::render(&spec)),
    }
}
