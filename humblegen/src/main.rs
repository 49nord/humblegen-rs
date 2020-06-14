//! Humblegen code application

mod cli;

fn main() {
    let args: cli::CliArgs = argh::from_env();
    
    let spec_file = std::fs::File::open(&args.input).expect("open input file");
    let spec = humblegen::parse(spec_file).expect("parse input file");

    args.code_generator()
        .expect("backend supports given arguments")
        .generate(&spec, &args.output)
        .expect("output generation succeeds");
}
