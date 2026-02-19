# 📗 Grammar and AST Reference

This document provides the formal grammar specification (EBNF) and Abstract Syntax Tree (AST) structure for Liva v0.6.

> **Note:** This is a technical reference for compiler developers and language designers. For user-facing syntax documentation, see [Syntax Overview](../language-reference/syntax-overview.md).

## Source Documentation

This is a consolidated view of the formal grammar. For the complete original specification, see:
- [`docs_old/Liva_v0.6_EBNF_AST.md`](../../docs_old/Liva_v0.6_EBNF_AST.md) - Complete EBNF grammar and AST definitions

## 1. Lexical Structure (Tokens)

### Identifiers

```ebnf
letter        = 'A'..'Z' | 'a'..'z' | '_' ;
digit         = '0'..'9' ;
hexdigit      = digit | 'a'..'f' | 'A'..'F' ;

IdentStart    = letter ;
IdentCont     = letter | digit ;
Identifier    = IdentStart , { IdentCont } ;

(* Visibility modifiers *)
PrivateIdent  = "__" , Identifier ;              (* private *)
ProtectedIdent= "_"  , Identifier ;              (* protected *)
```

### Literals

```ebnf
IntLiteral    = digit , { digit | '_' } ;
FloatLiteral  = digit , { digit | '_' } , '.' , digit , { digit | '_' } ;
CharLiteral   = "'" , ? any UTF-8 char except ' and \ ? , "'" ;
StringLiteral = '"' , { ? any char or escape ? } , '"' ;
BoolLiteral   = "true" | "false" ;
```

### Operators

```ebnf
(* Assignment *)
OpAssign      = "=" ;

(* Arithmetic *)
OpPlus        = "+" ;
OpMinus       = "-" ;
OpMul         = "*" ;
OpDiv         = "/" ;
OpMod         = "%" ;

(* Comparison *)
OpLT          = "<" ;
OpLE          = "<=" ;
OpGT          = ">" ;
OpGE          = ">=" ;
OpEQ          = "==" ;
OpNE          = "!=" ;

(* Logical - Word form *)
OpAnd         = "and" ;
OpOr          = "or" ;
OpNot         = "not" ;

(* Logical - Symbol form *)
OpAndSym      = "&&" ;
OpOrSym       = "||" ;
OpNotSym      = "!" ;
```

### Delimiters

```ebnf
Dot           = "." ;
Comma         = "," ;
Colon         = ":" ;
QMark         = "?" ;
LParen        = "(" ;
RParen        = ")" ;
LBrace        = "{" ;
RBrace        = "}" ;
LBracket      = "[" ;
RBracket      = "]" ;
Range         = ".." ;
Arrow         = "=>" ;
```

### Keywords

```
let, const, import, use, rust, type, test,
if, else, while, for, in, switch, case, default,
throw, try, catch, return,
async, par, parallel, task, fire, await,
fail,
true, false, this, constructor
```

## 2. Type System Grammar

```ebnf
Type
  = SimpleType
  | GenericType
  | ArrayType
  | FunctionType
  ;

SimpleType
  = "number" | "float" | "bool" | "char" | "string" | "bytes"
  | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
  | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
  | "f32" | "f64"
  | Identifier              (* user-defined types *)
  ;

GenericType
  = Identifier , "<" , Type , { "," , Type } , ">"
  ;

ArrayType
  = "[" , Type , "]"
  | Type , "[" , "]"
  ;

FunctionType
  = "(" , [ Type , { "," , Type } ] , ")" , "->" , Type
  ;
```

**Type Aliases:**
- `number` → `i32`
- `float` → `f64`

## 3. Program Structure

```ebnf
Program
  = { TopLevelItem }
  ;

TopLevelItem
  = FunctionDecl
  | ClassDecl
  | ConstDecl
  | ImportDecl
  | UseDecl
  ;
```

## 4. Function Declarations

```ebnf
FunctionDecl
  = Identifier , "(" , [ ParamList ] , ")" , [ ":" , Type ] , FunctionBody
  ;

ParamList
  = Param , { "," , Param }
  ;

Param
  = Identifier , [ ":" , Type ]
  ;

FunctionBody
  = "=>" , Expression                    (* one-liner *)
  | "{" , { Statement } , "}"            (* block *)
  ;
```

**Examples:**
```liva
// One-liner
add(a, b) => a + b
square(x: number): number => x * x

// Block
greet(name: string) {
  print($"Hello, {name}!")
}
```

## 5. Class Declarations

```ebnf
ClassDecl
  = Identifier , "{" , { ClassMember } , "}"
  ;

ClassMember
  = FieldDecl
  | MethodDecl
  | ConstructorDecl
  ;

FieldDecl
  = [ VisibilityModifier ] , Identifier , ":" , Type
  ;

MethodDecl
  = [ VisibilityModifier ] , FunctionDecl
  ;

ConstructorDecl
  = "constructor" , "(" , [ ParamList ] , ")" , "{" , { Statement } , "}"
  ;

VisibilityModifier
  = ""           (* public - default *)
  | "_"          (* protected *)
  | "__"         (* private *)
  ;
```

## 6. Statements

```ebnf
Statement
  = VariableDecl
  | ConstDecl
  | Assignment
  | IfStatement
  | WhileStatement
  | ForStatement
  | SwitchStatement
  | ReturnStatement
  | FailStatement
  | ExpressionStatement
  ;

VariableDecl
  = "let" , IdentifierList , [ ":" , Type ] , "=" , Expression
  ;

IdentifierList
  = Identifier , [ "," , Identifier ]      (* for error binding *)
  ;

ConstDecl
  = "const" , Identifier , [ ":" , Type ] , "=" , Expression
  ;

Assignment
  = Identifier , "=" , Expression
  ;

IfStatement
  = "if" , Expression , "{" , { Statement } , "}"
  , [ "else" , ( IfStatement | "{" , { Statement } , "}" ) ]
  ;

WhileStatement
  = "while" , Expression , "{" , { Statement } , "}"
  ;

ForStatement
  = "for" , [ ForModifier ] , Identifier , "in" , Expression
  , [ ForPolicy ] , "{" , { Statement } , "}"
  ;

ForModifier
  = "par"           (* parallel *)
  | "parvec"        (* SIMD *)
  ;

ForPolicy
  = "with" , PolicyOption , { PolicyOption }
  ;

PolicyOption
  = "chunk" , IntLiteral
  | "threads" , IntLiteral
  | "simdWidth" , IntLiteral
  | "ordered"
  | "unordered"
  ;

SwitchStatement
  = "switch" , Expression , "{"
  , { "case" , Expression , ":" , { Statement } }
  , [ "default" , ":" , { Statement } ]
  , "}"
  ;

ReturnStatement
  = "return" , [ Expression ]
  ;

FailStatement
  = "fail" , Expression
  ;

ExpressionStatement
  = Expression
  ;
```

## 7. Expressions

```ebnf
Expression
  = TernaryExpr
  ;

TernaryExpr
  = LogicalOrExpr , [ "?" , Expression , ":" , Expression ]
  ;

LogicalOrExpr
  = LogicalAndExpr , { ( "or" | "||" ) , LogicalAndExpr }
  ;

LogicalAndExpr
  = EqualityExpr , { ( "and" | "&&" ) , EqualityExpr }
  ;

EqualityExpr
  = RelationalExpr , { ( "==" | "!=" ) , RelationalExpr }
  ;

RelationalExpr
  = AdditiveExpr , { ( "<" | "<=" | ">" | ">=" ) , AdditiveExpr }
  ;

AdditiveExpr
  = MultiplicativeExpr , { ( "+" | "-" ) , MultiplicativeExpr }
  ;

MultiplicativeExpr
  = UnaryExpr , { ( "*" | "/" | "%" ) , UnaryExpr }
  ;

UnaryExpr
  = ( "not" | "!" | "-" ) , UnaryExpr
  | ConcurrencyExpr
  ;

ConcurrencyExpr
  = [ ConcurrencyModifier ] , PostfixExpr
  ;

ConcurrencyModifier
  = "async"
  | "par" | "parallel"
  | "fire" , ( "async" | "par" )
  | "task" , ( "async" | "par" )
  | "await"
  ;

PostfixExpr
  = PrimaryExpr , { PostfixOp }
  ;

PostfixOp
  = "." , Identifier                    (* field access *)
  | "(" , [ ArgumentList ] , ")"        (* function call *)
  | "[" , Expression , "]"              (* array index *)
  ;

PrimaryExpr
  = Literal
  | Identifier
  | "this"
  | "(" , Expression , ")"
  | ArrayLiteral
  | ObjectLiteral
  | StringTemplate
  ;

Literal
  = IntLiteral
  | FloatLiteral
  | CharLiteral
  | StringLiteral
  | BoolLiteral
  ;

ArrayLiteral
  = "[" , [ Expression , { "," , Expression } ] , "]"
  ;

ObjectLiteral
  = "{" , [ FieldInit , { "," , FieldInit } ] , "}"
  ;

FieldInit
  = Identifier , ":" , Expression
  ;

StringTemplate
  = "$" , '"' , { TemplateChar | TemplateExpr } , '"'
  ;

TemplateExpr
  = "{" , Expression , "}"
  ;

ArgumentList
  = Expression , { "," , Expression }
  ;
```

## 8. AST Node Types

### Top-Level Nodes

```rust
pub enum TopLevelItem {
    Function(Function),
    Class(Class),
    Const(Const),
    Import(Import),
    Use(Use),
}

pub struct Program {
    pub items: Vec<TopLevelItem>,
}
```

### Function AST

```rust
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: FunctionBody,
    pub is_async: bool,           // inferred by semantic pass
    pub visibility: Visibility,
}

pub struct Param {
    pub name: String,
    pub type_: Option<Type>,
}

pub enum FunctionBody {
    Expression(Box<Expr>),        // one-liner: => expr
    Block(Vec<Stmt>),             // block: { stmts }
}
```

### Class AST

```rust
pub struct Class {
    pub name: String,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub constructor: Option<Constructor>,
}

pub struct Field {
    pub name: String,
    pub type_: Type,
    pub visibility: Visibility,
}

pub struct Constructor {
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

pub enum Visibility {
    Public,           // default, no prefix
    Private,          // _name
}
```

### Statement AST

```rust
pub enum Stmt {
    VarDecl {
        names: Vec<String>,       // For error binding: let x, err = ...
        type_: Option<Type>,
        init: Expr,
    },
    ConstDecl {
        name: String,
        type_: Option<Type>,
        value: Expr,
    },
    Assignment {
        target: String,
        value: Expr,
    },
    If {
        cond: Expr,
        then: Vec<Stmt>,
        else_: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    For {
        modifier: Option<ForModifier>,
        var: String,
        iter: Expr,
        policy: Option<ForPolicy>,
        body: Vec<Stmt>,
    },
    Switch {
        expr: Expr,
        cases: Vec<SwitchCase>,
        default: Option<Vec<Stmt>>,
    },
    Return(Option<Expr>),
    Fail(Expr),
    Expression(Expr),
}

pub enum ForModifier {
    Par,              // Parallel threads
    ParVec,           // SIMD vectorization
}

pub struct ForPolicy {
    pub chunk_size: Option<usize>,
    pub thread_count: Option<usize>,
    pub simd_width: Option<usize>,
    pub ordered: bool,
}
```

### Expression AST

```rust
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    This,
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnOp,
        operand: Box<Expr>,
    },
    Ternary {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    StringTemplate {
        parts: Vec<Templatepart>,
    },
    Async(Box<Expr>),
    Par(Box<Expr>),
    Task {
        mode: TaskMode,
        expr: Box<Expr>,
    },
    Fire {
        mode: TaskMode,
        expr: Box<Expr>,
    },
    Await(Box<Expr>),
}

pub enum TaskMode {
    Async,
    Parallel,
}

pub enum Literal {
    Int(i64),
    Float(f64),
    Char(char),
    String(String),
    Bool(bool),
}

pub enum BinOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    // Logical
    And, Or,
}

pub enum UnOp {
    Neg,              // -
    Not,              // not, !
}
```

## 9. Operator Precedence

From lowest to highest precedence:

1. **Ternary** - `? :`
2. **Logical OR** - `or`, `||`
3. **Logical AND** - `and`, `&&`
4. **Equality** - `==`, `!=`
5. **Relational** - `<`, `<=`, `>`, `>=`
6. **Additive** - `+`, `-`
7. **Multiplicative** - `*`, `/`, `%`
8. **Unary** - `not`, `!`, `-`
9. **Concurrency** - `async`, `par`, `task`, `fire`, `await`
10. **Postfix** - `.`, `()`, `[]`
11. **Primary** - Literals, identifiers, `this`, `()`

**Associativity:**
- Ternary: Right-associative
- Binary operators: Left-associative
- Unary operators: Right-associative

## 10. Semantic Rules

### Auto-Async Inference

A function is automatically marked as `async` if its body contains:
- Direct `async` call: `let x = async foo()`
- Task async: `let t = task async foo()`
- Await expression: `let x = await task`

### Error Binding

When using error binding (`let value, err = ...`):
- If function is fallible: `err` contains error string, `value` contains result
- If function is not fallible: `err` is empty string `""`, `value` contains result
- Error type is always `String`

### Type Inference

- Variables: Inferred from initialization expression
- Functions: Return type inferred from return statements
- `number` resolves to `i32`
- `float` resolves to `f64`

## Complete Reference

For the complete, authoritative grammar specification with all edge cases and detailed AST node definitions, see:

- **[`docs_old/Liva_v0.6_EBNF_AST.md`](../../docs_old/Liva_v0.6_EBNF_AST.md)**

This document contains:
- Full EBNF grammar with all productions
- Complete AST node definitions in Rust
- Detailed semantic rules
- Edge cases and special handling
- Examples for every grammar rule

## See Also

- **[Parser Implementation](parser.md)** - How the grammar is implemented
- **[Syntax Overview](../language-reference/syntax-overview.md)** - User-facing syntax guide
- **[Desugaring Rules](desugaring.md)** - AST transformations
- **[Architecture](architecture.md)** - Compiler pipeline overview

---

**Note:** This grammar is for Liva v0.6. Future versions may extend or modify the grammar.
