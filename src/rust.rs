use crate::ast;
use crate::util::IdentifyFirstLast;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug)]
pub struct RustVisitor {
    tokens: TokenStream,
}

pub trait Visitor {
    fn begin_enum(&mut self, ident: &str);
    fn finish_enum(&mut self, ident: &str);

    fn begin_field(&mut self, ident: &str);
    fn finish_field(&mut self, ident: &str);

    fn begin_struct(&mut self, ident: &str);
    fn finish_struct(&mut self, ident: &str);

    fn begin_struct_fields(&mut self);
    fn finish_struct_fields(&mut self);

    fn begin_tuple(&mut self);
    fn finish_tuple(&mut self);

    fn begin_tuple_component(&mut self);
    fn finish_tuple_component(&mut self);

    fn begin_variant(&mut self, ident: &str);
    fn finish_variant(&mut self, ident: &str);

    fn finish_last_field(&mut self, ident: &str) {
        self.finish_field(ident);
    }

    fn finish_last_tuple_component(&mut self) {
        self.finish_tuple_component();
    }

    fn finish_last_variant(&mut self, ident: &str) {
        self.finish_variant(ident);
    }

    fn visit_spec(&mut self, spec: &ast::Spec) {
        for item in spec.iter() {
            self.visit_spec_item(item)
        }
    }

    fn visit_spec_item(&mut self, spec_item: &ast::SpecItem) {
        match spec_item {
            ast::SpecItem::StructDef(sdef) => self.visit_struct_def(sdef),
            ast::SpecItem::EnumDef(edef) => self.visit_enum_def(edef),
        }
    }

    fn visit_enum_def(&mut self, edef: &ast::EnumDef) {
        self.begin_enum(&edef.name);
        for (_, is_last, variant_def) in edef.variants.iter().identify_first_last() {
            self.visit_variant_def(variant_def, is_last);
        }
        self.finish_enum(&edef.name);
    }

    fn visit_variant_def(&mut self, vdef: &ast::VariantDef, is_last: bool) {
        match vdef.variant_type {
            ast::VariantType::Simple => self.visit_variant_def_simple(&vdef.name, is_last),
            ast::VariantType::Tuple(ref tdef) => {
                self.visit_variant_def_tuple(&vdef.name, tdef, is_last)
            }
            ast::VariantType::Struct(ref fields) => {
                self.visit_variant_def_struct(&vdef.name, fields, is_last)
            }
        }
    }

    fn visit_variant_def_simple(&mut self, ident: &str, is_last: bool) {
        self.begin_variant(ident);
        if is_last {
            self.finish_last_variant(ident);
        } else {
            self.finish_variant(ident);
        }
    }

    fn visit_variant_def_tuple(&mut self, ident: &str, tdef: &ast::TupleDef, is_last: bool) {
        self.begin_variant(ident);
        self.visit_tuple_def(tdef);
        if is_last {
            self.finish_last_variant(ident);
        } else {
            self.finish_variant(ident);
        }
    }

    fn visit_variant_def_struct(&mut self, ident: &str, sdef: &ast::StructFields, is_last: bool) {
        self.begin_variant(ident);
        self.visit_struct_fields(sdef);
        if is_last {
            self.finish_last_variant(ident);
        } else {
            self.finish_variant(ident);
        }
    }

    fn visit_struct_fields(&mut self, fields: &ast::StructFields) {
        self.begin_struct_fields();
        for (_, is_last, field) in fields.iter().identify_first_last() {
            self.visit_field_node(field, is_last);
        }
        self.finish_struct_fields();
    }

    fn visit_field_node(&mut self, field: &ast::FieldNode, is_last: bool) {
        self.begin_field(&field.name);
        self.visit_type_ident(&field.type_ident);
        if is_last {
            self.finish_last_field(&field.name)
        } else {
            self.finish_field(&field.name)
        };
    }

    fn visit_struct_def(&mut self, sdef: &ast::StructDef) {
        self.begin_struct(&sdef.name);
        self.visit_struct_fields(&sdef.fields);
        self.finish_struct(&sdef.name);
    }

    fn visit_tuple_def(&mut self, tdef: &ast::TupleDef) {
        self.begin_tuple();
        for (_, is_last, component) in tdef.components().iter().identify_first_last() {
            self.visit_tuple_component(component, is_last);
        }
        self.finish_tuple();
    }

    fn visit_tuple_component(&mut self, component: &ast::TypeIdent, is_last: bool) {
        self.begin_tuple_component();
        self.visit_type_ident(component);
        if is_last {
            self.finish_last_tuple_component()
        } else {
            self.finish_tuple_component()
        };
    }

    fn visit_type_ident(&mut self, type_ident: &ast::TypeIdent) {
        match type_ident {
            ast::TypeIdent::BuiltIn(at) => self.visit_type_ident_builtin(at),
            ast::TypeIdent::List(inner) => self.visit_type_ident_list(inner),
            ast::TypeIdent::Tuple(tdef) => self.visit_type_ident_tuple(tdef),
            ast::TypeIdent::UserDefined(id) => self.visit_type_ident_user(id),
        }
    }

    fn visit_type_ident_builtin(&mut self, atom: &ast::AtomType);
    fn visit_type_ident_list(&mut self, inner: &ast::TypeIdent);
    fn visit_type_ident_tuple(&mut self, tdef: &ast::TupleDef) {
        self.visit_tuple_def(tdef);
    }
    fn visit_type_ident_user(&mut self, id: &str);
}

impl Visitor for RustVisitor {
    fn visit_type_ident_builtin(&mut self, atom: &ast::AtomType) {
        let atom_quoted = match atom {
            ast::AtomType::Str => quote!(String),
            ast::AtomType::I32 => quote!(i32),
            ast::AtomType::U32 => quote!(u32),
            ast::AtomType::U8 => quote!(u8),
            ast::AtomType::F64 => quote!(f64),
        };

        self.tokens.extend(atom_quoted);
    }

    fn visit_type_ident_list(&mut self, inner: &ast::TypeIdent) {
        self.tokens.extend(quote!(Vec));
        self.push_punct('<');
        self.visit_type_ident(inner);
        self.push_punct('>');
    }

    fn visit_type_ident_user(&mut self, id: &str) {
        let ident = quote::format_ident!("{}", id);
        self.tokens.extend(quote!(#ident));
    }

    fn begin_enum(&mut self, ident: &str) {
        let ident = quote::format_ident!("{}", ident);
        self.tokens.extend(quote!(
            #[derive(Debug, Deserialize, Serialize)]
            enum #ident
        ));
        self.push_punct('{');
    }

    fn finish_enum(&mut self, _ident: &str) {
        self.push_punct('}');
    }

    fn begin_variant(&mut self, ident: &str) {
        let ident = quote::format_ident!("{}", ident);

        self.tokens.extend(quote!(#ident));
    }

    fn finish_variant(&mut self, _ident: &str) {
        self.push_punct(',');
    }

    fn begin_struct(&mut self, ident: &str) {
        let ident = quote::format_ident!("{}", ident);

        self.tokens.extend(quote!(
            #[derive(Debug, Deserialize, Serialize)]
            struct #ident
        ));
    }

    fn finish_struct(&mut self, _ident: &str) {}

    fn begin_tuple(&mut self) {
        self.push_punct('(');
    }

    fn finish_tuple(&mut self) {
        self.push_punct(')');
    }

    fn begin_tuple_component(&mut self) {}

    fn finish_tuple_component(&mut self) {
        self.push_punct(',');
    }

    fn begin_struct_fields(&mut self) {
        self.push_punct('{');
    }

    fn finish_struct_fields(&mut self) {
        self.push_punct('}');
    }

    fn begin_field(&mut self, ident: &str) {
        let ident = quote::format_ident!("{}", ident);
        self.tokens.extend(quote!(
            #ident:
        ));
    }

    fn finish_field(&mut self, _ident: &str) {
        self.push_punct(',');
    }

    fn finish_last_field(&mut self, _ident: &str) {}

    fn finish_last_tuple_component(&mut self) {}

    fn finish_last_variant(&mut self, _ident: &str) {}
}

impl RustVisitor {
    pub fn new() -> Self {
        RustVisitor {
            tokens: TokenStream::new(),
        }
    }

    fn push_punct(&mut self, op: char) {
        proc_macro2::Punct::new(op, proc_macro2::Spacing::Alone).to_tokens(&mut self.tokens);
    }

    pub fn into_inner(self) -> TokenStream {
        self.tokens
    }
}

pub fn render(spec: &ast::Spec) -> TokenStream {
    let mut visitor = RustVisitor::new();
    visitor.visit_spec(spec);
    visitor.into_inner()
}
