use anyhow::Context;

use std::path::PathBuf;

#[derive(Debug)]
struct RustTestCase {
    name: String,
    humble_spec: PathBuf,
    humble_rust_out: PathBuf,
    main: PathBuf,
}

impl RustTestCase {
    fn run(&self) {
        let spec_file = std::fs::File::open(&self.humble_spec).expect("open humble spec file");
        let spec = humblegen::parse(spec_file).expect("parse humble spec file");
        let spec_rust = humblegen::Language::Rust.render(&spec).to_string();
        std::fs::write(&self.humble_rust_out, spec_rust).expect("write generated rust code");

        let t = trybuild::TestCases::new();
        t.pass(&self.main);
        // cases run on drop of t
        drop(t);
    }

    fn from_test_dir(dir: &std::fs::DirEntry) -> anyhow::Result<RustTestCase> {
        let entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir.path())?
            .collect::<std::io::Result<Vec<std::fs::DirEntry>>>()
            .context("read test dir entries")?
            .into_iter()
            .filter(|e| e.file_type().expect("get file type").is_file())
            .collect();

        let name = dir
            .file_name()
            .into_string()
            .ok()
            .context("test case dir name must be a Rust String")?;

        struct RequiredFile(Option<PathBuf>, &'static str, &'static str);
        impl RequiredFile {
            fn must_exist(self) -> anyhow::Result<PathBuf> {
                self.0.clone().ok_or_else(|| {
                    anyhow::anyhow!("test case dir requires file {:?} ({})", self.1, self.2)
                })
            }
        }

        let mut humble_spec = RequiredFile(None, "spec.humble", "input humble spec");
        let mut humble_rust_out = RequiredFile(
            None,
            "spec.rs",
            "reference output of Rust backend for spec.humble",
        );
        let mut main = RequiredFile(
            None,
            "main.rs",
            "consumer of generated code (the test case)",
        );
        let mut required_files = vec![&mut humble_spec, &mut humble_rust_out, &mut main];

        for entry in entries {
            let name = entry
                .file_name()
                .into_string()
                .ok()
                .context("test case file names must be Rust strs")?;

            for required_file in required_files.iter_mut() {
                if required_file.1 == name.as_str() {
                    required_file.0 = Some(entry.path());
                }
            }
        }

        Ok(RustTestCase {
            name: name.to_string(),
            humble_spec: humble_spec.must_exist()?,
            humble_rust_out: humble_rust_out.must_exist()?,
            main: main.must_exist()?,
        })
    }
}

#[test]
fn rust() {
    // parse all the directories in ./tests/rust to RustTestCase instances
    let tests: Vec<RustTestCase> = std::fs::read_dir("./tests/rust/")
        .expect("read test dir")
        .collect::<std::io::Result<Vec<std::fs::DirEntry>>>()
        .expect("read test dir entries")
        .into_iter()
        .map(|dir_entry| RustTestCase::from_test_dir(&dir_entry))
        .collect::<anyhow::Result<Vec<RustTestCase>>>()
        .expect("parse test dirs as test cases");

    // run them
    for test in tests {
        println!("running test {:?}", test.name);
        test.run();
    }
}
