use std::{path, str};

/// Command-line arguments
#[derive(argh::FromArgs)]
#[argh(description = "generate code from humble protocol spec")]
pub(crate) struct CliArgs {
    /// language to generate code for
    #[argh(option, short = 'l', from_str_fn(str::FromStr::from_str))]
    pub(crate) lang: humblegen::Language,
    /// input path (humble file)
    #[argh(positional)]
    pub(crate) input: path::PathBuf,
}
