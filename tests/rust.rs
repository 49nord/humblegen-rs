use anyhow::Context;

use std::path::PathBuf;

struct RustTestCase {
    name: String,
    humble_spec: PathBuf,
    humble_rust_out: PathBuf,
    main: PathBuf,
}

impl RustTestCase {
    fn run(&self) {
        let spec_file = std::fs::read_to_string(&self.humble_spec).expect("read humble spec");
        let spec = humblegen::parse_spec_str_or_panic(&spec_file);
        let spec_rust = humblegen::Language::Rust.render(&spec).to_string();
        // TODO rustfmt
        std::fs::write(&self.humble_rust_out, spec_rust).expect("write generated rust code");

        let t = trybuild::TestCases::new();
        t.pass(&self.main);
        // cases run on drop of t
        drop(t);
    }

    fn from_test_dir(dir: &std::fs::DirEntry) -> anyhow::Result<RustTestCase> {
        let entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir.path())?
            .collect::<std::io::Result<Vec<std::fs::DirEntry>>>()
            .unwrap()
            .into_iter()
            .filter(|e| e.file_type().expect("get file type").is_file())
            .collect();

        let name = dir
            .file_name()
            .into_string()
            .ok()
            .context("test case dir name must be a Rust String")?;

        let mut humble_spec = None;
        let mut humble_rust_out = None;
        let mut main = None;

        for entry in entries {
            let name = entry
                .file_name()
                .into_string()
                .ok()
                .context("test case file names must be Rust strs")?;

            match name.as_str() {
                "main.rs" => main = Some(entry.path()),
                "spec.humble" => humble_spec = Some(entry.path()),
                "spec.rs" => humble_rust_out = Some(entry.path()),
                x if x.starts_with(".") => (), // ignore
                x if x.ends_with("~") => (),   // ignore
                x => return Err(anyhow::anyhow!("unexpected file in test case dir: {:?}", x)),
            }
        }

        Ok(RustTestCase {
            name: name.to_owned(),
            humble_spec: humble_spec.unwrap(),
            humble_rust_out: humble_rust_out.unwrap(),
            main: main.unwrap(),
        })
    }
}

#[test]
fn rust() {
    let tests: Vec<RustTestCase> = std::fs::read_dir("./tests/rust/")
        .expect("read test dir")
        .collect::<std::io::Result<Vec<std::fs::DirEntry>>>()
        .unwrap()
        .into_iter()
        .map(|dir_entry| RustTestCase::from_test_dir(&dir_entry))
        .collect::<anyhow::Result<Vec<RustTestCase>>>()
        .unwrap();

    for test in tests {
        println!("running test {:?}", test.name);
        test.run();
    }
}
