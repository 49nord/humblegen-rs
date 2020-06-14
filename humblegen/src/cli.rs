use std::{path, str, ops::Deref};
use anyhow::{self, Result};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("unknown code generation backend '{0}'")]
    UnknownBackend(String),
    #[error("unknown output artifact '{0}'")]
    UnknownArtifact(String),
    #[error("{0}")]
    LibraryError(#[from] humblegen::LibError),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Backend {
    Rust,
    Elm,
    Docs,
}

impl str::FromStr for Backend {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "RUST" => Ok(Backend::Rust),
            "ELM" => Ok(Backend::Elm),
            "DOCS" | "DOC" | "DOCUMENTATION" => Ok(Backend::Docs),
            _ => Err(CliError::UnknownBackend(s.to_string())),
        }
    }
}

#[derive(Default)]
pub(crate) struct Artifact(humblegen::Artifact);

impl str::FromStr for Artifact {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TYPES" => Ok(Artifact(humblegen::Artifact::TypesOnly)),
            "CLIENT" => Ok(Artifact(humblegen::Artifact::ClientEndpoints)),
            "SERVER" => Ok(Artifact(humblegen::Artifact::ServerEndpoints)),
            _ => Err(CliError::UnknownArtifact(s.to_string())),
        }
    }
}

impl Deref for Artifact {
    type Target = humblegen::Artifact;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Command-line arguments
// TODO: turn into enum separating language backends from docs backend, docs backend does not need a gen_server and gen_client field
#[derive(argh::FromArgs)]
#[argh(description = "generate code from humble protocol spec")]
pub(crate) struct CliArgs {
    /// language to generate code for
    #[argh(option, short = 'l', long = "language", from_str_fn(str::FromStr::from_str))]
    pub(crate) backend: Backend,
    /// generate REST endpoints for a server
    #[argh(option, short = 'a', from_str_fn(str::FromStr::from_str), default = "Artifact::default()")]
    pub(crate) artifacts: Artifact,
    /// input path to humble file
    #[argh(positional)]
    pub(crate) input: path::PathBuf,
    /// input path to humble file
    #[argh(option, short = 'o')]
    pub(crate) output: path::PathBuf,
}


impl CliArgs {
    /// Dynamcally select and instantiate the correct backend for the given
    /// command-line arguments.
    /// 
    /// Might fail because the backend cannot fulfill the request. For example,
    /// requesting server endpoints for elm -- a client-side programming language --
    /// will result in an error.
    pub fn code_generator(&self) -> Result<Box<dyn humblegen::CodeGenerator>, CliError> {
        match self.backend {
            Backend::Rust => Ok(Box::new(humblegen::backend::rust::Generator::new(*self.artifacts).map_err(CliError::LibraryError)?)),
            Backend::Elm => Ok(Box::new(humblegen::backend::elm::Generator::new(*self.artifacts).map_err(CliError::LibraryError)?)),
            Backend::Docs => Ok(Box::new(humblegen::backend::docs::Generator::default())),
        }
    }
}