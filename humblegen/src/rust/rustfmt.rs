//! rustfmt invocation helpers

/// The code in this file is based on
///     https://docs.rs/bindgen/0.51.1/src/bindgen/lib.rs.html#1945
use std::borrow::Cow;
use std::io;
use std::io::prelude::*;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

/// Gets the rustfmt path to rustfmt the generated bindings.
fn rustfmt_path() -> io::Result<PathBuf> {
    #[cfg(feature = "which-rustfmt")]
    match which::which("rustfmt") {
        Ok(p) => Ok(p),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e))),
    }
    #[cfg(not(feature = "which-rustfmt"))]
    Err(io::Error::new(
        io::ErrorKind::Other,
        "which wasn't enabled, and no rustfmt binary specified",
    ))
}

/// Checks if rustfmt_bindings is set and runs rustfmt on the string
pub(crate) fn rustfmt_2018_generated_string<'a>(source: &'a str) -> io::Result<Cow<'a, str>> {
    let rustfmt = rustfmt_path()?;
    let mut cmd = Command::new(&*rustfmt);

    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    cmd.args(&["--edition", "2018"]);

    let mut child = cmd.spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    let source = source.to_owned();

    // Write to stdin in a new thread, so that we can read from stdout on this
    // thread. This keeps the child from blocking on writing to its stdout which
    // might block us from writing to its stdin.
    let stdin_handle = ::std::thread::spawn(move || {
        let _ = child_stdin.write_all(source.as_bytes());
        source
    });

    let mut output = vec![];
    io::copy(&mut child_stdout, &mut output)?;

    let status = child.wait()?;
    let source = stdin_handle.join().expect(
        "The thread writing to rustfmt's stdin doesn't do \
             anything that could panic",
    );

    match String::from_utf8(output) {
        Ok(bindings) => match status.code() {
            Some(0) => Ok(Cow::Owned(bindings)),
            Some(2) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Rustfmt parsing errors.".to_string(),
            )),
            Some(3) => {
                log::warn!("Rustfmt could not format some lines.");
                Ok(Cow::Owned(bindings))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Internal rustfmt error".to_string(),
            )),
        },
        _ => Ok(Cow::Owned(source)),
    }
}
