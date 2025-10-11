# 📗 Liva v0.6 — Gramática (EBNF) y AST

> Incluye funciones de una línea, operadores `and`/`or`/`not` (y `&&`/`||`/`!), niveles de acceso `_` / `__`, y reglas de deducción automática de `async` y tipos `number`/`float`.

---

## 1) Léxico (tokens)

```ebnf
letter        = 'A'..'Z' | 'a'..'z' | '_' ;
digit         = '0'..'9' ;
hexdigit      = digit | 'a'..'f' | 'A'..'F' ;

IdentStart    = letter ;
IdentCont     = letter | digit ;
Identifier    = IdentStart , { IdentCont } ;

PrivateIdent  = "__" , Identifier ;              (* private real *)
ProtectedIdent= "_"  , [ "_" ] , Identifier ;    (* "_" protected; "__" ya cubierto por Private *)

IntLiteral    = digit , { digit | '_' } ;
FloatLiteral  = digit , { digit | '_' } , '.' , digit , { digit | '_' } ;
CharLiteral   = "'" , ? any UTF-8 char except ' and \ ? , "'" ;
StringLiteral = '"' , { ? any char or escape ? } , '"' ;

BoolLiteral   = "true" | "false" ;

WS            = { ' ' | '\t' | '\r' | '\n' } ;
LineComment   = "//" , { ? not EOL ? } ;
BlockComment  = "/*" , { ? not "*/" ? } , "*/" ;

(* Operadores *)
OpAssign      = "=" ;
OpPlus        = "+" ;
OpMinus       = "-" ;
OpMul         = "*" ;
OpDiv         = "/" ;
OpMod         = "%" ;

OpLT          = "<" ;
OpLE          = "<=" ;
OpGT          = ">" ;
OpGE          = ">=" ;
OpEQ          = "==" ;
OpNE          = "!=" ;

OpAnd         = "and" | "&&" ;
OpOr          = "or"  | "||" ;
OpNot         = "not" | "!" ;

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
Arrow         = "->" ;
```

**Palabras reservadas**:  
`let, const, import, use, rust, type, test, if, else, while, for, in, switch, case, default, throw, try, catch, return, async, parallel, task, fire, true, false`

---

## 2) Tipos

```ebnf
Type
  = SimpleType
  | GenericType
  | ArrayType
  | OptionalType
  ;

SimpleType
  = "number" | "float" | "bool" | "char" | "string" | "bytes"
  | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
  | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
  | "f32" | "f64"
  | Identifier              (* tipos de usuario *)
  ;

GenericType
  = Identifier , "<" , Type , { "," , Type } , ">"
  | "Option" , "<" , Type , ">"
  | "Result" , "<" , Type , "," , Type , ">"
  ;

ArrayType   = Type , "[" , "]" ;
OptionalType= Type , QMark ;
```

**Alias semánticos (fase semántica)**  
`number := i32` ; `float := f64`

---

## 3) Unidades de compilación

```ebnf
CompilationUnit
  = { ImportDecl | UseRustDecl | TypeDecl | ClassDecl | FunctionDecl | TestDecl } ;

ImportDecl   = "import" , Identifier , [ "as" , Identifier ] ;
UseRustDecl  = "use" , "rust" , StringLiteral , [ "as" , Identifier ] ;

TypeDecl     = "type" , Identifier , TypeBody ;
TypeBody     = LBrace , { FieldDecl | MethodDecl } , RBrace ;

ClassDecl    = Identifier , LBrace , { FieldDecl | MethodDecl } , RBrace ;

FieldDecl    = VisibilityIdent , ":" , Type , [ OpAssign , Expression ] ;
VisibilityIdent
  = PrivateIdent
  | ProtectedIdent
  | Identifier                   (* público por defecto *)
  ;

MethodDecl
  = VisibilityIdent , LParen , [ ParamList ] , RParen , Block
  | VisibilityIdent , LParen , [ ParamList ] , RParen , Colon , Type , OpAssign , Expression  (* una línea *)
  ;

ParamList    = Param , { "," , Param } ;
Param        = Identifier , [ Colon , Type ] , [ OpAssign , Expression ] ;

FunctionDecl
  = Identifier , LParen , [ ParamList ] , RParen , Block
  | Identifier , LParen , [ ParamList ] , RParen , Colon , Type , OpAssign , Expression  (* una línea *)
  ;

TestDecl     = "test" , StringLiteral , Block ;
```

**Notas**: sin `class` ni `fun`. El contexto determina clase (si hay métodos) o struct (solo campos).

---

## 4) Sentencias y bloques

```ebnf
Block        = LBrace , { Statement } , RBrace ;

Statement
  = VarDecl | ConstDecl | AssignStmt | IfStmt | WhileStmt | ForStmt
  | SwitchStmt | TryCatchStmt | ThrowStmt | ReturnStmt
  | ExprStmt
  ;

VarDecl      = "let" , Identifier , [ Colon , Type ] , [ OpAssign , Expression ] ;
ConstDecl    = "const" , Identifier , OpAssign , Expression ;

AssignStmt   = LValue , OpAssign , Expression ;
LValue       = Primary , { (Dot , Identifier) | (LBracket , Expression , RBracket) } ;

IfStmt       = "if" , "("? , Expression , ")?" , Block , [ "else" , Block ] ;
WhileStmt    = "while" , "("? , Expression , ")?" , Block ;

ForStmt      = "for" , Identifier , "in" , Expression , Block ;

SwitchStmt   = "switch" , Expression , LBrace , { CaseClause } , [ DefaultClause ] , RBrace ;
CaseClause   = "case" , Expression , ":" , { Statement } ;
DefaultClause= "default" , ":" , { Statement } ;

TryCatchStmt = "try" , Block , "catch" , "(" , Identifier , ")" , Block ;
ThrowStmt    = "throw" , Expression , ";"? ;

ReturnStmt   = "return" , [ Expression ] , ";"? ;
ExprStmt     = Expression , ";"? ;
```

---

## 5) Expresiones y precedencia

**Precedencia (alta → baja)**  
1) Llamadas / indexación / acceso miembro  
2) Unarios: `-`, `not`/`!`  
3) `*` `/` `%`  
4) `+` `-`  
5) Comparación: `< <= > >= == !=`  
6) Lógico `and` / `&&`  
7) Lógico `or`  / `||`  
8) Ternario `? :` (opcional)  
9) Asignación `=`

```ebnf
Expression
  = ConditionalExpr ;

ConditionalExpr
  = OrExpr , [ "?" , Expression , ":" , Expression ] ;

OrExpr
  = AndExpr , { (OpOr) , AndExpr } ;

AndExpr
  = CmpExpr , { (OpAnd) , CmpExpr } ;

CmpExpr
  = AddExpr , { (OpLT | OpLE | OpGT | OpGE | OpEQ | OpNE) , AddExpr } ;

AddExpr
  = MulExpr , { (OpPlus | OpMinus) , MulExpr } ;

MulExpr
  = UnaryExpr , { (OpMul | OpDiv | OpMod) , UnaryExpr } ;

UnaryExpr
  = (OpMinus | OpNot) , UnaryExpr
  | PostfixExpr ;

PostfixExpr
  = Primary , { CallSuffix | IndexSuffix | MemberSuffix } ;

CallSuffix   = LParen , [ ArgList ] , RParen ;
IndexSuffix  = LBracket , Expression , RBracket ;
MemberSuffix = Dot , Identifier ;

ArgList      = Expression , { "," , Expression } ;

Primary
  = Literal
  | "(" , Expression , ")"
  | ObjectLiteral
  | ArrayLiteral
  | AsyncCallExpr
  | ParallelCallExpr
  | TaskCallExpr
  | FireCallExpr
  | Identifier
  ;

Literal
  = IntLiteral | FloatLiteral | StringLiteral | CharLiteral | BoolLiteral ;

ObjectLiteral = LBrace , [ ObjField , { "," , ObjField } ] , RBrace ;
ObjField      = Identifier , ":" , Expression ;

ArrayLiteral  = LBracket , [ Expression , { "," , Expression } ] , RBracket ;

AsyncCallExpr    = "async" , Identifier , LParen , [ ArgList ] , RParen ;
ParallelCallExpr = "parallel" , Identifier , LParen , [ ArgList ] , RParen ;
TaskCallExpr
  = "task" , "async"    , Identifier , LParen , [ ArgList ] , RParen
  | "task" , "parallel" , Identifier , LParen , [ ArgList ] , RParen
  ;
FireCallExpr
  = "fire" , "async"    , Identifier , LParen , [ ArgList ] , RParen
  | "fire" , "parallel" , Identifier , LParen , [ ArgList ] , RParen
  ;
```

> `and`/`or`/`not` ≡ `&&`/`||`/`!` con la misma precedencia y cortocircuito.

---

## 6) Reglas semánticas clave

### 6.1 Encapsulación `_` / `__`
- Sin prefijo ⇒ **public** → `pub`
- `_name` ⇒ **protected** (clase + subclases) → `pub(super)`
- `__name` ⇒ **private** (solo clase) → *(sin `pub`)*
- Subclases se emiten en un submódulo para habilitar `pub(super)` como *protected real*.

### 6.2 Deducción automática de `async`
Una definición se marca `async fn` si su cuerpo contiene:
- `AsyncCallExpr`,
- llamada a función ya `async`,
- o llamada a API Rust `async` conocida (via `use rust`).

### 6.3 Concurrencia en la llamada
- `async f()` → tarea en runtime (Tokio). Lazy **await** al primer uso.
- `parallel f()` → tarea en hilo SO. Lazy **join** al primer uso.
- `task async|parallel` → devuelve handle; se **await** explícito.
- `fire async|parallel` → no guarda handle; **sin warnings**.

**Warnings**: resultado no usado de `async/parallel` que devuelve valor (excepto si `fire` o `_ = …`).

### 6.4 Tipos `number`/`float` y tipos Rust
- `number` ⇢ `i32`; `float` ⇢ `f64`.
- Se permiten tipos nativos de Rust (`u64`, `f32`, …).
- Sin promociones implícitas peligrosas; los casts son explícitos.

---

## 7) Esquema de AST (TypeScript-like)

```ts
interface Program { kind: "Program"; body: TopLevel[]; }
type TopLevel =
  | ImportDecl | UseRustDecl | TypeDecl | ClassDecl | FunctionDecl | TestDecl;

interface ImportDecl { kind: "ImportDecl"; name: Identifier; alias?: Identifier; }
interface UseRustDecl { kind: "UseRustDecl"; crate: StringLiteral; alias?: Identifier; }

interface TypeDecl { kind: "TypeDecl"; name: Identifier; members: (FieldDecl|MethodDecl)[]; }
interface ClassDecl {
  kind: "ClassDecl";
  name: Identifier;
  members: (FieldDecl|MethodDecl)[];
  isStructOnly: boolean;
}

type Visibility = "public" | "protected" | "private";

interface FieldDecl {
  kind: "FieldDecl";
  name: Identifier;
  visibility: Visibility;
  type?: TypeRef;
  init?: Expr;
}

interface MethodDecl {
  kind: "MethodDecl";
  name: Identifier;
  visibility: Visibility;
  params: Param[];
  returnType?: TypeRef;
  body?: BlockStmt;
  exprBody?: Expr;
  isOneLiner: boolean;
  isAsyncInferred: boolean;
}

interface FunctionDecl {
  kind: "FunctionDecl";
  name: Identifier;
  params: Param[];
  returnType?: TypeRef;
  body?: BlockStmt;
  exprBody?: Expr;
  isOneLiner: boolean;
  isAsyncInferred: boolean;
}

interface Param { name: Identifier; type?: TypeRef; defaultValue?: Expr; }

type TypeRef = SimpleTypeRef | GenericTypeRef | ArrayTypeRef | OptionalTypeRef;
interface SimpleTypeRef { kind: "SimpleTypeRef"; name: string; }
interface GenericTypeRef { kind: "GenericTypeRef"; base: string; args: TypeRef[]; }
interface ArrayTypeRef   { kind: "ArrayTypeRef"; elem: TypeRef; }
interface OptionalTypeRef{ kind: "OptionalTypeRef"; inner: TypeRef; }

type Stmt =
  | VarDecl | ConstDecl | AssignStmt | IfStmt | WhileStmt | ForStmt | SwitchStmt
  | TryCatchStmt | ThrowStmt | ReturnStmt | ExprStmt | BlockStmt ;

interface BlockStmt { kind: "BlockStmt"; stmts: Stmt[]; }
interface VarDecl   { kind: "VarDecl"; name: Identifier; type?: TypeRef; init?: Expr; isConst: false; }
interface ConstDecl { kind: "ConstDecl"; name: Identifier; init: Expr; isConst: true; }
interface AssignStmt{ kind: "AssignStmt"; target: LValue; value: Expr; }

interface IfStmt    { kind: "IfStmt"; test: Expr; consequent: BlockStmt; alternate?: BlockStmt; }
interface WhileStmt { kind: "WhileStmt"; test: Expr; body: BlockStmt; }
interface ForStmt   { kind: "ForStmt"; iterVar: Identifier; iterable: Expr; body: BlockStmt; }
interface SwitchStmt{ kind: "SwitchStmt"; discriminant: Expr; cases: { test: Expr; body: Stmt[]; }[]; defaultBody?: Stmt[]; }
interface TryCatchStmt { kind: "TryCatchStmt"; tryBlock: BlockStmt; catchIdent: Identifier; catchBlock: BlockStmt; }
interface ThrowStmt { kind: "ThrowStmt"; expr: Expr; }
interface ReturnStmt{ kind: "ReturnStmt"; expr?: Expr; }
interface ExprStmt  { kind: "ExprStmt"; expr: Expr; }

type Expr =
  | LiteralExpr | IdExpr | UnaryExpr | BinaryExpr | TernaryExpr
  | CallExpr | MemberExpr | IndexExpr
  | ObjectLiteralExpr | ArrayLiteralExpr
  | AsyncCallExpr | ParallelCallExpr | TaskCallExpr | FireCallExpr ;

interface LiteralExpr { kind: "LiteralExpr"; value: number|string|boolean; litType: "int"|"float"|"string"|"char"|"bool"; }
interface IdExpr      { kind: "IdExpr"; name: string; }
interface UnaryExpr   { kind: "UnaryExpr"; op: "neg"|"not"; argument: Expr; }
interface BinaryExpr  { kind: "BinaryExpr"; op: "+"|"-"|"*"|"/"|"%"|"<"|"<="|">"|">="|"=="|"!="|"and"|"or"; left: Expr; right: Expr; }
interface TernaryExpr { kind: "TernaryExpr"; test: Expr; consequent: Expr; alternate: Expr; }

interface MemberExpr  { kind: "MemberExpr"; object: Expr; property: Identifier; }
interface IndexExpr   { kind: "IndexExpr"; object: Expr; index: Expr; }
interface CallExpr    { kind: "CallExpr"; callee: Expr; args: Expr[]; }

interface ObjectLiteralExpr { kind: "ObjectLiteralExpr"; fields: { key: string; value: Expr; }[]; }
interface ArrayLiteralExpr  { kind: "ArrayLiteralExpr"; elements: Expr[]; }

interface AsyncCallExpr    { kind: "AsyncCallExpr"; callee: Identifier; args: Expr[]; }
interface ParallelCallExpr { kind: "ParallelCallExpr"; callee: Identifier; args: Expr[]; }
interface TaskCallExpr     { kind: "TaskCallExpr"; mode: "async"|"parallel"; callee: Identifier; args: Expr[]; }
interface FireCallExpr     { kind: "FireCallExpr"; mode: "async"|"parallel"; callee: Identifier; args: Expr[]; }
```

---

## 8) Algoritmos semánticos (pseudocódigo)

### 8.1 Visibilidad
```pseudo
function visibilityOf(name):
  if name startsWith "__": return "private"
  else if name startsWith "_": return "protected"
  else: return "public"
```

### 8.2 Detección automática de async
```pseudo
function isAsyncFunction(def):
  return containsAsync(def.body)

function containsAsync(node):
  if node is AsyncCallExpr: return true
  if node is CallExpr to known-async API: return true
  if node is CallExpr to FunctionDecl f and f.isAsyncInferred: return true
  for child in node.children: if containsAsync(child): return true
  return false
```

### 8.3 Tipos `number`/`float`
```pseudo
alias number := i32
alias float  := f64

typeCheck(binop):
  if (lhs: number, rhs: float) or viceversa: error "explicit cast required"
  else follow Rust typing
```

### 8.4 Operadores lógicos
- Normalizar `&&`/`||`/`!` a `and`/`or`/`not` en el AST.
- Cortocircuito garantizado en codegen.

### 8.5 Lazy await/join
```pseudo
// En el desugaring, al primer uso de un valor devuelto por async/parallel:
if value.origin == "async spawn" and value.notResolved:
  inject ".await" (and error handling)

if value.origin == "thread spawn" and value.notResolved:
  inject ".join()" (and error handling)
```

### 8.6 Warnings de resultado no usado
```pseudo
if call is async/parallel and returns non-void and handle is unused
   and not prefixed with "fire" and not assigned to "_" :
  warn "result of concurrent call is unused (did you mean 'fire ...' or '_ = ...'?)"
```

---

## 9) Reglas de desugaring → Rust (resumen)

- **Clases/structs**: `Identifier { ... }` → `pub struct` + `impl`  
  - Público: `pub` ; Protegido: `pub(super)` ; Privado: *(sin `pub`)*  
  - Subclases en `mod <type>_mod::subtypes` para que vean `pub(super)`.

- **Funciones**: one-liner → `fn name(..) -> T { expr }` ; si `isAsyncInferred` → `async fn`.

- **Concurrencia**:  
  - `async f(args)` → `let h = tokio::spawn(f(args));` (+ lazy `.await`)  
  - `parallel f(args)` → `let h = std::thread::spawn(|| f(args));` (+ lazy `.join()`)  
  - `task ...` → igual, pero el *handle* se expone y solo resuelve con `await`.  
  - `fire ...` → `spawn(...)` sin guardar *handle*.

- **Operadores**: `and`/`or`/`not` → `&&`/`||`/`!`.

- **Templates de string**: `$"Hola {x}"` → `format!("Hola {}", x)`.

---

## 10) Casos de prueba recomendados (parser)

- `sum(a,b): number = a+b`
- `if not a and b or c {}`
- `Persona { nombre:string, _edad:number, __dni:string }`
- `fetch(){ let r = async http.get(url); return r.text() }` (auto-async)
- `fire async send()` ; `let t = task parallel calc(x); let y = await t`
- `let c: u64 = 0`
```

---

_Archivo generado para uso directo en el repositorio del compilador (`livac`)._
