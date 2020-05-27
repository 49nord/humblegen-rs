//! Humble language abstract syntax tree

/// A spec node.
///
/// A spec is the top-level item in humble.
#[derive(Debug)]
pub struct Spec(pub Vec<SpecItem>);

impl Spec {
    /// Iterate over items in spec.
    pub fn iter(&self) -> impl Iterator<Item = &SpecItem> {
        self.0.iter()
    }

    /// Mutable iterator over items in spec.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SpecItem> {
        self.0.iter_mut()
    }
}

/// A Spec item node.
#[derive(Debug)]
pub enum SpecItem {
    /// `struct` definition.
    StructDef(StructDef),
    /// `enum` definition.
    EnumDef(EnumDef),
}

/// A struct definition.
#[derive(Debug)]
pub struct StructDef {
    /// Name of the struct.
    pub name: String,
    /// Fields of the struct.
    pub fields: StructFields,
    /// Documentation comment.
    pub doc_comment: Option<String>,
}

/// Container of struct fields.
#[derive(Debug)]
pub struct StructFields(pub Vec<FieldNode>);

impl StructFields {
    /// Iterate over all contained fields.
    pub fn iter(&self) -> impl Iterator<Item = &FieldNode> {
        self.0.iter()
    }
}

/// Enum definition.
#[derive(Debug)]
pub struct EnumDef {
    /// Name of the `enum`.
    pub name: String,
    /// Container of variants.
    pub variants: Vec<VariantDef>,
    /// Documentation comment.
    pub doc_comment: Option<String>,
}

impl EnumDef {
    /// Iterate over all complex variants.
    ///
    /// Complex variants are all that are not simple.
    pub fn complex_variants(&self) -> impl Iterator<Item = &VariantDef> {
        self.variants.iter().filter(|v| !v.is_simple())
    }

    /// Iterate over all simple variants.
    ///
    /// C-style enum variants are considered simple.
    pub fn simple_variants(&self) -> impl Iterator<Item = &VariantDef> {
        self.variants.iter().filter(|v| v.is_simple())
    }
}

/// A variant definition.
#[derive(Debug)]
pub struct VariantDef {
    /// Name of the variant.
    pub name: String,
    /// Type of the variant.
    pub variant_type: VariantType,
    /// Documentation comment.
    pub doc_comment: Option<String>,
}

/// An (enum-)variant type.
#[derive(Debug)]
pub enum VariantType {
    /// Simple C-style variant.
    Simple,
    /// Tuple variant.
    Tuple(TupleDef),
    /// Struct variant.
    Struct(StructFields),
    /// Newype variant.
    Newtype(TypeIdent),
}

impl VariantDef {
    /// Returns whether or not a variant is simple.
    fn is_simple(&self) -> bool {
        if let VariantType::Simple = self.variant_type {
            true
        } else {
            false
        }
    }
}

/// A field node (field definition inside struct).
#[derive(Debug, Clone)]
pub struct FieldNode {
    pub pair: FieldDefPair,
    /// Documentation comment.
    pub doc_comment: Option<String>,
}

impl TypeIdent {
    pub fn user_defined(&self) -> Option<&String> {
        match self {
            TypeIdent::UserDefined(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldDefPair {
    /// Name of the field.
    pub name: String,
    /// Type of the field.
    pub type_ident: TypeIdent,
}

impl FieldDefPair {
    /// Whether the given FieldDefPair is a humblespec embed
    /// (only valid if it is within a struct's `FieldNode`).
    pub fn is_embed(&self) -> bool {
        self.type_ident
            .user_defined()
            .map(|ident_name| &self.name == ident_name)
            .unwrap_or(false)
    }
}

/// A type identifier.
#[derive(Debug, Clone)]
pub enum TypeIdent {
    /// Built-in (atomic) type.
    BuiltIn(AtomType),
    /// `list[T]`
    List(Box<TypeIdent>),
    /// `option[T]`
    Option(Box<TypeIdent>),
    /// `result[T]`
    Result(Box<TypeIdent>, Box<TypeIdent>),
    /// `map[t][u]`
    Map(Box<TypeIdent>, Box<TypeIdent>),
    /// Tuple type.
    Tuple(TupleDef),
    /// Type defined in humble file.
    UserDefined(String),
}

/// An atomic type.
#[derive(Debug, Clone)]
pub enum AtomType {
    /// Empty type
    Empty,
    /// String.
    Str,
    /// Signed 32-bit integer.
    I32,
    /// Unsigned 32-bit integer.
    U32,
    /// Unsigned 8-bit integer.
    U8,
    /// 64-bit IEEE floating-point number.
    F64,
    /// Boolean value.
    Bool,
    /// Timestamp in UTC time.
    DateTime,
    /// Date value.
    Date,
}

/// A tuple definition.
#[derive(Debug, Clone)]
pub struct TupleDef(pub Vec<TypeIdent>);

impl TupleDef {
    /// Get a reference to the tuple components.
    pub fn components(&self) -> &Vec<TypeIdent> {
        &self.0
    }
}
