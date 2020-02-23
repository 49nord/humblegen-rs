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

impl EnumDef {
    pub fn complex_variants(&self) -> impl Iterator<Item = &VariantDef> {
        self.variants.iter().filter(|v| !v.is_simple())
    }

    pub fn simple_variants(&self) -> impl Iterator<Item = &VariantDef> {
        self.variants.iter().filter(|v| v.is_simple())
    }
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

impl VariantDef {
    fn is_simple(&self) -> bool {
        if let VariantType::Simple = self.variant_type {
            true
        } else {
            false
        }
    }
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
