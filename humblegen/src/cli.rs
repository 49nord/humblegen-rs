use anyhow::{self, Result};
use std::{ops::Deref, path, str};
use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("unknown code generation backend '{0}'")]
    UnknownBackend(String),
    #[error("unknown output artifact '{0}'")]
    UnknownArtifact(String),
    #[error(transparent)]
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

// This impl is necessary allow the usage of the structopt default_value attribute
impl ToString for Artifact {
    fn to_string(&self) -> String {
        match self.0 {
            // These strings have to match the ones in str::FromString
            humblegen::Artifact::TypesOnly => "TYPES".to_string(),
            humblegen::Artifact::ClientEndpoints => "CLIENT".to_string(),
            humblegen::Artifact::ServerEndpoints => "SERVER".to_string(),
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
#[derive(StructOpt)]
#[structopt(about = "generate code from humble protocol spec")]
pub(crate) struct CliArgs {
    /// language to generate code for
    #[structopt(short = "l", long = "language")]
    pub(crate) backend: Backend,
    /// generate REST endpoints for a server
    #[structopt(short = "a", long = "artifacts", default_value)]
    pub(crate) artifacts: Artifact,
    /// input path to humble file
    pub(crate) input: path::PathBuf,
    /// input path to humble file
    #[structopt(short = "o", long = "output")]
    pub(crate) output: path::PathBuf,
    /// prefix to be used in elm module declarations
    #[structopt(long, default_value = "\"Api\"")]
    pub(crate) elm_module_root: String,
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
            Backend::Rust => Ok(Box::new(
                humblegen::backend::rust::Generator::new(*self.artifacts)
                    .map_err(CliError::LibraryError)?,
            )),
            Backend::Elm => Ok(Box::new(
                humblegen::backend::elm::Generator::new(
                    *self.artifacts,
                    self.elm_module_root.clone(),
                )
                .map_err(CliError::LibraryError)?,
            )),
            Backend::Docs => Ok(Box::new(humblegen::backend::docs::Generator::default())),
        }
    }
}
