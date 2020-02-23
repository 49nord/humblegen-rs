use std::fs;

mod ast;
mod parser;
mod rust;
mod util;

fn main() {
    let input = fs::read_to_string("src/sample.humble").unwrap();
    let spec = parser::parse(&input);

    println!("{}", rust::render(&spec));
}
