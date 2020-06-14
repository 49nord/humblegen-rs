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
    /// `service` definition
    ServiceDef(ServiceDef),
}

impl SpecItem {
    /// The service definition if `self` is a `ServiceDef`.
    pub fn service_def(&self) -> Option<&ServiceDef> {
        match self {
            SpecItem::ServiceDef(s) => Some(s),
            _ => None,
        }
    }
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

impl VariantType {
    /// Returns the StructFields if the variant type is Struct.
    pub fn struct_fields(&self) -> Option<&StructFields> {
        match self {
            VariantType::Struct(f) => Some(f),
            _ => None,
        }
    }
    pub fn struct_fields_mut(&mut self) -> Option<&mut StructFields> {
        match self {
            VariantType::Struct(f) => Some(f),
            _ => None,
        }
    }
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

/// A service definition.
/// Example:
/// ```text
/// /// Monster management service.
/// service MonsterApi {
///    GET  /monsters -> vec[Monster],
///    POST /monsters -> MonsterData -> result[Monster][MonsterError]
/// }
/// ```
#[derive(Debug)]
pub struct ServiceDef {
    /// The service name. (example: `MonsterApi`)
    pub name: String,
    /// The doc comment of the service. (example: `Monster management service.`)
    pub doc_comment: Option<String>,
    /// The service endpoints. (example: see struct `ServiceEndpoint`)
    pub endpoints: Vec<ServiceEndpoint>,
}

/// An endpoint within a service definition.
/// Example:
/// ```text
/// /// Retrieve all monsters.
/// GET /monsters -> vec[Monster],
/// ```
#[derive(Debug)]
pub struct ServiceEndpoint {
    /// The doc comment of the endpoint. (example: `Retrieve all monsters.`)
    pub doc_comment: Option<String>,
    /// The route of the endpoint. (example: see struct `ServiceRoute`)
    pub route: ServiceRoute,
}

/// And endpoint's route.
/// Example:
/// ```text
/// GET  /monsters?{GetMonstersQuery} -> vec[Monster],
/// POST /monsters -> MonsterData -> result[Monster][MonsterError]
/// ```
#[derive(Debug)]
pub enum ServiceRoute {
    /// A GET endpoint.
    Get {
        /// The route components. See struct `ServiceRouteComponent`.
        components: Vec<ServiceRouteComponent>,
        /// The query type, if specified. (example: `GetMonstersQuery`)
        query: Option<TypeIdent>,
        /// The route return type.
        ret: TypeIdent,
    },
    /// A POST endpoint.
    Post {
        /// The route components. See struct `ServiceRouteComponent`.
        components: Vec<ServiceRouteComponent>,
        /// The query type, if specified. (example: `GetMonstersQuery`)
        query: Option<TypeIdent>,
        /// The POST body type. (example: `MonsterData`)
        body: TypeIdent,
        /// The route return type.
        ret: TypeIdent,
    },
    /// A DELETE endpoint
    Delete {
        /// The route components. See struct `ServiceRouteComponent`.
        components: Vec<ServiceRouteComponent>,
        /// The query type, if specified. (example: `GetMonstersQuery`)
        query: Option<TypeIdent>,
        /// The route return type.
        ret: TypeIdent,
    },
    /// A PUT endpoint.
    Put {
        /// The route components. See struct `ServiceRouteComponent`.
        components: Vec<ServiceRouteComponent>,
        /// The query type, if specified. (example: `GetMonstersQuery`)
        query: Option<TypeIdent>,
        /// The POST body type. (example: `MonsterData`)
        body: TypeIdent,
        /// The route return type.
        ret: TypeIdent,
    },
    /// A PATCH endpoint.
    Patch {
        /// The route components. See struct `ServiceRouteComponent`.
        components: Vec<ServiceRouteComponent>,
        /// The query type, if specified. (example: `GetMonstersQuery`)
        query: Option<TypeIdent>,
        /// The POST body type. (example: `MonsterData`)
        body: TypeIdent,
        /// The route return type.
        ret: TypeIdent,
    },
}

impl ServiceRoute {
    /// The route components. See struct `ServiceRouteComponent`.
    pub fn components(&self) -> &Vec<ServiceRouteComponent> {
        match self {
            ServiceRoute::Get { components, .. } => components,
            ServiceRoute::Delete { components, .. } => components,
            ServiceRoute::Post { components, .. } => components,
            ServiceRoute::Put { components, .. } => components,
            ServiceRoute::Patch { components, .. } => components,
        }
    }

    /// The query type, if specified. (example: `GetMonstersQuery`)
    pub fn query(&self) -> &Option<TypeIdent> {
        match self {
            ServiceRoute::Get { query, .. } => query,
            ServiceRoute::Delete { query, .. } => query,
            ServiceRoute::Post { query, .. } => query,
            ServiceRoute::Put { query, .. } => query,
            ServiceRoute::Patch { query, .. } => query,
        }
    }

    /// The return type.
    pub fn return_type(&self) -> &TypeIdent {
        match self {
            ServiceRoute::Get { ret, .. } => ret,
            ServiceRoute::Delete { ret, .. } => ret,
            ServiceRoute::Post { ret, .. } => ret,
            ServiceRoute::Put { ret, .. } => ret,
            ServiceRoute::Patch { ret, .. } => ret,
        }
    }

    pub fn http_method_as_str(&self) -> &'static str {
        match self {
            ServiceRoute::Get { .. } => "GET",
            ServiceRoute::Delete { .. } => "DELETE",
            ServiceRoute::Post { .. } => "POST",
            ServiceRoute::Put { .. } => "PUT",
            ServiceRoute::Patch { .. } => "PATCH",
        }
    }
}

/// A component of a `ServiceRoute`.
/// Example:
/// ```text
/// GET /monsters/{id: str}
/// ```
/// results in
/// - `Literal("monsters")
/// - `Variable(FieldDefPair{ name: "id", type_ident: TypeIdent::BuiltIn(AtomType::Str) })`
///
#[derive(Debug)]
pub enum ServiceRouteComponent {
    Literal(String),
    Variable(FieldDefPair),
}

/// A field node (field definition inside struct).
#[derive(Debug, Clone)]
pub struct FieldNode {
    pub pair: FieldDefPair,
    /// Documentation comment.
    pub doc_comment: Option<String>,
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

impl TypeIdent {
    pub fn user_defined(&self) -> Option<&String> {
        match self {
            TypeIdent::UserDefined(s) => Some(s),
            _ => None,
        }
    }
}

/// An atomic type.
#[derive(Debug, Clone, Copy)]
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
    pub fn elements(&self) -> &Vec<TypeIdent> {
        &self.0
    }
}
