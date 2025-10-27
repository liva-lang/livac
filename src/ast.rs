/// Abstract Syntax Tree for Liva language v0.9
use std::fmt;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Program {
    pub items: Vec<TopLevel>,
}

/// Type parameter with optional constraints
/// Example: `T`, `T: Comparable`, `T: Add + Sub`, `K: Hashable + Display`
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypeParameter {
    pub name: String,
    pub constraints: Vec<String>,  // Trait names that T must implement
}

impl TypeParameter {
    pub fn new(name: String) -> Self {
        TypeParameter {
            name,
            constraints: Vec::new(),
        }
    }

    pub fn with_constraint(name: String, constraint: String) -> Self {
        TypeParameter {
            name,
            constraints: vec![constraint],
        }
    }

    pub fn with_constraints(name: String, constraints: Vec<String>) -> Self {
        TypeParameter {
            name,
            constraints,
        }
    }
}

impl fmt::Display for TypeParameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.constraints.is_empty() {
            write!(f, ": {}", self.constraints.join(" + "))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TopLevel {
    Import(ImportDecl),
    UseRust(UseRustDecl),
    Type(TypeDecl),
    Class(ClassDecl),
    Function(FunctionDecl),
    Test(TestDecl),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImportDecl {
    pub imports: Vec<String>,      // List of imported symbols: ["add", "multiply"]
    pub source: String,             // Path to file: "./math.liva"
    pub is_wildcard: bool,          // true for `import *`
    pub alias: Option<String>,      // For wildcard: `import * as name`
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UseRustDecl {
    pub crate_name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypeDecl {
    pub name: String,
    pub type_params: Vec<TypeParameter>,  // Generic type parameters
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClassDecl {
    pub name: String,
    pub type_params: Vec<TypeParameter>,  // Generic type parameters
    pub base: Option<String>,
    pub members: Vec<Member>,
    #[serde(default)]
    pub needs_serde: bool,  // Phase 2: true if used with JSON.parse
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Member {
    Field(FieldDecl),
    Method(MethodDecl),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldDecl {
    pub name: String,
    pub visibility: Visibility,
    pub type_ref: Option<TypeRef>,
    pub init: Option<Expr>,
    #[serde(default)]
    pub is_optional: bool,  // true if field?: Type syntax
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MethodDecl {
    pub name: String,
    pub visibility: Visibility,
    pub type_params: Vec<TypeParameter>,  // Generic type parameters with constraints
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub body: Option<BlockStmt>,
    pub expr_body: Option<Expr>,
    pub is_async_inferred: bool,
    pub contains_fail: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FunctionDecl {
    pub name: String,
    pub type_params: Vec<TypeParameter>,  // Generic type parameters with constraints
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub body: Option<BlockStmt>,
    pub expr_body: Option<Expr>,
    pub is_async_inferred: bool,
    pub contains_fail: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Param {
    pub pattern: BindingPattern,  // Changed from `name: String` to support destructuring
    pub type_ref: Option<TypeRef>,
    pub default: Option<Expr>,
}

impl Param {
    /// Helper to get the name for simple identifier parameters (backward compatibility)
    pub fn name(&self) -> Option<&str> {
        match &self.pattern {
            BindingPattern::Identifier(name) => Some(name),
            _ => None,
        }
    }
    
    /// Check if this parameter uses destructuring
    pub fn is_destructuring(&self) -> bool {
        !self.pattern.is_simple()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TestDecl {
    pub name: String,
    pub body: BlockStmt,
}

#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize, serde::Deserialize)]
pub enum Visibility {
    Public,
    Private,
}

impl Visibility {
    pub fn from_name(name: &str) -> Self {
        if name.starts_with('_') {
            Visibility::Private
        } else {
            Visibility::Public
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TypeRef {
    Simple(String),
    Generic { base: String, args: Vec<TypeRef> },
    Array(Box<TypeRef>),
    Optional(Box<TypeRef>),
    Fallible(Box<TypeRef>),
    Tuple(Vec<TypeRef>),  // Tuple types: (int, string, bool)
}

impl TypeRef {
    pub fn to_rust_type(&self) -> String {
        match self {
            TypeRef::Simple(name) => match name.as_str() {
                "number" | "int" => "i32".to_string(),
                "float" => "f64".to_string(),
                "string" => "String".to_string(),
                "bytes" => "Vec<u8>".to_string(),
                "bool" => "bool".to_string(),
                "char" => "char".to_string(),
                "array" => "Vec<serde_json::Value>".to_string(),
                _ => name.clone(),
            },
            TypeRef::Generic { base, args } => {
                let args_str = args
                    .iter()
                    .map(|a| a.to_rust_type())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", base, args_str)
            }
            TypeRef::Array(inner) => format!("Vec<{}>", inner.to_rust_type()),
            TypeRef::Optional(inner) => format!("Option<{}>", inner.to_rust_type()),
            TypeRef::Fallible(inner) => format!("Result<{}, liva_rt::Error>", inner.to_rust_type()),
            TypeRef::Tuple(types) => {
                let types_str = types
                    .iter()
                    .map(|t| t.to_rust_type())
                    .collect::<Vec<_>>()
                    .join(", ");
                // Rust requires trailing comma for single-element tuples
                if types.len() == 1 {
                    format!("({},)", types_str)
                } else {
                    format!("({})", types_str)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Stmt {
    VarDecl(VarDecl),
    ConstDecl(ConstDecl),
    Assign(AssignStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Switch(SwitchStmt),
    TryCatch(TryCatchStmt),
    Throw(ThrowStmt),
    Fail(FailStmt),
    Return(ReturnStmt),
    Expr(ExprStmt),
    Block(BlockStmt),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BlockStmt {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VarDecl {
    pub bindings: Vec<VarBinding>,
    pub init: Expr,
    pub is_fallible: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VarBinding {
    pub pattern: BindingPattern,  // Changed from name: String to pattern
    pub type_ref: Option<TypeRef>,
    #[serde(skip)]
    pub span: Option<crate::span::Span>,
}

impl VarBinding {
    // Helper to get name for simple identifier patterns (backward compatibility)
    pub fn name(&self) -> Option<&str> {
        match &self.pattern {
            BindingPattern::Identifier(name) => Some(name),
            _ => None,
        }
    }
    
    // Helper to check if it's a simple identifier
    pub fn is_simple(&self) -> bool {
        matches!(self.pattern, BindingPattern::Identifier(_))
    }
}

/// Binding pattern for destructuring
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BindingPattern {
    Identifier(String),                    // Simple: x
    Object(ObjectPattern),                 // Object: {x, y}
    Array(ArrayPattern),                   // Array: [x, y, ...rest]
    Tuple(TuplePattern),                   // Tuple: (x, y, z)
}

impl BindingPattern {
    /// Check if this is a simple identifier pattern (not destructuring)
    pub fn is_simple(&self) -> bool {
        matches!(self, BindingPattern::Identifier(_))
    }
}

/// Object destructuring pattern: {name, age: userAge}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ObjectPattern {
    pub fields: Vec<ObjectPatternField>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ObjectPatternField {
    pub key: String,        // Field name in object
    pub binding: String,    // Variable name (may differ with rename syntax)
}

/// Array destructuring pattern: [first, second, ...rest]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArrayPattern {
    pub elements: Vec<Option<String>>,  // None = skip element
    pub rest: Option<String>,            // ...rest binding
}

/// Tuple destructuring pattern: (x, y, z)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TuplePattern {
    pub elements: Vec<String>,  // Variable names for each tuple element
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConstDecl {
    pub name: String,
    pub type_ref: Option<TypeRef>,
    pub init: Expr,
    #[serde(skip)]
    pub span: Option<crate::span::Span>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssignStmt {
    pub target: Expr,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: IfBody,
    pub else_branch: Option<IfBody>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum IfBody {
    Block(BlockStmt),
    Stmt(Box<Stmt>),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: BlockStmt,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ForStmt {
    pub var: String,
    pub iterable: Expr,
    #[serde(default)]
    pub policy: DataParallelPolicy,
    #[serde(default)]
    pub options: ForPolicyOptions,
    pub body: BlockStmt,
}

impl ForStmt {
    pub fn new(var: String, iterable: Expr, body: BlockStmt) -> Self {
        Self {
            var,
            iterable,
            policy: DataParallelPolicy::Seq,
            options: ForPolicyOptions::default(),
            body,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataParallelPolicy {
    Seq,
    Par,
    Vec,
    ParVec,
}

impl Default for DataParallelPolicy {
    fn default() -> Self {
        DataParallelPolicy::Seq
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct ForPolicyOptions {
    #[serde(default)]
    pub ordered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threads: Option<ThreadOption>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "simdWidth")]
    pub simd_width: Option<SimdWidthOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefetch: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduction: Option<ReductionOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<ScheduleOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detect: Option<DetectOption>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ThreadOption {
    #[serde(rename = "auto")]
    Auto,
    Count(i64),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum SimdWidthOption {
    #[serde(rename = "auto")]
    Auto,
    Width(i64),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReductionOption {
    Safe,
    Fast,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleOption {
    Static,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DetectOption {
    #[serde(rename = "auto")]
    Auto,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SwitchStmt {
    pub discriminant: Expr,
    pub cases: Vec<CaseClause>,
    pub default: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CaseClause {
    pub value: Expr,
    pub body: Vec<Stmt>,
}

// ===== Enhanced Pattern Matching (v0.9.5) =====

/// Switch expression for pattern matching that returns a value
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SwitchExpr {
    pub discriminant: Box<Expr>,
    pub arms: Vec<SwitchArm>,
}

/// A single arm in a switch expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SwitchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,  // Optional if condition
    pub body: SwitchBody,
}

/// Pattern for matching values
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Pattern {
    /// Literal value: 42, "hello", true
    Literal(Literal),
    /// Wildcard pattern: _
    Wildcard,
    /// Binding pattern: x (captures value)
    Binding(String),
    /// Range pattern: 1..10, 1..=10, ..10, 10..
    Range(RangePattern),
    /// Tuple pattern: (x, y, z)
    Tuple(Vec<Pattern>),
    /// Array pattern: [first, second, rest]
    Array(Vec<Pattern>),
    /// Or pattern: 1 | 2 | 3
    Or(Vec<Pattern>),
}

/// Range pattern for numeric matching
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RangePattern {
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
    pub inclusive: bool,  // true for ..=, false for ..
}

/// Body of a switch arm (can be expression or block)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SwitchBody {
    /// Single expression: => expr
    Expr(Box<Expr>),
    /// Block of statements: => { ... }
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TryCatchStmt {
    pub try_block: BlockStmt,
    pub catch_var: String,
    pub catch_block: BlockStmt,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ThrowStmt {
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FailStmt {
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReturnStmt {
    pub expr: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExprStmt {
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnOp,
        operand: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Call(CallExpr),
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
    Tuple(Vec<Expr>),  // Tuple literals: (10, 20, 30)
    Lambda(LambdaExpr),
    StringTemplate {
        parts: Vec<StringTemplatePart>,
    },
    Fail(Box<Expr>),
    MethodCall(MethodCallExpr),
    Switch(SwitchExpr),  // Enhanced pattern matching (v0.9.5)
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StringTemplatePart {
    Text(String),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    #[serde(default)]
    pub exec_policy: ExecPolicy,
    /// Optional type arguments for generic function calls (e.g., sum<int>(1, 2))
    #[serde(default)]
    pub type_args: Vec<TypeRef>,
}

impl CallExpr {
    pub fn new(callee: Expr, args: Vec<Expr>) -> Self {
        Self {
            callee: Box::new(callee),
            args,
            exec_policy: ExecPolicy::Normal,
            type_args: Vec::new(),
        }
    }

    pub fn with_type_args(callee: Expr, type_args: Vec<TypeRef>, args: Vec<Expr>) -> Self {
        Self {
            callee: Box::new(callee),
            args,
            exec_policy: ExecPolicy::Normal,
            type_args,
        }
    }
}

/// Method call expression for array methods and other instance methods
/// Example: arr.map(x => x * 2) or arr.par().map(x => x * 2)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MethodCallExpr {
    /// The object on which the method is called
    pub object: Box<Expr>,
    /// The method name (e.g., "map", "filter", "par", "vec")
    pub method: String,
    /// Arguments to the method
    pub args: Vec<Expr>,
    /// Execution policy adapter (if any)
    #[serde(default)]
    pub adapter: ArrayAdapter,
    /// Options for the adapter (threads, chunk, simdWidth, etc.)
    #[serde(default)]
    pub adapter_options: AdapterOptions,
}

impl MethodCallExpr {
    pub fn new(object: Expr, method: String, args: Vec<Expr>) -> Self {
        Self {
            object: Box::new(object),
            method,
            args,
            adapter: ArrayAdapter::Seq,
            adapter_options: AdapterOptions::default(),
        }
    }
}

/// Array execution adapters for performance policies
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArrayAdapter {
    /// Sequential execution (default)
    Seq,
    /// Parallel execution (.par())
    Par,
    /// Vectorized execution (.vec())
    Vec,
    /// Parallel + Vectorized (.parvec())
    ParVec,
}

impl Default for ArrayAdapter {
    fn default() -> Self {
        ArrayAdapter::Seq
    }
}

/// Options for array adapters
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct AdapterOptions {
    /// Number of threads for parallel execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threads: Option<i32>,
    /// Chunk size for work distribution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk: Option<i32>,
    /// SIMD width for vectorized execution
    #[serde(skip_serializing_if = "Option::is_none", rename = "simdWidth")]
    pub simd_width: Option<i32>,
    /// Whether to preserve order in results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordered: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecPolicy {
    Normal,
    Async,
    Par,
    TaskAsync,
    TaskPar,
    FireAsync,
    FirePar,
}

impl Default for ExecPolicy {
    fn default() -> Self {
        ExecPolicy::Normal
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LambdaExpr {
    pub is_move: bool,
    pub params: Vec<LambdaParam>,
    pub return_type: Option<TypeRef>,
    pub body: LambdaBody,
    #[serde(default)]
    pub captures: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LambdaParam {
    pub pattern: BindingPattern,
    pub type_ref: Option<TypeRef>,
}

impl LambdaParam {
    /// Get the parameter name if it's a simple identifier pattern
    pub fn name(&self) -> Option<&str> {
        match &self.pattern {
            BindingPattern::Identifier(name) => Some(name),
            _ => None,
        }
    }

    /// Check if this parameter uses destructuring
    pub fn is_destructuring(&self) -> bool {
        !matches!(self.pattern, BindingPattern::Identifier(_))
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LambdaBody {
    Expr(Box<Expr>),
    Block(BlockStmt),
}

#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize, serde::Deserialize)]
pub enum ConcurrencyMode {
    Async,
    Parallel,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize, serde::Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Range,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Lt => write!(f, "<"),
            BinOp::Le => write!(f, "<="),
            BinOp::Gt => write!(f, ">"),
            BinOp::Ge => write!(f, ">="),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Range => write!(f, ".."),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize, serde::Deserialize)]
pub enum UnOp {
    Neg,
    Not,
    Await,
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnOp::Neg => write!(f, "-"),
            UnOp::Not => write!(f, "!"),
            UnOp::Await => write!(f, "await"),
        }
    }
}

impl fmt::Display for ImportDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_wildcard {
            if let Some(alias) = &self.alias {
                write!(f, "import * as {} from \"{}\"", alias, self.source)
            } else {
                write!(f, "import * from \"{}\"", self.source)
            }
        } else {
            write!(f, "import {{ {} }} from \"{}\"", self.imports.join(", "), self.source)
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Literal(lit) => write!(f, "{:?}", lit),
            Pattern::Wildcard => write!(f, "_"),
            Pattern::Binding(name) => write!(f, "{}", name),
            Pattern::Range(range) => write!(f, "{}", range),
            Pattern::Tuple(patterns) => {
                write!(f, "(")?;
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", pat)?;
                }
                write!(f, ")")
            }
            Pattern::Array(patterns) => {
                write!(f, "[")?;
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", pat)?;
                }
                write!(f, "]")
            }
            Pattern::Or(patterns) => {
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", pat)?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for RangePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.start, &self.end, self.inclusive) {
            (Some(start), Some(end), true) => write!(f, "{:?}..={:?}", start, end),
            (Some(start), Some(end), false) => write!(f, "{:?}..{:?}", start, end),
            (Some(start), None, _) => write!(f, "{:?}..", start),
            (None, Some(end), true) => write!(f, "..={:?}", end),
            (None, Some(end), false) => write!(f, "..{:?}", end),
            (None, None, _) => write!(f, ".."),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_from_name_variants() {
        assert_eq!(Visibility::from_name("public_name"), Visibility::Public);
        assert_eq!(Visibility::from_name("_private"), Visibility::Private);
    }

    #[test]
    fn test_type_ref_to_rust_type_variants() {
        let simple = TypeRef::Simple("number".into());
        assert_eq!(simple.to_rust_type(), "i32");

        let generic = TypeRef::Generic {
            base: "Result".into(),
            args: vec![
                TypeRef::Simple("string".into()),
                TypeRef::Simple("Error".into()),
            ],
        };
        assert_eq!(generic.to_rust_type(), "Result<String, Error>");

        let array = TypeRef::Array(Box::new(TypeRef::Simple("bool".into())));
        assert_eq!(array.to_rust_type(), "Vec<bool>");

        let optional = TypeRef::Optional(Box::new(TypeRef::Simple("float".into())));
        assert_eq!(optional.to_rust_type(), "Option<f64>");
    }
}
