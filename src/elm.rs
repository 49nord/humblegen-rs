use crate::ast;

pub struct ElmTypesVisitor {
    output: String,
}

impl ElmTypesVisitor {
    pub fn new() -> ElmTypesVisitor {
        ElmTypesVisitor {
            output: "module Main exposing (..)\n\n".to_owned(),
        }
    }

    pub fn into_inner(self) -> String {
        self.output
    }
}

impl ast::Visitor for ElmTypesVisitor {
    fn begin_enum(&mut self, ident: &str) {
        self.output.push_str("type ");
        self.output.push_str(ident);
        self.output.push_str(" = ");
    }

    fn finish_enum(&mut self, _ident: &str) {
        self.output.push_str("\n");
    }

    fn begin_field(&mut self, ident: &str) {
        self.output.push_str(ident);
        self.output.push_str(": ");
    }

    fn finish_field(&mut self, _ident: &str) {
        self.output.push_str(", ");
    }

    fn finish_last_field(&mut self, _ident: &str) {}

    fn begin_struct(&mut self, ident: &str) {
        self.output.push_str("type alias ");
        self.output.push_str(ident);
        self.output.push_str(" = ");
    }

    fn finish_struct(&mut self, _ident: &str) {
        self.output.push_str("\n\n");
    }

    fn begin_struct_fields(&mut self) {
        self.output.push_str("{ ");
    }

    fn finish_struct_fields(&mut self) {
        self.output.push_str(" }");
    }

    fn begin_tuple(&mut self) {
        self.output.push_str("( ");
    }

    fn finish_tuple(&mut self) {
        self.output.push_str(" )");
    }

    fn begin_tuple_component(&mut self) {}

    fn finish_tuple_component(&mut self) {
        self.output.push_str(", ");
    }

    fn finish_last_tuple_component(&mut self) {}

    fn begin_variant(&mut self, ident: &str) {
        self.output.push_str(ident);
    }

    fn finish_variant(&mut self, _ident: &str) {
        self.output.push_str(" | ");
    }

    fn finish_last_variant(&mut self, _ident: &str) {}

    fn visit_type_ident_builtin(&mut self, atom: &ast::AtomType) {
        self.output.push_str(match atom {
            ast::AtomType::Str => "String",
            ast::AtomType::I32 => "Int",
            ast::AtomType::U32 => "Int",
            ast::AtomType::U8 => "Int",
            ast::AtomType::F64 => "Float",
        })
    }

    fn visit_type_ident_list(&mut self, inner: &ast::TypeIdent) {
        self.output.push_str("(List ");
        self.visit_type_ident(inner);
        self.output.push_str(")");
    }

    fn visit_type_ident_option(&mut self, inner: &ast::TypeIdent) {
        self.output.push_str("(Maybe ");
        self.visit_type_ident(inner);
        self.output.push_str(")");
    }

    fn visit_type_ident_map(&mut self, key: &ast::TypeIdent, value: &ast::TypeIdent) {
        self.output.push_str("(Dict ");
        self.visit_type_ident(key);
        self.output.push_str(" ");
        self.visit_type_ident(value);
        self.output.push_str(")");
    }

    fn visit_type_ident_user(&mut self, id: &str) {
        self.output.push_str(id);
    }
}

pub fn render(spec: &ast::Spec) -> String {
    let mut visitor = ElmTypesVisitor::new();
    ast::Visitor::visit_spec(&mut visitor, spec);
    visitor.into_inner()
}
