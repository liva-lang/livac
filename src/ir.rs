//! Intermediate representation for Liva → Rust lowering.
//!
//! The goal of this IR is to decouple high-level AST constructs from
//! Rust-specific code generation concerns.  It carries enough typing
//! and effect information to decide how to expand concurrency features,
//! intrinsic helpers, and formatting utilities before hitting `quote!`.

use crate::ast;

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub items: Vec<Item>,
    pub extern_crates: Vec<ExternCrate>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            extern_crates: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternCrate {
    pub crate_name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
    Test(Test),
    Unsupported(ast::TopLevel),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: Type,
    pub body: Block,
    pub async_kind: AsyncKind,
    pub visibility: Visibility,
    pub contains_fail: bool,
    pub source: ast::FunctionDecl,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    pub name: String,
    pub body: Block,
    pub source: ast::TestDecl,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        ty: Option<Type>,
        value: Expr,
    },
    Const {
        name: String,
        ty: Option<Type>,
        value: Expr,
    },
    Assign {
        target: Expr,
        value: Expr,
    },
    Return(Option<Expr>),
    Throw(Expr),
    Expr(Expr),
    If {
        condition: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        condition: Expr,
        body: Block,
    },
    For {
        var: String,
        iterable: Expr,
        policy: DataParallelPolicy,
        options: ForPolicyOptions,
        body: Block,
    },
    Block(Block),
    TryCatch {
        try_block: Block,
        error_var: String,
        catch_block: Block,
    },
    Switch {
        discriminant: Expr,
        cases: Vec<SwitchCase>,
        default: Option<Vec<Stmt>>,
    },
    Unsupported(ast::Stmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub value: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Await(Box<Expr>),
    AsyncCall {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    ParallelCall {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    TaskCall {
        mode: ConcurrencyMode,
        callee: String,
        args: Vec<Expr>,
    },
    FireCall {
        mode: ConcurrencyMode,
        callee: String,
        args: Vec<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    StringTemplate(Vec<TemplatePart>),
    Member {
        object: Box<Expr>,
        property: String,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    ObjectLiteral(Vec<(String, Expr)>),
    StructLiteral {
        type_name: String,
        fields: Vec<(String, Expr)>,
    },
    ArrayLiteral(Vec<Expr>),
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },
    Lambda(LambdaExpr),
    Unsupported(ast::Expr),
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub exec_policy: ExecPolicy,
    pub callee_name: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ExecPolicy {
    Normal,
    Async,
    Par,
    TaskAsync,
    TaskPar,
    FireAsync,
    FirePar,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Text(String),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unit,
    Number,
    Float,
    Bool,
    String,
    Bytes,
    Char,
    Array(Box<Type>),
    Optional(Box<Type>),
    Custom(String),
    Inferred,
}

impl Type {
    pub fn from_ast(node: &Option<ast::TypeRef>) -> Self {
        match node {
            Some(ast::TypeRef::Simple(name)) => match name.as_str() {
                "number" => Type::Number,
                "float" => Type::Float,
                "bool" => Type::Bool,
                "string" => Type::String,
                "bytes" => Type::Bytes,
                "char" => Type::Char,
                "array" => Type::Array(Box::new(Type::Custom("serde_json::Value".into()))),
                other => Type::Custom(other.to_string()),
            },
            Some(ast::TypeRef::Array(inner)) => {
                Type::Array(Box::new(Type::from_ast(&Some((**inner).clone()))))
            }
            Some(ast::TypeRef::Optional(inner)) => {
                Type::Optional(Box::new(Type::from_ast(&Some((**inner).clone()))))
            }
            Some(ast::TypeRef::Generic { base, .. }) => Type::Custom(base.clone()),
            Some(ast::TypeRef::Fallible(inner)) => {
                let inner_type = (**inner).clone();
                let inner_rust = inner_type.to_rust_type();
                Type::Custom(format!("Result<{}, liva_rt::Error>", inner_rust))
            }
            None => Type::Inferred,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AsyncKind {
    NotAsync,
    Async,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConcurrencyMode {
    Async,
    Parallel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataParallelPolicy {
    Seq,
    Par,
    Vec,
    Boost,
}

impl Default for DataParallelPolicy {
    fn default() -> Self {
        DataParallelPolicy::Seq
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ForPolicyOptions {
    pub ordered: bool,
    pub chunk: Option<i64>,
    pub threads: Option<ThreadOption>,
    pub simd_width: Option<SimdWidthOption>,
    pub prefetch: Option<i64>,
    pub reduction: Option<ReductionOption>,
    pub schedule: Option<ScheduleOption>,
    pub detect: Option<DetectOption>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThreadOption {
    Auto,
    Count(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimdWidthOption {
    Auto,
    Width(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReductionOption {
    Safe,
    Fast,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleOption {
    Static,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DetectOption {
    Auto,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

impl From<ast::Visibility> for Visibility {
    fn from(value: ast::Visibility) -> Self {
        match value {
            ast::Visibility::Public => Visibility::Public,
            ast::Visibility::Protected => Visibility::Protected,
            ast::Visibility::Private => Visibility::Private,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LambdaExpr {
    pub is_move: bool,
    pub params: Vec<LambdaParam>,
    pub return_type: Option<String>,
    pub body: LambdaBody,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LambdaParam {
    pub name: String,
    pub type_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LambdaBody {
    Expr(Box<Expr>),
    Block(Vec<Stmt>),
}
