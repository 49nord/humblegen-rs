//! Humblegen compiler library

use std::error::Error;
use std::{env, io, path, str};

mod ast;
mod elm;
mod parser;
mod rust;

/// Codegen language
#[derive(Debug, Copy, Clone)]
pub enum Language {
    /// Rust
    Rust,
    /// Elm
    Elm,
}

impl Language {
    /// Render output for spec.
    pub fn render(self, spec: &ast::Spec) -> String {
        match self {
            Language::Rust => {
                let src = rust::render(&spec).to_string();
                rust::rustfmt::rustfmt_2018_generated_string(&src)
                    .map(|s| s.into_owned())
                    // if rustfmt doesn't work, use the unformatted source
                    .unwrap_or(src)
            }
            Language::Elm => elm::render(&spec),
        }
    }
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

/// This method is intended for use form within a `build.rs` file.
///
/// Builds the specified humblefile using the Rust builder
/// and writes the generated code to `$OUT_DIR/protocol.rs`.
///
/// Outputs `rerun-if-changed` instructions for the given `src` path.
pub fn build<P: AsRef<path::Path>>(src: P) -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={}", src.as_ref().display());
    let out_dir: path::PathBuf = env::var("OUT_DIR").expect("read OUT_DIR envvar").into();
    let out_path = out_dir.join("protocol.rs");
    generate(std::fs::File::open(src)?, std::fs::File::create(&out_path)?)
}

pub fn generate<I: io::Read, O: io::Write>(src: I, mut dst: O) -> Result<(), Box<dyn Error>> {
    let rendered = Language::Rust.render(&parse(src)?);
    Ok(dst.write_all(rendered.as_bytes())?)
}

pub fn parse<I: io::Read>(mut src: I) -> Result<ast::Spec, Box<dyn Error>> {
    let mut input = String::new();
    src.read_to_string(&mut input)?;
    parser::parse(&input).map_err(|e| Box::new(e) as Box<dyn Error>)
}
