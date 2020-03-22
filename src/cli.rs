use std::{path, str};

/// Codegen language
#[derive(Debug, Copy, Clone)]
pub(crate) enum Language {
    /// Rust
    Rust,
    /// Elm
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

/// Command-line arguments
#[derive(argh::FromArgs)]
#[argh(description = "generate code from humble protocol spec")]
pub(crate) struct CliArgs {
    /// language to generate code for
    #[argh(option, short = 'l', from_str_fn(str::FromStr::from_str))]
    pub(crate) lang: Language,
    /// input path (humble file)
    #[argh(positional)]
    pub(crate) input: path::PathBuf,
}
