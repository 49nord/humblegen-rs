//! Elm code generator.

use crate::{ast, Artifact, LibError, Spec};
use anyhow::{Context, Result};
use std::io::{self, BufWriter};
use inflector::cases::camelcase::to_camel_case;
use itertools::Itertools;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub mod decoder_generation;
pub mod encoder_generation;

const BACKEND_NAME: &str = "elm";

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
        write!(self.outstream, "{}", "\n".repeat(num))?;
        Ok(())
    }
}

/// Generate elm code for a docstring.
///
/// If not present, generates an empty string.
fn generate_doc_comment(doc_comment: &Option<String>) -> String {
    match doc_comment {
        Some(ref ds) => format!("{{-| {ds}\n-}}\n", ds = ds),
        None => "".to_owned(),
    }
}

// TODO: Elm does not allow documentation on members, so the docs need to be converted to markdown
//       lists instead. This is true for `type alias` struct fields as well as enum variants.

pub(crate) fn generate_struct_def(def: &ast::StructDef, file :&mut IndentWriter) -> Result<(), LibError> {
    file.kill_indent();

    write!(file.start_line()?, "{doc_comment}type alias {name} =",
        doc_comment = generate_doc_comment(&def.doc_comment),
        name = def.name)?;

    generate_struct_fields(&def.fields, file)?;

    file.empty_lines(2)?;

    Ok(())
}

pub(crate) fn generate_struct_fields(fields: &ast::StructFields, file :&mut IndentWriter) -> Result<(), LibError> {
        
    file.increase_indent();

    for (idx, field) in fields.iter().enumerate() {
        let first = idx == 0;
        generate_struct_field(field, first, file)?;
    }
    
    write!(file.start_line()?, "}}")?;

    file.decrease_indent();

    Ok(())
}


fn generate_struct_field(field: &ast::FieldNode, first : bool, file :&mut IndentWriter) -> Result<(), LibError> {
    write!(file.start_line()?, "{delimiter}{name}: {ty}",
        delimiter = if first { "{ " } else { ", " }, 
        name = field_name(&field.pair.name),
        ty = generate_type_ident(&field.pair.type_ident)
    )?;

    Ok(())
}

/// Generate elm code for an enum definition.
pub(crate) fn generate_enum_def(def: &ast::EnumDef, file :&mut IndentWriter) -> Result<(), LibError> {
    file.kill_indent();

    write!(file.start_line()?, "{doc_comment}type {name}",
         doc_comment = generate_doc_comment(&def.doc_comment),
         name = def.name,)?;
    
    file.increase_indent();

    for (idx, field) in def.variants.iter().enumerate() {
        let first = idx == 0;
        generate_variant_def(field, first, file)?;
    }

    file.empty_lines(2)?;

    Ok(())
}


/// Add parenthesis if necessary.
///
/// Wraps `s` in parentheses if it contains a space.
fn to_atom(s: String) -> String {
    if s.contains(' ') {
        format!("({})", s)
    } else {
        s
    }
}

/// Generate elm code for a variant definition.
fn generate_variant_def(variant: &ast::VariantDef, first : bool, file :&mut IndentWriter) -> Result<(), LibError> {
    let delimiter = if first { "= " } else { "| " };
    match variant.variant_type {
        ast::VariantType::Simple => {
            write!(file.start_line()?, "{delimiter}{name}",
                delimiter = delimiter, 
                name = variant.name,
            )?;
        },
        ast::VariantType::Tuple(ref fields) => {
            write!(file.start_line()?, "{delimiter}{name} {fields}",
                delimiter = delimiter, 
                name = variant.name,
                fields = fields
                .elements()
                .iter()
                .map(generate_type_ident)
                .map(to_atom)
                .join(" ")
            )?;
        }
        ast::VariantType::Struct(ref fields) => {
            write!(file.start_line()?, "{delimiter}{name}",
                delimiter = delimiter, 
                name = variant.name,
            )?;
            generate_struct_fields(fields, file)?;
        }
        ast::VariantType::Newtype(ref ty) => {
            write!(file.start_line()?, "{delimiter}{name} {field}",
                delimiter = delimiter, 
                name = variant.name,
                field = to_atom(generate_type_ident(ty))
            )?;
        }
    }

    Ok(())
}

/// Generate elm code for a type identifier.
fn generate_type_ident(type_ident: &ast::TypeIdent) -> String {
    match type_ident {
        ast::TypeIdent::BuiltIn(atom) => generate_atom(atom),
        ast::TypeIdent::List(inner) => format!("List {}", to_atom(generate_type_ident(inner))),
        ast::TypeIdent::Option(inner) => format!("Maybe {}", to_atom(generate_type_ident(inner))),
        ast::TypeIdent::Result(ok, err) => format!(
            "Result {} {}",
            to_atom(generate_type_ident(err)),
            to_atom(generate_type_ident(ok)),
        ),
        ast::TypeIdent::Map(key, value) => format!(
            "Dict {} {}",
            to_atom(generate_type_ident(key)),
            to_atom(generate_type_ident(value)),
        ),
        ast::TypeIdent::Tuple(tdef) => generate_tuple_def(tdef),
        ast::TypeIdent::UserDefined(ident) => ident.to_owned(),
    }
}

/// Generate elm code for a tuple definition.
fn generate_tuple_def(tdef: &ast::TupleDef) -> String {
    format!(
        "({})",
        tdef.elements().iter().map(generate_type_ident).join(", ")
    )
}

/// Generate elm code for an atomic type.
fn generate_atom(atom: &ast::AtomType) -> String {
    match atom {
        ast::AtomType::Empty => "()",
        ast::AtomType::Str => "String",
        ast::AtomType::I32 => "Int",
        ast::AtomType::U32 => "Int",
        ast::AtomType::U8 => "Int",
        ast::AtomType::F64 => "Float",
        ast::AtomType::Bool => "Bool",
        ast::AtomType::DateTime => "Time.Posix",
        ast::AtomType::Date => "Date.Date",
        ast::AtomType::Uuid => "String",
        ast::AtomType::Bytes => "String",
    }
    .to_owned()
}

/// Construct name for a field.
fn field_name(ident: &str) -> String {
    to_camel_case(ident)
}

fn generate_rest_api_client_helpers(spec: &ast::Spec) -> String {
    spec.iter()
        .filter_map(|spec_item| match spec_item {
            ast::SpecItem::StructDef(_) | ast::SpecItem::ServiceDef(_) => None,
            ast::SpecItem::EnumDef(edef) => Some(decoder_generation::generate_enum_helpers(edef)),
        })
        .join("")
}

fn generate_rest_api_clients(spec: &ast::Spec) -> String {
    generate_rest_api_client_helpers(spec);
    spec.iter()
        .filter_map(|spec_item| match spec_item {
            // No helpers for structs.
            ast::SpecItem::StructDef(_) | ast::SpecItem::EnumDef(_) => None,
            ast::SpecItem::ServiceDef(service) => Some(generate_rest_api_client(service)),
        })
        .join("")
}

fn generate_rest_api_client(spec: &ast::ServiceDef) -> String {
    todo!()
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
        write!(file.handle(), "module Api.{} exposing (..)", name)?;
        file.empty_lines(2)?;

        // TODO: write timestamp and info that this file is generated
        Ok(file)
    }

    pub fn generate_user_defined_types(spec :&Spec, outdir: &Path) -> Result<(), LibError> {

        let mut file = Self::make_file(spec, outdir, "Data")?;
        
        for spec_item in spec.iter() {
            match spec_item {
                ast::SpecItem::StructDef(sdef) => generate_struct_def(sdef, &mut file)?,
                ast::SpecItem::EnumDef(edef) => generate_enum_def(edef, &mut file)?,
                ast::SpecItem::ServiceDef(_) => {},
            };
        }

        Ok(())
    }

    pub fn generate_decoders(spec :&Spec, outdir: &Path) -> Result<(), LibError> {
        let mut file = Self::make_file(spec, outdir, "Decode")?;
        write!(file.handle(), "{}", decoder_generation::generate_type_decoders(spec))?;
        Ok(())
    }

    pub fn generate_encoders(spec :&Spec, outdir: &Path) -> Result<(), LibError> {
        let mut file = Self::make_file(spec, outdir, "Encode")?;
        write!(file.handle(), "{}", encoder_generation::generate_type_encoders(spec))?;
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
        //let generated_code = self.generate_spec(spec);

        //let mut outdir = PathBuf::from(&output);

        Ok(())
    }
}
