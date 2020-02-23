use crate::ast;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug)]
pub struct RustVisitor {
    tokens: TokenStream,
}

impl ast::Visitor for RustVisitor {
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
    ast::Visitor::visit_spec(&mut visitor, spec);
    visitor.into_inner()
}
