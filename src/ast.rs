use crate::util::IdentifyFirstLast;

#[derive(Debug)]
pub struct Spec(pub Vec<SpecItem>);

impl Spec {
    pub fn iter(&self) -> impl Iterator<Item = &SpecItem> {
        self.0.iter()
    }
}

#[derive(Debug)]
pub enum SpecItem {
    StructDef(StructDef),
    EnumDef(EnumDef),
}

#[derive(Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: StructFields,
}

#[derive(Debug)]
pub struct StructFields(pub Vec<FieldNode>);

impl StructFields {
    pub fn iter(&self) -> impl Iterator<Item = &FieldNode> {
        self.0.iter()
    }
}

#[derive(Debug)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<VariantDef>,
}

#[derive(Debug)]
pub struct VariantDef {
    pub name: String,
    pub variant_type: VariantType,
}

#[derive(Debug)]
pub enum VariantType {
    Simple,
    Tuple(TupleDef),
    Struct(StructFields),
}

#[derive(Debug)]
pub struct FieldNode {
    pub name: String,
    pub type_ident: TypeIdent,
}

#[derive(Debug)]
pub enum TypeIdent {
    BuiltIn(AtomType),
    List(Box<TypeIdent>),
    Option(Box<TypeIdent>),
    Map(Box<TypeIdent>, Box<TypeIdent>),
    Tuple(TupleDef),
    UserDefined(String),
}

#[derive(Debug)]
pub enum AtomType {
    Str,
    I32,
    U32,
    U8,
    F64,
}

#[derive(Debug)]
pub struct TupleDef(pub Vec<TypeIdent>);

impl TupleDef {
    pub fn components(&self) -> &Vec<TypeIdent> {
        &self.0
    }
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

    fn visit_spec(&mut self, spec: &Spec) {
        for item in spec.iter() {
            self.visit_spec_item(item)
        }
    }

    fn visit_spec_item(&mut self, spec_item: &SpecItem) {
        match spec_item {
            SpecItem::StructDef(sdef) => self.visit_struct_def(sdef),
            SpecItem::EnumDef(edef) => self.visit_enum_def(edef),
        }
    }

    fn visit_enum_def(&mut self, edef: &EnumDef) {
        self.begin_enum(&edef.name);
        for (_, is_last, variant_def) in edef.variants.iter().identify_first_last() {
            self.visit_variant_def(variant_def, is_last);
        }
        self.finish_enum(&edef.name);
    }

    fn visit_variant_def(&mut self, vdef: &VariantDef, is_last: bool) {
        match vdef.variant_type {
            VariantType::Simple => self.visit_variant_def_simple(&vdef.name, is_last),
            VariantType::Tuple(ref tdef) => self.visit_variant_def_tuple(&vdef.name, tdef, is_last),
            VariantType::Struct(ref fields) => {
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

    fn visit_variant_def_tuple(&mut self, ident: &str, tdef: &TupleDef, is_last: bool) {
        self.begin_variant(ident);
        self.visit_tuple_def(tdef);
        if is_last {
            self.finish_last_variant(ident);
        } else {
            self.finish_variant(ident);
        }
    }

    fn visit_variant_def_struct(&mut self, ident: &str, sdef: &StructFields, is_last: bool) {
        self.begin_variant(ident);
        self.visit_struct_fields(sdef);
        if is_last {
            self.finish_last_variant(ident);
        } else {
            self.finish_variant(ident);
        }
    }

    fn visit_struct_fields(&mut self, fields: &StructFields) {
        self.begin_struct_fields();
        for (_, is_last, field) in fields.iter().identify_first_last() {
            self.visit_field_node(field, is_last);
        }
        self.finish_struct_fields();
    }

    fn visit_field_node(&mut self, field: &FieldNode, is_last: bool) {
        self.begin_field(&field.name);
        self.visit_type_ident(&field.type_ident);
        if is_last {
            self.finish_last_field(&field.name)
        } else {
            self.finish_field(&field.name)
        };
    }

    fn visit_struct_def(&mut self, sdef: &StructDef) {
        self.begin_struct(&sdef.name);
        self.visit_struct_fields(&sdef.fields);
        self.finish_struct(&sdef.name);
    }

    fn visit_tuple_def(&mut self, tdef: &TupleDef) {
        self.begin_tuple();
        for (_, is_last, component) in tdef.components().iter().identify_first_last() {
            self.visit_tuple_component(component, is_last);
        }
        self.finish_tuple();
    }

    fn visit_tuple_component(&mut self, component: &TypeIdent, is_last: bool) {
        self.begin_tuple_component();
        self.visit_type_ident(component);
        if is_last {
            self.finish_last_tuple_component()
        } else {
            self.finish_tuple_component()
        };
    }

    fn visit_type_ident(&mut self, type_ident: &TypeIdent) {
        match type_ident {
            TypeIdent::BuiltIn(at) => self.visit_type_ident_builtin(at),
            TypeIdent::List(inner) => self.visit_type_ident_list(inner),
            TypeIdent::Option(inner) => self.visit_type_ident_option(inner),
            TypeIdent::Map(key, value) => self.visit_type_ident_map(key, value),
            TypeIdent::Tuple(tdef) => self.visit_type_ident_tuple(tdef),
            TypeIdent::UserDefined(id) => self.visit_type_ident_user(id),
        }
    }

    fn visit_type_ident_builtin(&mut self, atom: &AtomType);
    fn visit_type_ident_list(&mut self, inner: &TypeIdent);
    fn visit_type_ident_option(&mut self, inner: &TypeIdent);
    fn visit_type_ident_map(&mut self, key: &TypeIdent, value: &TypeIdent);
    fn visit_type_ident_tuple(&mut self, tdef: &TupleDef) {
        self.visit_tuple_def(tdef);
    }
    fn visit_type_ident_user(&mut self, id: &str);
}
