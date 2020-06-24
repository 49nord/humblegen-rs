use std::{env, path::PathBuf};

fn main() {
    let this_project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let humble_file = this_project_dir.join("../humblegen/tests/rust/service/spec.humble");
    humblegen::build(humble_file).expect("compile humble");
}
