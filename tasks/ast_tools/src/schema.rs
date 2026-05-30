use oxc_index::IndexVec;

oxc_index::define_index_type! {
    pub struct TypeId = u32;
}

impl TypeId {
    pub const DUMMY: Self = Self::from_raw_unchecked(0);
}

pub struct Schema {
    pub types: IndexVec<TypeId, AstType>,
}

#[derive(Debug)]
pub enum AstType {
    Struct(AstStruct),
    Enum(AstEnum),
    Box(AstBox),
    Vec(AstVec),
    Option(AstOption),
    Primitive(AstPrimitive),
}

impl AstType {
    pub fn name(&self) -> &str {
        match self {
            AstType::Struct(ast) => &ast.name,
            AstType::Enum(ast) => &ast.name,
            AstType::Box(ast) => &ast.name,
            AstType::Vec(ast) => &ast.name,
            AstType::Option(ast) => &ast.name,
            AstType::Primitive(ast) => ast.name,
        }
    }
}

#[derive(Debug)]
pub struct AstStruct {
    pub type_id: TypeId,
    pub name: String,
    pub fields: Vec<AstStructField>,
}

#[derive(Debug)]
pub struct AstStructField {
    pub type_id: TypeId,
    pub name: String,
}

#[derive(Debug)]
pub struct AstEnum {
    pub type_id: TypeId,
    pub name: String,
    pub variants: Vec<AstEnumVariant>,
}

#[derive(Debug)]
pub struct AstEnumVariant {
    pub type_id: Option<TypeId>,
    pub name: String,
}

#[derive(Debug)]
pub struct AstBox {
    pub type_id: TypeId,
    pub name: String,
    pub inner_type_id: TypeId,
}

#[derive(Debug)]
pub struct AstVec {
    pub type_id: TypeId,
    pub name: String,
    pub inner_type_id: TypeId,
}

#[derive(Debug)]
pub struct AstOption {
    pub type_id: TypeId,
    pub name: String,
    pub inner_type_id: TypeId,
}

#[derive(Debug)]
#[allow(unused)]
pub struct AstNode {
    pub type_id: TypeId,
    pub name: String,
}

#[derive(Debug)]
pub struct AstPrimitive {
    pub type_id: TypeId,
    pub name: &'static str,
}
