use std::{fs, path, str};

mod ast;
mod elm;
mod parser;
mod rust;

#[derive(Debug, Copy, Clone)]
enum Language {
    Rust,
    Elm,
}

impl str::FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rust" => Ok(Language::Rust),
            "elm" => Ok(Language::Elm),
            other => Err(format!("unknown language: {}", other)),
        }
    }
}

#[derive(argh::FromArgs)]
#[argh(description = "generate code from humble protocol spec")]
struct CliArgs {
    /// language to generate code for
    #[argh(option, short = 'l', from_str_fn(str::FromStr::from_str))]
    lang: Language,
    /// input path (humble file)
    #[argh(positional)]
    input: path::PathBuf,
}

fn main() {
    let args: CliArgs = argh::from_env();

    let input = fs::read_to_string(args.input).expect("could not read input file");
    let spec = parser::parse(&input);

    match args.lang {
        Language::Rust => println!("{}", rust::render(&spec)),
        Language::Elm => println!("{}", elm::render(&spec)),
    }
}
