//! Humblegen compiler library

use std::error::Error;
use std::{env, fs, path, str};

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
            Language::Rust => rust::render(&spec).to_string(),
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

/// Parse a specification from a file path.
pub fn parse_spec<P: AsRef<path::Path>>(src: P) -> Result<ast::Spec, Box<dyn Error>> {
    let input = fs::read_to_string(src).map(Box::new)?;
    Ok(parser::parse(&input))
}

/// Build the specified humblefile using the Rust builder.
///
/// Outputs `rerun-if-changed` instructions for the given `src` path.
pub fn build<P: AsRef<path::Path>>(src: P) -> Result<(), Box<dyn Error>> {
    let spec = parse_spec(src.as_ref())?;
    let rendered = Language::Rust.render(&spec);

    let out_dir: path::PathBuf = env::var("OUT_DIR").expect("read OUT_DIR envvar").into();
    fs::write(out_dir.join("protocol.rs"), rendered.as_bytes()).map(Box::new)?;

    println!("cargo:rerun-if-changed={}", src.as_ref().display());

    Ok(())
}
