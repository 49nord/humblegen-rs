use super::{generate_doc_comment, to_atom, field_name, IndentWriter};
use crate::{LibError, ast};
use itertools::Itertools;

// TODO: Elm does not allow documentation on members, so the docs need to be converted to markdown
//       lists instead. This is true for `type alias` struct fields as well as enum variants.

pub(crate) fn generate_struct_def(def: &ast::StructDef, file :&mut IndentWriter) -> Result<(), LibError> {
    file.kill_indent();

    write!(file.start_line()?, "{doc_comment}\ntype alias {name} =",
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

    write!(file.start_line()?, "{doc_comment}\ntype {name}",
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