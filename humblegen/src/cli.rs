use std::{path, str};
use std::error::Error;

#[derive(Debug, Copy, Clone)]
pub enum Backend {
    Rust,
    Elm,
    Docs,
}

impl str::FromStr for Backend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "RUST" => Ok(Backend::Rust),
            "ELM" => Ok(Backend::Elm),
            "DOCS" | "DOC" | "DOCUMENTATION" => Ok(Backend::Docs),
            other => Err(format!("unknown language: {}", other)),
        }
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
    #[argh(switch, short = 's', long = "server")]
    pub(crate) gen_server: bool,
    /// generate REST endpoints for a client
    #[argh(switch, short = 'c', long = "client")]
    pub(crate) gen_client: bool,
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
    pub fn code_generator(&self) -> Result<Box<dyn humblegen::CodeGenerator>, Box<dyn Error>> {
        match self.backend {
            Backend::Rust => Ok(Box::new(humblegen::backend::rust::Generator::default())),
            Backend::Elm => Ok(Box::new(humblegen::backend::elm::Generator::default())),
            Backend::Docs => Ok(Box::new(humblegen::backend::docs::Generator::default())),
        }
    }
}