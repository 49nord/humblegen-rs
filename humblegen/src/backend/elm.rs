//! Elm code generator.

use crate::{ast, Artifact, LibError, Spec};
use anyhow::Result;
use inflector::cases::camelcase::to_camel_case;
use std::io::{self, BufWriter};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

const BACKEND_NAME: &str = "elm";

pub mod type_generation;
pub mod decoder_generation;
pub mod encoder_generation;
pub mod endpoint_generation;


pub(crate) struct IndentWriter {
    indent: usize,
    outstream : Box<dyn io::Write>,
}

impl IndentWriter {
    pub(crate) fn for_file(outdir : &Path, filename :&str) -> Result<Self, LibError> {
        let data_path = { let mut p = PathBuf::from(outdir); p.push(filename); p };

        let outfile = File::create(&data_path).map_err(LibError::IoError)?;
        let outstream = BufWriter::new(outfile);

        Ok(Self { outstream: Box::new(outstream), indent: 0 })
    }

    fn kill_indent(&mut self) {
        self.indent = 0;
    }

    fn increase_indent(&mut self) -> String {
        self.indent += 1;
        self.newline()
    }

    fn decrease_indent(&mut self) -> String {
        self.indent -= 1;
        self.newline()
    }

    fn tabs(&self) -> String {
        "    ".repeat(self.indent)
    } 

    fn newline(&self) -> String {
        format!("\n{}", self.tabs())
    } 

    fn start_line(&mut self) -> Result<&mut dyn io::Write, LibError> {
        write!(self.outstream, "\n{}", self.tabs())?;
        Ok(&mut self.outstream)
    }

    fn handle(&mut self) -> &mut dyn io::Write {
        &mut self.outstream
    }

    fn empty_lines(&mut self, num : usize) -> Result<(), LibError> {
        write!(self.outstream, "{}", "\n".repeat(num + 1))?;
        Ok(())
    }
}


fn generate_doc_comment(doc_comment: &Option<String>) -> String {
    // TODO: escape comment
    match doc_comment {
        Some(ref ds) => format!("{{-| {ds}\n-}}", ds = ds),
        None => "".to_owned(),
    }
}


fn to_atom(s: String) -> String {
    if s.contains(' ') {
        format!("({})", s)
    } else {
        s
    }
}


fn field_name(ident: &str) -> String {
    to_camel_case(ident)
}

pub struct Generator {
    _artifact: Artifact,
}

impl Generator {
    pub fn new(artifact: Artifact) -> Result<Self, LibError> {
        match artifact {
            Artifact::TypesOnly | Artifact::ClientEndpoints => Ok(Self { _artifact: artifact }),
            Artifact::ServerEndpoints => Err(LibError::UnsupportedArtifact {
                artifact,
                backend: BACKEND_NAME,
            }),
        }
    }

    fn make_file(_spec :&Spec, outdir: &Path, name :&str) -> Result<IndentWriter, LibError> {
        // TODO: populate mem filesystem or temp folder first, then make everything visible at once
        // to avoid partial write out on error
        let mut file = IndentWriter::for_file(outdir, &format!("{}.elm", name))?;

        // TODO: make module path prefix configurable
        write!(file.handle(), "module Api.{} exposing (..)", name.replace("/","."))?;
        file.empty_lines(2)?;

        // TODO: write timestamp and info that this file is generated
        Ok(file)
    }

    pub fn generate_user_defined_types(spec :&Spec, outdir: &Path) -> Result<(), LibError> {

        {
            let mut builtin_dir = PathBuf::from(outdir);
            builtin_dir.push("BuiltIn");
            fs::create_dir(builtin_dir)?;
        }

        {
            let mut file = Self::make_file(spec, outdir, "BuiltIn/Bytes")?;
            write!(file.handle(), "{}", include_str!("./elm/builtin_type_bytes.elm"))?;
        }

        {
            let mut file = Self::make_file(spec, outdir, "BuiltIn/Uuid")?;
            write!(file.handle(), "{}", include_str!("./elm/builtin_type_uuid.elm"))?;
        }

        let mut file = Self::make_file(spec, outdir, "Data")?;
        
        for spec_item in spec.iter() {
            match spec_item {
                ast::SpecItem::StructDef(sdef) => type_generation::generate_struct_def(sdef, &mut file)?,
                ast::SpecItem::EnumDef(edef) => type_generation::generate_enum_def(edef, &mut file)?,
                ast::SpecItem::ServiceDef(_) => {},
            };
        }

        Ok(())
    }

    pub fn generate_decoders(spec :&Spec, outdir: &Path) -> Result<(), LibError> {
        let mut file = Self::make_file(spec, outdir, "Decode")?;
        write!(file.handle(), "{}", include_str!("./elm/preamble_decoder.elm"))?;
        write!(file.start_line()?, "{}", "import Api.Data exposing (..)")?;
        file.empty_lines(2)?;
        write!(file.handle(), "{}", decoder_generation::generate_type_decoders(spec))?;
        Ok(())
    }

    pub fn generate_encoders(spec :&Spec, outdir: &Path) -> Result<(), LibError> {
        let mut file = Self::make_file(spec, outdir, "Encode")?;
        write!(file.handle(), "{}", include_str!("./elm/preamble_encoder.elm"))?;
        write!(file.start_line()?, "{}", "import Api.Data exposing (..)")?;
        file.empty_lines(2)?;
        write!(file.handle(), "{}", encoder_generation::generate_type_encoders(spec))?;
        Ok(())
    }

    pub fn generate_endpoints(spec :&Spec, outdir: &Path) -> Result<(), LibError> {

        let mut service_dir = PathBuf::from(outdir);
        service_dir.push("Service");
        fs::create_dir(service_dir)?;

        for spec_item in spec.iter() {
            match spec_item {
                ast::SpecItem::StructDef(..) | ast::SpecItem::EnumDef(..) => {},
                ast::SpecItem::ServiceDef(service) => {
                    let mut file = Self::make_file(spec, outdir, &format!("Service/{}", service.name))?;
                    write!(file.handle(), "{}", include_str!("./elm/preamble_service.elm"))?;
                    write!(file.start_line()?, "{}", "import Api.Data exposing (..)")?;
                    write!(file.start_line()?, "{}", "import Api.Encode as AE")?;
                    write!(file.start_line()?, "{}", "import Api.Decode as AD")?;
                    file.empty_lines(2)?;
                    endpoint_generation::generate(service, &mut file)?;
                },
            };
        }

        Ok(())
    }

    // pub fn generate_spec(&self, spec: &Spec) -> String {
    //     let generate_client_side_services = self.artifact == Artifact::ClientEndpoints
    //         && spec
    //             .iter()
    //             .find(|item| item.service_def().is_some())
    //             .is_some();

    //     let defs = generate_def(spec);

    //     let mut outfile = vec![
    //         include_str!("elm/module_header.elm"),
    //         include_str!("elm/preamble_types.elm"),
    //         if generate_client_side_services {
    //             include_str!("elm/preamble_services.elm")
    //         } else {
    //             ""
    //         },
    //         &defs,
    //         include_str!("elm/utils_types.elm"),
    //     ];

    //     if generate_client_side_services {
    //         let decoders = decoder_generation::generate_type_decoders(spec);
    //         let encoders = encoder_generation::generate_type_encoders(spec);
    //         let clients = ""; //generate_rest_api_clients(spec);
    //         let client_side_code: Vec<&str> = vec![&decoders, &encoders, &clients];
    //         outfile.extend(client_side_code);
    //         outfile.join("\n")
    //     } else {
    //         outfile.join("\n")
    //     }
    // }

    pub fn validate_output_dir(path: &Path) -> Result<(), LibError> {
        if !path.is_dir() {
            return Err(LibError::OutputMustBeFolder {
                backend: BACKEND_NAME,
            });
        }

        let is_empty = path.read_dir().map_err(LibError::IoError)?.next().is_none();

        if !is_empty {
            return Err(LibError::OutputFolderNotEmpty {
                backend: BACKEND_NAME,
            });
        }

        Ok(())
    }
}

impl crate::CodeGenerator for Generator {
    fn generate(&self, spec: &Spec, output: &Path) -> Result<(), LibError> {
        Self::validate_output_dir(&output)?;

        Self::generate_user_defined_types(&spec, &output)?;
        Self::generate_decoders(&spec, &output)?;
        Self::generate_encoders(&spec, &output)?;
        Self::generate_endpoints(&spec, &output)?;

        Ok(())
    }
}
