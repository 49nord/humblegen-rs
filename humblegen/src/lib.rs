//! Humblegen compiler library

use std::{fmt, io, path::Path, path::PathBuf};

pub use ast::Spec;

pub mod ast;
pub mod backend;
pub mod parser;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibError {
    #[error("backend '{backend}' does not support artifact '{artifact}'")]
    UnsupportedArtifact {
        backend: &'static str,
        artifact: Artifact,
    },
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    ParseError(#[from] pest::error::Error<parser::Rule>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// Which artifacts to produce in addition to user defined types
pub enum Artifact {
    /// Only generate user defined type definitions
    TypesOnly,
    /// Generate encoders, decoders and client-side REST API endpoints
    ClientEndpoints,
    /// Generate encoders, decoders and server-side REST API endpoints
    ServerEndpoints,
}

impl Default for Artifact {
    fn default() -> Self {
        Artifact::TypesOnly
    }
}

impl fmt::Display for Artifact {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Artifact::TypesOnly => "TypesOnly",
            Artifact::ClientEndpoints => "ClientEndpoints",
            Artifact::ServerEndpoints => "ServerEndpoints",
        };
        write!(f, "{}", printable)
    }
}

// Common interface of all backends
pub trait CodeGenerator {
    fn generate(&self, spec: &Spec, output: &Path) -> Result<(), LibError>;
}

pub fn parse<I: io::Read>(mut src: I) -> Result<ast::Spec, LibError> {
    let mut input = String::new();
    src.read_to_string(&mut input).map_err(LibError::IoError)?;
    Ok(parser::parse(&input).map_err(LibError::ParseError)?)
}

/// This method is intended for use form within a `build.rs` file.
///
/// Builds the specified humblefile using the Rust builder
/// and writes the generated code to `$OUT_DIR/protocol.rs`.
///
/// Outputs `rerun-if-changed` instructions for the given `src` path.
pub fn build<P: AsRef<Path>>(src: P) -> Result<(), LibError> {
    println!("cargo:rerun-if-changed={}", src.as_ref().display());
    let out_dir: PathBuf = std::env::var("OUT_DIR")
        .expect("read OUT_DIR envvar")
        .into();
    let out_path = out_dir.join("protocol.rs");

    let infile = std::fs::File::open(src)?;
    let spec = parse(infile)?;
    let generator = backend::rust::Generator::new(Artifact::ServerEndpoints)?;
    generator.generate(&spec, &out_path)?;

    Ok(())
}
