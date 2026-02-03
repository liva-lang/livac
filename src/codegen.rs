use crate::ast::*;
use crate::desugaring::DesugarContext;
use crate::error::{CompilerError, Result, SemanticErrorInfo};
use crate::ir;
use crate::traits::TraitRegistry;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

/// Capitalize the first letter of a string
fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Information about a pending Task (async/par) that hasn't been awaited yet
#[derive(Debug, Clone)]
struct TaskInfo {
    /// Whether this is error binding (two variables: value, err)
    is_error_binding: bool,
    /// The names of all variables in the binding (1 for simple, 2 for error binding)
    binding_names: Vec<String>,
    /// Whether the task has already been awaited
    awaited: bool,
    /// The execution policy (Async or Par)
    exec_policy: ExecPolicy,
    /// Whether the task returns a tuple directly (vs Result)
    returns_tuple: bool,
    /// Whether this is an HTTP call that returns (Option<T>, String) needing unwrap
    is_http_call: bool,
}

pub struct CodeGenerator {
    pub(crate) output: String,
    indent_level: usize,
    ctx: DesugarContext,
    in_method: bool,
    in_assignment_target: bool,
    in_fallible_function: bool,
    in_string_template: bool, // Track if we're inside a string template
    bracket_notation_vars: std::collections::HashSet<String>,
    class_instance_vars: std::collections::HashSet<String>,
    array_vars: std::collections::HashSet<String>, // Track which variables are arrays
    json_value_vars: std::collections::HashSet<String>, // Track which variables are JsonValue
    string_vars: std::collections::HashSet<String>, // Track which variables are strings
    native_vec_string_vars: std::collections::HashSet<String>, // Track Vec<String> from Sys.args() - use direct indexing
    // --- Class/type metadata (for field resolution)
    class_fields: std::collections::HashMap<String, std::collections::HashSet<String>>,
    class_optional_fields: std::collections::HashMap<String, std::collections::HashSet<String>>, // Track optional fields per class
    var_types: std::collections::HashMap<String, String>, // var -> ClassName
    fallible_functions: std::collections::HashSet<String>, // Track which functions are fallible
    // --- Type aliases (for expansion during codegen)
    type_aliases: std::collections::HashMap<String, (Vec<TypeParameter>, TypeRef)>,
    // --- Union types (for enum generation)
    union_types: std::collections::HashSet<Vec<String>>, // Track all union types used: [(i32, String), ...]
    // --- Phase 2: Lazy await/join tracking
    pending_tasks: std::collections::HashMap<String, TaskInfo>, // Variables that hold unawaited Tasks
    // --- Phase 3: Error binding variables (Option<String> type)
    error_binding_vars: std::collections::HashSet<String>, // Variables from error binding (second variable in let x, err = ...)
    string_error_vars: std::collections::HashSet<String>, // String error variables from HTTP/File calls (for `if err` sugar)
    option_value_vars: std::collections::HashSet<String>, // Variables from error binding (first variable in let value, err = ..., which is Option<T>)
    struct_destructured_vars: std::collections::HashSet<String>, // Variables from struct destructuring (may be Option<T>)
    rust_struct_vars: std::collections::HashSet<String>, // Variables that are Rust structs (HTTP response, etc.), not JsonValue
    typed_array_vars: std::collections::HashMap<String, String>, // Track arrays with element type: var_name -> element_class_name (e.g., "posts" -> "Post")
    current_lambda_element_type: Option<String>, // Temporarily track element type when generating lambdas in forEach/map/etc
    // --- Phase 4: Join combining optimization
    #[allow(dead_code)]
    awaitable_tasks: Vec<String>, // Tasks that can be combined with tokio::join!
    // --- Phase 5: Generic constraints
    trait_registry: TraitRegistry, // Trait registry for constraint validation
    // --- Async user functions
    async_functions: std::collections::BTreeSet<String>, // User-defined async functions (BTreeSet from DesugarContext)
    // --- Phase 6: Interface method signatures (for type inference)
    interface_methods: std::collections::HashMap<String, std::collections::HashMap<String, TypeRef>>, // interface_name -> (method_name -> return_type)
}

impl CodeGenerator {
    fn new(ctx: DesugarContext) -> Self {
        let async_funcs = ctx.async_functions.clone();
        Self {
            output: String::new(),
            indent_level: 0,
            ctx,
            in_method: false,
            in_assignment_target: false,
            in_fallible_function: false,
            in_string_template: false,
            bracket_notation_vars: std::collections::HashSet::new(),
            class_instance_vars: std::collections::HashSet::new(),
            array_vars: std::collections::HashSet::new(),
            json_value_vars: std::collections::HashSet::new(),
            string_vars: std::collections::HashSet::new(),
            native_vec_string_vars: std::collections::HashSet::new(),
            class_fields: std::collections::HashMap::new(),
            class_optional_fields: std::collections::HashMap::new(),
            var_types: std::collections::HashMap::new(),
            fallible_functions: std::collections::HashSet::new(),
            type_aliases: std::collections::HashMap::new(),
            union_types: std::collections::HashSet::new(),
            pending_tasks: std::collections::HashMap::new(),
            error_binding_vars: std::collections::HashSet::new(),
            string_error_vars: std::collections::HashSet::new(),
            option_value_vars: std::collections::HashSet::new(),
            struct_destructured_vars: std::collections::HashSet::new(),
            rust_struct_vars: std::collections::HashSet::new(),
            typed_array_vars: std::collections::HashMap::new(),
            current_lambda_element_type: None,
            awaitable_tasks: Vec::new(),
            trait_registry: TraitRegistry::new(),
            async_functions: async_funcs,
            interface_methods: std::collections::HashMap::new(),
        }
    }

    fn is_class_instance(&self, var_name: &str) -> bool {
        // For method contexts (self), always use dot notation
        if var_name == "self" || var_name == "this" {
            return true;
        }

        // Check if the variable was assigned from a class constructor
        self.class_instance_vars.contains(var_name) ||
        // Temporary heuristic: single character variables are likely class instances
        var_name.len() == 1
    }

    /// Check if an expression is a JsonValue (for lambda pattern detection)
    /// Returns true for both JsonValue direct and Vec<JsonValue>
    fn is_json_value_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(var_name) => self.json_value_vars.contains(var_name),
            Expr::MethodCall(mc) => {
                // If the method returns a JsonValue (e.g., .get_field(), .get())
                matches!(mc.method.as_str(), "get" | "get_field") || self.is_json_value_expr(&mc.object)
            }
            _ => false,
        }
    }
    
    /// Check if an expression is a DIRECT JsonValue (not Vec<JsonValue>)
    /// Direct means: from JSON.parse(), .get(), .get_field()
    /// Not from: .map(), .filter() (those return Vec<JsonValue>)
    fn is_direct_json_value(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(var_name) => {
                // Check if it's in json_value_vars AND not in array_vars
                // (array_vars includes Vec<JsonValue> from map/filter)
                self.json_value_vars.contains(var_name) && !self.array_vars.contains(var_name)
            }
            Expr::MethodCall(mc) => {
                // Only .get() and .get_field() return direct JsonValue
                matches!(mc.method.as_str(), "get" | "get_field")
            }
            _ => false,
        }
    }
    
    /// Check if expression is JSON.parse method call (Phase 1: JSON Typed Parsing)
    fn is_json_parse_call(&self, expr: &Expr) -> bool {
        match expr {
            // JSON.parse() call
            Expr::MethodCall(mc) if matches!(&*mc.object, Expr::Identifier(id) if id == "JSON") && mc.method == "parse" => true,
            // response.json() or any object's .json() method
            Expr::MethodCall(mc) if mc.method == "json" => true,
            _ => false
        }
    }
    
    /// Check if expression is an HTTP call (GET/POST/PUT/DELETE)
    fn is_http_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                // Check if callee is HTTP method call (async HTTP.get, etc.)
                if let Expr::MethodCall(mc) = call.callee.as_ref() {
                    if let Expr::Identifier(obj) = mc.object.as_ref() {
                        return (obj == "HTTP" || obj == "Http") && matches!(mc.method.as_str(), "get" | "post" | "put" | "delete");
                    }
                }
                false
            }
            Expr::MethodCall(mc) => {
                if let Expr::Identifier(obj) = mc.object.as_ref() {
                    return (obj == "HTTP" || obj == "Http") && matches!(mc.method.as_str(), "get" | "post" | "put" | "delete");
                }
                false
            }
            _ => false
        }
    }
    
    /// Check if expression is a File call (read/write/append/delete)
    fn is_file_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::MethodCall(mc) => {
                if let Expr::Identifier(obj) = mc.object.as_ref() {
                    return obj == "File" && matches!(mc.method.as_str(), "read" | "write" | "append" | "delete");
                }
                false
            }
            _ => false
        }
    }
    
    /// Check if expression is an await of a pending HTTP task
    /// Returns the task variable name if it's an HTTP task await
    fn is_await_http_task(&self, expr: &Expr) -> Option<String> {
        if let Expr::Unary { op: crate::ast::UnOp::Await, operand } = expr {
            if let Expr::Identifier(name) = operand.as_ref() {
                let sanitized = self.sanitize_name(name);
                if let Some(task_info) = self.pending_tasks.get(&sanitized) {
                    if task_info.is_http_call {
                        return Some(sanitized);
                    }
                }
            }
        }
        None
    }
    
    /// Check if an async expression contains an HTTP call
    /// e.g., async HTTP.get(url) -> true
    fn is_http_call_in_async(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                // Check if callee is HTTP method call
                if let Expr::MethodCall(mc) = call.callee.as_ref() {
                    if let Expr::Identifier(obj) = mc.object.as_ref() {
                        return (obj == "HTTP" || obj == "Http") && matches!(mc.method.as_str(), "get" | "post" | "put" | "delete");
                    }
                }
                false
            }
            _ => false,
        }
    }
    
    /// Check if a method modifies self fields (requires &mut self)
    fn method_modifies_self(&self, method: &MethodDecl) -> bool {
        if let Some(body) = &method.body {
            return self.block_modifies_self(body);
        }
        false
    }
    
    /// Check if a block contains assignments to self fields
    fn block_modifies_self(&self, block: &BlockStmt) -> bool {
        for stmt in &block.stmts {
            if self.stmt_modifies_self(stmt) {
                return true;
            }
        }
        false
    }
    
    /// Check if a statement modifies self fields
    fn stmt_modifies_self(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr_stmt) => self.expr_modifies_self(&expr_stmt.expr),
            Stmt::Return(return_stmt) => {
                return_stmt.expr.as_ref().map_or(false, |e| self.expr_modifies_self(e))
            }
            Stmt::VarDecl(var_decl) => self.expr_modifies_self(&var_decl.init),
            Stmt::Assign(assign_stmt) => {
                // Check if left side is this.field
                if let Expr::Member { object, .. } = &assign_stmt.target {
                    if let Expr::Identifier(obj) = object.as_ref() {
                        if obj == "this" || obj == "self" {
                            return true;
                        }
                    }
                }
                self.expr_modifies_self(&assign_stmt.value)
            }
            Stmt::If(if_stmt) => {
                let cond_modifies = self.expr_modifies_self(&if_stmt.condition);
                let then_modifies = match &if_stmt.then_branch {
                    IfBody::Block(b) => self.block_modifies_self(b),
                    IfBody::Stmt(s) => self.stmt_modifies_self(s),
                };
                let else_modifies = if_stmt.else_branch.as_ref().map_or(false, |eb| {
                    match eb {
                        IfBody::Block(b) => self.block_modifies_self(b),
                        IfBody::Stmt(s) => self.stmt_modifies_self(s),
                    }
                });
                cond_modifies || then_modifies || else_modifies
            }
            Stmt::While(while_stmt) => {
                self.expr_modifies_self(&while_stmt.condition) || self.block_modifies_self(&while_stmt.body)
            }
            Stmt::For(for_stmt) => {
                self.expr_modifies_self(&for_stmt.iterable) || self.block_modifies_self(&for_stmt.body)
            }
            _ => false,
        }
    }
    
    /// Check if an expression modifies self fields (assignment to this.field)
    fn expr_modifies_self(&self, expr: &Expr) -> bool {
        match expr {
            Expr::MethodCall(mc) => {
                // Bug #20 fix: Check if calling mutating methods on self fields
                // e.g., self.notes.push(note) means we modify self
                let is_mutating_method = matches!(mc.method.as_str(), 
                    "push" | "pop" | "remove" | "clear" | "insert" | 
                    "sort" | "reverse" | "extend" | "retain" | "truncate"
                );
                
                if is_mutating_method {
                    // Check if the base is this.something
                    if let Some(base_name) = self.get_base_var_name(&mc.object) {
                        if base_name == "this" || base_name == "self" {
                            return true;
                        }
                    }
                    // Check if it's this.field.method()
                    if let Expr::Member { object, .. } = mc.object.as_ref() {
                        if let Expr::Identifier(obj_name) = object.as_ref() {
                            if obj_name == "this" || obj_name == "self" {
                                return true;
                            }
                        }
                    }
                }
                
                self.expr_modifies_self(&mc.object) || mc.args.iter().any(|a| self.expr_modifies_self(a))
            }
            Expr::Call(call) => {
                self.expr_modifies_self(&call.callee) || call.args.iter().any(|a| self.expr_modifies_self(a))
            }
            Expr::Binary { left, right, .. } => {
                self.expr_modifies_self(left) || self.expr_modifies_self(right)
            }
            _ => false,
        }
    }
    
    /// Extract the base variable name from an expression
    /// e.g., posts.parvec() -> "posts", myArray -> "myArray"
    fn get_base_var_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Identifier(name) => Some(name.clone()),
            Expr::MethodCall(mc) => self.get_base_var_name(&mc.object),
            _ => None
        }
    }
    
    /// Generate typed JSON parsing code (Phase 1: JSON Typed Parsing)
    /// Generates: serde_json::from_str::<Type>(&json_string)
    fn generate_typed_json_parse(&mut self, method_call: &MethodCallExpr, type_ref: &TypeRef) -> Result<()> {
        // Convert Liva type to Rust type
        let rust_type = type_ref.to_rust_type();
        
        // Generate: serde_json::from_str::<RustType>(&json_arg)
        self.output.push_str("serde_json::from_str::<");
        self.output.push_str(&rust_type);
        self.output.push_str(">(&");
        
        // Check if this is JSON.parse() or response.json()
        if method_call.method == "json" && !matches!(&*method_call.object, Expr::Identifier(id) if id == "JSON") {
            // This is response.json() - use the response body
            self.generate_expr(&method_call.object)?;
            self.output.push_str(".body");
        } else {
            // This is JSON.parse() - use the argument
            if let Some(arg) = method_call.args.first() {
                self.generate_expr(arg)?;
            } else {
                return Err(CompilerError::CodegenError(
                    SemanticErrorInfo::new(
                        "E3001",
                        "JSON.parse requires a string argument",
                        "JSON.parse must be called with a JSON string"
                    )
                ));
            }
        }
        
        self.output.push(')');
        Ok(())
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn generate_program(&mut self, program: &Program) -> Result<()> {
        // Generate use statements for Rust crates
        for (crate_name, alias) in &self.ctx.rust_crates {
            if let Some(alias_name) = alias {
                writeln!(self.output, "use {} as {};", crate_name, alias_name).unwrap();
            } else {
                writeln!(self.output, "use {};", crate_name).unwrap();
            }
        }

        if !self.ctx.rust_crates.is_empty() {
            self.output.push('\n');
        }

        // Build class metadata maps
        self.class_fields.clear();
        self.class_optional_fields.clear();
        for item in &program.items {
            if let TopLevel::Class(cls) = item {
                let mut fields = std::collections::HashSet::new();
                let mut optional_fields = std::collections::HashSet::new();
                for m in &cls.members {
                    if let Member::Field(f) = m {
                        fields.insert(f.name.clone());
                        if f.is_optional {
                            optional_fields.insert(f.name.clone());
                        }
                    }
                }
                self.class_fields.insert(cls.name.clone(), fields);
                self.class_optional_fields.insert(cls.name.clone(), optional_fields);
            }
        }
        
        // Build interface method signatures map (for type inference in implementing classes)
        // Note: Due to parser design, interfaces may be parsed as Class without constructor
        self.interface_methods.clear();
        for item in &program.items {
            // Check TopLevel::Type (explicit interface syntax via 'type' keyword)
            if let TopLevel::Type(type_decl) = item {
                let mut methods: std::collections::HashMap<String, TypeRef> = std::collections::HashMap::new();
                for member in &type_decl.members {
                    if let Member::Method(m) = member {
                        if let Some(ret_type) = &m.return_type {
                            methods.insert(m.name.clone(), ret_type.clone());
                        }
                    }
                }
                if !methods.is_empty() {
                    self.interface_methods.insert(type_decl.name.clone(), methods);
                }
            }
            // Also check Class that is really an interface (no constructor, only method signatures)
            if let TopLevel::Class(class) = item {
                let has_constructor = class.members.iter().any(|m| {
                    matches!(m, Member::Method(method) if method.name == "constructor")
                });
                let has_method_bodies = class.members.iter().any(|m| {
                    matches!(m, Member::Method(method) if method.body.is_some() || method.expr_body.is_some())
                });
                // If no constructor and no method bodies, it's an interface
                if !has_constructor && !has_method_bodies {
                    let mut methods: std::collections::HashMap<String, TypeRef> = std::collections::HashMap::new();
                    for member in &class.members {
                        if let Member::Method(m) = member {
                            if let Some(ret_type) = &m.return_type {
                                methods.insert(m.name.clone(), ret_type.clone());
                            }
                        }
                    }
                    if !methods.is_empty() {
                        self.interface_methods.insert(class.name.clone(), methods);
                    }
                }
            }
        }

    // Always include concurrency runtime for now
    if std::env::var("LIVA_DEBUG").is_ok() { println!("DEBUG: Including liva_rt module"); }
        self.writeln("mod liva_rt {");
        self.indent();
        self.writeln("use std::future::Future;");
        self.writeln("use tokio::task::JoinHandle;");
        self.writeln("");

        // Add Error type for fallibility system
        self.writeln("/// Runtime error type for fallible operations");
        self.writeln("#[derive(Debug, Clone, PartialEq)]");
        self.writeln("pub struct Error {");
        self.indent();
        self.writeln("pub message: String,");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("impl Error {");
        self.indent();
        self.writeln("pub fn from<S: Into<String>>(message: S) -> Self {");
        self.indent();
        self.writeln("Error {");
        self.indent();
        self.writeln("message: message.into(),");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("impl std::fmt::Display for Error {");
        self.indent();
        self.writeln("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {");
        self.indent();
        self.writeln("write!(f, \"{}\", self.message)");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("impl std::error::Error for Error {}");
        self.writeln("");

        // Always include both async and parallel functions
        self.writeln("/// Spawn an async task");
        self.writeln("pub fn spawn_async<F, T>(future: F) -> JoinHandle<T>");
        self.writeln("where");
        self.indent();
        self.writeln("F: Future<Output = T> + Send + 'static,");
        self.writeln("T: Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("tokio::spawn(future)");
        self.dedent();
        self.writeln("}");

        self.writeln("/// Fire and forget async task");
        self.writeln("pub fn fire_async<F>(future: F)");
        self.writeln("where");
        self.indent();
        self.writeln("F: Future<Output = ()> + Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("tokio::spawn(future);");
        self.dedent();
        self.writeln("}");

        self.writeln("/// Spawn a parallel task");
        self.writeln("pub fn spawn_parallel<F, T>(f: F) -> JoinHandle<T>");
        self.writeln("where");
        self.indent();
        self.writeln("F: FnOnce() -> T + Send + 'static,");
        self.writeln("T: Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("// For simplicity, just execute synchronously and wrap in JoinHandle");
        self.writeln("// In a real implementation, this would use rayon or std::thread");
        self.writeln("let result = f();");
        self.writeln("tokio::spawn(async move { result })");
        self.dedent();
        self.writeln("}");

        self.writeln("/// Fire and forget parallel task");
        self.writeln("pub fn fire_parallel<F>(f: F)");
        self.writeln("where");
        self.indent();
        self.writeln("F: FnOnce() + Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("// For simplicity, just spawn a thread");
        self.writeln("std::thread::spawn(f);");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // String multiplication helper
        self.writeln("/// String multiplication helper");
        self.writeln("/// Supports both String*int and int*String patterns");
        self.writeln("pub fn string_mul<L: StringOrInt, R: StringOrInt>(left: L, right: R) -> String {");
        self.indent();
        self.writeln("match (left.as_string_or_int(), right.as_string_or_int()) {");
        self.indent();
        self.writeln("(StringOrIntValue::String(s), StringOrIntValue::Int(n)) => {");
        self.indent();
        self.writeln("if n <= 0 { String::new() } else { s.repeat(n as usize) }");
        self.dedent();
        self.writeln("}");
        self.writeln("(StringOrIntValue::Int(n), StringOrIntValue::String(s)) => {");
        self.indent();
        self.writeln("if n <= 0 { String::new() } else { s.repeat(n as usize) }");
        self.dedent();
        self.writeln("}");
        self.writeln("(StringOrIntValue::Int(a), StringOrIntValue::Int(b)) => {");
        self.indent();
        self.writeln("(a * b).to_string()");
        self.dedent();
        self.writeln("}");
        self.writeln("(StringOrIntValue::String(_), StringOrIntValue::String(_)) => {");
        self.indent();
        self.writeln("panic!(\"Cannot multiply two strings\")");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub enum StringOrIntValue {");
        self.indent();
        self.writeln("String(String),");
        self.writeln("Int(i64),");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub trait StringOrInt {");
        self.indent();
        self.writeln("fn as_string_or_int(self) -> StringOrIntValue;");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("impl StringOrInt for String {");
        self.indent();
        self.writeln("fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::String(self) }");
        self.dedent();
        self.writeln("}");
        self.writeln("impl StringOrInt for &str {");
        self.indent();
        self.writeln("fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::String(self.to_string()) }");
        self.dedent();
        self.writeln("}");
        self.writeln("impl StringOrInt for i32 {");
        self.indent();
        self.writeln("fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::Int(self as i64) }");
        self.dedent();
        self.writeln("}");
        self.writeln("impl StringOrInt for i64 {");
        self.indent();
        self.writeln("fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::Int(self) }");
        self.dedent();
        self.writeln("}");
        self.writeln("impl StringOrInt for f64 {");
        self.indent();
        self.writeln("fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::Int(self as i64) }");
        self.dedent();
        self.writeln("}");
        self.writeln("impl StringOrInt for usize {");
        self.indent();
        self.writeln("fn as_string_or_int(self) -> StringOrIntValue { StringOrIntValue::Int(self as i64) }");
        self.dedent();
        self.writeln("}");
        self.writeln("");

        // HTTP Client Runtime Functions
        self.writeln("// HTTP Client");
        self.writeln("#[derive(Debug, Clone, Default)]");
        self.writeln("pub struct LivaHttpResponse {");
        self.indent();
        self.writeln("pub status: i32,");
        self.writeln("pub status_text: String,");
        self.writeln("pub body: String,");
        self.writeln("pub headers: Vec<String>,");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        // Add impl block with json() method
        self.writeln("impl LivaHttpResponse {");
        self.indent();
        self.writeln("pub fn json(&self) -> (JsonValue, String) {");
        self.indent();
        self.writeln("match serde_json::from_str(&self.body) {");
        self.indent();
        self.writeln("Ok(value) => (JsonValue(value), String::new()),");
        self.writeln("Err(e) => (JsonValue(serde_json::Value::Null), format!(\"JSON parse error: {}\", e)),");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        self.writeln("pub async fn liva_http_get(url: String) -> (Option<LivaHttpResponse>, String) {");
        self.indent();
        self.writeln("liva_http_request(\"GET\", url, None).await");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        self.writeln("pub async fn liva_http_post(url: String, body: String) -> (Option<LivaHttpResponse>, String) {");
        self.indent();
        self.writeln("liva_http_request(\"POST\", url, Some(body)).await");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        self.writeln("pub async fn liva_http_put(url: String, body: String) -> (Option<LivaHttpResponse>, String) {");
        self.indent();
        self.writeln("liva_http_request(\"PUT\", url, Some(body)).await");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        self.writeln("pub async fn liva_http_delete(url: String) -> (Option<LivaHttpResponse>, String) {");
        self.indent();
        self.writeln("liva_http_request(\"DELETE\", url, None).await");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        self.writeln("async fn liva_http_request(method: &str, url: String, body: Option<String>) -> (Option<LivaHttpResponse>, String) {");
        self.indent();
        self.writeln("if !url.starts_with(\"http://\") && !url.starts_with(\"https://\") {");
        self.indent();
        self.writeln("return (None, format!(\"Invalid URL format: '{}'. URLs must start with http:// or https://\", url));");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("let client = match reqwest::Client::builder().timeout(std::time::Duration::from_secs(30)).build() {");
        self.indent();
        self.writeln("Ok(c) => c,");
        self.writeln("Err(e) => return (None, format!(\"Failed to create HTTP client: {}\", e)),");
        self.dedent();
        self.writeln("};");
        self.writeln("");
        self.writeln("let request_builder = match method {");
        self.indent();
        self.writeln("\"GET\" => client.get(&url),");
        self.writeln("\"POST\" => {");
        self.indent();
        self.writeln("let mut builder = client.post(&url);");
        self.writeln("if let Some(body_content) = body {");
        self.indent();
        self.writeln("builder = builder.header(\"Content-Type\", \"application/json\").body(body_content);");
        self.dedent();
        self.writeln("}");
        self.writeln("builder");
        self.dedent();
        self.writeln("}");
        self.writeln("\"PUT\" => {");
        self.indent();
        self.writeln("let mut builder = client.put(&url);");
        self.writeln("if let Some(body_content) = body {");
        self.indent();
        self.writeln("builder = builder.header(\"Content-Type\", \"application/json\").body(body_content);");
        self.dedent();
        self.writeln("}");
        self.writeln("builder");
        self.dedent();
        self.writeln("}");
        self.writeln("\"DELETE\" => client.delete(&url),");
        self.writeln("_ => return (None, format!(\"Unknown HTTP method: {}\", method)),");
        self.dedent();
        self.writeln("};");
        self.writeln("");
        self.writeln("// Add User-Agent header required by GitHub and other APIs");
        self.writeln("let request_builder = request_builder.header(\"User-Agent\", \"Liva-HTTP-Client/1.0\");");
        self.writeln("");
        self.writeln("let response = match request_builder.send().await {");
        self.indent();
        self.writeln("Ok(resp) => resp,");
        self.writeln("Err(e) => {");
        self.indent();
        self.writeln("let error_msg = if e.is_timeout() { \"Request timeout (30s)\".to_string() }");
        self.writeln("else if e.is_connect() { format!(\"Connection error: {}\", e) }");
        self.writeln("else { format!(\"Network error: {}\", e) };");
        self.writeln("return (None, error_msg);");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("};");
        self.writeln("");
        self.writeln("let status = response.status();");
        self.writeln("let status_code = status.as_u16() as i32;");
        self.writeln("let status_text = status.canonical_reason().unwrap_or(\"Unknown\").to_string();");
        self.writeln("");
        self.writeln("let mut headers = Vec::new();");
        self.writeln("for (key, value) in response.headers() {");
        self.indent();
        self.writeln("if let Ok(value_str) = value.to_str() {");
        self.indent();
        self.writeln("headers.push(format!(\"{}: {}\", key.as_str(), value_str));");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("let body = match response.text().await {");
        self.indent();
        self.writeln("Ok(text) => text,");
        self.writeln("Err(e) => return (None, format!(\"Failed to read response body: {}\", e)),");
        self.dedent();
        self.writeln("};");
        self.writeln("");
        self.writeln("(Some(LivaHttpResponse { status: status_code, status_text, body, headers }), String::new())");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        // JSON Support - JsonValue wrapper
        self.writeln("// JSON Support");
        self.writeln("#[derive(Debug, Clone)]");
        self.writeln("pub struct JsonValue(pub serde_json::Value);");
        self.writeln("");
        self.writeln("impl Default for JsonValue {");
        self.indent();
        self.writeln("fn default() -> Self { JsonValue(serde_json::Value::Null) }");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("impl JsonValue {");
        self.indent();
        self.writeln("pub fn new(value: serde_json::Value) -> Self { JsonValue(value) }");
        self.writeln("");
        self.writeln("pub fn length(&self) -> usize {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::Array(arr) => arr.len(),");
        self.writeln("serde_json::Value::Object(obj) => obj.len(),");
        self.writeln("serde_json::Value::String(s) => s.len(),");
        self.writeln("_ => 0,");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn get(&self, index: usize) -> Option<JsonValue> {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::Array(arr) => arr.get(index).map(|v| JsonValue(v.clone())),");
        self.writeln("_ => None,");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn get_field(&self, key: &str) -> Option<JsonValue> {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::Object(obj) => obj.get(key).map(|v| JsonValue(v.clone())),");
        self.writeln("_ => None,");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        // Type conversion methods
        self.writeln("pub fn as_i32(&self) -> Option<i32> {");
        self.indent();
        self.writeln("self.0.as_i64().map(|n| n as i32)");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn as_f64(&self) -> Option<f64> {");
        self.indent();
        self.writeln("self.0.as_f64()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        // as_float is an alias for as_f64
        self.writeln("pub fn as_float(&self) -> f64 {");
        self.indent();
        self.writeln("self.0.as_f64().unwrap_or(0.0)");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn as_string(&self) -> Option<String> {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::String(s) => Some(s.clone()),");
        self.writeln("_ => None,");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn as_bool(&self) -> Option<bool> {");
        self.indent();
        self.writeln("self.0.as_bool()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn is_null(&self) -> bool {");
        self.indent();
        self.writeln("self.0.is_null()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn is_array(&self) -> bool {");
        self.indent();
        self.writeln("self.0.is_array()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn is_object(&self) -> bool {");
        self.indent();
        self.writeln("self.0.is_object()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn to_json_string(&self) -> String {");
        self.indent();
        self.writeln("self.0.to_string()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        // as_array() returns Vec<JsonValue> directly (unwraps automatically)
        self.writeln("pub fn as_array(&self) -> Vec<JsonValue> {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::Array(arr) => arr.iter().map(|v| JsonValue(v.clone())).collect(),");
        self.writeln("_ => Vec::new(),");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        // Add to_vec and iter methods for array operations
        self.writeln("pub fn to_vec(&self) -> Vec<JsonValue> {");
        self.indent();
        self.writeln("self.as_array()");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        self.writeln("pub fn iter(&self) -> std::vec::IntoIter<JsonValue> {");
        self.indent();
        self.writeln("self.to_vec().into_iter()");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        self.writeln("impl std::fmt::Display for JsonValue {");
        self.indent();
        self.writeln("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {");
        self.indent();
        // For strings, display without quotes; for other types, use JSON representation
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::String(s) => write!(f, \"{}\", s),");
        self.writeln("serde_json::Value::Null => write!(f, \"null\"),");
        self.writeln("other => write!(f, \"{}\", other),");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        // Add Index<&str> for nested JSON access: json["field"]["nested"]
        self.writeln("impl std::ops::Index<&str> for JsonValue {");
        self.indent();
        self.writeln("type Output = JsonValue;");
        self.writeln("");
        self.writeln("fn index(&self, key: &str) -> &Self::Output {");
        self.indent();
        self.writeln("// This is a bit of a hack - we leak the value to get a static reference");
        self.writeln("// In practice, this is safe for our use case since we don't mutate");
        self.writeln("static NULL_VALUE: std::sync::OnceLock<JsonValue> = std::sync::OnceLock::new();");
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::Object(obj) => {");
        self.indent();
        self.writeln("if let Some(v) = obj.get(key) {");
        self.indent();
        self.writeln("// Leak to get 'static lifetime - acceptable for read-only JSON access");
        self.writeln("Box::leak(Box::new(JsonValue(v.clone())))");
        self.dedent();
        self.writeln("} else {");
        self.indent();
        self.writeln("NULL_VALUE.get_or_init(|| JsonValue(serde_json::Value::Null))");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("_ => NULL_VALUE.get_or_init(|| JsonValue(serde_json::Value::Null)),");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        // Add IntoIterator for for...in loop support
        self.writeln("impl IntoIterator for JsonValue {");
        self.indent();
        self.writeln("type Item = JsonValue;");
        self.writeln("type IntoIter = std::vec::IntoIter<JsonValue>;");
        self.writeln("");
        self.writeln("fn into_iter(self) -> Self::IntoIter {");
        self.indent();
        self.writeln("match self.0 {");
        self.indent();
        self.writeln("serde_json::Value::Array(arr) => {");
        self.indent();
        self.writeln("arr.into_iter().map(|v| JsonValue(v)).collect::<Vec<_>>().into_iter()");
        self.dedent();
        self.writeln("}");
        self.writeln("_ => Vec::new().into_iter(),");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        // Add PartialEq<bool> for comparing JSON booleans
        self.writeln("impl PartialEq<bool> for JsonValue {");
        self.indent();
        self.writeln("fn eq(&self, other: &bool) -> bool {");
        self.indent();
        self.writeln("self.as_bool() == Some(*other)");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        // Add PartialEq<&str> for comparing JSON strings
        self.writeln("impl PartialEq<&str> for JsonValue {");
        self.indent();
        self.writeln("fn eq(&self, other: &&str) -> bool {");
        self.indent();
        self.writeln("match &self.0 {");
        self.indent();
        self.writeln("serde_json::Value::String(s) => s == *other,");
        self.writeln("_ => false,");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.writeln("");
        
        // Add is_null check method and comparison - null generates .is_null() check
        // No PartialEq needed - we'll translate `x != null` to `!x.is_null()`

        self.dedent();
        self.writeln("}");
        self.writeln("");

        // Add rayon imports if parallel execution is used (at top level, after liva_rt module)
        if self.ctx.has_parallel {
            self.writeln("// Rayon parallel iterator support");
            self.writeln("use rayon::prelude::*;");
            self.writeln("");
        }

        // Pre-pass: Collect all type aliases
        for item in &program.items {
            if let TopLevel::TypeAlias(alias) = item {
                self.type_aliases.insert(
                    alias.name.clone(),
                    (alias.type_params.clone(), alias.target_type.clone())
                );
            }
        }

        // Pre-pass: Collect all union types by scanning type annotations
        // This happens during generation, unions are registered in expand_type_alias

        // Generate top-level items (first pass to collect unions)
        for item in &program.items {
            if std::env::var("LIVA_DEBUG").is_ok() { println!("DEBUG: Processing top-level item: {:?}", item); }
            match item {
                TopLevel::Class(cls) => if std::env::var("LIVA_DEBUG").is_ok() { println!("DEBUG: Found class: {}", cls.name) },
                TopLevel::Function(func) => if std::env::var("LIVA_DEBUG").is_ok() { println!("DEBUG: Found function: {}", func.name) },
                _ => if std::env::var("LIVA_DEBUG").is_ok() { println!("DEBUG: Found other item: {:?}", item) },
            }
            self.generate_top_level(item)?;
            self.output.push('\n');
        }

        // After first pass, generate union type enum definitions
        let unions_to_generate: Vec<Vec<String>> = self.union_types.iter().cloned().collect();
        if !unions_to_generate.is_empty() {
            // Insert union enums before the generated code
            let mut union_defs = String::new();
            union_defs.push_str("\n// Union type definitions\n");
            
            for union_types in unions_to_generate {
                let enum_name = format!("Union_{}", union_types.join("_"));
                union_defs.push_str(&format!("#[derive(Debug, Clone)]\n"));
                union_defs.push_str(&format!("enum {} {{\n", enum_name));
                
                // Generate variant for each type in the union
                for (_i, rust_type) in union_types.iter().enumerate() {
                    let variant_name = self.type_to_variant_name(rust_type);
                    union_defs.push_str(&format!("    {}({}),\n", variant_name, rust_type));
                }
                
                union_defs.push_str("}\n\n");
                
                // Implement Display for the union enum
                union_defs.push_str(&format!("impl std::fmt::Display for {} {{\n", enum_name));
                union_defs.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
                union_defs.push_str("        match self {\n");
                
                for rust_type in &union_types {
                    let variant_name = self.type_to_variant_name(rust_type);
                    union_defs.push_str(&format!("            {}::{}(val) => write!(f, \"{{}}\", val),\n", 
                        enum_name, variant_name));
                }
                
                union_defs.push_str("        }\n");
                union_defs.push_str("    }\n");
                union_defs.push_str("}\n\n");
            }
            
            // Find where to insert (after liva_rt module, before first function)
            // For now, prepend to output (will fix positioning later)
            let temp = self.output.clone();
            self.output = union_defs;
            self.output.push_str(&temp);
        }

        Ok(())
    }

    /// Convert a Rust type string to a valid enum variant name
    fn type_to_variant_name(&self, rust_type: &str) -> String {
        match rust_type {
            "i32" => "Int".to_string(),
            "String" => "Str".to_string(),
            "f64" => "Float".to_string(),
            "bool" => "Bool".to_string(),
            other => {
                // Remove special characters and capitalize
                other
                    .replace("<", "")
                    .replace(">", "")
                    .replace(",", "")
                    .replace(" ", "")
                    .replace("(", "")
                    .replace(")", "")
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().to_string())
                    .unwrap_or_default()
                    + &other[1..]
                        .replace("<", "")
                        .replace(">", "")
                        .replace(",", "")
                        .replace(" ", "")
                        .replace("(", "")
                        .replace(")", "")
            }
        }
    }

    /// Generate union wrapper if needed (e.g., Union_i32_String::Int(42))
    /// Returns (needs_close, needs_to_string) tuple
    fn maybe_wrap_in_union(&mut self, dest_type_ref: &TypeRef, expr: &Expr) -> (bool, bool) {
        // Check if destination is a union type
        if let TypeRef::Union(members) = dest_type_ref {
            // Infer the type of the expression (using existing infer_expr_type)
            if let Some(type_with_arrow) = self.infer_expr_type(expr, None) {
                // Extract type from " -> i32" format
                let expr_type = type_with_arrow.trim_start_matches(" -> ").to_string();
                
                // Generate the union type name
                let expanded_members: Vec<String> = members
                    .iter()
                    .map(|m| self.expand_type_alias(m))
                    .collect();
                let union_name = format!("Union_{}", expanded_members.join("_"));
                
                // Find which variant to use
                let variant = self.type_to_variant_name(&expr_type);
                
                // Check if this is a string literal that needs .to_string()
                let needs_to_string = expr_type == "String" 
                    && matches!(expr, Expr::Literal(Literal::String(_)));
                
                // Generate wrapper
                write!(self.output, "{}::{}(", union_name, variant).unwrap();
                return (true, needs_to_string);
            }
        }
        (false, false)
    }

    fn generate_top_level(&mut self, item: &TopLevel) -> Result<()> {
        match item {
            TopLevel::Import(_) => {
                // Imports are handled differently in Rust
                // We'd need to map to actual module paths
                Ok(())
            }
            TopLevel::UseRust(_) => {
                // Already handled in use statements
                Ok(())
            }
            TopLevel::Type(type_decl) => self.generate_type_decl(type_decl),
            TopLevel::TypeAlias(alias) => self.generate_type_alias(alias),
            TopLevel::Class(class) => {
                if std::env::var("LIVA_DEBUG").is_ok() { println!("DEBUG: Generating class {}", class.name); }
                self.generate_class(class)
            }
            TopLevel::Function(func) => self.generate_function(func),
            TopLevel::Test(test) => self.generate_test(test),
        }
    }

    fn generate_type_decl(&mut self, type_decl: &TypeDecl) -> Result<()> {
        // Interfaces in Liva are compile-time only contracts.
        // They are validated by the semantic analyzer but do NOT generate any Rust code.
        // Classes implementing interfaces just need to have the required methods.
        // Interface method signatures are collected in generate_program() for type inference.
        
        // Only generate a comment for documentation purposes
        self.writeln(&format!("// Interface: {} (compile-time validation only)", type_decl.name));
        
        Ok(())
    }

    fn generate_type_alias(&mut self, alias: &TypeAliasDecl) -> Result<()> {
        // Store type alias for expansion during type annotation generation
        self.type_aliases.insert(
            alias.name.clone(),
            (alias.type_params.clone(), alias.target_type.clone())
        );
        // Type aliases in Liva are expanded inline during type checking
        // We don't generate Rust type aliases to keep codegen simple
        Ok(())
    }

    /// Expand type aliases in a TypeRef to get the final Rust type string
    fn expand_type_alias(&mut self, type_ref: &TypeRef) -> String {
        match type_ref {
            TypeRef::Simple(name) => {
                // Check if it's a type alias
                if let Some((alias_params, target_type)) = self.type_aliases.get(name).cloned() {
                    // If the alias has no type parameters, just expand the target
                    if alias_params.is_empty() {
                        return self.expand_type_alias(&target_type);
                    }
                    // If it has type parameters but no args, just expand (error should be caught in semantic)
                    return self.expand_type_alias(&target_type);
                }
                // Not a type alias, use the normal to_rust_type conversion
                type_ref.to_rust_type()
            }
            TypeRef::Generic { base, args } => {
                // Check if the base is a type alias
                if let Some((alias_params, target_type)) = self.type_aliases.get(base).cloned() {
                    // Substitute type parameters
                    let substituted = self.substitute_type_params_codegen(
                        &target_type,
                        &alias_params,
                        args,
                    );
                    return self.expand_type_alias(&substituted);
                }
                // Not a type alias, recursively expand arguments
                let expanded_args: Vec<String> = args
                    .iter()
                    .map(|arg| self.expand_type_alias(arg))
                    .collect();
                format!("{}<{}>", base, expanded_args.join(", "))
            }
            TypeRef::Array(inner) => {
                format!("Vec<{}>", self.expand_type_alias(inner))
            }
            TypeRef::Optional(inner) => {
                format!("Option<{}>", self.expand_type_alias(inner))
            }
            TypeRef::Fallible(inner) => {
                format!("Result<{}, liva_rt::Error>", self.expand_type_alias(inner))
            }
            TypeRef::Tuple(types) => {
                let types_str: Vec<String> = types
                    .iter()
                    .map(|t| self.expand_type_alias(t))
                    .collect();
                // Rust requires trailing comma for single-element tuples
                if types.len() == 1 {
                    format!("({},)", types_str.join(", "))
                } else {
                    format!("({})", types_str.join(", "))
                }
            }
            TypeRef::Union(types) => {
                // For union types, register and generate a Rust enum
                let type_names: Vec<String> = types
                    .iter()
                    .map(|t| self.expand_type_alias(t))
                    .collect();
                
                // Register this union for enum generation
                self.union_types.insert(type_names.clone());
                
                // Generate union enum name
                format!("Union_{}", type_names.join("_"))
            }
        }
    }

    /// Substitute type parameters in a TypeRef (for codegen)
    fn substitute_type_params_codegen(
        &self,
        type_ref: &TypeRef,
        params: &[TypeParameter],
        args: &[TypeRef],
    ) -> TypeRef {
        match type_ref {
            TypeRef::Simple(name) => {
                // Check if this name is one of the type parameters
                for (i, param) in params.iter().enumerate() {
                    if &param.name == name {
                        return args[i].clone();
                    }
                }
                // Not a type parameter, return as-is
                type_ref.clone()
            }
            TypeRef::Array(inner) => {
                TypeRef::Array(Box::new(self.substitute_type_params_codegen(inner, params, args)))
            }
            TypeRef::Optional(inner) => {
                TypeRef::Optional(Box::new(self.substitute_type_params_codegen(inner, params, args)))
            }
            TypeRef::Fallible(inner) => {
                TypeRef::Fallible(Box::new(self.substitute_type_params_codegen(inner, params, args)))
            }
            TypeRef::Tuple(elements) => {
                TypeRef::Tuple(
                    elements
                        .iter()
                        .map(|elem| self.substitute_type_params_codegen(elem, params, args))
                        .collect(),
                )
            }
            TypeRef::Union(types) => {
                TypeRef::Union(
                    types
                        .iter()
                        .map(|ty| self.substitute_type_params_codegen(ty, params, args))
                        .collect(),
                )
            }
            TypeRef::Generic { base, args: inner_args } => {
                // Recursively substitute in base and all arguments
                let substituted_base = match self.substitute_type_params_codegen(
                    &TypeRef::Simple(base.clone()),
                    params,
                    args,
                ) {
                    TypeRef::Simple(name) => name,
                    _ => base.clone(), // Shouldn't happen
                };
                
                TypeRef::Generic {
                    base: substituted_base,
                    args: inner_args
                        .iter()
                        .map(|arg| self.substitute_type_params_codegen(arg, params, args))
                        .collect(),
                }
            }
        }
    }


    fn generate_class(&mut self, class: &ClassDecl) -> Result<()> {
        // Check if this is actually an interface (no constructor, methods without bodies)
        // Interfaces are compile-time only and don't generate Rust code
        let has_constructor = class.members.iter().any(|m| {
            matches!(m, Member::Method(method) if method.name == "constructor")
        });
        let all_methods_abstract = class.members.iter().all(|m| {
            match m {
                Member::Method(method) => method.body.is_none() && method.expr_body.is_none(),
                Member::Field(_) => false, // Fields without init are fine for interfaces
            }
        });
        let has_only_methods = class.members.iter().all(|m| matches!(m, Member::Method(_)));
        
        // If no constructor and all methods are abstract (no body), it's an interface
        if !has_constructor && all_methods_abstract && has_only_methods {
            self.writeln(&format!("// Interface: {} (compile-time validation only)", class.name));
            return Ok(());
        }

        // Generate default functions for optional fields with init values
        if class.needs_serde {
            for member in &class.members {
                if let Member::Field(field) = member {
                    if field.is_optional && field.init.is_some() {
                        self.generate_field_default_function(&class.name, field)?;
                    }
                }
            }
        }
        
        // Format type parameters
        let type_params_str = if !class.type_params.is_empty() {
            let params: Vec<String> = class.type_params.iter().map(|tp| {
                if !tp.constraints.is_empty() {
                    // Use trait registry to get complete Rust trait bounds
                    let rust_bounds = self.trait_registry.generate_rust_bounds(&tp.constraints);
                    format!("{}{}", tp.name, rust_bounds)
                } else {
                    tp.name.clone()
                }
            }).collect();
            format!("<{}>", params.join(", "))
        } else {
            String::new()
        };
        
        // Phase 2: Generate serde derives if class is used with JSON.parse
        let derives = if class.needs_serde {
            "#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]"
        } else {
            "#[derive(Debug, Clone, Default)]"
        };
        
        // Generate struct (interfaces are validated at compile-time, no runtime representation)
        if !class.implements.is_empty() {
            self.writeln(&format!("// {} implements {}", class.name, class.implements.join(", ")));
        }
        self.writeln(derives);
        self.writeln(&format!("pub struct {}{} {{", class.name, type_params_str));
        self.indent();

        for member in &class.members {
            if let Member::Field(field) = member {
                self.generate_field(field, class.needs_serde, Some(&class.name))?;
            }
        }

        self.dedent();
        self.writeln("}");
        self.output.push('\n');

        // Generate impl block
        let _has_methods = class.members.iter().any(|m| matches!(m, Member::Method(_)));
        let has_fields = class.members.iter().any(|m| matches!(m, Member::Field(_)));

        // Find constructor method
        let constructor = class.members.iter().find_map(|m| {
            if let Member::Method(method) = m {
                if method.name == "constructor" {
                    Some(method)
                } else {
                    None
                }
            } else {
                None
            }
        });

        // Format type parameters for impl block
        let impl_type_params = if !class.type_params.is_empty() {
            let params: Vec<String> = class.type_params.iter().map(|tp| {
                if !tp.constraints.is_empty() {
                    // Use trait registry to get complete Rust trait bounds
                    let rust_bounds = self.trait_registry.generate_rust_bounds(&tp.constraints);
                    format!("{}{}", tp.name, rust_bounds)
                } else {
                    tp.name.clone()
                }
            }).collect();
            format!("<{}>", params.join(", "))
        } else {
            String::new()
        };
        
        let impl_type_args = if !class.type_params.is_empty() {
            let args: Vec<String> = class.type_params.iter().map(|tp| tp.name.clone()).collect();
            format!("<{}>", args.join(", "))
        } else {
            String::new()
        };
        
        self.writeln(&format!("impl{} {}{} {{", impl_type_params, class.name, impl_type_args));
        self.indent();

        // Generate constructor
        if let Some(constructor_method) = constructor {
            // Custom constructor - generate new() with parameters
            self.write_indent();
            write!(self.output, "pub fn new(").unwrap();
            let params_str = self.generate_params(
                &constructor_method.params,
                false,
                Some(class),
                Some("constructor"),
                None,
            )?;
            write!(self.output, "{}) -> Self {{\n", params_str).unwrap();
            self.indent();
            self.write_indent();
            self.output.push_str("Self {\n");
            self.indent();

            // Bug #19 fix: Parse constructor body to find this.field = value assignments
            // Build a map of field_name -> assigned_value
            let mut field_assignments: std::collections::HashMap<String, &Expr> = std::collections::HashMap::new();
            
            if let Some(body) = &constructor_method.body {
                for stmt in &body.stmts {
                    // Look for assignments like: this.field = value
                    if let Stmt::Assign(assign) = stmt {
                        if let Expr::Member { object, property } = &assign.target {
                            if let Expr::Identifier(obj_name) = object.as_ref() {
                                if obj_name == "this" {
                                    // Found this.field = value
                                    let field_name = self.sanitize_name(property);
                                    field_assignments.insert(field_name, &assign.value);
                                }
                            }
                        }
                    }
                }
            }
            
            // Generate field initializations from the body assignments
            for member in &class.members {
                if let Member::Field(field) = member {
                    let field_name = self.sanitize_name(&field.name);
                    
                    if let Some(value_expr) = field_assignments.get(&field_name) {
                        // Use the value from the constructor body
                        self.write_indent();
                        write!(self.output, "{}: ", field_name).unwrap();
                        
                        // Check if value is a string literal and needs .to_string()
                        let needs_to_string = matches!(value_expr, Expr::Literal(Literal::String(_)));
                        
                        self.generate_expr(value_expr)?;
                        
                        if needs_to_string {
                            self.output.push_str(".to_string()");
                        }
                        self.output.push_str(",\n");
                    } else {
                        // Field not assigned in constructor - use default value
                        let default_value = if field.is_optional {
                            "None".to_string()
                        } else if let Some(init_expr) = &field.init {
                            // Field has an initializer in its definition
                            let mut value = String::new();
                            let needs_string_conversion = matches!(init_expr, Expr::Literal(Literal::String(_)))
                                && field.type_ref.as_ref().map(|t| matches!(t, TypeRef::Simple(s) if s == "string" || s == "String")).unwrap_or(false);
                            
                            // Generate the init expression to a temporary buffer
                            let old_output = std::mem::take(&mut self.output);
                            self.generate_expr(init_expr)?;
                            value = std::mem::replace(&mut self.output, old_output);
                            
                            if needs_string_conversion {
                                format!("{}.to_string()", value)
                            } else {
                                value
                            }
                        } else {
                            match field.type_ref.as_ref() {
                                Some(type_ref) => match type_ref {
                                    TypeRef::Simple(name) => match name.as_str() {
                                        "number" | "int" => "0".to_string(),
                                        "float" => "0.0".to_string(),
                                        "string" => "String::new()".to_string(),
                                        "bool" => "false".to_string(),
                                        "char" => "'\\0'".to_string(),
                                        _ => "Default::default()".to_string(),
                                    },
                                    TypeRef::Array(_) => "Vec::new()".to_string(),
                                    _ => "Default::default()".to_string(),
                                },
                                None => "Default::default()".to_string(),
                            }
                        };
                        self.write_indent();
                        self.writeln(&format!("{}: {},", field_name, default_value));
                    }
                }
            }

            self.dedent();
            self.write_indent();
            self.output.push_str("}\n");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');
        } else if has_fields {
            // Default constructor
            self.writeln(&format!("pub fn new() -> Self {{"));
            self.indent();
            self.writeln("Self {");
            self.indent();

            for member in &class.members {
                if let Member::Field(field) = member {
                    self.write_indent();
                    let field_name = self.sanitize_name(&field.name);
                    
                    // Use explicit init value if provided, otherwise use defaults
                    if let Some(init_expr) = &field.init {
                        write!(self.output, "{}: ", field_name).unwrap();
                        
                        // Check if we need to convert string literal to String
                        let needs_string_conversion = matches!(init_expr, Expr::Literal(Literal::String(_)))
                            && field.type_ref.as_ref().map(|t| matches!(t, TypeRef::Simple(s) if s == "string" || s == "String")).unwrap_or(false);
                        
                        // If field is optional, wrap the init value in Some()
                        if field.is_optional {
                            self.output.push_str("Some(");
                        }
                        
                        if needs_string_conversion {
                            self.generate_expr(init_expr)?;
                            self.output.push_str(".to_string()");
                        } else {
                            self.generate_expr(init_expr)?;
                        }
                        
                        if field.is_optional {
                            self.output.push_str(")");
                        }
                        
                        self.output.push_str(",\n");
                    } else {
                        // Optional fields should default to None
                        let default_value = if field.is_optional {
                            "None".to_string()
                        } else {
                            match field.type_ref.as_ref() {
                                Some(type_ref) => match type_ref {
                                    TypeRef::Simple(name) => match name.as_str() {
                                        "number" | "int" => "0".to_string(),
                                        "float" => "0.0".to_string(),
                                        "string" => "String::new()".to_string(),
                                        "bool" => "false".to_string(),
                                        "char" => "'\\0'".to_string(),
                                        _ => "Default::default()".to_string(),
                                    },
                                    _ => "Default::default()".to_string(),
                                },
                                None => "Default::default()".to_string(),
                            }
                        };
                        self.writeln(&format!("{}: {},", field_name, default_value));
                    }
                }
            }

            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');
        }

        // Generate other methods (excluding constructor)
        for member in &class.members {
            if let Member::Method(method) = member {
                if method.name != "constructor" {
                    self.write_indent();
                    write!(self.output, "// Generating method: {}\n", method.name).unwrap();
                    self.generate_method(method, Some(class))?;
                    self.output.push('\n');
                }
            }
        }

        self.dedent();
        self.writeln("}");

        Ok(())
    }

    fn generate_field_default_function(&mut self, class_name: &str, field: &FieldDecl) -> Result<()> {
        let field_name = self.sanitize_name(&field.name);
        let func_name = format!("default_{}_{}", class_name.to_lowercase(), field_name);
        
        let base_type = if let Some(type_ref) = &field.type_ref {
            type_ref.to_rust_type()
        } else {
            "String".to_string()
        };
        
        self.writeln(&format!("fn {}() -> Option<{}>{{", func_name, base_type));
        self.indent();
        
        if let Some(init_expr) = &field.init {
            self.write_indent();
            self.output.push_str("Some(");
            
            // Check if we need to convert string literal to String
            let needs_string_conversion = matches!(init_expr, Expr::Literal(Literal::String(_)))
                && field.type_ref.as_ref().map(|t| matches!(t, TypeRef::Simple(s) if s == "string" || s == "String")).unwrap_or(false);
            
            if needs_string_conversion {
                self.generate_expr(init_expr)?;
                self.output.push_str(".to_string()");
            } else {
                self.generate_expr(init_expr)?;
            }
            
            self.output.push_str(")\n");
        }
        
        self.dedent();
        self.writeln("}");
        self.output.push('\n');
        
        Ok(())
    }

    fn generate_field(&mut self, field: &FieldDecl, needs_serde: bool, class_name: Option<&str>) -> Result<()> {
        let vis = match field.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
        };

        let base_type = if let Some(type_ref) = &field.type_ref {
            type_ref.to_rust_type()
        } else {
            "()".to_string()
        };

        // Wrap in Option<T> if field is optional
        let type_str = if field.is_optional {
            format!("Option<{}>", base_type)
        } else {
            base_type
        };

        let field_name_rust = self.sanitize_name(&field.name);
        
        // Add serde attributes for optional fields
        if needs_serde && field.is_optional {
            // If the field has a default value, use serde default function
            if field.init.is_some() && class_name.is_some() {
                let func_name = format!("default_{}_{}", class_name.unwrap().to_lowercase(), field_name_rust);
                self.writeln(&format!("#[serde(default = \"{}\")]", func_name));
            }
            self.writeln("#[serde(skip_serializing_if = \"Option::is_none\")]");
        }
        
        // If serde is needed and the original name differs from snake_case, add rename attribute
        if needs_serde && field.name != field_name_rust {
            self.writeln(&format!("#[serde(rename = \"{}\")]", field.name));
        }
        
        self.writeln(&format!(
            "{}{}: {},",
            vis,
            field_name_rust,
            type_str
        ));
        Ok(())
    }

    fn infer_expr_type(&self, expr: &Expr, class: Option<&ClassDecl>) -> Option<String> {
        match expr {
            Expr::Tuple(elements) => {
                // Infer tuple type from element expressions
                let mut element_types = Vec::new();
                for elem in elements {
                    if let Some(type_str) = self.infer_expr_type(elem, class) {
                        // Extract just the type part (remove " -> " prefix)
                        let type_part = type_str.trim_start_matches(" -> ");
                        element_types.push(type_part.to_string());
                    } else {
                        // If we can't infer an element type, fall back to generic
                        element_types.push("i32".to_string());
                    }
                }
                if !element_types.is_empty() {
                    Some(format!(" -> ({})", element_types.join(", ")))
                } else {
                    Some(" -> ()".to_string()) // Empty tuple
                }
            }
            Expr::Member { object, property } => {
                // Check if this is accessing a field of 'this'
                if let Expr::Identifier(obj) = object.as_ref() {
                    if obj == "this" && class.is_some() {
                        // Find the field type
                        for member in &class.unwrap().members {
                            if let Member::Field(field) = member {
                                if field.name == *property {
                                    let rust_type = field
                                        .type_ref
                                        .as_ref()
                                        .map(|t| t.to_rust_type())
                                        .unwrap_or_else(|| "String".to_string());
                                    // Return owned type - cloning will be handled in code generation
                                    return Some(format!(" -> {}", rust_type));
                                }
                            }
                        }
                    }
                }
                None
            }
            Expr::Binary { op, .. } => {
                // Simple heuristics for binary operations
                match op {
                    BinOp::Lt
                    | BinOp::Le
                    | BinOp::Gt
                    | BinOp::Ge
                    | BinOp::Eq
                    | BinOp::Ne
                    | BinOp::And
                    | BinOp::Or => Some(" -> bool".to_string()),
                    _ => None,
                }
            }
            Expr::Literal(lit) => match lit {
                Literal::String(_) => Some(" -> String".to_string()), // String literals converted to String in tuples
                Literal::Int(_) => Some(" -> i32".to_string()),
                Literal::Float(_) => Some(" -> f64".to_string()),
                Literal::Bool(_) => Some(" -> bool".to_string()),
                _ => None,
            },
            // String templates (format! calls) return String
            Expr::Call(call) => {
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    if name.starts_with("$") || name == "format" {
                        return Some(" -> String".to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn infer_param_type_from_class(
        &self,
        param_name: &str,
        class: &ClassDecl,
        method_name: Option<&str>,
    ) -> Option<String> {
        // Determine candidate field name based on method/prefix
        let field_name = if let Some(method) = method_name {
            if method.starts_with("set") || method.starts_with("get") {
                let without_prefix = if method.starts_with("set") {
                    method.strip_prefix("set").unwrap_or(method)
                } else if method.starts_with("get") {
                    method.strip_prefix("get").unwrap_or(method)
                } else {
                    method
                };
                let mut chars = without_prefix.chars();
                if let Some(first) = chars.next() {
                    format!("{}{}", first.to_lowercase(), chars.as_str())
                } else {
                    param_name.to_string()
                }
            } else {
                param_name.to_string()
            }
        } else {
            param_name.to_string()
        };

        // Look for matching field in class
        if let Some(fields) = self.class_fields.get(&class.name) {
            if fields.contains(&field_name) || fields.contains(&format!("_{}", field_name)) {
                // Search for type in class members
                for m in &class.members {
                    if let Member::Field(f) = m {
                        if f.name == field_name || f.name == format!("_{}", field_name) {
                            return f.type_ref.as_ref().map(|t| t.to_rust_type());
                        }
                    }
                }
                // Fallback for common field names
                return Some(match field_name.as_str() {
                    "name" => "String".to_string(),
                    "age" => "i32".to_string(),
                    _ => "i32".to_string(),
                });
            }
        }
        None
    }

    #[allow(dead_code)]
    fn generate_constructor_method(&mut self, method: &MethodDecl) -> Result<()> {
        let vis = "pub";
        let _async_kw = "";
        let _type_params = String::new();

        let params_str = self.generate_params(&method.params, false, None, None, None)?; // false because constructor is not a method

        // Constructor returns Self
        let return_type = " -> Self".to_string();

        self.write_indent();
        write!(
            self.output,
            "{}{}fn {}({}){}",
            vis,
            if vis.is_empty() { "" } else { " " },
            "new", // Always generate as new()
            params_str,
            return_type
        )
        .unwrap();

        // For custom constructor, map parameters to fields by name
        // Assume parameter names match field names
        self.output.push_str(" {\n");
        self.indent();
        self.write_indent();
        self.output.push_str("Self {\n");
        self.indent();

        // Generate field assignments based on parameters
        for param in &method.params {
            self.write_indent();
            let field_name = self.sanitize_name(param.name().unwrap());
            // Add conversion for string fields
            if param.name().unwrap() == "name" {
                self.output
                    .push_str(&format!("{}: {}.to_string()", field_name, field_name));
            } else {
                self.output
                    .push_str(&format!("{}: {}", field_name, field_name));
            }
            self.output.push_str(",\n");
        }

        // Add default values for fields not covered by parameters
        // For now, assume all fields are covered by parameters

        self.dedent();
        self.write_indent();
        self.output.push_str("}\n");
        self.dedent();
        self.writeln("}");

        Ok(())
    }

    fn generate_method(&mut self, method: &MethodDecl, class: Option<&ClassDecl>) -> Result<()> {
        let vis = match method.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
        };

        let async_kw = if method.is_async_inferred {
            "async "
        } else {
            ""
        };

        let type_params = if !method.type_params.is_empty() {
            let bounded: Vec<String> = method
                .type_params
                .iter()
                .map(|param| {
                    if !param.constraints.is_empty() {
                        // Use trait registry to get complete Rust trait bounds
                        let rust_bounds = self.trait_registry.generate_rust_bounds(&param.constraints);
                        format!("{}{}", param.name, rust_bounds)
                    } else {
                        param.name.clone()
                    }
                })
                .collect();
            format!("<{}>", bounded.join(", "))
        } else {
            String::new()
        };

        let params_str = self.generate_params(&method.params, true, class, Some(&method.name), Some(method))?;

        let return_type = if let Some(ret) = &method.return_type {
            format!(" -> {}", ret.to_rust_type())
        } else {
            // First, try to find return type from implemented interfaces
            let interface_return_type = class.and_then(|c| {
                for iface_name in &c.implements {
                    if let Some(methods) = self.interface_methods.get(iface_name) {
                        if let Some(ret_type) = methods.get(&method.name) {
                            return Some(format!(" -> {}", ret_type.to_rust_type()));
                        }
                    }
                }
                None
            });
            
            if let Some(ret) = interface_return_type {
                ret
            } else if let Some(expr) = &method.expr_body {
                // Try to infer return type from expression
                self.infer_expr_type(expr, class)
                    .unwrap_or_else(|| " -> ()".to_string())
            } else {
                String::new()
            }
        };

        self.write_indent();
        write!(
            self.output,
            "{}{}fn {}{}({}){}",
            vis,
            async_kw,
            self.sanitize_name(&method.name),
            type_params,
            params_str,
            return_type
        )
        .unwrap();

        self.in_method = true;
        if let Some(expr) = &method.expr_body {
            self.output.push_str(" {\n");
            self.indent();
            
            // Generate destructuring code for parameters
            self.generate_param_destructuring(&method.params)?;
            
            self.write_indent();
            self.generate_expr(expr)?;
            self.output.push('\n');
            self.dedent();
            self.writeln("}");
        } else if let Some(body) = &method.body {
            self.output.push_str(" {\n");
            self.indent();
            
            // Generate destructuring code for parameters
            self.generate_param_destructuring(&method.params)?;
            
            self.generate_block_inner(body)?;
            self.dedent();
            self.writeln("}");
        }
        self.in_method = false;

        Ok(())
    }

    fn generate_function(&mut self, func: &FunctionDecl) -> Result<()> {
        let (async_kw, tokio_attr) = if func.name == "main" && func.is_async_inferred {
            // For main function with async, use tokio::main attribute with async keyword
            ("async ", "#[tokio::main]\n")
        } else if func.is_async_inferred {
            ("async ", "")
        } else {
            ("", "")
        };

        let type_params = if !func.type_params.is_empty() {
            let bounded: Vec<String> = func
                .type_params
                .iter()
                .map(|param| {
                    if !param.constraints.is_empty() {
                        // Use trait registry to get complete Rust trait bounds
                        let rust_bounds = self.trait_registry.generate_rust_bounds(&param.constraints);
                        format!("{}{}", param.name, rust_bounds)
                    } else {
                        param.name.clone()
                    }
                })
                .collect();
            format!("<{}>", bounded.join(", "))
        } else {
            String::new()
        };
        let params_str = self.generate_params(&func.params, false, None, None, None)?;

        // Handle fallibility - wrap return type in Result if function contains fail
        let return_type = if func.contains_fail {
            if let Some(ret) = &func.return_type {
                format!(" -> Result<{}, liva_rt::Error>", ret.to_rust_type())
            } else if let Some(expr) = &func.expr_body {
                let inner_type = self
                    .infer_expr_type(expr, None)
                    .unwrap_or_else(|| " -> i32".to_string())
                    .trim_start_matches(" -> ")
                    .to_string();
                format!(" -> Result<{}, liva_rt::Error>", inner_type)
            } else {
                " -> Result<(), liva_rt::Error>".to_string()
            }
        } else {
            if let Some(ret) = &func.return_type {
                format!(" -> {}", ret.to_rust_type())
            } else if let Some(expr) = &func.expr_body {
                // For expression-bodied functions without explicit return type, infer from the expression
                self.infer_expr_type(expr, None)
                    .unwrap_or_else(|| " -> i32".to_string())
            } else if func.body.is_some() {
                // For block-bodied functions, try to infer from return statements
                if let Some(body) = &func.body {
                    if self.block_has_return(body) {
                        // Try to infer type from return statement
                        self.infer_return_type_from_block(body)
                            .unwrap_or_else(|| " -> f64".to_string()) // Default to f64 as fallback
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        };

        write!(
            self.output,
            "{}{}fn {}{}({})",
            tokio_attr,
            async_kw,
            self.sanitize_name(&func.name),
            type_params,
            params_str
        )
        .unwrap();

        if !return_type.is_empty() {
            write!(self.output, "{}", return_type).unwrap();
        }

        if let Some(expr) = &func.expr_body {
            self.output.push_str(" {\n");
            self.indent();
            
            // Generate destructuring code for parameters
            self.generate_param_destructuring(&func.params)?;
            
            self.write_indent();
            let was_fallible = self.in_fallible_function;
            self.in_fallible_function = func.contains_fail;
            if func.contains_fail {
                // Check if the expression already returns a Result (like a fallible ternary)
                let expr_returns_result = matches!(expr, Expr::Ternary { then_expr, else_expr, .. }
                    if self.expr_contains_fail(then_expr) || self.expr_contains_fail(else_expr));

                if !expr_returns_result {
                    self.output.push_str("Ok(");
                }
                self.generate_expr(expr)?;
                if !expr_returns_result {
                    self.output.push(')');
                }
            } else {
                self.generate_expr(expr)?;
            }
            self.in_fallible_function = was_fallible;
            self.output.push('\n');

            // Phase 4.2: Check for dead tasks
            self.check_dead_tasks();
            self.pending_tasks.clear();

            self.dedent();
            self.writeln("}");
        } else if let Some(body) = &func.body {
            self.output.push_str(" {\n");
            self.indent();
            
            // Generate destructuring code for parameters
            self.generate_param_destructuring(&func.params)?;
            
            let was_fallible = self.in_fallible_function;
            self.in_fallible_function = func.contains_fail;
            self.generate_block_inner(body)?;
            // If function is fallible and doesn't end with explicit return, add Ok(())
            if func.contains_fail && !self.block_ends_with_return(body) {
                self.write_indent();
                self.writeln("Ok(())");
            }
            self.in_fallible_function = was_fallible;

            // Phase 4.2: Check for dead tasks (tasks that were never awaited)
            self.check_dead_tasks();

            // Clear pending tasks for next function
            self.pending_tasks.clear();

            self.dedent();
            self.writeln("}");
        }

        Ok(())
    }

    fn generate_test(&mut self, test: &TestDecl) -> Result<()> {
        self.writeln("#[test]");
        self.writeln(&format!(
            "fn test_{}() {{",
            self.sanitize_test_name(&test.name)
        ));
        self.indent();
        self.generate_block_inner(&test.body)?;
        self.dedent();
        self.writeln("}");
        Ok(())
    }

    fn generate_params(
        &mut self,
        params: &[Param],
        is_method: bool,
        class: Option<&ClassDecl>,
        method_name: Option<&str>,
        method: Option<&MethodDecl>,
    ) -> Result<String> {
        let mut result = String::new();

        if is_method {
            // Use &mut self for methods that modify fields
            let is_setter = method_name.map_or(false, |name| name.starts_with("set"));
            let modifies_self = method.map_or(false, |m| self.method_modifies_self(m));
            
            if is_setter || modifies_self {
                result.push_str("&mut self");
            } else {
                result.push_str("&self");
            }
            if !params.is_empty() {
                result.push_str(", ");
            }
        }

        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }

            // For destructured parameters, use temporary names (_param_0, _param_1, etc.)
            // Otherwise use the actual parameter name
            let param_name = if param.is_destructuring() {
                format!("_param_{}", i)
            } else {
                self.sanitize_name(param.name().unwrap())
            };
            
            let type_str = if let Some(type_ref) = &param.type_ref {
                let rust_type = type_ref.to_rust_type();
                
                // Register parameter as class instance if its type is a known class
                if !param.is_destructuring() {
                    let type_name = match type_ref {
                        TypeRef::Simple(name) => Some(name.clone()),
                        _ => None,
                    };
                    if let Some(tname) = &type_name {
                        // Track string parameters for proper .length -> .len() translation
                        if matches!(tname.as_str(), "string" | "String") {
                            self.string_vars.insert(param_name.clone());
                        }
                        // Check if this is a class type (starts with uppercase, not primitive)
                        else if !matches!(tname.as_str(), "number" | "i32" | "i64" | "f64" | "bool" | "char" | "Vec" | "Option") 
                           && tname.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) 
                        {
                            self.class_instance_vars.insert(param_name.clone());
                            self.var_types.insert(param_name.clone(), tname.clone());
                        }
                    }
                }
                
                rust_type
            } else if let Some(cls) = class {
                // Try to infer from field types in the class
                // For destructured params, use the type annotation if present
                if param.is_destructuring() {
                    "serde_json::Value".to_string() // Default for destructured params without type
                } else {
                    self.infer_param_type_from_class(param.name().unwrap(), cls, method_name)
                        .unwrap_or_else(|| "i32".to_string())
                }
            } else {
                // Infer type based on parameter name (hack for constructor)
                if param.is_destructuring() {
                    "serde_json::Value".to_string() // Default for destructured params without type
                } else {
                    match param.name().unwrap() {
                        "name" => "String".to_string(),
                        "age" => "i32".to_string(),
                        "items" => "Vec<serde_json::Value>".to_string(),
                        _ => "i32".to_string(),
                    }
                }
            };

            write!(result, "{}: {}", param_name, type_str).unwrap();
        }

        Ok(result)
    }

    fn generate_block_inner(&mut self, block: &BlockStmt) -> Result<()> {
        for stmt in &block.stmts {
            self.generate_stmt(stmt)?;
        }
        Ok(())
    }

    fn generate_if_body(&mut self, body: &IfBody) -> Result<()> {
        match body {
            IfBody::Block(block) => {
                for stmt in &block.stmts {
                    self.generate_stmt(stmt)?;
                }
            }
            IfBody::Stmt(stmt) => {
                self.generate_stmt(stmt)?;
            }
        }
        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        // Phase 4: Check if this statement uses multiple pending tasks (join combining optimization)
        let used_tasks = self.stmt_uses_pending_tasks(stmt);

        if used_tasks.len() > 1 {
            // Multiple tasks used - generate tokio::join! for parallel await
            self.generate_tasks_join(&used_tasks)?;
        } else if used_tasks.len() == 1 {
            // Single task - use regular await (Phase 2 behavior)
            self.generate_task_await(&used_tasks[0])?;
        }
        // Phase 2 fallback: Check if this statement uses a pending task for the first time
        // (This is kept for backwards compatibility, but should not trigger if Phase 4 works)
        else if let Some(var_name) = self.stmt_uses_pending_task(stmt) {
            self.generate_task_await(&var_name)?;
        }

        match stmt {
            Stmt::VarDecl(var) => {
                self.write_indent();

                // Phase 2: Check if init expression is a Task (async/par call)
                let task_exec_policy = self.is_task_expr(&var.init);

                if var.bindings.len() > 1 {
                    // Multiple bindings - fallible pattern (error binding)
                    let is_fallible_call = self.is_fallible_expr(&var.init);

                    // Collect binding names for tracking
                    let binding_names: Vec<String> = var
                        .bindings
                        .iter()
                        .filter_map(|b| b.name().map(|n| self.sanitize_name(n)))
                        .collect();

                    // Check if the init expression returns a tuple directly (before tracking)
                    let returns_tuple = self.is_builtin_conversion_call(&var.init);
                    
                    // Check if this is an await of HTTP task (also returns tuple with String error)
                    let is_await_http = self.is_await_http_task(&var.init).is_some();
                    
                    // Phase 1: Check if this is typed JSON.parse (returns direct values, not Option)
                    let is_typed_json_parse = self.is_json_parse_call(&var.init) 
                        && var.bindings.first().and_then(|b| b.type_ref.as_ref()).is_some();

                    // Phase 3: Track the error variable (second binding) as Option<String>
                    // BUT: Only track as Option if NOT a tuple-returning function AND NOT typed JSON.parse
                    // AND NOT an await of HTTP task (which returns String error, not Option)
                    // Tuple functions return (Option<T>, String) - err is String, not Option
                    // Typed JSON.parse returns (T, String) - value is T, not Option<T>
                    if binding_names.len() == 2 && !returns_tuple && !is_typed_json_parse && !is_await_http {
                        self.error_binding_vars.insert(binding_names[1].clone());
                        self.option_value_vars.insert(binding_names[0].clone()); // Also track the value (first binding)
                    } else if binding_names.len() == 2 && (returns_tuple || is_typed_json_parse) {
                        // For tuple-returning functions AND typed JSON.parse: response is T (not Option), err is String
                        // Don't add to option_value_vars or error_binding_vars
                        
                        // Track the error variable (second binding) as string_error_vars for `if err` sugar
                        self.string_error_vars.insert(binding_names[1].clone());
                        
                        // Check if this is an HTTP call - mark first binding as rust_struct
                        if self.is_http_call(&var.init) {
                            self.rust_struct_vars.insert(binding_names[0].clone());
                        }
                        
                        // Check if this is typed JSON.parse with array type - track element type
                        if is_typed_json_parse {
                            if let Some(first_binding) = var.bindings.first() {
                                if let Some(type_ref) = &first_binding.type_ref {
                                    // Check if it's an array type like [Post] or [string]
                                    if let TypeRef::Array(element_type) = type_ref {
                                        match element_type.as_ref() {
                                            TypeRef::Simple(type_name) => {
                                                // Track arrays of classes and strings (both need .cloned())
                                                self.typed_array_vars.insert(binding_names[0].clone(), type_name.clone());
                                            }
                                            _ => {}
                                        }
                                    }
                                    // Also track the data variable as an array for .length -> .len() conversion
                                    if matches!(type_ref, TypeRef::Array(_)) {
                                        self.array_vars.insert(binding_names[0].clone());
                                    }
                                }
                            }
                        }
                    }

                    if let Some(exec_policy) = task_exec_policy {
                        // Phase 2: Error binding with Task - store task without awaiting
                        // Generate: let task_name = async/par call();
                        let task_var_name = format!("{}_task", binding_names[0]);
                        write!(self.output, "let {} = ", task_var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str(";\n");

                        // Register as pending task with error binding
                        let is_http = self.is_http_call(&var.init);
                        self.pending_tasks.insert(
                            binding_names[0].clone(),
                            TaskInfo {
                                is_error_binding: true,
                                binding_names: binding_names.clone(),
                                awaited: false,
                                exec_policy,
                                returns_tuple,
                                is_http_call: is_http,
                            },
                        );
                    } else {
                        // Non-Task error binding (original behavior)
                        write!(self.output, "let (").unwrap();
                        for (i, binding) in var.bindings.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            if let Some(name) = binding.name() {
                                write!(self.output, "{}", self.sanitize_name(name)).unwrap();
                            }
                        }

                        if is_fallible_call {
                            // Generate: let (value, err) = match expr { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };
                            self.output.push_str(") = match ");
                            self.generate_expr(&var.init)?;
                            self.output.push_str(" { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };\n");
                        } else {
                            // Check if the expression is a built-in conversion function that returns a tuple
                            let returns_tuple = self.is_builtin_conversion_call(&var.init);
                            
                            // Phase 1: Check if this is JSON.parse with type hint
                            let is_json_parse = self.is_json_parse_call(&var.init);
                            let has_type_hint = var.bindings.first().and_then(|b| b.type_ref.as_ref()).is_some();
                            
                            if is_json_parse && has_type_hint {
                                // Typed JSON parsing with error binding: let nums: [i32], err = JSON.parse("[1,2,3]")
                                // Generate: let (nums, err): (Vec<i32>, String) = match serde_json::from_str::<Vec<i32>>(...) { Ok(v) => (v, String::new()), Err(e) => (Vec::new(), format!("{}", e)) };
                                let type_ref = var.bindings.first().unwrap().type_ref.as_ref().unwrap();
                                let rust_type = self.expand_type_alias(type_ref);
                                
                                write!(self.output, "): ({}, String) = match ", rust_type).unwrap();
                                
                                if let Expr::MethodCall(method_call) = &var.init {
                                    self.generate_typed_json_parse(method_call, type_ref)?;
                                }
                                
                                // Generate default value for error case
                                let default_value = match type_ref {
                                    TypeRef::Array(_) => "Vec::new()".to_string(),
                                    TypeRef::Optional(_) => "None".to_string(),
                                    TypeRef::Simple(name) => match name.as_str() {
                                        "int" | "i8" | "i16" | "i32" | "i64" | "i128" |
                                        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "isize" => "0".to_string(),
                                        "float" | "f32" | "f64" => "0.0".to_string(),
                                        "bool" => "false".to_string(),
                                        "string" | "String" => "String::new()".to_string(),
                                        _ => "Default::default()".to_string(),
                                    },
                                    _ => "Default::default()".to_string(),
                                };
                                
                                write!(self.output, " {{ Ok(v) => (v, String::new()), Err(e) => ({}, format!(\"JSON parse error: {{}}\", e)) }};\n", default_value).unwrap();
                                
                                // Phase 2.2: Track class instances for proper member access codegen
                                if let TypeRef::Simple(class_name) = type_ref {
                                    // Check if this is a class type (not a primitive)
                                    if self.class_fields.contains_key(class_name) {
                                        let binding = &var.bindings[0];
                                        if let Some(name) = binding.name() {
                                            self.class_instance_vars.insert(self.sanitize_name(name));
                                        }
                                    }
                                } else if let TypeRef::Array(elem_type) = type_ref {
                                    // Track array variable and element type
                                    let binding = &var.bindings[0];
                                    if let Some(name) = binding.name() {
                                        let sanitized_name = self.sanitize_name(name);
                                        self.array_vars.insert(sanitized_name.clone());
                                        
                                        // Track element type (class, string, etc.) for proper forEach/map patterns
                                        if let TypeRef::Simple(type_name) = elem_type.as_ref() {
                                            // Track in typed_array_vars for both classes and primitives like "string"
                                            // This ensures forEach uses correct lambda pattern |p| instead of |&p|
                                            self.typed_array_vars.insert(sanitized_name.clone(), type_name.clone());
                                            
                                            // Also track class instances for member access
                                            if self.class_fields.contains_key(type_name) {
                                                // Already handled by typed_array_vars
                                            }
                                        }
                                    }
                                }
                            } else if is_json_parse && !has_type_hint {
                                // Untyped JSON parsing with error binding: let data, err = JSON.parse(body)
                                // JSON.parse returns (Option<JsonValue>, String)
                                // Generate: let (data, err) = expr;
                                self.output.push_str(") = ");
                                self.generate_expr(&var.init)?;
                                self.output.push_str(";\n");
                                
                                // Track this variable as Option<JsonValue> so we can unwrap it before field access
                                if let Some(first_binding) = var.bindings.first() {
                                    if let Some(name) = first_binding.name() {
                                        let sanitized = self.sanitize_name(name);
                                        self.option_value_vars.insert(sanitized.clone());
                                        self.json_value_vars.insert(sanitized);
                                    }
                                }
                            } else if self.is_http_call(&var.init) {
                                // HTTP call without async - generate direct .await
                                // let response, err = HTTP.get(url)
                                // Generate: let (response, err) = { let (opt, err) = liva_http_get(url).await; (opt.unwrap_or_default(), err) };
                                self.output.push_str(") = { let (opt, err) = ");
                                self.generate_expr(&var.init)?;
                                self.output.push_str(".await; (opt.unwrap_or_default(), err) };\n");
                                
                                // Track the response variable as rust_struct
                                if let Some(first_binding) = var.bindings.first() {
                                    if let Some(name) = first_binding.name() {
                                        self.rust_struct_vars.insert(self.sanitize_name(name));
                                    }
                                }
                                // Track the error variable as string_error_vars (for `if err` sugar)
                                if var.bindings.len() >= 2 {
                                    if let Some(name) = var.bindings[1].name() {
                                        self.string_error_vars.insert(self.sanitize_name(name));
                                    }
                                }
                            } else if self.is_file_call(&var.init) {
                                // File call - returns (Option<T>, String)
                                // let content, err = File.read(path)
                                // Generate: let (content, err) = { let (opt, err) = expr; (opt.unwrap_or_default(), err) };
                                self.output.push_str(") = { let (opt, err) = ");
                                self.generate_expr(&var.init)?;
                                self.output.push_str("; (opt.unwrap_or_default(), err) };\n");
                                // Track the error variable as string_error_vars (for `if err` sugar)
                                if var.bindings.len() >= 2 {
                                    if let Some(name) = var.bindings[1].name() {
                                        self.string_error_vars.insert(self.sanitize_name(name));
                                    }
                                }
                            } else if let Some(task_name) = self.is_await_http_task(&var.init) {
                                // Await of pending HTTP task - unwrap JoinHandle and extract result
                                // let res, err = await task1
                                // Generate: let (res, err) = { let (opt, err) = task1_task.await.unwrap(); (opt.unwrap_or_default(), err) };
                                write!(self.output, ") = {{ let (opt, err) = {}_task.await.unwrap(); (opt.unwrap_or_default(), err) }};\n", task_name).unwrap();
                                
                                // Track the response variable as rust_struct
                                if let Some(first_binding) = var.bindings.first() {
                                    if let Some(name) = first_binding.name() {
                                        self.rust_struct_vars.insert(self.sanitize_name(name));
                                    }
                                }
                                // Track the error variable as string_error_vars (for `if err` sugar)
                                if var.bindings.len() >= 2 {
                                    if let Some(name) = var.bindings[1].name() {
                                        self.string_error_vars.insert(self.sanitize_name(name));
                                    }
                                }
                                // Mark task as awaited
                                if let Some(task_info) = self.pending_tasks.get_mut(&task_name) {
                                    task_info.awaited = true;
                                }
                            } else if returns_tuple {
                                // Built-in conversion functions (parseInt, parseFloat) already return (value, Option<Error>)
                                // Generate: let (value, err) = expr;
                                self.output.push_str(") = ");
                                self.generate_expr(&var.init)?;
                                self.output.push_str(";\n");
                            } else {
                                // Non-fallible function called with fallible binding pattern
                                // Generate: let (value, err) = (expr, None);
                                self.output.push_str("): (_, Option<liva_rt::Error>) = (");
                                self.generate_expr(&var.init)?;
                                self.output.push_str(", None);\n");
                            }
                        }
                    }
                } else {
                    // Normal binding: let a = expr (only one binding expected)
                    if var.bindings.len() != 1 {
                        return Err(CompilerError::CodegenError(
                            SemanticErrorInfo::new(
                                "E3000",
                                "Invalid binding pattern",
                                "Let statement should have exactly one binding when not using fallible pattern"
                            )
                            .with_help("Use fallible binding pattern 'let result, err = ...' or single binding 'let result = ...'")
                        ));
                    }
                    let binding = &var.bindings[0];
                    
                    // Check if this is a destructuring pattern
                    if !binding.pattern.is_simple() {
                        // Handle destructuring patterns
                        self.generate_destructuring_pattern(&binding.pattern, &var.init)?;
                        return Ok(());
                    }
                    
                    // Simple identifier binding
                    let var_name = self.sanitize_name(binding.name().unwrap());

                    // Phase 2: Check if this is a Task assignment
                    if let Some(exec_policy) = task_exec_policy {
                        // Simple task binding (no error handling)
                        // Generate: let var_name_task = async/par call();
                        let task_var_name = format!("{}_task", var_name);
                        write!(self.output, "let {} = ", task_var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str(";\n");

                        // Check if the inner call is HTTP (for await unwrapping later)
                        let is_http = self.is_http_call_in_async(&var.init);

                        // Register as pending task
                        self.pending_tasks.insert(
                            var_name.clone(),
                            TaskInfo {
                                is_error_binding: false,
                                binding_names: vec![var_name.clone()],
                                awaited: false,
                                exec_policy,
                                returns_tuple: false, // Simple binding, no tuple destructuring
                                is_http_call: is_http,
                            },
                        );
                    } else {
                        // Non-Task normal binding (original behavior)

                        // Check if initializing with a string literal or string expression - mark variable as string
                        if let Expr::Literal(Literal::String(_)) = &var.init {
                            if let Some(name) = binding.name() {
                                self.string_vars.insert(self.sanitize_name(name));
                            }
                        }
                        // Also mark as string if initialized with a string concatenation
                        else if self.expr_is_stringy(&var.init) {
                            if let Some(name) = binding.name() {
                                self.string_vars.insert(self.sanitize_name(name));
                            }
                        }

                        // Check if initializing with an object literal - mark variable for bracket notation
                        if let Expr::ObjectLiteral(_) = &var.init {
                            if let Some(name) = binding.name() {
                                self.bracket_notation_vars.insert(name.to_string());
                            }
                        }

                        // Check if initializing with an array literal - mark variable as array
                        if let Expr::ArrayLiteral(elements) = &var.init {
                            if let Some(name) = binding.name() {
                                self.array_vars.insert(name.to_string());
                                
                                // If array contains anonymous objects, mark as json_value
                                // e.g., let users = [{ id: 1, name: "Alice" }, { id: 2, name: "Bob" }]
                                if !elements.is_empty() {
                                    if matches!(elements[0], Expr::ObjectLiteral(_)) {
                                        self.json_value_vars.insert(name.to_string());
                                    }
                                    // If array contains class instances, track the element type
                                    // e.g., let items = [Item("one", true), Item("two", false)]
                                    else if let Expr::Call(call) = &elements[0] {
                                        if let Expr::Identifier(class_name) = &*call.callee {
                                            // Check if first letter is uppercase (likely a class)
                                            if class_name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                                                self.typed_array_vars.insert(name.to_string(), class_name.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // Check if initializing with a method call that returns an array (map, filter, etc.)
                        // or Option (find)
                        else if let Expr::MethodCall(method_call) = &var.init {
                            if matches!(method_call.method.as_str(), "map" | "filter") {
                                if let Some(name) = binding.name() {
                                    self.array_vars.insert(name.to_string());
                                    
                                    // If the method is called on a JsonValue, the result is also Vec<JsonValue>
                                    // BUT we need to mark it as json_value for proper forEach iteration
                                    if self.is_json_value_expr(&method_call.object) {
                                        self.json_value_vars.insert(name.to_string());
                                    }
                                }
                            }
                            // Bug #35: .split() returns array of strings
                            else if method_call.method.as_str() == "split" {
                                if let Some(name) = binding.name() {
                                    self.array_vars.insert(name.to_string());
                                    self.typed_array_vars.insert(name.to_string(), "string".to_string());
                                }
                            }
                            // .find() returns Option<T> - mark variable as option_value_vars
                            else if method_call.method.as_str() == "find" {
                                if let Some(name) = binding.name() {
                                    self.option_value_vars.insert(name.to_string());
                                }
                            }
                            // Sys.args() returns Vec<String> - need direct indexing
                            else if method_call.method.as_str() == "args" {
                                if let Expr::Identifier(obj_name) = method_call.object.as_ref() {
                                    if obj_name == "Sys" {
                                        if let Some(name) = binding.name() {
                                            self.native_vec_string_vars.insert(name.to_string());
                                            self.array_vars.insert(name.to_string());
                                        }
                                    }
                                }
                            }
                            // .as_array() returns Vec<JsonValue> - mark as array for .length -> .len()
                            else if method_call.method.as_str() == "as_array" {
                                if let Some(name) = binding.name() {
                                    self.array_vars.insert(name.to_string());
                                }
                            }
                        }
                        // Mark instances created via constructor call: let x = ClassName(...)
                        else if let Expr::Call(call) = &var.init {
                            if let Expr::Identifier(class_name) = &*call.callee {
                                if let Some(name) = binding.name() {
                                    self.class_instance_vars.insert(name.to_string());
                                    self.var_types
                                        .insert(name.to_string(), class_name.clone());
                                }
                            }
                        }
                        // Mark instances created via struct literal: let x = ClassName { ... }
                        else if let Expr::StructLiteral { type_name, .. } = &var.init {
                            if let Some(name) = binding.name() {
                                self.class_instance_vars.insert(name.to_string());
                                self.var_types
                                    .insert(name.to_string(), type_name.clone());
                            }
                        }
                        // Mark variables initialized from JSON indexing as json_value
                        // e.g., let items = result["items"] where result is a JsonValue
                        else if let Expr::Index { object, .. } = &var.init {
                            // Check if the object being indexed is a JsonValue
                            if self.is_json_value_expr(object) {
                                if let Some(name) = binding.name() {
                                    self.json_value_vars.insert(name.to_string());
                                }
                            }
                        }

                        write!(self.output, "let mut {}", var_name).unwrap();

                        if let Some(type_ref) = &binding.type_ref {
                            let rust_type = self.expand_type_alias(type_ref);
                            write!(self.output, ": {}", rust_type).unwrap();
                            
                            // Track string type for .length -> .len() conversion
                            if matches!(type_ref, TypeRef::Simple(name) if name == "string") {
                                self.string_vars.insert(var_name.clone());
                            }
                            
                            // Bug #35 fix: Track array types for proper forEach/map lambda patterns
                            // e.g., let parts: [string] = text.split(",") should use |p| not |&p|
                            if let TypeRef::Array(elem_type) = type_ref {
                                self.array_vars.insert(var_name.clone());
                                if let TypeRef::Simple(type_name) = elem_type.as_ref() {
                                    self.typed_array_vars.insert(var_name.clone(), type_name.clone());
                                }
                            }
                        }

                        self.output.push_str(" = ");
                        
                        // Check if we need to wrap in a union variant
                        let (needs_union_close, mut needs_to_string) = if let Some(type_ref) = &binding.type_ref {
                            self.maybe_wrap_in_union(type_ref, &var.init)
                        } else {
                            (false, false)
                        };
                        
                        // Bug #17 fix: String literals should always be converted to String
                        // to avoid &str vs String type mismatch when variable is reassigned
                        if matches!(&var.init, Expr::Literal(Literal::String(_))) {
                            needs_to_string = true;
                        }
                        
                        // Phase 1: Check if this is JSON.parse with type hint (typed parsing)
                        let is_json_parse = self.is_json_parse_call(&var.init);
                        let has_type_hint = binding.type_ref.is_some();
                        
                        if is_json_parse && has_type_hint {
                            // Typed JSON parsing: let nums: [i32] = JSON.parse("[1,2,3]")
                            // Generate: let nums: Vec<i32> = serde_json::from_str::<Vec<i32>>(&"[1,2,3]").expect("JSON parse failed");
                            if let Expr::MethodCall(method_call) = &var.init {
                                self.generate_typed_json_parse(method_call, binding.type_ref.as_ref().unwrap())?;
                                self.output.push_str(".expect(\"JSON parse failed\")");
                            }
                        } else if is_json_parse {
                            // Untyped JSON parsing (original behavior): let data = JSON.parse(body)
                            // Mark this variable as JsonValue for lambda pattern detection
                            if let Some(name) = binding.name() {
                                self.json_value_vars.insert(name.to_string());
                            }
                            
                            // Generate: let posts = JSON.parse(body).0.expect("JSON parse failed");
                            self.generate_expr(&var.init)?;
                            self.output.push_str(".0.expect(\"JSON parse failed\")");
                        } else {
                            // Auto-clone when assigning this.field to a local variable
                            let needs_clone = self.expr_is_self_field(&var.init);
                            self.generate_expr(&var.init)?;
                            if needs_clone {
                                self.output.push_str(".clone()");
                            }
                        }
                        
                        // Add .to_string() if needed for string literals
                        if needs_to_string {
                            self.output.push_str(".to_string()");
                        }
                        
                        // Close union wrapper if opened
                        if needs_union_close {
                            self.output.push(')');
                        }
                        
                        self.output.push_str(";\n");
                    }
                }
            }
            Stmt::ConstDecl(const_decl) => {
                self.write_indent();
                write!(self.output, "const {}: ", const_decl.name.to_uppercase()).unwrap();
                let type_str = if let Some(type_ref) = &const_decl.type_ref {
                    type_ref.to_rust_type()
                } else {
                    self.infer_const_type(&const_decl.init)
                };
                self.output.push_str(&type_str);
                self.output.push_str(" = ");
                self.generate_expr(&const_decl.init)?;
                self.output.push_str(";\n");
            }
            Stmt::Assign(assign) => {
                // Check if assigning an object literal - mark variable for bracket notation
                if let Expr::Identifier(var_name) = &assign.target {
                    if let Expr::ObjectLiteral(_) = &assign.value {
                        self.bracket_notation_vars.insert(var_name.clone());
                    }
                    // Check if assigning from a class constructor call - mark variable as class instance
                    else if let Expr::Call(call) = &assign.value {
                        if let Expr::Identifier(_class_name) = &*call.callee {
                            // Check if this is a class constructor call by looking at the context
                            // For now, assume any call to an identifier is a potential class constructor
                            // This is a heuristic - we could improve this by checking against known class names
                            self.class_instance_vars.insert(var_name.clone());
                        }
                    }
                }

                self.write_indent();
                self.in_assignment_target = true;
                self.generate_expr(&assign.target)?;
                self.in_assignment_target = false;
                self.output.push_str(" = ");
                // If assigning a string literal to what might be a String field, add .to_string()
                if let Expr::Literal(Literal::String(_)) = &assign.value {
                    self.generate_expr(&assign.value)?;
                    self.output.push_str(".to_string()");
                } else {
                    self.generate_expr(&assign.value)?;
                }
                self.output.push_str(";\n");
            }
            Stmt::If(if_stmt) => {
                self.write_indent();
                self.output.push_str("if ");
                self.generate_condition_expr(&if_stmt.condition)?;
                self.output.push_str(" {\n");
                self.indent();
                self.generate_if_body(&if_stmt.then_branch)?;
                self.dedent();
                self.write_indent();
                self.output.push('}');

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.output.push_str(" else {\n");
                    self.indent();
                    self.generate_if_body(else_branch)?;
                    self.dedent();
                    self.write_indent();
                    self.output.push('}');
                }
                self.output.push('\n');
            }
            Stmt::While(while_stmt) => {
                self.write_indent();
                self.output.push_str("while ");
                self.generate_expr(&while_stmt.condition)?;
                self.output.push_str(" {\n");
                self.indent();
                self.generate_block_inner(&while_stmt.body)?;
                self.dedent();
                self.writeln("}");
            }
            Stmt::For(for_stmt) => {
                // Mark the loop variable for bracket notation (likely iterating over JSON objects)
                let var_name = self.sanitize_name(&for_stmt.var);
                self.bracket_notation_vars.insert(var_name.clone());

                self.write_indent();
                write!(self.output, "for {} in ", var_name).unwrap();
                self.generate_expr(&for_stmt.iterable)?;
                self.output.push_str(" {\n");
                self.indent();
                self.generate_block_inner(&for_stmt.body)?;
                self.dedent();
                self.writeln("}");
            }
            Stmt::Switch(switch_stmt) => {
                // Check if this is a string-based switch (if any case value is a string literal)
                let is_string_switch = switch_stmt.cases.iter().any(|case| {
                    matches!(&case.value, Expr::Literal(Literal::String(_)))
                });
                
                self.write_indent();
                self.output.push_str("match ");
                self.generate_expr(&switch_stmt.discriminant)?;
                // Add .as_str() for string-based switches so literals match
                if is_string_switch {
                    self.output.push_str(".as_str()");
                }
                self.output.push_str(" {\n");
                self.indent();

                for case in &switch_stmt.cases {
                    self.write_indent();
                    self.generate_expr(&case.value)?;
                    self.output.push_str(" => {\n");
                    self.indent();
                    for stmt in &case.body {
                        self.generate_stmt(stmt)?;
                    }
                    self.dedent();
                    self.writeln("}");
                }

                if let Some(default) = &switch_stmt.default {
                    self.writeln("_ => {");
                    self.indent();
                    for stmt in default {
                        self.generate_stmt(stmt)?;
                    }
                    self.dedent();
                    self.writeln("}");
                }

                self.dedent();
                self.writeln("}");
            }
            Stmt::TryCatch(try_catch) => {
                self.writeln("match (|| -> Result<(), Box<dyn std::error::Error>> {");
                self.indent();
                self.generate_block_inner(&try_catch.try_block)?;
                self.writeln("Ok(())");
                self.dedent();
                self.writeln("})() {");
                self.indent();
                self.writeln("Ok(_) => {},");
                self.write_indent();
                write!(
                    self.output,
                    "Err({}) => {{\n",
                    self.sanitize_name(&try_catch.catch_var)
                )
                .unwrap();
                self.indent();
                self.generate_block_inner(&try_catch.catch_block)?;
                self.dedent();
                self.writeln("}");
                self.dedent();
                self.writeln("}");
            }
            Stmt::Throw(throw_stmt) => {
                self.write_indent();
                self.output.push_str("return Err(");
                self.generate_expr(&throw_stmt.expr)?;
                self.output.push_str(".into());\n");
            }
            Stmt::Return(ret) => {
                self.write_indent();
                self.output.push_str("return");
                if let Some(expr) = &ret.expr {
                    self.output.push(' ');
                    if self.in_fallible_function {
                        self.output.push_str("Ok(");
                        self.generate_return_expr(expr)?;
                        self.output.push(')');
                    } else {
                        self.generate_return_expr(expr)?;
                    }
                }
                self.output.push_str(";\n");
            }
            Stmt::Expr(expr_stmt) => {
                self.write_indent();
                self.generate_expr(&expr_stmt.expr)?;
                self.output.push_str(";\n");
            }
            Stmt::Block(block) => {
                self.writeln("{");
                self.indent();
                self.generate_block_inner(block)?;
                self.dedent();
                self.writeln("}");
            }
            Stmt::Fail(fail_stmt) => {
                self.write_indent();
                self.output.push_str("return Err(liva_rt::Error::from(");
                self.generate_expr(&fail_stmt.expr)?;
                self.output.push_str("));\n");
            }
        }
        Ok(())
    }

    /// Checks if an expression is a member access on 'this' (self.field)
    /// Returns true if the expression needs .clone() when used in assignment or return
    fn expr_is_self_field(&self, expr: &Expr) -> bool {
        if let Expr::Member { object, property: _ } = expr {
            if let Expr::Identifier(obj) = object.as_ref() {
                return obj == "this" && self.in_method;
            }
        }
        false
    }

    /// Generate return expression with auto-clone for non-Copy types
    /// Detects when returning a field from self and automatically adds .clone()
    fn generate_return_expr(&mut self, expr: &Expr) -> Result<()> {
        // Check if this is a string literal - needs .to_string() for String return type
        if let Expr::Literal(Literal::String(_)) = expr {
            self.generate_expr(expr)?;
            self.output.push_str(".to_string()");
            return Ok(());
        }
        
        // Check if this is a member access on 'this' (self.field)
        // Use the helper function for this check
        let needs_clone = self.expr_is_self_field(expr);

        if needs_clone {
            self.generate_expr(expr)?;
            self.output.push_str(".clone()");
        } else {
            self.generate_expr(expr)?;
        }
        
        Ok(())
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(lit) => self.generate_literal(lit)?,
            Expr::Identifier(name) => {
                // Convert 'this' to 'self' when inside a method
                let actual_name = if self.in_method && name == "this" {
                    "self"
                } else {
                    name
                };

                // Check if this is a constant (uppercase identifier)
                if actual_name.chars().all(|c| c.is_uppercase() || c == '_') {
                    write!(self.output, "{}", actual_name).unwrap();
                } else {
                    write!(self.output, "{}", self.sanitize_name(actual_name)).unwrap();
                }
            }
            Expr::Binary { op, left, right } => {
                if matches!(op, BinOp::Add)
                    && (self.expr_is_stringy(left) || self.expr_is_stringy(right))
                {
                    self.output.push_str("format!(\"{}{}\", ");
                    self.generate_expr_for_string_concat(left)?;
                    self.output.push_str(", ");
                    self.generate_expr_for_string_concat(right)?;
                    self.output.push(')');
                } else {
                    self.generate_binary_operation(op, left, right)?;
                }
            }
            Expr::Unary { op, operand } => match op {
                crate::ast::UnOp::Await => {
                    // Check if we're awaiting a pending task variable
                    if let Expr::Identifier(name) = operand.as_ref() {
                        let sanitized = self.sanitize_name(name);
                        if self.pending_tasks.contains_key(&sanitized) {
                            // Generate task_name_task.await instead of task_name.await
                            write!(self.output, "{}_task.await", sanitized).unwrap();
                            return Ok(());
                        }
                    }
                    self.generate_expr(operand)?;
                    self.output.push_str(".await");
                }
                crate::ast::UnOp::Not => {
                    // Special handling for !error_var -> error_var.is_none()
                    if let Expr::Identifier(name) = operand.as_ref() {
                        let sanitized = self.sanitize_name(name);
                        if self.error_binding_vars.contains(&sanitized) {
                            write!(self.output, "{}.is_none()", sanitized).unwrap();
                            return Ok(());
                        }
                        // String error vars (from HTTP/File): !err -> err.is_empty()
                        if self.string_error_vars.contains(&sanitized) {
                            write!(self.output, "{}.is_empty()", sanitized).unwrap();
                            return Ok(());
                        }
                    }
                    write!(self.output, "{}", op).unwrap();
                    self.generate_expr(operand)?;
                }
                _ => {
                    write!(self.output, "{}", op).unwrap();
                    self.generate_expr(operand)?;
                }
            },
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                // Check if this ternary contains a fail - if so, generate as Result
                let has_fail =
                    self.expr_contains_fail(then_expr) || self.expr_contains_fail(else_expr);

                if has_fail {
                    // Generate as Result: if cond { Ok(then) or Err(...) } else { Err(...) or Ok(else) }
                    self.output.push_str("if ");
                    self.generate_expr(condition)?;
                    self.output.push_str(" { ");
                    // For the then branch, check if it's a fail
                    if let Expr::Fail(expr) = then_expr.as_ref() {
                        self.output.push_str("return Err(liva_rt::Error::from(");
                        self.generate_expr(expr)?;
                        self.output.push_str("))");
                    } else {
                        self.output.push_str("Ok(");
                        self.generate_expr(then_expr)?;
                        self.output.push_str(")");
                    }
                    self.output.push_str(" } else { ");
                    // For the else branch, check if it's a fail
                    if let Expr::Fail(expr) = else_expr.as_ref() {
                        self.output.push_str("Err(liva_rt::Error::from(");
                        self.generate_expr(expr)?;
                        self.output.push_str("))");
                    } else {
                        self.output.push_str("Ok(");
                        self.generate_expr(else_expr)?;
                        self.output.push_str(")");
                    }
                    self.output.push_str(" }");
                } else {
                    self.output.push_str("if ");
                    self.generate_expr(condition)?;
                    self.output.push_str(" { ");
                    self.generate_expr(then_expr)?;
                    self.output.push_str(" } else { ");
                    self.generate_expr(else_expr)?;
                    self.output.push_str(" }");
                }
            }
            Expr::Call(call) => {
                // Check if this is a .count() call on a sequence
                if let Expr::Member { object, property } = call.callee.as_ref() {
                    if property == "count" {
                        self.generate_expr(object)?;
                        self.output.push_str(".count()");
                        return Ok(());
                    }
                }
                self.generate_call_expr(call)?;
            }
            Expr::Member { object, property } => {
                // Phase 3.5: Special handling for error.message
                if let Expr::Identifier(name) = object.as_ref() {
                    let sanitized = self.sanitize_name(name);
                    if self.error_binding_vars.contains(&sanitized) && property == "message" {
                        write!(
                            self.output,
                            "{}.as_ref().map(|e| e.message.as_str()).unwrap_or(\"None\")",
                            sanitized
                        )
                        .unwrap();
                        return Ok(());
                    }
                    
                    // Special handling for Option<Struct> from tuple-returning functions
                    // For HTTP responses, File contents, JSON values, etc. - unwrap before accessing field
                    if self.option_value_vars.contains(&sanitized) {
                        if property == "length" {
                            // For JSON values: use .length() method
                            write!(self.output, "{}.as_ref().unwrap().length()", sanitized).unwrap();
                            return Ok(());
                        }
                        
                        // Check if this is a JSON value (from JSON.parse, HTTP, etc.)
                        let is_json_value = self.json_value_vars.contains(&sanitized);
                        
                        // Check if this is a struct field access (not JSON)
                        // Common struct fields: status, statusText, body, headers, content, etc.
                        let is_http_struct_field = matches!(
                            property.as_str(),
                            "status" | "statusText" | "body" | "headers" | "content" | "data"
                        );
                        
                        if is_http_struct_field {
                            // Convert camelCase to snake_case for Rust structs
                            let rust_field = self.to_snake_case(property);
                            write!(self.output, "{}.as_ref().unwrap().{}", sanitized, rust_field).unwrap();
                            return Ok(());
                        }
                        
                        // For Option<T> from .find() on class arrays, unwrap before field access
                        // If it's not a JSON value, it's a class instance wrapped in Option
                        if !is_json_value {
                            let rust_field = self.to_snake_case(property);
                            write!(self.output, "{}.as_ref().unwrap().{}", sanitized, rust_field).unwrap();
                            return Ok(());
                        }
                    }
                }

                // v0.11.0: Tuple member access - check if property is numeric (tuple.0, tuple.1, etc.)
                if property.parse::<usize>().is_ok() {
                    self.generate_expr(object)?;
                    write!(self.output, ".{}", property).unwrap();
                    return Ok(());
                }

                if property == "length" {
                    // Check if this is a JsonValue (not an array, string, or class instance)
                    // JsonValue uses .length(), Rust arrays/strings use .len()
                    // Note: .len() returns usize, but Liva uses i32 (number), so we cast
                    // Bug #31 fix: Wrap in parens so .toString() works: (x.len() as i32).to_string()
                    match object.as_ref() {
                        Expr::Identifier(var_name) => {
                            let sanitized = self.sanitize_name(var_name);
                            // Check if this is a known JsonValue variable
                            let is_json_value = self.json_value_vars.contains(&sanitized)
                                || self.json_value_vars.contains(var_name);
                            
                            if is_json_value {
                                // JsonValue uses .length() (already returns i32)
                                self.generate_expr(object)?;
                                self.output.push_str(".length()");
                            } else {
                                // Default to (obj.len() as i32) for strings, arrays, and other types
                                self.output.push('(');
                                self.generate_expr(object)?;
                                self.output.push_str(".len() as i32)");
                            }
                        }
                        _ => {
                            self.output.push('(');
                            self.generate_expr(object)?;
                            self.output.push_str(".len() as i32)");
                        }
                    }
                    return Ok(());
                }
                
                self.generate_expr(object)?;
                
                // Use bracket notation for JSON objects, dot notation for structs
                match object.as_ref() {
                    Expr::Identifier(var_name) => {
                        // Check if this is a Rust struct (HTTP response, etc.)
                        // Sanitize the name to match how it was stored in rust_struct_vars
                        let sanitized_name = self.sanitize_name(var_name);
                        let is_rust_struct = self.rust_struct_vars.contains(&sanitized_name);
                        
                        // Check if this is likely a JsonValue (not array, not class instance, not rust struct)
                        if !is_rust_struct
                            && !self.is_class_instance(var_name) 
                            && !self.array_vars.contains(var_name)
                            && !var_name.contains("person")
                            && !var_name.contains("user")
                        {
                            // Likely a JsonValue - use get_field()
                            write!(self.output, ".get_field(\"{}\").unwrap_or_default()", property).unwrap();
                            return Ok(());
                        }
                        
                        // For class instances and Rust structs, use dot notation
                        if is_rust_struct || self.is_class_instance(var_name)
                            || var_name.contains("person")
                            || var_name.contains("user")
                        {
                            // Convert camelCase to snake_case for Rust structs
                            let rust_field = self.to_snake_case(property);
                            write!(self.output, ".{}", rust_field).unwrap();

                            // Clone common owned types when returning by value and not assigning
                            if self.in_method
                                && !self.in_assignment_target
                                && (property == "title"
                                    || property == "author"
                                    || property == "name"
                                    || property.contains("dni")
                                    || property.ends_with("text")
                                    || property.ends_with("data"))
                            {
                                self.output.push_str(".clone()");
                            }
                            return Ok(());
                        }
                        
                        // For JSON access, generate bracket notation
                        write!(self.output, "[\"{}\"]", property).unwrap();

                        // Convert numeric properties automatically (but not in string templates - format! handles it)
                        if !self.in_string_template {
                            if property == "price"
                                || property == "age"
                                || property.contains("count")
                                || property.contains("total")
                                || property.contains("sum")
                            {
                                self.output.push_str(".as_f64().unwrap_or(0.0)");
                            } else if property == "name"
                                || property.contains("text")
                                || property.contains("data")
                            {
                                self.output.push_str(".as_string().unwrap_or_default()");
                            }
                        }
                    }
                    Expr::Index { .. } => {
                        // Indexed access like array[index] - the result is typically a JSON object
                        // So property access should use bracket notation
                        write!(self.output, "[\"{}\"]", property).unwrap();

                        // For numeric fields in JSON objects, convert to appropriate type (but not in string templates)
                        if !self.in_string_template {
                            // Always convert price to f64 since it's commonly used in arithmetic
                            if property == "price" {
                                self.output.push_str(".as_f64().unwrap_or(0.0)");
                            } else if property == "age"
                                || property.contains("count")
                                || property.contains("total")
                                || property.contains("sum")
                            {
                                self.output.push_str(".as_f64().unwrap_or(0.0)");
                            } else if property == "name"
                                || property.contains("text")
                                || property.contains("data")
                            {
                                self.output.push_str(".as_string().unwrap_or_default()");
                            }
                        }
                    }
                    _ => {
                        // For other expressions, use dot notation
                        write!(self.output, ".{}", self.sanitize_name(property)).unwrap();
                    }
                }
            }
            Expr::Index { object, index } => {
                // Special handling for JsonValue (both Option<JsonValue> and JsonValue)
                // BUT: Skip this if we're in a string template (handled separately there)
                if !self.in_string_template {
                    if let Expr::Identifier(var_name) = object.as_ref() {
                        let sanitized = self.sanitize_name(var_name);
                        let is_option_json = self.option_value_vars.contains(&sanitized);
                        
                        // Check if this might be a JsonValue (either Option or direct)
                        // We detect Option<JsonValue> via option_value_vars
                        // For direct JsonValue, we'll try to generate the method call
                        // and let Rust's type system validate it
                        if is_option_json {
                            // Option<JsonValue> case
                            match index.as_ref() {
                                Expr::Literal(Literal::String(key)) => {
                                    write!(self.output, "{}.as_ref().unwrap().get_field(\"{}\").unwrap_or_default()", sanitized, key).unwrap();
                                }
                                Expr::Identifier(index_var) => {
                                    // If the index is a variable, check if it's a string variable
                                    let index_sanitized = self.sanitize_name(index_var);
                                    if self.string_vars.contains(&index_sanitized) {
                                        // String variable - use get_field for object access
                                        write!(self.output, "{}.as_ref().unwrap().get_field(&{}).unwrap_or_default()", sanitized, index_sanitized).unwrap();
                                    } else {
                                        // Assume numeric index for array access
                                        write!(self.output, "{}.as_ref().unwrap().get(", sanitized).unwrap();
                                        self.generate_expr(index)?;
                                        self.output.push_str(").unwrap_or_default()");
                                    }
                                }
                                _ => {
                                    write!(self.output, "{}.as_ref().unwrap().get(", sanitized).unwrap();
                                    self.generate_expr(index)?;
                                    self.output.push_str(").unwrap_or_default()");
                                }
                            }
                            return Ok(());
                        }
                    }
                }
                
                // Special handling for string indexing: s[i] -> s.chars().nth(i).unwrap_or_default()
                // Rust strings are UTF-8 and don't support direct indexing
                // This must come BEFORE generate_expr(object) to prevent emitting the object first
                if let Expr::Identifier(var_name) = object.as_ref() {
                    let sanitized = self.sanitize_name(var_name);
                    if self.string_vars.contains(&sanitized) {
                        // String indexing - use .chars().nth(i)
                        self.generate_expr(object)?;
                        self.output.push_str(".chars().nth(");
                        self.generate_expr(index)?;
                        self.output.push_str(" as usize).map(|c| c.to_string()).unwrap_or_default()");
                        return Ok(());
                    }
                }
                
                self.generate_expr(object)?;
                
                // For native Vec<String> (from Sys.args()), use direct indexing with .clone()
                if let Expr::Identifier(var_name) = object.as_ref() {
                    let sanitized = self.sanitize_name(var_name);
                    if self.native_vec_string_vars.contains(&sanitized) {
                        self.output.push('[');
                        self.generate_expr(index)?;
                        self.output.push_str("].clone()");
                        return Ok(());
                    }
                }
                
                // For JsonValue direct access (not Option), check if object looks like JsonValue
                // This is a heuristic: if object is an identifier that's not in our known sets,
                // it might be a JsonValue. We'll generate .get_field() / .get() instead of []
                if let Expr::Identifier(var_name) = object.as_ref() {
                    let sanitized = self.sanitize_name(var_name);
                    // If it's not a known array or class instance, try JsonValue access
                    if !self.array_vars.contains(&sanitized) && !self.class_instance_vars.contains(&sanitized) {
                        match index.as_ref() {
                            Expr::Literal(Literal::String(key)) => {
                                // Try JsonValue object access
                                write!(self.output, ".get_field(\"{}\").unwrap_or_default()", key).unwrap();
                                return Ok(());
                            }
                            Expr::Literal(Literal::Int(num)) => {
                                // Try JsonValue array access with numeric literal
                                write!(self.output, ".get({} as usize).cloned().unwrap_or_default()", num).unwrap();
                                return Ok(());
                            }
                            Expr::Identifier(index_var) => {
                                // If the index is a variable, check if it's a string variable
                                // String variables should use get_field(), numeric should use get()
                                let index_sanitized = self.sanitize_name(index_var);
                                if self.string_vars.contains(&index_sanitized) {
                                    // String variable - use get_field for object access
                                    self.output.push_str(".get_field(&");
                                    self.output.push_str(&index_sanitized);
                                    self.output.push_str(").unwrap_or_default()");
                                } else {
                                    // Assume numeric index for array access
                                    self.output.push_str(".get(");
                                    self.generate_expr(index)?;
                                    self.output.push_str(" as usize).cloned().unwrap_or_default()");
                                }
                                return Ok(());
                            }
                            _ => {
                                // Try JsonValue array access with expression
                                self.output.push_str(".get(");
                                self.generate_expr(index)?;
                                self.output.push_str(" as usize).cloned().unwrap_or_default()");
                                return Ok(());
                            }
                        }
                    }
                }
                
                // Handle nested JSON access: when object is another Index expression (e.g., issue["user"]["login"])
                // The object was already generated above, now we need to chain .get_field() for the nested access
                if let Expr::Index { .. } = object.as_ref() {
                    // Object is another Index, which means this is nested JSON access
                    // Generate .get_field("key") for the next level
                    match index.as_ref() {
                        Expr::Literal(Literal::String(key)) => {
                            write!(self.output, ".get_field(\"{}\").unwrap_or_default()", key).unwrap();
                            return Ok(());
                        }
                        Expr::Literal(Literal::Int(num)) => {
                            write!(self.output, ".get({} as usize).cloned().unwrap_or_default()", num).unwrap();
                            return Ok(());
                        }
                        _ => {
                            self.output.push_str(".get(");
                            self.generate_expr(index)?;
                            self.output.push_str(" as usize).cloned().unwrap_or_default()");
                            return Ok(());
                        }
                    }
                }
                
                // Fall back to standard array indexing
                // Bug #34: For arrays with non-literal index (e.g., lines[i] where i is int),
                // we need to add `as usize` because Rust Vec indexing requires usize
                self.output.push('[');
                self.generate_expr(index)?;
                
                // Add `as usize` for non-literal indexes on arrays
                // Literal integers (0, 1, 2) work fine, but variables (i) are i32 and need conversion
                let needs_clone = if let Expr::Identifier(var_name) = object.as_ref() {
                    let sanitized = self.sanitize_name(var_name);
                    if self.array_vars.contains(&sanitized) {
                        // Only add conversion for non-literal indexes
                        match index.as_ref() {
                            Expr::Literal(Literal::Int(_)) => {
                                // Literal integers don't need conversion
                            }
                            _ => {
                                // Variables and expressions need `as usize`
                                self.output.push_str(" as usize");
                            }
                        }
                        // Check if this is a string array - need .clone() for String
                        if let Some(elem_type) = self.typed_array_vars.get(&sanitized) {
                            elem_type == "string"
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };
                
                self.output.push(']');
                
                // For string arrays, add .clone() because indexing returns &String
                if needs_clone {
                    self.output.push_str(".clone()");
                }

                // Convert numeric properties automatically
                if let Expr::Literal(Literal::String(prop)) = index.as_ref() {
                    if prop == "price"
                        || prop == "age"
                        || prop.contains("count")
                        || prop.contains("total")
                        || prop.contains("sum")
                    {
                        self.output.push_str(".as_f64().unwrap_or(0.0)");
                    } else if prop == "name" || prop.contains("text") || prop.contains("data") {
                        self.output.push_str(".as_string().unwrap_or_default()");
                    }
                }
            }
            Expr::ObjectLiteral(fields) => {
                // Generate as a struct initialization or JSON
                self.output.push_str("serde_json::json!({\n");
                self.indent();
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(",\n");
                    }
                    self.write_indent();
                    write!(self.output, "\"{}\": ", key).unwrap();
                    self.generate_expr(value)?;
                }
                self.output.push('\n');
                self.dedent();
                self.write_indent();
                self.output.push_str("})");
            }
            Expr::StructLiteral { type_name, fields } => {
                // Generate Rust struct literal directly instead of constructor call
                // This works for all cases: with or without explicit constructor
                write!(self.output, "{} {{ ", type_name).unwrap();

                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Convert field name to snake_case
                    let field_name = self.sanitize_name(key);
                    write!(self.output, "{}: ", field_name).unwrap();
                    
                    // Add .to_string() for string literals
                    if let Expr::Literal(Literal::String(_)) = value {
                        self.generate_expr(value)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(value)?;
                    }
                }

                self.output.push_str(" }");
            }
            Expr::ArrayLiteral(elements) => {
                self.output.push_str("vec![");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(elem)?;
                }
                self.output.push(']');
            }
            Expr::Tuple(elements) => {
                self.output.push('(');
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Convert string literals to String for tuple compatibility
                    if matches!(elem, Expr::Literal(Literal::String(_))) {
                        self.generate_expr(elem)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(elem)?;
                    }
                }
                // Rust requires trailing comma for single-element tuples
                if elements.len() == 1 {
                    self.output.push(',');
                }
                self.output.push(')');
            }
            Expr::StringTemplate { parts } => {
                self.output.push_str("format!(\"");

                for part in parts.iter() {
                    match part {
                        StringTemplatePart::Text(text) => {
                            for ch in text.chars() {
                                match ch {
                                    '"' => self.output.push_str("\\\""),
                                    '\\' => self.output.push_str("\\\\"),
                                    '\n' => self.output.push_str("\\n"),
                                    '\r' => self.output.push_str("\\r"),
                                    '\t' => self.output.push_str("\\t"),
                                    '{' => self.output.push_str("{{"),
                                    '}' => self.output.push_str("}}"),
                                    _ => self.output.push(ch),
                                }
                            }
                        }
                        StringTemplatePart::Expr(expr) => match expr.as_ref() {
                            // Literals always use Display
                            Expr::Literal(_) => {
                                self.output.push_str("{}");
                            }
                            // Simple identifiers: check if they're arrays or option values
                            Expr::Identifier(name) => {
                                if self.array_vars.contains(name) {
                                    self.output.push_str("{:?}");
                                } else if self.option_value_vars.contains(name) || self.error_binding_vars.contains(name) {
                                    // Option values need special handling - will be unwrapped
                                    self.output.push_str("{}");
                                } else {
                                    self.output.push_str("{}");
                                }
                            }
                            // Member access uses Display
                            Expr::Member { .. } => {
                                self.output.push_str("{}");
                            }
                            // Index access uses Display
                            Expr::Index { .. } => {
                                self.output.push_str("{}");
                            }
                            // Binary operations (including comparisons) use Display
                            Expr::Binary { .. } => {
                                self.output.push_str("{}");
                            }
                            // Ternary/If expressions with string results use Display
                            Expr::Ternary { .. } => {
                                self.output.push_str("{}");
                            }
                            // Function calls use Display
                            Expr::Call { .. } | Expr::MethodCall { .. } => {
                                self.output.push_str("{}");
                            }
                            // Arrays and objects use Debug
                            Expr::ArrayLiteral(_) | Expr::ObjectLiteral(_) | Expr::Tuple(_) => {
                                self.output.push_str("{:?}");
                            }
                            // Everything else uses Debug
                            _ => {
                                self.output.push_str("{:?}");
                            }
                        },
                    }
                }

                self.output.push('"');

                let exprs: Vec<&Expr> = parts
                    .iter()
                    .filter_map(|part| match part {
                        StringTemplatePart::Expr(expr) => Some(expr.as_ref()),
                        _ => None,
                    })
                    .collect();

                if !exprs.is_empty() {
                    self.output.push_str(", ");

                    // Mark that we're inside a string template for proper member access generation
                    let was_in_template = self.in_string_template;
                    self.in_string_template = true;

                    for (idx, expr) in exprs.iter().enumerate() {
                        if idx > 0 {
                            self.output.push_str(", ");
                        }
                        // Phase 3.5: If expr is an error binding variable, use .message
                        if let Expr::Identifier(name) = expr {
                            let sanitized = self.sanitize_name(name);
                            if self.error_binding_vars.contains(&sanitized) {
                                write!(
                                    self.output,
                                    "{}.as_ref().map(|e| e.message.as_str()).unwrap_or(\"\")",
                                    sanitized
                                )
                                .unwrap();
                                continue;
                            }
                            if self.option_value_vars.contains(&sanitized) {
                                write!(
                                    self.output,
                                    "{}.as_ref().map(|v| v.to_string()).unwrap_or_default()",
                                    sanitized
                                )
                                .unwrap();
                                continue;
                            }
                        }
                        // Phase 3.6: If expr is index access on JSON value
                        if let Expr::Index { object, index } = expr {
                            if let Expr::Identifier(var_name) = object.as_ref() {
                                let sanitized = self.sanitize_name(var_name);
                                if self.option_value_vars.contains(&sanitized) {
                                    // Generate unwrapped index access for string template
                                    match index.as_ref() {
                                        Expr::Literal(Literal::String(key)) => {
                                            write!(self.output, "{}.as_ref().unwrap().get_field(\"{}\").unwrap_or_default()", sanitized, key).unwrap();
                                        }
                                        _ => {
                                            write!(self.output, "{}.as_ref().unwrap().get(", sanitized).unwrap();
                                            self.generate_expr(index)?;
                                            self.output.push_str(").unwrap_or_default()");
                                        }
                                    }
                                    continue;
                                }
                            }
                        }
                        self.generate_expr(expr)?;
                    }

                    self.in_string_template = was_in_template;
                }

                self.output.push(')');
            }
            Expr::Lambda(lambda) => {
                if lambda.is_move {
                    self.output.push_str("move ");
                }
                self.output.push('|');

                for (idx, param) in lambda.params.iter().enumerate() {
                    if idx > 0 {
                        self.output.push_str(", ");
                    }
                    
                    // For destructured parameters, use temporary names
                    let param_name = if param.is_destructuring() {
                        format!("_param_{}", idx)
                    } else {
                        self.sanitize_name(param.name().unwrap())
                    };
                    
                    self.output.push_str(&param_name);
                    if let Some(type_ref) = &param.type_ref {
                        self.output.push_str(": ");
                        self.output.push_str(&type_ref.to_rust_type());
                    }
                }

                self.output.push_str("| ");

                // Check if we need to generate destructuring code
                let has_destructuring = lambda.params.iter().any(|p| p.is_destructuring());

                match &lambda.body {
                    LambdaBody::Expr(expr) => {
                        if has_destructuring {
                            // Need to wrap in a block to insert destructuring
                            self.output.push('{');
                            self.indent();
                            self.output.push('\n');
                            
                            // Capture element type before mutable borrows
                            let element_type_for_destr = self.current_lambda_element_type.clone();
                            
                            // Generate destructuring for lambda params
                            for (idx, param) in lambda.params.iter().enumerate() {
                                if param.is_destructuring() {
                                    let temp_name = format!("_param_{}", idx);
                                    self.write_indent();
                                    self.generate_lambda_param_destructuring(&param.pattern, &temp_name, false, element_type_for_destr.as_deref())?;
                                }
                            }
                            
                            self.write_indent();
                            self.generate_expr(expr)?;
                            self.output.push('\n');
                            self.dedent();
                            self.write_indent();
                            self.output.push('}');
                        } else {
                            self.generate_expr(expr)?;
                        }
                    }
                    LambdaBody::Block(block) => {
                        // For lambdas, we need to generate a block that returns a value
                        // Check if the last statement is a return statement
                        if let Some(last_stmt) = block.stmts.last() {
                            if let Stmt::Return(return_stmt) = last_stmt {
                                if let Some(expr) = &return_stmt.expr {
                                    // Generate the block with statements except the last return
                                    self.output.push('{');
                                    self.indent();
                                    self.output.push('\n');
                                    
                                    // Capture element type before mutable borrows
                                    let element_type_for_destr = self.current_lambda_element_type.clone();
                                    
                                    // Generate destructuring for lambda params (if any)
                                    if has_destructuring {
                                        for (idx, param) in lambda.params.iter().enumerate() {
                                            if param.is_destructuring() {
                                                let temp_name = format!("_param_{}", idx);
                                                self.write_indent();
                                                self.generate_lambda_param_destructuring(&param.pattern, &temp_name, false, element_type_for_destr.as_deref())?;
                                                self.output.push('\n');
                                            }
                                        }
                                    }
                                    
                                    self.write_indent();

                                    // Generate all statements except the last return
                                    for stmt in &block.stmts[..block.stmts.len() - 1] {
                                        self.generate_stmt(stmt)?;
                                        self.output.push('\n');
                                        self.write_indent();
                                    }

                                    // Generate the return expression without the return keyword
                                    self.generate_expr(expr)?;

                                    self.dedent();
                                    self.output.push('\n');
                                    self.write_indent();
                                    self.output.push('}');
                                } else {
                                    // Empty return, generate unit type
                                    self.output.push_str("()");
                                }
                            } else {
                                // No return statement, generate block as-is
                                self.output.push('{');
                                self.indent();
                                self.output.push('\n');
                                
                                // Capture element type before mutable borrows
                                let element_type_for_destr = self.current_lambda_element_type.clone();
                                
                                // Generate destructuring for lambda params (if any)
                                if has_destructuring {
                                    for (idx, param) in lambda.params.iter().enumerate() {
                                        if param.is_destructuring() {
                                            let temp_name = format!("_param_{}", idx);
                                            self.write_indent();
                                            self.generate_lambda_param_destructuring(&param.pattern, &temp_name, false, element_type_for_destr.as_deref())?;
                                            self.output.push('\n');
                                        }
                                    }
                                }
                                
                                self.write_indent();
                                for stmt in &block.stmts {
                                    self.generate_stmt(stmt)?;
                                    self.output.push('\n');
                                    self.write_indent();
                                }
                                self.dedent();
                                self.output.push('}');
                            }
                        } else {
                            // Empty block
                            self.output.push_str("()");
                        }
                    }
                }
            }
            Expr::Fail(expr) => {
                self.write_indent();
                self.output.push_str("return Err(liva_rt::Error::from(");
                self.generate_expr(expr)?;
                self.output.push_str("));\n");
            }
            Expr::MethodCall(method_call) => {
                // TODO: Implement method call code generation (stdlib Phase 2)
                // For now, just generate a placeholder
                self.generate_method_call_expr(method_call)?;
            }
            Expr::Switch(switch_expr) => {
                self.generate_switch_expr(switch_expr)?;
            }
        }
        Ok(())
    }

    fn generate_switch_expr(&mut self, switch_expr: &SwitchExpr) -> Result<()> {
        // Check if this is a union type match by examining the first Typed pattern
        let union_type_name = self.detect_union_switch(switch_expr);
        
        // Generate Rust match expression
        self.output.push_str("match ");
        self.generate_expr(&switch_expr.discriminant)?;
        self.output.push_str(" {");
        self.indent();

        for arm in &switch_expr.arms {
            self.output.push('\n');
            self.write_indent();

            // Generate pattern (with union context if applicable)
            if let Some(ref union_name) = union_type_name {
                self.generate_union_pattern(&arm.pattern, union_name)?;
            } else {
                self.generate_pattern(&arm.pattern)?;
            }

            // Generate guard if present
            if let Some(guard) = &arm.guard {
                self.output.push_str(" if ");
                self.generate_expr(guard)?;
            }

            self.output.push_str(" => ");

            // Generate body
            match &arm.body {
                SwitchBody::Expr(expr) => {
                    self.generate_expr(expr)?;
                }
                SwitchBody::Block(stmts) => {
                    self.output.push('{');
                    self.indent();
                    for stmt in stmts {
                        self.output.push('\n');
                        self.write_indent();
                        self.generate_stmt(stmt)?;
                    }
                    self.dedent();
                    self.output.push('\n');
                    self.write_indent();
                    self.output.push('}');
                }
            }

            self.output.push(',');
        }

        self.dedent();
        self.output.push('\n');
        self.write_indent();
        self.output.push('}');

        Ok(())
    }

    /// Detect if this is a union type switch by checking for Typed patterns
    fn detect_union_switch(&mut self, switch_expr: &SwitchExpr) -> Option<String> {
        // Collect types from Typed patterns in order
        let mut pattern_types = Vec::new();
        
        for arm in &switch_expr.arms {
            if let Pattern::Typed { type_ref, .. } = &arm.pattern {
                let rust_type = self.expand_type_alias(type_ref);
                if !pattern_types.contains(&rust_type) {
                    pattern_types.push(rust_type);
                }
            }
        }
        
        // If we found typed patterns, construct union name from pattern order
        if pattern_types.len() >= 2 {
            Some(format!("Union_{}", pattern_types.join("_")))
        } else {
            None
        }
    }

    /// Generate a pattern in the context of a union match
    fn generate_union_pattern(&mut self, pattern: &Pattern, union_name: &str) -> Result<()> {
        match pattern {
            Pattern::Typed { name, type_ref } => {
                let rust_type = self.expand_type_alias(type_ref);
                let variant_name = self.type_to_variant_name(&rust_type);
                write!(self.output, "{}::{}({})", union_name, variant_name, self.sanitize_name(name)).unwrap();
            }
            Pattern::Wildcard => {
                self.output.push('_');
            }
            _ => {
                // For other patterns, fallback to normal generation
                self.generate_pattern(pattern)?;
            }
        }
        Ok(())
    }

    fn generate_pattern(&mut self, pattern: &Pattern) -> Result<()> {
        match pattern {
            Pattern::Literal(lit) => {
                self.generate_literal(lit)?;
            }
            Pattern::Wildcard => {
                self.output.push('_');
            }
            Pattern::Binding(name) => {
                self.output.push_str(&self.sanitize_name(name));
            }
            Pattern::Range(range) => {
                match (&range.start, &range.end, range.inclusive) {
                    (Some(start), Some(end), true) => {
                        self.generate_expr(start)?;
                        self.output.push_str("..=");
                        self.generate_expr(end)?;
                    }
                    (Some(start), Some(end), false) => {
                        self.generate_expr(start)?;
                        self.output.push_str("..");
                        self.generate_expr(end)?;
                    }
                    (Some(start), None, _) => {
                        self.generate_expr(start)?;
                        self.output.push_str("..");
                    }
                    (None, Some(end), true) => {
                        self.output.push_str("..=");
                        self.generate_expr(end)?;
                    }
                    (None, Some(end), false) => {
                        self.output.push_str("..");
                        self.generate_expr(end)?;
                    }
                    (None, None, _) => {
                        self.output.push_str("..");
                    }
                }
            }
            Pattern::Tuple(patterns) => {
                self.output.push('(');
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_pattern(pat)?;
                }
                self.output.push(')');
            }
            Pattern::Array(patterns) => {
                self.output.push('[');
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_pattern(pat)?;
                }
                self.output.push(']');
            }
            Pattern::Or(patterns) => {
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" | ");
                    }
                    self.generate_pattern(pat)?;
                }
            }
            Pattern::Typed { name, type_ref } => {
                // Type pattern for union narrowing: name: type
                // When used outside of union context, just bind the variable
                // (Union context is handled in generate_union_pattern)
                self.output.push_str(&self.sanitize_name(name));
            }
        }
        Ok(())
    }

    fn generate_call_expr(&mut self, call: &CallExpr) -> Result<()> {
        match call.exec_policy {
            ExecPolicy::Normal => self.generate_normal_call(call),
            ExecPolicy::Async => self.generate_async_call(call),
            ExecPolicy::Par => self.generate_parallel_call(call),
            ExecPolicy::TaskAsync => self.generate_task_call(call, ConcurrencyMode::Async),
            ExecPolicy::TaskPar => self.generate_task_call(call, ConcurrencyMode::Parallel),
            ExecPolicy::FireAsync => self.generate_fire_call(call, ConcurrencyMode::Async),
            ExecPolicy::FirePar => self.generate_fire_call(call, ConcurrencyMode::Parallel),
        }
    }

    fn generate_normal_call(&mut self, call: &CallExpr) -> Result<()> {
        if let Expr::Identifier(name) = call.callee.as_ref() {
            // Handle parseInt(str) -> (i32, Option<Error>)
            if name == "parseInt" {
                if call.args.is_empty() {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "parseInt requires 1 argument",
                            "parseInt(str) takes exactly one string argument"
                        )
                    ));
                }
                self.output.push_str("match ");
                self.generate_expr(&call.args[0])?;
                self.output.push_str(".parse::<i32>() { Ok(v) => (v, None), Err(_) => (0, Some(liva_rt::Error::from(\"Invalid integer format\"))) }");
                return Ok(());
            }
            
            // Handle parseFloat(str) -> (f64, Option<Error>)
            if name == "parseFloat" {
                if call.args.is_empty() {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "parseFloat requires 1 argument",
                            "parseFloat(str) takes exactly one string argument"
                        )
                    ));
                }
                self.output.push_str("match ");
                self.generate_expr(&call.args[0])?;
                self.output.push_str(".parse::<f64>() { Ok(v) => (v, None), Err(_) => (0.0_f64, Some(liva_rt::Error::from(\"Invalid float format\"))) }");
                return Ok(());
            }
            
            // Handle toString(value) -> String
            if name == "toString" {
                if call.args.is_empty() {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "toString requires 1 argument",
                            "toString(value) takes exactly one argument"
                        )
                    ));
                }
                self.output.push_str("format!(\"{}\", ");
                self.generate_expr(&call.args[0])?;
                self.output.push_str(")");
                return Ok(());
            }
            
            // Handle readLine() -> String (read from stdin)
            if name == "readLine" {
                self.output.push_str("{ let mut input = String::new(); std::io::stdin().read_line(&mut input).expect(\"Failed to read line\"); input.trim().to_string() }");
                return Ok(());
            }
            
            // Handle prompt(message) -> String (display message and read input)
            if name == "prompt" {
                if call.args.is_empty() {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "prompt requires 1 argument",
                            "prompt(message) takes exactly one string argument"
                        )
                    ));
                }
                self.output.push_str("{ print!(\"{}\", ");
                self.generate_expr(&call.args[0])?;
                self.output.push_str("); std::io::stdout().flush().expect(\"Failed to flush stdout\"); let mut input = String::new(); std::io::stdin().read_line(&mut input).expect(\"Failed to read line\"); input.trim().to_string() }");
                return Ok(());
            }
            
            if name == "print" {
                if call.args.is_empty() {
                    self.output.push_str("println!()");
                } else {
                    self.output.push_str("println!(\"");
                    for _arg in call.args.iter() {
                        // print() uses Display format {} for clean, user-facing output
                        self.output.push_str("{}");
                    }
                    self.output.push_str("\", ");
                    for (i, arg) in call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        // Phase 3.5: If arg is an error binding variable, print .message
                        if let Expr::Identifier(name) = arg {
                            let sanitized = self.sanitize_name(name);
                            if self.error_binding_vars.contains(&sanitized) {
                                write!(
                                    self.output,
                                    "{}.as_ref().map(|e| e.message.as_str()).unwrap_or(\"None\")",
                                    sanitized
                                )
                                .unwrap();
                                continue;
                            }
                        }
                        self.generate_expr(arg)?;
                    }
                    self.output.push(')');
                }
                return Ok(());
            }

            // Check if this is a constructor call (starts with uppercase)
            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                // Assume it's a constructor call like ClassName(args...)
                write!(self.output, "{}::new(", name).unwrap();
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Add .to_string() for string literals (hack for constructor parameters)
                    if let Expr::Literal(Literal::String(_)) = arg {
                        self.generate_expr(arg)?;
                        self.output.push_str(".to_string()");
                    } else if let Expr::Identifier(var_name) = arg {
                        // Bug #32: Clone string variables when passing to constructors
                        // to allow them to be used after the constructor call
                        let sanitized = self.sanitize_name(var_name);
                        let is_string_var = self.string_vars.contains(&sanitized);
                        let is_class_instance = self.class_instance_vars.contains(&sanitized);
                        if is_string_var || is_class_instance {
                            self.generate_expr(arg)?;
                            self.output.push_str(".clone()");
                        } else {
                            self.generate_expr(arg)?;
                        }
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push(')');
                return Ok(());
            }
        }

        // Check if this is a call to a user-defined async function
        let is_async_call = if let Expr::Identifier(name) = call.callee.as_ref() {
            self.async_functions.contains(name)
        } else {
            false
        };

        self.generate_expr(&call.callee)?;
        
        // Add type arguments if present (turbofish syntax)
        if !call.type_args.is_empty() {
            self.output.push_str("::<");
            for (i, type_arg) in call.type_args.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                write!(self.output, "{}", type_arg.to_rust_type()).unwrap();
            }
            self.output.push('>');
        }
        
        self.output.push('(');
        for (i, arg) in call.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            // Convert string literals to String automatically
            if let Expr::Literal(Literal::String(_)) = arg {
                self.generate_expr(arg)?;
                self.output.push_str(".to_string()");
            } else if let Expr::Identifier(name) = arg {
                // Clone class instances when passing to functions to avoid ownership issues
                let sanitized = self.sanitize_name(name);
                if self.class_instance_vars.contains(&sanitized) {
                    self.generate_expr(arg)?;
                    self.output.push_str(".clone()");
                } else {
                    self.generate_expr(arg)?;
                }
            } else {
                self.generate_expr(arg)?;
            }
        }
        self.output.push(')');
        
        // Add .await for async function calls
        if is_async_call {
            self.output.push_str(".await");
        }
        
        Ok(())
    }

    /// Check if an expression is an async or par call (returns Task)
    fn is_task_expr(&self, expr: &Expr) -> Option<ExecPolicy> {
        match expr {
            Expr::Call(call) => match call.exec_policy {
                ExecPolicy::Async | ExecPolicy::Par => Some(call.exec_policy.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    /// Check if an expression uses a variable (recursively)
    fn expr_uses_var(&self, expr: &Expr, var_name: &str) -> bool {
        match expr {
            Expr::Identifier(name) => {
                let sanitized = if self.in_method && name == "this" {
                    "self"
                } else {
                    name
                };
                self.sanitize_name(sanitized) == var_name
            }
            Expr::Binary { left, right, .. } => {
                self.expr_uses_var(left, var_name) || self.expr_uses_var(right, var_name)
            }
            Expr::Unary { operand, .. } => self.expr_uses_var(operand, var_name),
            Expr::Call(call) => {
                self.expr_uses_var(&call.callee, var_name)
                    || call
                        .args
                        .iter()
                        .any(|arg| self.expr_uses_var(arg, var_name))
            }
            Expr::Member { object, .. } => self.expr_uses_var(object, var_name),
            Expr::Index { object, index } => {
                self.expr_uses_var(object, var_name) || self.expr_uses_var(index, var_name)
            }
            Expr::StringTemplate { parts } => parts.iter().any(|p| {
                if let crate::ast::StringTemplatePart::Expr(e) = p {
                    self.expr_uses_var(e, var_name)
                } else {
                    false
                }
            }),
            _ => false,
        }
    }

    /// Check if a statement uses a pending task variable
    fn stmt_uses_pending_task(&self, stmt: &Stmt) -> Option<String> {
        for (var_name, task_info) in &self.pending_tasks {
            if task_info.awaited {
                continue; // Already awaited
            }

            // For error binding, check if ANY of the binding variables is used
            let check_vars: Vec<&String> = if task_info.is_error_binding {
                task_info.binding_names.iter().collect()
            } else {
                vec![var_name]
            };

            let uses_var = match stmt {
                Stmt::Expr(expr_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&expr_stmt.expr, v)),
                Stmt::If(if_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&if_stmt.condition, v)),
                Stmt::Return(ret_stmt) => ret_stmt.expr.as_ref().map_or(false, |e| {
                    check_vars.iter().any(|v| self.expr_uses_var(e, v))
                }),
                Stmt::While(while_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&while_stmt.condition, v)),
                Stmt::Assign(assign) => check_vars.iter().any(|v| {
                    self.expr_uses_var(&assign.target, v) || self.expr_uses_var(&assign.value, v)
                }),
                _ => false,
            };

            if uses_var {
                return Some(var_name.clone());
            }
        }
        None
    }

    /// Phase 4: Get ALL pending tasks used in a statement (for join combining)
    fn stmt_uses_pending_tasks(&self, stmt: &Stmt) -> Vec<String> {
        let mut used_tasks = Vec::new();

        for (var_name, task_info) in &self.pending_tasks {
            if task_info.awaited {
                continue; // Already awaited
            }

            // For error binding, check if ANY of the binding variables is used
            let check_vars: Vec<&String> = if task_info.is_error_binding {
                task_info.binding_names.iter().collect()
            } else {
                vec![var_name]
            };

            let uses_var = match stmt {
                Stmt::Expr(expr_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&expr_stmt.expr, v)),
                Stmt::If(if_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&if_stmt.condition, v)),
                Stmt::Return(ret_stmt) => ret_stmt.expr.as_ref().map_or(false, |e| {
                    check_vars.iter().any(|v| self.expr_uses_var(e, v))
                }),
                Stmt::While(while_stmt) => check_vars
                    .iter()
                    .any(|v| self.expr_uses_var(&while_stmt.condition, v)),
                Stmt::Assign(assign) => check_vars.iter().any(|v| {
                    self.expr_uses_var(&assign.target, v) || self.expr_uses_var(&assign.value, v)
                }),
                _ => false,
            };

            if uses_var {
                used_tasks.push(var_name.clone());
            }
        }

        used_tasks
    }

    /// Phase 4.2: Check for dead tasks (never awaited) and emit warnings
    fn check_dead_tasks(&self) {
        for (var_name, task_info) in &self.pending_tasks {
            if !task_info.awaited {
                eprintln!(
                    "⚠️  Warning: Task '{}' was created but never used",
                    var_name
                );
                eprintln!("   → Consider removing the task creation or using the variable");
                eprintln!("   → This creates an async/parallel task that does nothing");
            }
        }
    }

    /// Phase 4: Generate tokio::join! for multiple pending tasks (optimization)
    fn generate_tasks_join(&mut self, task_vars: &[String]) -> Result<()> {
        if task_vars.is_empty() {
            return Ok(());
        }

        // Collect task infos and skip already awaited tasks
        let mut tasks_to_join: Vec<(String, TaskInfo)> = Vec::new();
        for var_name in task_vars {
            if let Some(task_info) = self.pending_tasks.get(var_name) {
                if !task_info.awaited {
                    tasks_to_join.push((var_name.clone(), task_info.clone()));
                }
            }
        }

        if tasks_to_join.is_empty() {
            return Ok(());
        }

        // If only one task, use regular await
        if tasks_to_join.len() == 1 {
            return self.generate_task_await(&tasks_to_join[0].0);
        }

        // Generate tokio::join! for multiple tasks
        self.write_indent();
        self.output.push_str("let (");

        // Generate tuple of result variables
        for (i, (var_name, task_info)) in tasks_to_join.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }

            if task_info.is_error_binding {
                // Error binding: (value, err)
                self.output.push('(');
                for (j, binding_name) in task_info.binding_names.iter().enumerate() {
                    if j > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(binding_name);
                }
                self.output.push(')');
            } else {
                // Simple binding: value
                self.output.push_str(var_name);
            }
        }

        self.output.push_str(") = ");

        // Check if all tasks are the same type (all async or all par)
        let all_same_type = tasks_to_join
            .iter()
            .all(|(_, info)| info.exec_policy == tasks_to_join[0].1.exec_policy);

        if !all_same_type {
            // Mixed async/par - fall back to sequential awaits
            // Drop the "let (" we just wrote
            let output_len = self.output.len();
            let last_line_start = self.output[..output_len]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            self.output.truncate(last_line_start);

            // Generate sequential awaits instead
            for (var_name, _) in &tasks_to_join {
                self.generate_task_await(var_name)?;
            }
            return Ok(());
        }

        // Generate tokio::join! macro call
        self.output.push_str("tokio::join!(");

        for (i, (var_name, task_info)) in tasks_to_join.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }

            let task_var_name = format!("{}_task", var_name);

            if task_info.is_error_binding {
                // Error binding: async { match task.await.unwrap() { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) } }
                write!(self.output, "async {{ match {}.await.unwrap() {{ Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) }} }}", 
                    task_var_name).unwrap();
            } else {
                // Simple binding: async { task.await.unwrap() }
                write!(self.output, "async {{ {}.await.unwrap() }}", task_var_name).unwrap();
            }
        }

        self.output.push_str(");\n");

        // Mark all tasks as awaited
        for (var_name, _) in &tasks_to_join {
            if let Some(task_info) = self.pending_tasks.get_mut(var_name) {
                task_info.awaited = true;
            }
        }

        Ok(())
    }

    /// Generate the await code for a pending task (Phase 2: Lazy await)
    fn generate_task_await(&mut self, var_name: &str) -> Result<()> {
        let task_info = self.pending_tasks.get(var_name).cloned();
        if task_info.is_none() {
            return Ok(()); // Not a task or already awaited
        }

        let task_info = task_info.unwrap();
        if task_info.awaited {
            return Ok(()); // Already awaited
        }

        let task_var_name = format!("{}_task", var_name);

        self.write_indent();

        if task_info.is_error_binding {
            // Error binding: Check if the function returns a tuple directly or Result
            write!(self.output, "let (").unwrap();
            for (i, binding_name) in task_info.binding_names.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                write!(self.output, "{}", binding_name).unwrap();
            }
            
            if task_info.returns_tuple {
                // Function returns (Option<T>, String) or (T, String) directly - destructure
                self.output.push_str(") = ");
                
                if task_info.is_http_call {
                    // HTTP calls return (Option<T>, String), unwrap the Option too
                    write!(self.output, "{{ let (opt, err) = {}.await.unwrap(); (opt.unwrap_or_default(), err) }};\n", task_var_name).unwrap();
                } else {
                    // Other tuple-returning functions return (T, String) directly
                    write!(self.output, "{}.await.unwrap();\n", task_var_name).unwrap();
                }
            } else {
                // Function returns Result - match and convert
                self.output.push_str(") = match ");
                write!(self.output, "{}.await.unwrap()", task_var_name).unwrap();
                self.output
                    .push_str(" { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };\n");
            }
        } else {
            // Simple binding: let var_name = var_name_task.await.unwrap();
            write!(
                self.output,
                "let mut {} = {}.await.unwrap();\n",
                var_name, task_var_name
            )
            .unwrap();
        }

        // Mark as awaited
        self.pending_tasks.get_mut(var_name).unwrap().awaited = true;

        Ok(())
    }

    /// Generate code for method calls (stdlib Phase 2 - array methods)
    fn generate_method_call_expr(&mut self, method_call: &crate::ast::MethodCallExpr) -> Result<()> {
        use crate::ast::ArrayAdapter;
        
        // Check if this is a Math function call (Math.sqrt, Math.pow, etc.)
        if let Expr::Identifier(name) = method_call.object.as_ref() {
            if name == "Math" {
                return self.generate_math_function_call(method_call);
            }
            
            // Check if this is a console function call (console.log, console.error, etc.)
            if name == "console" {
                return self.generate_console_function_call(method_call);
            }
            
            // Check if this is a JSON function call (JSON.parse, JSON.stringify)
            if name == "JSON" {
                return self.generate_json_function_call(method_call);
            }
            
            // Check if this is a File function call (File.read, File.write, etc.)
            if name == "File" {
                return self.generate_file_function_call(method_call);
            }
            
            // Check if this is an HTTP function call (HTTP.get, HTTP.post, etc.)
            if name == "HTTP" || name == "Http" {
                return self.generate_http_function_call(method_call);
            }
            
            // Check if this is a Sys function call (Sys.args, Sys.env, etc.)
            if name == "Sys" {
                return self.generate_sys_function_call(method_call);
            }
        }
        
        // Check if this is a string method (no adapter means it's not an array method)
        // Special case: indexOf can be both string and array method
        // We detect string indexOf if:
        // 1. The argument is a string literal, OR
        // 2. The argument is a known string variable, OR
        // 3. The object is a member access on 'this' (class field likely string)
        let is_string_indexof = method_call.method == "indexOf" && !method_call.args.is_empty() && {
            // Check if argument is string literal
            let arg_is_string_lit = matches!(&method_call.args[0], Expr::Literal(Literal::String(_)));
            // Check if argument is a known string variable
            let arg_is_string_var = if let Expr::Identifier(var_name) = &method_call.args[0] {
                self.string_vars.contains(&self.sanitize_name(var_name))
            } else { false };
            // Check if object is this.field (member access on this/self)
            let object_is_this_field = if let Expr::Member { object, .. } = method_call.object.as_ref() {
                matches!(object.as_ref(), Expr::Identifier(name) if name == "this" || name == "self")
            } else { false };
            
            arg_is_string_lit || arg_is_string_var || object_is_this_field
        };
        
        let is_string_method = (matches!(method_call.adapter, ArrayAdapter::Seq) 
            && matches!(
                method_call.method.as_str(),
                "split" | "replace" | "toUpperCase" | "toLowerCase" | 
                "trim" | "trimStart" | "trimEnd" | "startsWith" | "endsWith" |
                "substring" | "charAt"
            )) || is_string_indexof;
        
        if is_string_method {
            // Handle string methods
            return self.generate_string_method_call(method_call);
        }
        
        // Check if this is a method on HTTP Response (e.g., response.json())
        if let Expr::Identifier(var_name) = method_call.object.as_ref() {
            if self.rust_struct_vars.contains(var_name) && method_call.method == "json" {
                // This is response.json() - generate as method call
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".json()");
                return Ok(());
            }
        }
        
        // Check if the object is Option<JsonValue> (from JSON.parse with error binding)
        // If so, we need to unwrap it before calling array methods
        let is_option_json_value = if let Expr::Identifier(var_name) = method_call.object.as_ref() {
            let sanitized = self.sanitize_name(var_name);
            self.option_value_vars.contains(&sanitized) && self.json_value_vars.contains(&sanitized)
        } else {
            false
        };
        
        // Bug #36: For binary expressions as method call object, we need parentheses
        // e.g., (arr.length - 1).toString() should generate ((arr.len() as i32) - 1).to_string()
        // Without parens, `- 1.to_string()` has wrong precedence
        let needs_parens_for_binary = matches!(method_call.object.as_ref(), Expr::Binary { .. });
        
        if needs_parens_for_binary {
            self.output.push('(');
        }
        
        // Generate the object
        self.generate_expr(&method_call.object)?;
        
        if needs_parens_for_binary {
            self.output.push(')');
        }
        
        // Unwrap Option<JsonValue> before calling methods
        if is_option_json_value {
            self.output.push_str(".as_ref().unwrap()");
        }
        
        // Check if operating on JsonValue
        let is_json_value = self.is_json_value_expr(&method_call.object);
        let is_direct_json = self.is_direct_json_value(&method_call.object);
        
        // Handle array methods with adapters
        match method_call.adapter {
            ArrayAdapter::Seq => {
                // Sequential: use .iter() but with special handling for JsonValue
                match method_call.method.as_str() {
                    "map" => {
                        // For map, use iter()
                        // For Vec<JsonValue>, add .cloned() to clone elements
                        self.output.push_str(".iter()");
                        if is_json_value && !is_direct_json {
                            // Vec<JsonValue> needs cloned()
                            self.output.push_str(".cloned()");
                        }
                    }
                    "filter" => {
                        // For filter, use iter()
                        // For Vec<JsonValue>, add .cloned() to clone elements
                        self.output.push_str(".iter()");
                        if is_json_value && !is_direct_json {
                            // Vec<JsonValue> needs cloned()
                            self.output.push_str(".cloned()");
                        }
                    }
                    "reduce" => {
                        // reduce doesn't use .iter() - it operates directly on the vector
                        // We use .fold() which requires initial value and accumulator
                    }
                    "forEach" => {
                        // For forEach, use iter()
                        // For Vec<JsonValue>, add .cloned() to clone elements
                        self.output.push_str(".iter()");
                        if is_json_value && !is_direct_json {
                            // Vec<JsonValue> needs cloned()
                            self.output.push_str(".cloned()");
                        }
                    }
                    "find" | "some" | "every" | "indexOf" | "includes" => {
                        self.output.push_str(".iter()");
                        if is_json_value && !is_direct_json {
                            self.output.push_str(".cloned()");
                        }
                    }
                    _ => {
                        // For other methods, call directly
                    }
                }
            }
            ArrayAdapter::Par => {
                // Parallel: use rayon's .par_iter()
                // For JsonValue, need to convert to Vec first
                if is_direct_json {
                    self.output.push_str(".to_vec().into_par_iter()");
                } else {
                    self.output.push_str(".par_iter()");
                }
                
                // TODO: Handle adapter options (threads, chunk, ordered)
                if method_call.adapter_options.threads.is_some()
                    || method_call.adapter_options.chunk.is_some()
                {
                    // For now, just use default parallel iterator
                    // TODO: Configure rayon thread pool with options
                }
            }
            ArrayAdapter::Vec => {
                // Vectorized: use SIMD
                // TODO: Implement SIMD version
                self.output.push_str(".into_iter()");
            }
            ArrayAdapter::ParVec => {
                // Parallel + Vectorized: use rayon parallel iterator
                // For JsonValue, need to convert to Vec first
                if is_direct_json {
                    self.output.push_str(".to_vec().into_par_iter()");
                } else {
                    self.output.push_str(".par_iter()");
                }
                // TODO: Implement SIMD optimizations on top of parallel
            }
        }
        
        // Generate the method call
        
        // Special handling for reduce: it uses .iter() on the vector itself
        if method_call.method == "reduce" && matches!(method_call.adapter, ArrayAdapter::Seq) {
            self.output.push_str(".iter()");
        }
        
        self.output.push('.');
        
        // Map Liva method names to Rust iterator method names
        let rust_method = match method_call.method.as_str() {
            "forEach" => "for_each".to_string(),
            "indexOf" => "position".to_string(),
            "includes" => "any".to_string(),
            "reduce" => "fold".to_string(),  // Rust uses fold instead of reduce
            "some" => "any".to_string(),      // Liva: some, Rust: any
            "every" => "all".to_string(),     // Liva: every, Rust: all
            method_name => self.sanitize_name(method_name),  // Sanitize custom method names (e.g., isAdult -> is_adult)
        };
        
        self.output.push_str(&rust_method);
        self.output.push('(');
        
        // Generate arguments
        // Special case: reduce needs arguments reversed (initial first, then lambda)
        let args_to_generate: Vec<&Expr> = if method_call.method == "reduce" && method_call.args.len() == 2 {
            // Liva: .reduce(lambda, initial) -> Rust: .fold(initial, lambda)
            vec![&method_call.args[1], &method_call.args[0]]
        } else {
            method_call.args.iter().collect()
        };
        
        for (i, arg) in args_to_generate.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            
            // Special handling for includes/indexOf: wrap value in closure
            if method_call.method == "includes" || method_call.method == "indexOf" {
                self.output.push_str("|&x| x == ");
                self.generate_expr(arg)?;
                continue;
            }
            
            // Convert string literals to String for methods/functions
            // This avoids "expected String, found &str" errors
            if matches!(arg, Expr::Literal(Literal::String(_))) {
                // For array methods, JsonValue methods, and join (which expects &str), don't convert
                let is_array_or_json_method = matches!(
                    method_call.method.as_str(),
                    "map" | "filter" | "reduce" | "forEach" | "find" | "some" | "every" | "indexOf" | "includes" | "get" | "get_field" | "join"
                );
                
                if !is_array_or_json_method {
                    self.generate_expr(arg)?;
                    self.output.push_str(".to_string()");
                    continue;
                }
            }
            
            // For map/filter/reduce/forEach/find/some/every with .iter(), we need to dereference in the lambda
            // map: |&x| - filter: |&&x| (for .copied()) or |x| (for .cloned())
            // reduce: |acc, &x| - forEach: |&x| - find: |&&x| or |x| - some: |&&x| or |x| - every: |&&x| or |x|
            // EXCEPTION: JsonValue.iter() and JsonValue.to_vec().into_par_iter() return owned values, so no dereferencing needed
            // For parallel: .par_iter() uses &T, but .into_par_iter() (from .to_vec()) uses T (owned)
            let is_json_value = self.is_json_value_expr(&method_call.object);
            
            // Determine if we'll use .cloned() (for non-Copy types) which changes the lambda pattern
            // With .copied() (Copy types): filter(|&&x| ...) 
            // With .cloned() (Clone but not Copy types): filter(|x| ...)
            let will_use_cloned = if let Some(base_var_name) = self.get_base_var_name(&method_call.object) {
                if let Some(element_type) = self.typed_array_vars.get(&base_var_name) {
                    // Check if element type is a Copy type (class names are non-Copy)
                    // Bug #35: "string" is not Copy, so forEach needs |p| not |&p|
                    !matches!(element_type.as_str(), "number" | "int" | "i32" | "float" | "f64" | "bool" | "char")
                } else if self.string_vars.contains(&base_var_name) {
                    true
                } else if self.array_vars.contains(&base_var_name) && !self.json_value_vars.contains(&base_var_name) {
                    true
                } else {
                    false
                }
            } else {
                true // Default to cloned for safety
            };
            
            let needs_lambda_pattern = 
                (method_call.method == "map" || method_call.method == "filter" || method_call.method == "reduce" || method_call.method == "forEach" || method_call.method == "find" || method_call.method == "some" || method_call.method == "every")
                && (matches!(method_call.adapter, ArrayAdapter::Seq) 
                    || (matches!(method_call.adapter, ArrayAdapter::Par | ArrayAdapter::ParVec) && is_json_value));
            
            if needs_lambda_pattern {
                if let Expr::Lambda(lambda) = arg {
                    // Track lambda parameter types for typed arrays
                    // If the object is a typed array (e.g., posts: [Post]), track that the param is Post
                    let element_type = if let Some(base_var_name) = self.get_base_var_name(&method_call.object) {
                        self.typed_array_vars.get(&base_var_name).cloned()
                    } else {
                        None
                    };
                    
                    if let Some(ref elem_type) = element_type {
                        // Track the lambda parameter as an instance of this class type
                        for param in &lambda.params {
                            if let Some(name) = param.name() {
                                let param_name = self.sanitize_name(name);
                                self.class_instance_vars.insert(param_name);
                            }
                        }
                    }
                    
                    // Generate lambda with pattern |&x| or |&&x| or |acc, &x| (unless JsonValue)
                    if lambda.is_move {
                        self.output.push_str("move ");
                    }
                    self.output.push('|');
                    for (idx, param) in lambda.params.iter().enumerate() {
                        if idx > 0 {
                            self.output.push_str(", ");
                        }
                        
                        // Get parameter name (temp name if destructured)
                        let param_name = if param.is_destructuring() {
                            format!("_param_{}", idx)
                        } else {
                            self.sanitize_name(param.name().unwrap())
                        };
                        
                        // reduce: first param (acc) no pattern, second param (&x) gets &
                        if method_call.method == "reduce" {
                            if idx == 0 {
                                // Accumulator: no dereferencing
                                self.output.push_str(&param_name);
                            } else {
                                // Element: dereference once (unless JsonValue or destructured)
                                if !is_json_value && !param.is_destructuring() {
                                    self.output.push('&');
                                }
                                self.output.push_str(&param_name);
                            }
                        } else {
                            // filter/find/some/every need different patterns based on whether we'll use .cloned() or .copied()
                            // - With .copied() (Copy types): filter(|&&x| ...) - double deref
                            // - With .cloned() (Clone types): filter(|x| x...) - no deref, closure receives &&T but cloned() handles it
                            // map/forEach need & for Copy types (closure takes &T), but for non-Copy types,
                            // we work with references directly to avoid moving
                            // UNLESS it's JsonValue, then no dereferencing at all
                            // ALSO: if parameter is destructured, don't add & because we'll clone inside
                            if !is_json_value && !param.is_destructuring() {
                                if method_call.method == "filter" || method_call.method == "find" || method_call.method == "some" || method_call.method == "every" {
                                    // For .cloned(), don't add any pattern - we work with references
                                    if !will_use_cloned {
                                        self.output.push_str("&&");
                                    }
                                    // If will_use_cloned, no prefix needed - the closure will receive &&T 
                                    // but we just use it as a reference
                                } else if method_call.method == "map" || method_call.method == "forEach" {
                                    // Bug #22/#35 fix: For non-Copy types (class instances, strings),
                                    // don't add & because we can't move out of a shared reference
                                    if !will_use_cloned {
                                        self.output.push('&');
                                    }
                                    // For non-Copy types (will_use_cloned = true), no prefix - work with &T directly
                                } else {
                                    self.output.push('&');
                                }
                            }
                            self.output.push_str(&param_name);
                        }
                    }
                    self.output.push_str("| ");
                    
                    // Check if we need to generate destructuring code for lambda params
                    let has_destructuring = lambda.params.iter().any(|p| p.is_destructuring());
                    
                    match &lambda.body {
                        LambdaBody::Expr(expr) => {
                            if has_destructuring {
                                // Need to wrap in block to add destructuring
                                self.output.push('{');
                                self.indent();
                                self.output.push('\n');
                                
                                // Generate destructuring for each param
                                for (idx, param) in lambda.params.iter().enumerate() {
                                    if param.is_destructuring() {
                                        let temp_name = format!("_param_{}", idx);
                                        self.write_indent();
                                        self.generate_lambda_param_destructuring(&param.pattern, &temp_name, is_json_value, element_type.as_deref())?;
                                        self.output.push('\n');
                                    }
                                }
                                
                                self.write_indent();
                                self.generate_expr(expr)?;
                                self.output.push('\n');
                                self.dedent();
                                self.write_indent();
                                self.output.push('}');
                            } else {
                                self.generate_expr(expr)?;
                            }
                        }
                        LambdaBody::Block(block) => {
                            self.output.push('{');
                            self.indent();
                            self.output.push('\n');
                            
                            // Generate destructuring for lambda params (if any)
                            if has_destructuring {
                                for (idx, param) in lambda.params.iter().enumerate() {
                                    if param.is_destructuring() {
                                        let temp_name = format!("_param_{}", idx);
                                        self.write_indent();
                                        self.generate_lambda_param_destructuring(&param.pattern, &temp_name, is_json_value, element_type.as_deref())?;
                                        self.output.push('\n');
                                    }
                                }
                            }
                            
                            self.write_indent();
                            for stmt in &block.stmts[..block.stmts.len().saturating_sub(1)] {
                                self.generate_stmt(stmt)?;
                                self.output.push('\n');
                                self.write_indent();
                            }
                            if let Some(last_stmt) = block.stmts.last() {
                                if let Stmt::Return(return_stmt) = last_stmt {
                                    if let Some(expr) = &return_stmt.expr {
                                        self.generate_expr(expr)?;
                                    }
                                } else {
                                    self.generate_stmt(last_stmt)?;
                                }
                            }
                            self.dedent();
                            self.output.push('\n');
                            self.write_indent();
                            self.output.push('}');
                        }
                    }
                    continue;
                }
            }
            
            // Track lambda parameter types for typed arrays BEFORE generating the lambda
            // This handles ParVec/Par forEach/map/etc with typed arrays (not JsonValue)
            if let Expr::Lambda(lambda) = arg {
                if matches!(method_call.method.as_str(), "forEach" | "map" | "filter" | "reduce" | "find" | "some" | "every") {
                    if let Some(base_var_name) = self.get_base_var_name(&method_call.object) {
                        if let Some(element_type) = self.typed_array_vars.get(&base_var_name).cloned() {
                            // Set current element type for lambda generation
                            self.current_lambda_element_type = Some(element_type.clone());
                            
                            // Track the lambda parameter as an instance of this class type
                            for param in &lambda.params {
                                if let Some(name) = param.name() {
                                    let param_name = self.sanitize_name(name);
                                    self.class_instance_vars.insert(param_name);
                                }
                            }
                        }
                    }
                }
            }
            
            self.generate_expr(arg)?;
            
            // Clear current element type after generating lambda
            self.current_lambda_element_type = None;
        }
        
        self.output.push(')');
        
        // Add transformations after the method call
        let is_json_value = self.is_json_value_expr(&method_call.object);
        
        // Determine if the array contains non-Copy types (String, classes, etc.)
        // Non-Copy types need .cloned() instead of .copied()
        // Copy types: number, int, i32, float, f64, bool, char
        // Non-Copy types: string, String, and any class name
        let needs_clone_not_copy = if let Some(base_var_name) = self.get_base_var_name(&method_call.object) {
            if let Some(element_type) = self.typed_array_vars.get(&base_var_name) {
                // Check if element type is a Copy type (class names are non-Copy)
                !matches!(element_type.as_str(), "number" | "int" | "i32" | "float" | "f64" | "bool" | "char")
            } else if self.string_vars.contains(&base_var_name) {
                // String arrays explicitly need .cloned()
                true
            } else if self.array_vars.contains(&base_var_name) && !self.json_value_vars.contains(&base_var_name) {
                // For arrays without explicit type info but not JsonValue,
                // default to .cloned() as it's safer (works for both Copy and non-Copy)
                true
            } else {
                false
            }
        } else {
            // No base variable name - default to .cloned() for safety
            true
        };
        
        match (method_call.adapter, method_call.method.as_str()) {
            // Sequential map: just collect (lambda already returns owned values)
            (ArrayAdapter::Seq, "map") => {
                self.output.push_str(".collect::<Vec<_>>()");
            }
            // Sequential filter: copy/clone values after filtering, then collect
            // - JsonValue: no copy needed (already returns owned values)
            // - Non-Copy types (String, classes): use .cloned()
            // - Copy types (i32, f64, bool, char): use .copied()
            (ArrayAdapter::Seq, "filter") => {
                if is_json_value {
                    self.output.push_str(".collect::<Vec<_>>()");
                } else if needs_clone_not_copy {
                    self.output.push_str(".cloned().collect::<Vec<_>>()");
                } else {
                    self.output.push_str(".copied().collect::<Vec<_>>()");
                }
            }
            // Parallel map/filter with rayon
            (ArrayAdapter::Par, "map") | (ArrayAdapter::ParVec, "map") => {
                // Map returns owned values (from the lambda), just collect
                self.output.push_str(".collect::<Vec<_>>()");
            }
            (ArrayAdapter::Par, "filter") | (ArrayAdapter::ParVec, "filter") => {
                // Filter returns references, need to clone before collecting
                self.output.push_str(".cloned().collect::<Vec<_>>()");
            }
            // Find returns Option<&T>, copy/clone it
            // - JsonValue: no copy needed (already returns owned values)
            // - Non-Copy types: use .cloned()
            // - Copy types: use .copied()
            (_, "find") => {
                if !is_json_value {
                    if needs_clone_not_copy {
                        self.output.push_str(".cloned()");
                    } else {
                        self.output.push_str(".copied()");
                    }
                }
            }
            // indexOf/position returns Option<usize>
            (_, "indexOf") => {
                self.output.push_str(".map(|i| i as i32).unwrap_or(-1)");
            }
            // some, every, includes return bool - no transformation needed
            (_, "some") | (_, "every") | (_, "includes") => {}
            // Bug #38: JsonValue conversion methods return Option<T>, unwrap to T
            // asString -> as_string().unwrap_or_default()
            // asBool -> as_bool().unwrap_or_default()
            // asInt -> as_i32().unwrap_or_default()
            // asFloat -> as_f64().unwrap_or_default()
            (_, "asString") | (_, "asInt") | (_, "asFloat") | (_, "asBool") => {
                self.output.push_str(".unwrap_or_default()");
            }
            // Default: no transformation
            _ => {}
        }
        
        Ok(())
    }

    fn generate_string_method_call(&mut self, method_call: &crate::ast::MethodCallExpr) -> Result<()> {
        // Special handling for substring and charAt - they need different syntax
        match method_call.method.as_str() {
            "substring" => {
                // substring(start, end) -> &str[start..end].to_string()
                self.generate_expr(&method_call.object)?;
                self.output.push('[');
                if method_call.args.len() >= 1 {
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(" as usize");
                }
                self.output.push_str("..");
                if method_call.args.len() >= 2 {
                    self.generate_expr(&method_call.args[1])?;
                    self.output.push_str(" as usize");
                }
                self.output.push_str("].to_string()");
                return Ok(());
            }
            "charAt" => {
                // charAt(index) -> str.chars().nth(index).unwrap_or(' ')
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".chars().nth(");
                if !method_call.args.is_empty() {
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(" as usize");
                }
                self.output.push_str(").unwrap_or(' ')");
                return Ok(());
            }
            "indexOf" => {
                // indexOf(substring) -> str.find(substring).map(|i| i as i32).unwrap_or(-1)
                self.generate_expr(&method_call.object)?;
                self.output.push_str(".find(");
                if !method_call.args.is_empty() {
                    // If the argument is a String variable, we need &arg for find() to work
                    // find() expects a Pattern, and &String implements Pattern but String doesn't
                    let needs_ref = match &method_call.args[0] {
                        Expr::Identifier(var_name) => {
                            self.string_vars.contains(&self.sanitize_name(var_name))
                        }
                        Expr::Member { .. } => true, // Member access on fields - likely strings
                        Expr::Literal(Literal::String(_)) => false, // String literals are fine
                        _ => true, // Be safe and add & for other cases
                    };
                    if needs_ref {
                        self.output.push('&');
                    }
                    self.generate_expr(&method_call.args[0])?;
                }
                self.output.push_str(").map(|i| i as i32).unwrap_or(-1)");
                return Ok(());
            }
            _ => {}
        }
        
        // Generate the string object
        self.generate_expr(&method_call.object)?;
        
        // Map Liva string method names to Rust method names
        let rust_method = match method_call.method.as_str() {
            "toUpperCase" => "to_uppercase",
            "toLowerCase" => "to_lowercase",
            "trimStart" => "trim_start",
            "trimEnd" => "trim_end",
            "startsWith" => "starts_with",
            "endsWith" => "ends_with",
            method_name => method_name,  // split, replace, trim, substring, charAt
        };
        
        // Generate the method call
        self.output.push('.');
        self.output.push_str(rust_method);
        self.output.push('(');
        
        // Generate arguments
        for (i, arg) in method_call.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.generate_expr(arg)?;
        }
        
        self.output.push(')');
        
        // Post-processing for specific methods
        match method_call.method.as_str() {
            "split" => {
                // split returns an iterator, collect to Vec<String>
                self.output.push_str(".map(|s| s.to_string()).collect::<Vec<String>>()");
            }
            _ => {}
        }
        
        Ok(())
    }

    fn generate_math_function_call(&mut self, method_call: &crate::ast::MethodCallExpr) -> Result<()> {
        // Math functions: sqrt, pow, abs, floor, ceil, round, min, max, random
        match method_call.method.as_str() {
            "sqrt" | "abs" => {
                // sqrt(x) -> x.sqrt() or abs(x) -> x.abs()
                if method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            &format!("Math.{} requires 1 argument", method_call.method),
                            &format!("Math.{} takes exactly one argument", method_call.method)
                        )
                    ));
                }
                
                // Wrap argument in parentheses if it's a unary expression to avoid precedence issues
                let needs_parens = matches!(&method_call.args[0], Expr::Unary { .. });
                
                if needs_parens {
                    self.output.push('(');
                }
                self.generate_expr(&method_call.args[0])?;
                if needs_parens {
                    self.output.push(')');
                }
                
                self.output.push('.');
                self.output.push_str(&method_call.method);
                self.output.push_str("()");
            }
            "pow" => {
                // pow(base, exp) -> base.powf(exp)
                if method_call.args.len() < 2 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "Math.pow requires 2 arguments",
                            "Math.pow(base, exponent) takes exactly two arguments"
                        )
                    ));
                }
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".powf(");
                self.generate_expr(&method_call.args[1])?;
                self.output.push(')');
            }
            "floor" | "ceil" | "round" => {
                // floor(x) -> x.floor() as i32
                if method_call.args.is_empty() {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            &format!("Math.{} requires 1 argument", method_call.method),
                            &format!("Math.{} takes exactly one argument", method_call.method)
                        )
                    ));
                }
                self.generate_expr(&method_call.args[0])?;
                self.output.push('.');
                self.output.push_str(&method_call.method);
                self.output.push_str("() as i32");
            }
            "min" | "max" => {
                // min(a, b) -> a.min(b) or max(a, b) -> a.max(b)
                if method_call.args.len() < 2 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            &format!("Math.{} requires 2 arguments", method_call.method),
                            &format!("Math.{} takes exactly two arguments", method_call.method)
                        )
                    ));
                }
                self.generate_expr(&method_call.args[0])?;
                self.output.push('.');
                self.output.push_str(&method_call.method);
                self.output.push('(');
                self.generate_expr(&method_call.args[1])?;
                self.output.push(')');
            }
            "random" => {
                // random() -> rand::random::<f64>()
                // Note: requires use rand::Rng in the generated code
                self.output.push_str("rand::random::<f64>()");
            }
            _ => {
                return Err(CompilerError::CodegenError(
                    SemanticErrorInfo::new(
                        "E3000",
                        &format!("Unknown Math function: {}", method_call.method),
                        "Available Math functions: sqrt, pow, abs, floor, ceil, round, min, max, random"
                    )
                ));
            }
        }
        
        Ok(())
    }

    fn generate_console_function_call(&mut self, method_call: &crate::ast::MethodCallExpr) -> Result<()> {
        // Console functions: log, error, warn
        match method_call.method.as_str() {
            "log" => {
                // console.log(...) -> println!(...)
                if method_call.args.is_empty() {
                    self.output.push_str("println!()");
                } else {
                    // Check if first arg is a string literal with format placeholders
                    // If so, use it as the format string, otherwise generate default format
                    self.output.push_str("println!(");
                    
                    for (i, arg) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg)?;
                    }
                    self.output.push(')');
                }
            }
            "error" => {
                // console.error(...) -> eprintln!(...) in red color (ANSI escape codes)
                if method_call.args.is_empty() {
                    self.output.push_str("eprintln!()");
                } else {
                    self.output.push_str("eprintln!(\"\\x1b[31m");  // Red color start
                    // Check if first arg is format string
                    if method_call.args.len() == 1 {
                        // Single arg - just print it with reset
                        self.output.push_str("{}\\x1b[0m\", ");
                        self.generate_expr(&method_call.args[0])?;
                    } else if let Expr::Literal(Literal::String(fmt)) = &method_call.args[0] {
                        // First arg is string literal - use as format string
                        // Escape the format string properly
                        for ch in fmt.chars() {
                            match ch {
                                '"' => self.output.push_str("\\\""),
                                '\\' => self.output.push_str("\\\\"),
                                '\n' => self.output.push_str("\\n"),
                                '\r' => self.output.push_str("\\r"),
                                '\t' => self.output.push_str("\\t"),
                                _ => self.output.push(ch),
                            }
                        }
                        self.output.push_str("\\x1b[0m\", ");
                        for (i, arg) in method_call.args.iter().skip(1).enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            self.generate_expr(arg)?;
                        }
                    } else {
                        // Multiple args without format string - generate default
                        for (i, _) in method_call.args.iter().enumerate() {
                            if i > 0 {
                                self.output.push(' ');
                            }
                            self.output.push_str("{}");
                        }
                        self.output.push_str("\\x1b[0m\", ");
                        for (i, arg) in method_call.args.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            self.generate_expr(arg)?;
                        }
                    }
                    self.output.push(')');
                }
            }
            "warn" => {
                // console.warn(...) -> eprintln!(...) in amber/yellow color (ANSI escape codes)
                // Yellow: \x1b[33m ... \x1b[0m
                if method_call.args.is_empty() {
                    self.output.push_str("eprintln!()");
                } else {
                    self.output.push_str("eprintln!(\"\\x1b[33m");  // Yellow color
                    for (i, _) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push(' ');  // Space between arguments
                        }
                        self.output.push_str("{}");
                    }
                    self.output.push_str("\\x1b[0m\", ");  // Reset color
                    for (i, arg) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg)?;
                    }
                    self.output.push(')');
                }
            }
            "success" => {
                // console.success(...) -> println!(...) in green color (ANSI escape codes)
                // Green: \x1b[32m ... \x1b[0m
                if method_call.args.is_empty() {
                    self.output.push_str("println!()");
                } else {
                    self.output.push_str("println!(\"\\x1b[32m");  // Green color
                    for (i, _) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push(' ');  // Space between arguments
                        }
                        self.output.push_str("{}");
                    }
                    self.output.push_str("\\x1b[0m\", ");  // Reset color
                    for (i, arg) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg)?;
                    }
                    self.output.push(')');
                }
            }
            "input" => {
                // console.input() -> reads user input from stdin without prompt
                // console.input(message) -> prints message, then reads input
                // Returns a string with the user input (trimmed)
                // Similar to Python's input() and input("prompt")
                self.output.push_str("{\n");
                self.output.push_str("use std::io::{self, Write};\n");
                
                // Print the prompt message if provided
                if !method_call.args.is_empty() {
                    self.output.push_str("print!(");
                    self.generate_expr(&method_call.args[0])?;
                    self.output.push_str(");\n");
                    // Flush to ensure prompt is displayed before reading
                    self.output.push_str("io::stdout().flush().unwrap();\n");
                }
                
                // Read user input
                self.output.push_str("let mut input = String::new();\n");
                self.output.push_str("io::stdin().read_line(&mut input).unwrap();\n");
                self.output.push_str("input.trim().to_string()\n");
                self.output.push_str("}");
            }
            _ => {
                return Err(CompilerError::CodegenError(
                    SemanticErrorInfo::new(
                        "E3000",
                        &format!("Unknown console function: {}", method_call.method),
                        "Available console functions: log, error, warn, success, input"
                    )
                ));
            }
        }
        
        Ok(())
    }

    fn generate_json_function_call(&mut self, method_call: &crate::ast::MethodCallExpr) -> Result<()> {
        // JSON functions: parse, stringify
        match method_call.method.as_str() {
            "parse" => {
                // JSON.parse(json_str) returns (Option<JsonValue>, String)
                // Generates: match serde_json::from_str(...) { Ok(v) => (Some(JsonValue(v)), String::new()), Err(e) => (None, format!("...")) }
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "JSON.parse requires exactly 1 argument",
                            "Usage: JSON.parse(json_string)"
                        )
                    ));
                }
                
                self.output.push_str("(match serde_json::from_str::<serde_json::Value>(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(v) => (Some(liva_rt::JsonValue::new(v)), String::new()), Err(e) => (None, format!(\"JSON parse error: {}\", e)) })");
            }
            "stringify" => {
                // JSON.stringify(value) returns (Option<String>, String)
                // Generates: match serde_json::to_string(...) { Ok(s) => (Some(s), String::new()), Err(e) => (None, format!("...")) }
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "JSON.stringify requires exactly 1 argument",
                            "Usage: JSON.stringify(value)"
                        )
                    ));
                }
                
                self.output.push_str("(match serde_json::to_string(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(s) => (Some(s), String::new()), Err(e) => (None, format!(\"JSON stringify error: {}\", e)) })");
            }
            _ => {
                return Err(CompilerError::CodegenError(
                    SemanticErrorInfo::new(
                        "E3000",
                        &format!("Unknown JSON function: {}", method_call.method),
                        "Available JSON functions: parse, stringify"
                    )
                ));
            }
        }
        
        Ok(())
    }

    fn generate_file_function_call(&mut self, method_call: &crate::ast::MethodCallExpr) -> Result<()> {
        // File functions: read, write, append, exists, delete
        match method_call.method.as_str() {
            "read" => {
                // File.read(path) returns (Option<String>, String)
                // Error is "" on success, error message on failure
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "File.read requires exactly 1 argument",
                            "Usage: File.read(path)"
                        )
                    ));
                }
                
                self.output.push_str("(match std::fs::read_to_string(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(content) => (Some(content), String::new()), Err(e) => (None, format!(\"File read error: {}\", e)) })");
            }
            "write" => {
                // File.write(path, content) returns (Option<bool>, String)
                // Error is "" on success, error message on failure
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "File.write requires exactly 2 arguments",
                            "Usage: File.write(path, content)"
                        )
                    ));
                }
                
                self.output.push_str("(match std::fs::write(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(", &");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(") { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"File write error: {}\", e)) })");
            }
            "append" => {
                // File.append(path, content) returns (Option<bool>, String)
                // Error is "" on success, error message on failure
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "File.append requires exactly 2 arguments",
                            "Usage: File.append(path, content)"
                        )
                    ));
                }
                
                self.output.push_str("(match std::fs::OpenOptions::new().create(true).append(true).open(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").and_then(|mut file| { use std::io::Write; file.write_all(");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(".as_bytes()) }) { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"File append error: {}\", e)) })");
            }
            "exists" => {
                // File.exists(path) returns bool (no error binding)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "File.exists requires exactly 1 argument",
                            "Usage: File.exists(path)"
                        )
                    ));
                }
                
                self.output.push_str("std::path::Path::new(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").exists()");
            }
            "delete" => {
                // File.delete(path) returns (Option<bool>, String)
                // Error is "" on success, error message on failure
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "File.delete requires exactly 1 argument",
                            "Usage: File.delete(path)"
                        )
                    ));
                }
                
                self.output.push_str("(match std::fs::remove_file(&");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(") { Ok(_) => (Some(true), String::new()), Err(e) => (Some(false), format!(\"File delete error: {}\", e)) })");
            }
            _ => {
                return Err(CompilerError::CodegenError(
                    SemanticErrorInfo::new(
                        "E3000",
                        &format!("Unknown File function: {}", method_call.method),
                        "Available File functions: read, write, append, exists, delete"
                    )
                ));
            }
        }
        
        Ok(())
    }

    fn generate_http_function_call(&mut self, method_call: &crate::ast::MethodCallExpr) -> Result<()> {
        // HTTP functions: get, post, put, delete
        // All return (Option<LivaHttpResponse>, Option<String>)
        match method_call.method.as_str() {
            "get" => {
                // HTTP.get(url) returns (Option<LivaHttpResponse>, Option<String>)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "HTTP.get requires exactly 1 argument",
                            "Usage: HTTP.get(url)"
                        )
                    ));
                }
                
                self.output.push_str("liva_rt::liva_http_get(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".to_string())");
            }
            "post" => {
                // HTTP.post(url, body) returns (Option<LivaHttpResponse>, Option<String>)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "HTTP.post requires exactly 2 arguments",
                            "Usage: HTTP.post(url, body)"
                        )
                    ));
                }
                
                self.output.push_str("liva_rt::liva_http_post(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".to_string(), ");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(".to_string())");
            }
            "put" => {
                // HTTP.put(url, body) returns (Option<LivaHttpResponse>, Option<String>)
                if method_call.args.len() != 2 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "HTTP.put requires exactly 2 arguments",
                            "Usage: HTTP.put(url, body)"
                        )
                    ));
                }
                
                self.output.push_str("liva_rt::liva_http_put(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".to_string(), ");
                self.generate_expr(&method_call.args[1])?;
                self.output.push_str(".to_string())");
            }
            "delete" => {
                // HTTP.delete(url) returns (Option<LivaHttpResponse>, Option<String>)
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "HTTP.delete requires exactly 1 argument",
                            "Usage: HTTP.delete(url)"
                        )
                    ));
                }
                
                self.output.push_str("liva_rt::liva_http_delete(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(".to_string())");
            }
            _ => {
                return Err(CompilerError::CodegenError(
                    SemanticErrorInfo::new(
                        "E3000",
                        &format!("Unknown HTTP function: {}", method_call.method),
                        "Available HTTP functions: get, post, put, delete"
                    )
                ));
            }
        }
        
        Ok(())
    }

    fn generate_sys_function_call(&mut self, method_call: &crate::ast::MethodCallExpr) -> Result<()> {
        // Sys functions: args, env, exit
        match method_call.method.as_str() {
            "args" => {
                // Sys.args() returns [string] - command line arguments
                // Returns all args including program name at index 0
                self.output.push_str("std::env::args().collect::<Vec<String>>()");
            }
            "env" => {
                // Sys.env(key) returns string - environment variable value
                // Returns empty string if not found
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "Sys.env requires exactly 1 argument",
                            "Usage: Sys.env(\"VAR_NAME\")"
                        )
                    ));
                }
                
                self.output.push_str("std::env::var(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(").unwrap_or_default()");
            }
            "exit" => {
                // Sys.exit(code) - exit program with code
                if method_call.args.len() != 1 {
                    return Err(CompilerError::CodegenError(
                        SemanticErrorInfo::new(
                            "E3000",
                            "Sys.exit requires exactly 1 argument",
                            "Usage: Sys.exit(0)"
                        )
                    ));
                }
                
                self.output.push_str("std::process::exit(");
                self.generate_expr(&method_call.args[0])?;
                self.output.push_str(" as i32)");
            }
            _ => {
                return Err(CompilerError::CodegenError(
                    SemanticErrorInfo::new(
                        "E3000",
                        &format!("Unknown Sys function: {}", method_call.method),
                        "Available Sys functions: args, env, exit"
                    )
                ));
            }
        }
        
        Ok(())
    }

    fn generate_async_call(&mut self, call: &CallExpr) -> Result<()> {
        // Phase 2: NO await here - just create the Task
        // The await will be inserted at first use of the variable
        self.output.push_str("liva_rt::spawn_async(async move { ");
        
        // Check if callee is a MethodCall (e.g., HTTP.get())
        if let Expr::MethodCall(_) = &*call.callee {
            // MethodCall already generates the full call, just output it with .await
            self.generate_expr(&call.callee)?;
            self.output.push_str(".await");
        } else {
            // Regular function call
            self.generate_expr(&call.callee)?;
            self.output.push('(');
            for (i, arg) in call.args.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                // Convert string literals to String automatically
                if let Expr::Literal(Literal::String(_)) = arg {
                    self.generate_expr(arg)?;
                    self.output.push_str(".to_string()");
                } else {
                    self.generate_expr(arg)?;
                }
            }
            self.output.push(')');
        }
        
        self.output.push_str(" })");
        // Note: NO .await.unwrap() here anymore!
        Ok(())
    }

    fn generate_parallel_call(&mut self, call: &CallExpr) -> Result<()> {
        // Phase 2: NO await here - just create the Task
        // The await will be inserted at first use of the variable
        self.output.push_str("liva_rt::spawn_parallel(move || ");
        self.generate_expr(&call.callee)?;
        self.output.push('(');
        for (i, arg) in call.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            // Convert string literals to String automatically
            if let Expr::Literal(Literal::String(_)) = arg {
                self.generate_expr(arg)?;
                self.output.push_str(".to_string()");
            } else {
                self.generate_expr(arg)?;
            }
        }
        self.output.push(')');
        self.output.push(')');
        // Note: NO .await.unwrap() here anymore!
        Ok(())
    }

    fn generate_task_call(&mut self, call: &CallExpr, mode: ConcurrencyMode) -> Result<()> {
        let callee_name = match call.callee.as_ref() {
            Expr::Identifier(name) => name.clone(),
            _ => {
                return Err(CompilerError::CodegenError(
                    "Task calls currently only support simple function names".into(),
                ));
            }
        };

        let rust_name = self.sanitize_name(&callee_name);

        match mode {
            ConcurrencyMode::Async => {
                self.output.push_str("liva_rt::spawn_async(async move { ");
                write!(self.output, "{}(", rust_name).unwrap();
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push_str(") })");
            }
            ConcurrencyMode::Parallel => {
                self.output.push_str("liva_rt::spawn_parallel(move || { ");
                write!(self.output, "{}(", rust_name).unwrap();
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push_str(") })");
            }
        }
        Ok(())
    }

    fn generate_fire_call(&mut self, call: &CallExpr, mode: ConcurrencyMode) -> Result<()> {
        let callee_name = match call.callee.as_ref() {
            Expr::Identifier(name) => name.clone(),
            _ => {
                return Err(CompilerError::CodegenError(
                    "Fire calls currently only support simple function names".into(),
                ));
            }
        };

        let rust_name = self.sanitize_name(&callee_name);

        match mode {
            ConcurrencyMode::Async => {
                self.output.push_str("liva_rt::fire_async(async move {");
                self.output.push(' ');
                write!(self.output, "{}(", rust_name).unwrap();
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push_str("); })");
            }
            ConcurrencyMode::Parallel => {
                self.output.push_str("liva_rt::fire_parallel(move || {");
                self.output.push(' ');
                write!(self.output, "{}(", rust_name).unwrap();
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push_str("); })");
            }
        }
        Ok(())
    }

    fn generate_condition_expr(&mut self, expr: &Expr) -> Result<()> {
        // Special handling for error variables in conditions
        // if error_var -> error_var.is_some()
        // if !error_var -> error_var.is_none() (handled in Unary)
        if let Expr::Identifier(name) = expr {
            let sanitized = self.sanitize_name(name);
            if self.error_binding_vars.contains(&sanitized) {
                write!(self.output, "{}.is_some()", sanitized).unwrap();
                return Ok(());
            }
            // String error vars (from HTTP/File): if err -> !err.is_empty()
            if self.string_error_vars.contains(&sanitized) {
                write!(self.output, "!{}.is_empty()", sanitized).unwrap();
                return Ok(());
            }
        }
        
        // Otherwise, generate normally
        self.generate_expr(expr)
    }

    fn generate_binary_operation(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Result<()> {
        // Phase 3: Special handling for error binding variable comparisons with ""
        // Transform: err != "" to err.is_some()
        // Transform: err == "" to err.is_none()
        if matches!(op, BinOp::Ne | BinOp::Eq) {
            let is_error_var_comparison = match (left, right) {
                (Expr::Identifier(name), Expr::Literal(Literal::String(s))) if s.is_empty() => {
                    let sanitized = self.sanitize_name(name);
                    self.error_binding_vars.contains(&sanitized)
                }
                (Expr::Literal(Literal::String(s)), Expr::Identifier(name)) if s.is_empty() => {
                    let sanitized = self.sanitize_name(name);
                    self.error_binding_vars.contains(&sanitized)
                }
                _ => false,
            };

            if is_error_var_comparison {
                // Generate err.is_some() or err.is_none()
                if let Expr::Identifier(name) = left {
                    write!(self.output, "{}", self.sanitize_name(name)).unwrap();
                } else if let Expr::Identifier(name) = right {
                    write!(self.output, "{}", self.sanitize_name(name)).unwrap();
                }

                if matches!(op, BinOp::Ne) {
                    self.output.push_str(".is_some()");
                } else {
                    self.output.push_str(".is_none()");
                }
                return Ok(());
            }
            
            // Special handling for Option variable comparison with null
            // Transform: x != null -> x.is_some()
            // Transform: x == null -> x.is_none()
            let is_option_null_comparison = match (left, right) {
                (Expr::Identifier(name), Expr::Literal(Literal::Null)) => {
                    let sanitized = self.sanitize_name(name);
                    self.option_value_vars.contains(&sanitized)
                }
                (Expr::Literal(Literal::Null), Expr::Identifier(name)) => {
                    let sanitized = self.sanitize_name(name);
                    self.option_value_vars.contains(&sanitized)
                }
                _ => false,
            };
            
            if is_option_null_comparison {
                // Generate x.is_some() or x.is_none()
                let var_name = match (left, right) {
                    (Expr::Identifier(name), _) => name,
                    (_, Expr::Identifier(name)) => name,
                    _ => unreachable!(),
                };
                write!(self.output, "{}", self.sanitize_name(var_name)).unwrap();
                
                if matches!(op, BinOp::Ne) {
                    self.output.push_str(".is_some()");
                } else {
                    self.output.push_str(".is_none()");
                }
                return Ok(());
            }
            
            // Special handling for JsonValue comparison with null
            // Transform: jsonVar != null -> !jsonVar.is_null()
            // Transform: jsonVar == null -> jsonVar.is_null()
            let is_json_null_comparison = match (left, right) {
                (Expr::Identifier(name), Expr::Literal(Literal::Null)) => {
                    let sanitized = self.sanitize_name(name);
                    self.json_value_vars.contains(&sanitized)
                }
                (Expr::Literal(Literal::Null), Expr::Identifier(name)) => {
                    let sanitized = self.sanitize_name(name);
                    self.json_value_vars.contains(&sanitized)
                }
                _ => false,
            };
            
            if is_json_null_comparison {
                // Generate !x.is_null() or x.is_null()
                let var_name = match (left, right) {
                    (Expr::Identifier(name), _) => name,
                    (_, Expr::Identifier(name)) => name,
                    _ => unreachable!(),
                };
                
                if matches!(op, BinOp::Ne) {
                    self.output.push_str("!");
                }
                write!(self.output, "{}.is_null()", self.sanitize_name(var_name)).unwrap();
                return Ok(());
            }
        }

        // Special handling for string multiplication (String * int or int * String)
        if matches!(op, BinOp::Mul) {
            let has_string_literal = matches!(left, Expr::Literal(Literal::String(_)))
                || matches!(right, Expr::Literal(Literal::String(_)));
            
            if has_string_literal {
                // Use string_mul helper only when we have a string literal
                self.output.push_str("liva_rt::string_mul(");
                self.generate_expr(left)?;
                self.output.push_str(", ");
                self.generate_expr(right)?;
                self.output.push(')');
                return Ok(());
            }
        }

        // Original logic for other binary operations
        // Only add parentheses when necessary for precedence
        let left_needs_parens = self.expr_needs_parens_for_binop(left, op);
        let right_needs_parens = self.expr_needs_parens_for_binop(right, op);

        if left_needs_parens {
            self.output.push('(');
        }
        self.generate_expr(left)?;
        if left_needs_parens {
            self.output.push(')');
        }

        write!(self.output, " {} ", op).unwrap();

        if right_needs_parens {
            self.output.push('(');
        }
        self.generate_expr(right)?;
        if right_needs_parens {
            self.output.push(')');
        }

        Ok(())
    }

    fn expr_needs_parens_for_binop(&self, expr: &Expr, parent_op: &BinOp) -> bool {
        match expr {
            Expr::Literal(_) | Expr::Identifier(_) => false,
            Expr::Binary { op, .. } => {
                // Parentheses needed if this expression has lower precedence than parent
                self.binop_precedence(op) < self.binop_precedence(parent_op)
            }
            _ => true, // Default to needing parentheses for complex expressions
        }
    }

    fn binop_precedence(&self, op: &BinOp) -> i32 {
        match op {
            BinOp::Mul | BinOp::Div | BinOp::Mod => 100,
            BinOp::Add | BinOp::Sub => 90,
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 80,
            BinOp::Eq | BinOp::Ne => 70,
            BinOp::And => 60,
            BinOp::Or => 50,
            BinOp::Range => 40,
        }
    }

    fn block_has_return(&self, block: &BlockStmt) -> bool {
        block
            .stmts
            .iter()
            .any(|stmt| matches!(stmt, Stmt::Return(_)))
    }

    /// Infer return type from the first return statement in a block
    fn infer_return_type_from_block(&self, block: &BlockStmt) -> Option<String> {
        for stmt in &block.stmts {
            if let Stmt::Return(return_stmt) = stmt {
                if let Some(expr) = &return_stmt.expr {
                    return self.infer_expr_type(expr, None);
                }
            }
        }
        None
    }

    fn block_ends_with_return(&self, block: &BlockStmt) -> bool {
        block
            .stmts
            .last()
            .map(|stmt| matches!(stmt, Stmt::Return(_) | Stmt::Fail(_)))
            .unwrap_or(false)
    }

    fn is_fallible_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                // Check if calling a fallible function
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    self.fallible_functions.contains(name)
                } else {
                    false
                }
            }
            Expr::Ternary {
                condition: _,
                then_expr,
                else_expr,
            } => {
                // A ternary is fallible if either branch contains a fail or calls a fallible function
                self.expr_contains_fail(then_expr) || self.expr_contains_fail(else_expr)
            }
            _ => false,
        }
    }

    /// Check if expression is a built-in conversion function call that returns (value, Option<Error>)
    fn is_builtin_conversion_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                // Check if callee is an identifier (parseInt, parseFloat)
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    return name == "parseInt" || name == "parseFloat";
                }
                // Check if callee is a MethodCall (async HTTP.get, etc.)
                // This handles the async/par wrapper around method calls
                if let Expr::MethodCall(_) = call.callee.as_ref() {
                    return self.is_builtin_conversion_call(call.callee.as_ref());
                }
                false
            }
            Expr::MethodCall(method_call) => {
                // Check for .json() method on HTTP responses
                if method_call.method == "json" {
                    // response.json() returns (Option<JsonValue>, String)
                    return true;
                }
                
                if let Expr::Identifier(object_name) = method_call.object.as_ref() {
                    // Check for JSON methods
                    if object_name == "JSON" && (method_call.method == "parse" || method_call.method == "stringify") {
                        return true;
                    }
                    // Check for File methods (all except exists return tuples)
                    if object_name == "File" && (
                        method_call.method == "read" ||
                        method_call.method == "write" ||
                        method_call.method == "append" ||
                        method_call.method == "delete"
                    ) {
                        return true;
                    }
                    // Check for HTTP methods (all return tuples)
                    if (object_name == "HTTP" || object_name == "Http") && (
                        method_call.method == "get" ||
                        method_call.method == "post" ||
                        method_call.method == "put" ||
                        method_call.method == "delete"
                    ) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn expr_contains_fail(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Fail(_) => true,
            Expr::Call(call) => self.is_fallible_expr(&Expr::Call(call.clone())),
            Expr::Ternary {
                then_expr,
                else_expr,
                ..
            } => self.expr_contains_fail(then_expr) || self.expr_contains_fail(else_expr),
            _ => false,
        }
    }

    fn expr_is_stringy(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Literal(Literal::String(_)) => true,
            Expr::StringTemplate { .. } => true,
            Expr::Binary {
                op: BinOp::Add,
                left,
                right,
            } => self.expr_is_stringy(left) || self.expr_is_stringy(right),
            // Bug #18 fix: Variables known to be strings should trigger format! usage
            Expr::Identifier(name) => {
                let sanitized = self.sanitize_name(name);
                self.string_vars.contains(&sanitized)
            }
            _ => false,
        }
    }

    /// Generate an expression with special handling for error binding variables in string context
    fn generate_expr_for_string_concat(&mut self, expr: &Expr) -> Result<()> {
        // Check if this is an error binding variable
        if let Expr::Identifier(name) = expr {
            let sanitized = self.sanitize_name(name);
            if self.error_binding_vars.contains(&sanitized) {
                // For error variables, extract the message: err.as_ref().map(|e| e.message.as_str()).unwrap_or("")
                write!(
                    self.output,
                    "{}.as_ref().map(|e| e.message.as_str()).unwrap_or(\"\")",
                    sanitized
                ).unwrap();
                return Ok(());
            }
            if self.option_value_vars.contains(&sanitized) {
                // For option value variables, unwrap with default: value.as_ref().map(|v| v.to_string()).unwrap_or_default()
                write!(
                    self.output,
                    "{}.as_ref().map(|v| v.to_string()).unwrap_or_default()",
                    sanitized
                ).unwrap();
                return Ok(());
            }
            if self.struct_destructured_vars.contains(&sanitized) {
                // For struct destructured variables (may be Option<T>), use as_ref().map().unwrap_or_default()
                write!(
                    self.output,
                    "{}.as_ref().map(|v| format!(\"{{}}\", v)).unwrap_or_default()",
                    sanitized
                ).unwrap();
                return Ok(());
            }
        }
        // Otherwise, generate normally
        self.generate_expr(expr)
    }

    fn generate_literal(&mut self, lit: &Literal) -> Result<()> {
        match lit {
            Literal::Int(n) => write!(self.output, "{}", n).unwrap(),
            Literal::Float(f) => {
                // Always add _f64 suffix to avoid ambiguous numeric type errors
                write!(self.output, "{}_f64", f).unwrap();
            }
            Literal::String(s) => {
                // Write string with proper escape sequences interpreted
                // Don't use escape_default() as it would escape the escapes (\\n instead of \n)
                self.output.push('"');
                self.output.push_str(s);
                self.output.push('"');
            }
            Literal::Char(c) => write!(self.output, "'{}'", c.escape_default()).unwrap(),
            Literal::Bool(b) => write!(self.output, "{}", b).unwrap(),
            Literal::Null => self.output.push_str("None"),
        }
        Ok(())
    }

    fn infer_const_type(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(Literal::Int(_)) => "i32".to_string(),
            Expr::Literal(Literal::Float(_)) => "f64".to_string(),
            Expr::Literal(Literal::String(_)) => "&str".to_string(),
            Expr::Literal(Literal::Bool(_)) => "bool".to_string(),
            Expr::Literal(Literal::Char(_)) => "char".to_string(),
            Expr::Literal(Literal::Null) => "Option<()>".to_string(),
            _ => "i32".to_string(),
        }
    }

    /// Generate destructuring code for a binding pattern
    fn generate_destructuring_pattern(
        &mut self,
        pattern: &BindingPattern,
        init_expr: &Expr,
    ) -> Result<()> {
        match pattern {
            BindingPattern::Identifier(name) => {
                // Simple binding - should not reach here since we check is_simple() before calling
                // But handle it anyway for completeness
                let var_name = self.sanitize_name(name);
                
                // Track variable types based on init expression
                if let Expr::ObjectLiteral(_) = init_expr {
                    self.bracket_notation_vars.insert(name.clone());
                }
                if let Expr::ArrayLiteral(_) = init_expr {
                    self.array_vars.insert(name.clone());
                }
                if let Expr::Call(call) = init_expr {
                    if let Expr::Identifier(class_name) = &*call.callee {
                        self.class_instance_vars.insert(name.clone());
                        self.var_types.insert(name.clone(), class_name.clone());
                    }
                }
                
                write!(self.output, "let mut {}", var_name).unwrap();
                self.output.push_str(" = ");
                self.generate_expr(init_expr)?;
                self.output.push_str(";\n");
            }
            BindingPattern::Object(obj_pattern) => {
                // Object destructuring: let {x, y} = point
                // First, generate temporary variable for the object
                let temp_var = format!("_temp_{}", self.gen_unique_id());
                write!(self.output, "let {} = ", temp_var).unwrap();
                self.generate_expr(init_expr)?;
                self.output.push_str(";\n");

                // Then extract each field
                for field in &obj_pattern.fields {
                    self.write_indent();
                    let binding_name = self.sanitize_name(&field.binding);
                    
                    // Check if temp var is a JsonValue or needs bracket notation
                    // We need to check the init_expr to determine the type
                    let needs_bracket_notation = matches!(init_expr, Expr::ObjectLiteral(_)) ||
                        (matches!(init_expr, Expr::Identifier(id) if 
                            self.bracket_notation_vars.contains(id) || 
                            self.json_value_vars.contains(id)));
                    
                    if needs_bracket_notation {
                        // JSON object access using bracket notation
                        write!(
                            self.output,
                            "let mut {} = {}[\"{}\"].clone();\n",
                            binding_name, temp_var, field.key
                        ).unwrap();
                    } else {
                        // Struct field access - clone to handle non-Copy types (String, Vec, etc.)
                        write!(
                            self.output,
                            "let mut {} = {}.{}.clone();\n",
                            binding_name, temp_var, field.key
                        ).unwrap();
                    }
                }
            }
            BindingPattern::Array(arr_pattern) => {
                // Array destructuring: let [first, second] = array
                // First, generate temporary variable for the array (clone to avoid move)
                let temp_var = format!("_temp_{}", self.gen_unique_id());
                write!(self.output, "let {} = ", temp_var).unwrap();
                self.generate_expr(init_expr)?;
                self.output.push_str(".clone();\n");

                // Extract individual elements
                for (i, element) in arr_pattern.elements.iter().enumerate() {
                    if let Some(name) = element {
                        self.write_indent();
                        let binding_name = self.sanitize_name(name);
                        write!(
                            self.output,
                            "let mut {} = {}[{}].clone();\n",
                            binding_name, temp_var, i
                        ).unwrap();
                    }
                }

                // Handle rest pattern: [...rest]
                if let Some(rest_name) = &arr_pattern.rest {
                    self.write_indent();
                    let binding_name = self.sanitize_name(rest_name);
                    let start_index = arr_pattern.elements.len();
                    write!(
                        self.output,
                        "let mut {}: Vec<_> = {}[{}..].to_vec();\n",
                        binding_name, temp_var, start_index
                    ).unwrap();
                    
                    // Track the rest variable as an array
                    self.array_vars.insert(rest_name.clone());
                }
            }
            BindingPattern::Tuple(tuple_pattern) => {
                // Tuple destructuring: let (x, y, z) = tuple
                // Generate tuple pattern on left side
                self.output.push_str("let (");
                for (i, name) in tuple_pattern.elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let var_name = self.sanitize_name(name);
                    write!(self.output, "mut {}", var_name).unwrap();
                }
                self.output.push_str(") = ");
                self.generate_expr(init_expr)?;
                self.output.push_str(";\n");
            }
        }

        Ok(())
    }

    /// Generate destructuring code for function parameters at the start of function body
    /// For each destructured parameter, generate let statements to extract values
    fn generate_param_destructuring(&mut self, params: &[Param]) -> Result<()> {
        for (param_idx, param) in params.iter().enumerate() {
            if param.is_destructuring() {
                // Parameter is destructured, need to generate let statements
                // The parameter in the function signature uses a temp name: _param_0, _param_1, etc.
                let temp_name = format!("_param_{}", param_idx);
                
                match &param.pattern {
                    BindingPattern::Object(obj_pattern) => {
                        // Object destructuring: {x, y} => extract each field
                        for field in &obj_pattern.fields {
                            self.write_indent();
                            let binding_name = self.sanitize_name(&field.binding);
                            
                            // For parameters, we always use struct field access (not JSON)
                            // Clone to handle non-Copy types
                            write!(
                                self.output,
                                "let mut {} = {}.{}.clone();\n",
                                binding_name, temp_name, field.key
                            ).unwrap();
                        }
                    }
                    BindingPattern::Array(arr_pattern) => {
                        // Array destructuring: [first, second] => extract by index
                        for (i, element) in arr_pattern.elements.iter().enumerate() {
                            if let Some(name) = element {
                                self.write_indent();
                                let binding_name = self.sanitize_name(name);
                                write!(
                                    self.output,
                                    "let mut {} = {}[{}].clone();\n",
                                    binding_name, temp_name, i
                                ).unwrap();
                            }
                        }
                        
                        // Handle rest pattern: [...rest]
                        if let Some(rest_name) = &arr_pattern.rest {
                            self.write_indent();
                            let binding_name = self.sanitize_name(rest_name);
                            let start_index = arr_pattern.elements.len();
                            write!(
                                self.output,
                                "let mut {}: Vec<_> = {}[{}..].to_vec();\n",
                                binding_name, temp_name, start_index
                            ).unwrap();
                            
                            // Track as array variable
                            self.array_vars.insert(rest_name.clone());
                        }
                    }
                    BindingPattern::Tuple(tuple_pattern) => {
                        // Tuple destructuring: (x, y, z) => extract by position
                        self.write_indent();
                        self.output.push_str("let (");
                        for (i, name) in tuple_pattern.elements.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            let var_name = self.sanitize_name(name);
                            write!(self.output, "mut {}", var_name).unwrap();
                        }
                        write!(self.output, ") = {};\n", temp_name).unwrap();
                    }
                    BindingPattern::Identifier(_) => {
                        // Not destructured, nothing to do
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Generate destructuring code for lambda parameters
    /// Similar to generate_param_destructuring but for lambdas (no indent on first line)
    fn generate_lambda_param_destructuring(&mut self, pattern: &BindingPattern, temp_name: &str, is_json_value: bool, class_name: Option<&str>) -> Result<()> {
        match pattern {
            BindingPattern::Object(obj_pattern) => {
                // Object destructuring: extract each field
                for field in &obj_pattern.fields {
                    let binding_name = self.sanitize_name(&field.binding);
                    
                    if is_json_value {
                        // For JsonValue, use .get("field") access
                        write!(
                            self.output,
                            "let {} = {}[\"{}\"].clone();\n",
                            binding_name, temp_name, field.key
                        ).unwrap();
                    } else {
                        // For structs, check if field is optional in the class definition
                        let is_field_optional = if let Some(cls_name) = class_name {
                            if let Some(optional_fields) = self.class_optional_fields.get(cls_name) {
                                optional_fields.contains(&field.key)
                            } else {
                                false
                            }
                        } else {
                            false
                        };
                        
                        if is_field_optional {
                            // For optional fields, unwrap or use default
                            write!(
                                self.output,
                                "let {} = {}.{}.as_ref().map(|v| v.clone()).unwrap_or_default();\n",
                                binding_name, temp_name, field.key
                            ).unwrap();
                            // Only register optional fields for special string template handling
                            self.struct_destructured_vars.insert(binding_name.clone());
                        } else {
                            // For required fields, just clone
                            write!(
                                self.output,
                                "let {} = {}.{}.clone();\n",
                                binding_name, temp_name, field.key
                            ).unwrap();
                        }
                        
                        // Check if this field is itself a class type, and register it as a class instance
                        // This is important for nested struct access (e.g., address.zipcode)
                        if let Some(cls_name) = class_name {
                            // Try to get the field type from class_fields metadata
                            // For now, we'll use a heuristic: if the field name starts with lowercase
                            // and there's a corresponding capitalized class, mark it as class instance
                            let potential_class = capitalize_first_letter(&field.key);
                            if self.class_fields.contains_key(&potential_class) {
                                self.class_instance_vars.insert(binding_name.clone());
                                self.var_types.insert(binding_name.clone(), potential_class);
                            }
                        }
                    }
                    
                    if field != obj_pattern.fields.last().unwrap() {
                        self.write_indent();
                    }
                }
            }
            BindingPattern::Array(arr_pattern) => {
                // Array destructuring: extract by index
                for (i, element) in arr_pattern.elements.iter().enumerate() {
                    if let Some(name) = element {
                        let binding_name = self.sanitize_name(name);
                        write!(
                            self.output,
                            "let {} = {}[{}].clone();\n",
                            binding_name, temp_name, i
                        ).unwrap();
                        if i < arr_pattern.elements.len() - 1 || arr_pattern.rest.is_some() {
                            self.write_indent();
                        }
                    }
                }
                
                // Handle rest pattern
                if let Some(rest_name) = &arr_pattern.rest {
                    let binding_name = self.sanitize_name(rest_name);
                    let start_index = arr_pattern.elements.len();
                    write!(
                        self.output,
                        "let {}: Vec<_> = {}[{}..].to_vec();\n",
                        binding_name, temp_name, start_index
                    ).unwrap();
                }
            }
            BindingPattern::Tuple(tuple_pattern) => {
                // Tuple destructuring: extract by position
                self.output.push_str("let (");
                for (i, name) in tuple_pattern.elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let var_name = self.sanitize_name(name);
                    write!(self.output, "{}", var_name).unwrap();
                }
                write!(self.output, ") = {};\n", temp_name).unwrap();
            }
            BindingPattern::Identifier(_) => {
                // Not destructured, nothing to do
            }
        }
        
        Ok(())
    }

    /// Generate a unique ID for temporary variables
    fn gen_unique_id(&self) -> usize {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    fn sanitize_name(&self, name: &str) -> String {
        // Convert to snake_case, preserving leading underscore for private fields
        let has_leading_underscore = name.starts_with('_');
        let name_without_prefix = name.trim_start_matches('_');
        let snake = self.to_snake_case(name_without_prefix);
        
        if has_leading_underscore {
            format!("_{}", snake)
        } else {
            snake
        }
    }

    fn sanitize_test_name(&self, name: &str) -> String {
        name.replace(' ', "_").replace('-', "_").to_lowercase()
    }

    fn to_snake_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut prev_lowercase = false;

        for (i, ch) in s.chars().enumerate() {
            if ch.is_uppercase() {
                if i > 0 && prev_lowercase {
                    result.push('_');
                }
                result.push(ch.to_lowercase().next().unwrap());
                prev_lowercase = false;
            } else {
                result.push(ch);
                prev_lowercase = ch.is_lowercase();
            }
        }

        result
    }

    /// Check if a string is in camelCase (has uppercase letters that aren't at the start)
    fn is_camel_case(&self, s: &str) -> bool {
        s.chars().enumerate().any(|(i, ch)| i > 0 && ch.is_uppercase())
    }
}

struct IrCodeGenerator<'a> {
    output: String,
    indent_level: usize,
    #[allow(dead_code)]
    ctx: &'a DesugarContext,
    #[allow(dead_code)]
    scope_formats: Vec<HashMap<String, FormatKind>>,
    #[allow(dead_code)]
    in_method: bool,
    #[allow(dead_code)]
    error_binding_vars: HashSet<String>,
}

#[derive(Copy, Clone, PartialEq)]
#[allow(dead_code)]
enum FormatKind {
    Display,
    Debug,
}

impl FormatKind {
    #[allow(dead_code)]
    fn placeholder(self) -> &'static str {
        match self {
            FormatKind::Display => "{}",
            FormatKind::Debug => "{:?}",
        }
    }
}

#[allow(dead_code)]
impl<'a> IrCodeGenerator<'a> {
    fn new(ctx: &'a DesugarContext) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            ctx,
            scope_formats: vec![HashMap::new()],
            in_method: false,
            error_binding_vars: HashSet::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scope_formats.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scope_formats.pop();
    }

    fn set_var_format(&mut self, name: &str, format: FormatKind) {
        let sanitized = self.sanitize_name(name);
        if let Some(scope) = self.scope_formats.last_mut() {
            scope.insert(sanitized, format);
        }
    }

    fn record_from_type(&mut self, name: &str, ty: &ir::Type) {
        let format = self.format_from_type(ty);
        self.set_var_format(name, format);
    }

    fn record_from_expr(&mut self, name: &str, expr: &ir::Expr) {
        let format = self.format_from_expr(expr);
        self.set_var_format(name, format);
    }

    fn lookup_var_format(&self, name: &str) -> Option<FormatKind> {
        let sanitized = self.sanitize_name(name);
        for scope in self.scope_formats.iter().rev() {
            if let Some(format) = scope.get(&sanitized) {
                return Some(*format);
            }
        }
        None
    }

    fn format_from_type(&self, ty: &ir::Type) -> FormatKind {
        match ty {
            ir::Type::Array(_) => FormatKind::Debug,
            ir::Type::Custom(name) if name == "serde_json::Value" => FormatKind::Display,
            _ => FormatKind::Display,
        }
    }

    fn format_from_expr(&self, expr: &ir::Expr) -> FormatKind {
        match expr {
            ir::Expr::ArrayLiteral(_) => FormatKind::Debug,
            ir::Expr::TupleLiteral(_) => FormatKind::Debug,
            ir::Expr::ObjectLiteral(_) => FormatKind::Debug,
            ir::Expr::StructLiteral { .. } => FormatKind::Display,
            _ => FormatKind::Display,
        }
    }

    fn template_placeholder_for_expr(&self, expr: &ir::Expr) -> &'static str {
        match expr {
            ir::Expr::Member { object, .. } => {
                if matches!(object.as_ref(), ir::Expr::Identifier(name) if name == "self")
                    && self.expr_needs_debug(object)
                {
                    FormatKind::Debug.placeholder()
                } else {
                    FormatKind::Display.placeholder()
                }
            }
            ir::Expr::Index { object, .. } => {
                if self.expr_needs_debug(object) {
                    FormatKind::Debug.placeholder()
                } else {
                    FormatKind::Display.placeholder()
                }
            }
            _ => {
                if self.expr_needs_debug(expr) {
                    FormatKind::Debug.placeholder()
                } else {
                    FormatKind::Display.placeholder()
                }
            }
        }
    }

    fn expr_needs_debug(&self, expr: &ir::Expr) -> bool {
        match expr {
            ir::Expr::ArrayLiteral(_) => true,
            ir::Expr::TupleLiteral(_) => true,
            ir::Expr::ObjectLiteral(_) => true,
            ir::Expr::StructLiteral { .. } => false,
            ir::Expr::Identifier(name) => {
                matches!(self.lookup_var_format(name), Some(FormatKind::Debug))
            }
            ir::Expr::Binary { left, right, .. } => {
                self.expr_needs_debug(left) || self.expr_needs_debug(right)
            }
            ir::Expr::Member { object, .. } => self.expr_needs_debug(object),
            ir::Expr::Index { object, .. } => self.expr_needs_debug(object),
            _ => false,
        }
    }

    fn is_self_reference(&self, expr: &ir::Expr) -> bool {
        match expr {
            ir::Expr::Identifier(name) => name == "self",
            ir::Expr::Member { object, .. } | ir::Expr::Index { object, .. } => {
                self.is_self_reference(object)
            }
            _ => false,
        }
    }

    /// Checks if an expression is a member access on 'self' (self.field)
    /// Returns true if the expression needs .clone() when used in assignment or return
    fn expr_is_self_field(&self, expr: &ir::Expr) -> bool {
        if let ir::Expr::Member { object, property: _ } = expr {
            if let ir::Expr::Identifier(obj) = object.as_ref() {
                return obj == "self" && self.in_method;
            }
        }
        false
    }

    fn is_liva_rt_member(&self, expr: &ir::Expr) -> bool {
        match expr {
            ir::Expr::Identifier(name) => name == "liva_rt",
            ir::Expr::Member { object, .. } => self.is_liva_rt_member(object),
            _ => false,
        }
    }

    fn expr_is_json_access(&self, expr: &ir::Expr) -> bool {
        match expr {
            ir::Expr::Member { object, .. } | ir::Expr::Index { object, .. } => {
                !self.is_self_reference(object)
            }
            _ => false,
        }
    }

    fn generate_numeric_operand(&mut self, expr: &ir::Expr) -> Result<()> {
        if self.expr_is_json_access(expr) {
            self.output.push('(');
            self.generate_expr(expr)?;
            self.output.push_str(").as_f64().unwrap_or(0.0)");
            Ok(())
        } else {
            self.generate_expr(expr)
        }
    }

    fn infer_const_type_name(&self, expr: &ir::Expr) -> String {
        match expr {
            ir::Expr::Literal(ir::Literal::String(_)) => "&str".into(),
            ir::Expr::Literal(ir::Literal::Float(_)) => "f64".into(),
            ir::Expr::Literal(ir::Literal::Bool(_)) => "bool".into(),
            ir::Expr::Literal(ir::Literal::Char(_)) => "char".into(),
            _ => "i32".into(),
        }
    }

    fn generate(mut self, module: &ir::Module) -> Result<String> {
        self.emit_use_statements(module);

    // Always include runtime for now
    if std::env::var("LIVA_DEBUG").is_ok() { println!("DEBUG: IR generator including liva_rt module"); }
        self.writeln("mod liva_rt {");
        self.indent();
        self.writeln("use std::future::Future;");
        self.writeln("use tokio::task::JoinHandle;");

        // Always include both async and parallel functions
        self.writeln("/// Spawn an async task");
        self.writeln("pub fn spawn_async<F, T>(future: F) -> JoinHandle<T>");
        self.writeln("where");
        self.indent();
        self.writeln("F: Future<Output = T> + Send + 'static,");
        self.writeln("T: Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("tokio::spawn(future)");
        self.dedent();
        self.writeln("}");

        self.writeln("/// Fire and forget async task");
        self.writeln("pub fn fire_async<F>(future: F)");
        self.writeln("where");
        self.indent();
        self.writeln("F: Future<Output = ()> + Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("tokio::spawn(future);");
        self.dedent();
        self.writeln("}");

        self.writeln("/// Spawn a parallel task");
        self.writeln("pub fn spawn_parallel<F, T>(f: F) -> JoinHandle<T>");
        self.writeln("where");
        self.indent();
        self.writeln("F: FnOnce() -> T + Send + 'static,");
        self.writeln("T: Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("// For simplicity, just execute synchronously and wrap in JoinHandle");
        self.writeln("// In a real implementation, this would use rayon or std::thread");
        self.writeln("let result = f();");
        self.writeln("tokio::spawn(async move { result })");
        self.dedent();
        self.writeln("}");

        self.writeln("/// Fire and forget parallel task");
        self.writeln("pub fn fire_parallel<F>(f: F)");
        self.writeln("where");
        self.indent();
        self.writeln("F: FnOnce() + Send + 'static,");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("// For simplicity, just spawn a thread");
        self.writeln("std::thread::spawn(f);");
        self.dedent();
        self.writeln("}");

        // Add type definitions needed for parallel operations
        self.writeln("#[derive(Clone, Copy, Debug)]");
        self.writeln("pub enum ThreadOption {");
        self.indent();
        self.writeln("Auto,");
        self.writeln("Count(usize),");
        self.dedent();
        self.writeln("}");

        self.writeln("#[derive(Clone, Copy, Debug)]");
        self.writeln("pub enum SimdWidthOption {");
        self.indent();
        self.writeln("Auto,");
        self.writeln("Width(usize),");
        self.dedent();
        self.writeln("}");

        self.writeln("#[derive(Clone, Copy, Debug)]");
        self.writeln("pub enum ReductionOption {");
        self.indent();
        self.writeln("Safe,");
        self.writeln("Fast,");
        self.dedent();
        self.writeln("}");

        self.writeln("#[derive(Clone, Copy, Debug)]");
        self.writeln("pub enum ScheduleOption {");
        self.indent();
        self.writeln("Static,");
        self.writeln("Dynamic,");
        self.dedent();
        self.writeln("}");

        self.writeln("#[derive(Clone, Copy, Debug)]");
        self.writeln("pub enum DetectOption {");
        self.indent();
        self.writeln("Auto,");
        self.dedent();
        self.writeln("}");

        self.writeln("#[derive(Clone, Copy, Debug)]");
        self.writeln("pub struct ParallelForOptions {");
        self.indent();
        self.writeln("pub ordered: bool,");
        self.writeln("pub chunk: Option<usize>,");
        self.writeln("pub threads: Option<ThreadOption>,");
        self.writeln("pub simd_width: Option<SimdWidthOption>,");
        self.writeln("pub reduction: Option<ReductionOption>,");
        self.writeln("pub schedule: Option<ScheduleOption>,");
        self.writeln("pub prefetch: Option<i64>,");
        self.writeln("pub detect: Option<DetectOption>,");
        self.dedent();
        self.writeln("}");

        self.writeln("pub fn normalize_size(value: i64, fallback: usize) -> usize {");
        self.indent();
        self.writeln("if value > 0 { value as usize } else { fallback }");
        self.dedent();
        self.writeln("}");

        self.writeln("pub fn for_par<T, F>(iter: Vec<T>, f: F, _options: ParallelForOptions)");
        self.indent();
        self.writeln("where");
        self.indent();
        self.writeln("F: Fn(T),");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("for item in iter {");
        self.indent();
        self.writeln("f(item);");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");

        self.writeln("pub fn for_vec<T, F>(iter: Vec<T>, f: F, _options: ParallelForOptions)");
        self.indent();
        self.writeln("where");
        self.indent();
        self.writeln("F: Fn(T),");
        self.dedent();
        self.writeln("{");
        self.indent();
        self.writeln("for item in iter {");
        self.indent();
        self.writeln("f(item);");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");

        self.dedent();
        self.writeln("}");
        self.writeln("");

        for item in &module.items {
            match item {
                ir::Item::Function(func) => {
                    self.generate_function(func)?;
                    self.output.push('\n');
                }
                ir::Item::Test(test) => {
                    self.generate_test(test)?;
                    self.output.push('\n');
                }
                ir::Item::Unsupported(_) => {
                    return Err(CompilerError::CodegenError(
                        "Unsupported item in IR module".into(),
                    ))
                }
            }
        }

        Ok(self.output)
    }

    fn emit_use_statements(&mut self, module: &ir::Module) {
        use std::collections::BTreeSet;

        let mut emitted = BTreeSet::new();
        for (crate_name, alias) in &self.ctx.rust_crates {
            emitted.insert((crate_name.clone(), alias.clone()));
        }
        for ext in &module.extern_crates {
            emitted.insert((ext.crate_name.clone(), ext.alias.clone()));
        }

        // Always include liva_rt for runtime support
        writeln!(self.output, "use liva_rt;").unwrap();

        for (crate_name, alias) in emitted {
            if let Some(alias_name) = alias {
                writeln!(self.output, "use {} as {};", crate_name, alias_name).unwrap();
            } else {
                writeln!(self.output, "use {};", crate_name).unwrap();
            }
        }

        // Add import for SequenceCount trait if count() is used and runtime module will be generated
        // Note: This is a placeholder - in future versions, count() will be available without runtime module

        if !self.output.trim().is_empty() {
            self.output.push('\n');
        }
    }

    #[allow(dead_code)]
    fn emit_runtime_module(&mut self, use_async: bool, use_parallel: bool) {
        self.writeln("mod liva_rt {");
        self.indent();

        if use_async {
            self.writeln("use std::future::Future;");
            self.writeln(
                "pub fn spawn_async<Fut>(future: Fut) -> tokio::task::JoinHandle<Fut::Output>",
            );
            self.writeln("where Fut: Future + Send + 'static, Fut::Output: Send + 'static,");
            self.writeln("{");
            self.indent();
            self.writeln("tokio::spawn(future)");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("pub fn fire_async<Fut>(future: Fut)");
            self.writeln("where Fut: Future<Output = ()> + Send + 'static,");
            self.writeln("{");
            self.indent();
            self.writeln("let _ = tokio::spawn(future);");
            self.dedent();
            self.writeln("}");

            if use_parallel {
                self.output.push('\n');
            }
        }

        if use_parallel {
            self.writeln("use rayon::prelude::*;");
            self.writeln("use rayon::ThreadPoolBuilder;");
            self.writeln("use std::sync::Arc;");
            self.output.push('\n');

            self.writeln("#[derive(Clone, Copy, Debug)]");
            self.writeln("pub enum ThreadOption {");
            self.indent();
            self.writeln("Auto,");
            self.writeln("Count(usize),");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("#[derive(Clone, Copy, Debug)]");
            self.writeln("pub enum SimdWidthOption {");
            self.indent();
            self.writeln("Auto,");
            self.writeln("Width(usize),");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("#[derive(Clone, Copy, Debug)]");
            self.writeln("pub enum ReductionOption {");
            self.indent();
            self.writeln("Safe,");
            self.writeln("Fast,");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("#[derive(Clone, Copy, Debug)]");
            self.writeln("pub enum ScheduleOption {");
            self.indent();
            self.writeln("Static,");
            self.writeln("Dynamic,");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("#[derive(Clone, Copy, Debug)]");
            self.writeln("pub enum DetectOption {");
            self.indent();
            self.writeln("Auto,");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("#[derive(Clone, Copy, Debug)]");
            self.writeln("pub struct ParallelForOptions {");
            self.indent();
            self.writeln("pub ordered: bool,");
            self.writeln("pub chunk: Option<usize>,");
            self.writeln("pub threads: Option<ThreadOption>,");
            self.writeln("pub simd_width: Option<SimdWidthOption>,");
            self.writeln("pub prefetch: Option<usize>,");
            self.writeln("pub reduction: Option<ReductionOption>,");
            self.writeln("pub schedule: Option<ScheduleOption>,");
            self.writeln("pub detect: Option<DetectOption>,");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("pub fn normalize_size(value: i64, fallback: usize) -> usize {");
            self.indent();
            self.writeln("if value > 0 { value as usize } else { fallback }");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("fn emit_option_warnings(label: &str, options: &ParallelForOptions) {");
            self.indent();
            self.writeln("if let Some(prefetch) = options.prefetch {");
            self.indent();
            self.writeln("eprintln!(\"[liva_rt] `prefetch = {prefetch}` is not supported yet in `{label}`; ignoring.\");");
            self.dedent();
            self.writeln("}");
            self.writeln("if matches!(options.reduction, Some(ReductionOption::Safe) | Some(ReductionOption::Fast)) {");
            self.indent();
            self.writeln("eprintln!(\"[liva_rt] `reduction` is not supported yet in `{label}`; executing without reduction semantics.\");");
            self.dedent();
            self.writeln("}");
            self.writeln("if matches!(options.schedule, Some(ScheduleOption::Static) | Some(ScheduleOption::Dynamic)) {");
            self.indent();
            self.writeln(
                "eprintln!(\"[liva_rt] `schedule` hints are not supported yet in `{label}`.\");",
            );
            self.dedent();
            self.writeln("}");
            self.writeln("if matches!(options.detect, Some(DetectOption::Auto)) {");
            self.indent();
            self.writeln(
                "eprintln!(\"[liva_rt] `detect` hints are not supported yet in `{label}`.\");",
            );
            self.dedent();
            self.writeln("}");
            self.writeln("if matches!(options.simd_width, Some(SimdWidthOption::Auto) | Some(SimdWidthOption::Width(_))) {");
            self.indent();
            self.writeln("eprintln!(\"[liva_rt] `simdWidth` is not supported yet in `{label}`; falling back to scalar execution.\");");
            self.dedent();
            self.writeln("}");
            self.writeln("if options.ordered {");
            self.indent();
            self.writeln(
                "eprintln!(\"[liva_rt] `ordered` execution for `{label}` runs sequentially.\");",
            );
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("fn run_parallel_items<T, F>(items: Vec<T>, func: Arc<F>, options: ParallelForOptions)");
            self.writeln("where T: Send + 'static, F: Fn(T) + Send + Sync + 'static,");
            self.writeln("{");
            self.indent();
            self.writeln("if options.ordered {");
            self.indent();
            self.writeln("for item in items { func(item); }");
            self.writeln("return;");
            self.dedent();
            self.writeln("}");
            self.writeln("if let Some(chunk) = options.chunk.map(|c| c.max(1)) {");
            self.indent();
            self.writeln("items");
            self.writeln("    .into_par_iter()");
            self.writeln("    .with_min_len(chunk)");
            self.writeln("    .with_max_len(chunk)");
            self.writeln("    .for_each(move |item| {");
            self.indent();
            self.writeln("let func = func.clone();");
            self.writeln("func(item);");
            self.dedent();
            self.writeln("});");
            self.dedent();
            self.writeln("} else {");
            self.indent();
            self.writeln("items.into_par_iter().for_each(move |item| {");
            self.indent();
            self.writeln("let func = func.clone();");
            self.writeln("func(item);");
            self.dedent();
            self.writeln("});");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("fn execute_parallel<T, F>(items: Vec<T>, func: F, options: ParallelForOptions, label: &str)");
            self.writeln("where T: Send + 'static, F: Fn(T) + Send + Sync + 'static,");
            self.writeln("{");
            self.indent();
            self.writeln("emit_option_warnings(label, &options);");
            self.writeln("let func = Arc::new(func);");
            self.writeln("if let Some(ThreadOption::Count(count)) = options.threads {");
            self.indent();
            self.writeln("let count = count.max(1);");
            self.writeln("if let Ok(pool) = ThreadPoolBuilder::new().num_threads(count).build() {");
            self.indent();
            self.writeln("let func_clone = func.clone();");
            self.writeln("let options_clone = options;");
            self.writeln(
                "pool.install(move || run_parallel_items(items, func_clone, options_clone));",
            );
            self.writeln("return;");
            self.dedent();
            self.writeln("}");
            self.dedent();
            self.writeln("}");
            self.writeln("run_parallel_items(items, func, options);");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("pub fn spawn_parallel<F, T>(job: F) -> std::thread::JoinHandle<T>");
            self.writeln("where F: FnOnce() -> T + Send + 'static, T: Send + 'static,");
            self.writeln("{");
            self.indent();
            self.writeln("std::thread::spawn(job)");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln("pub fn fire_parallel<F, T>(job: F)");
            self.writeln("where F: FnOnce() -> T + Send + 'static, T: Send + 'static,");
            self.writeln("{");
            self.indent();
            self.writeln("let _ = std::thread::spawn(job);");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln(
                "pub fn for_par<I, T, F>(iterable: I, func: F, options: ParallelForOptions)",
            );
            self.writeln("where I: IntoIterator<Item = T>, T: Send + 'static, F: Fn(T) + Send + Sync + 'static,");
            self.writeln("{");
            self.indent();
            self.writeln("let items: Vec<T> = iterable.into_iter().collect();");
            self.writeln("execute_parallel(items, func, options, \"for par\");");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln(
                "pub fn for_vec<I, T, F>(iterable: I, func: F, options: ParallelForOptions)",
            );
            self.writeln("where I: IntoIterator<Item = T>, T: Send + 'static, F: Fn(T) + Send + Sync + 'static,");
            self.writeln("{");
            self.indent();
            self.writeln("let items: Vec<T> = iterable.into_iter().collect();");
            self.writeln("execute_parallel(items, func, options, \"for vec\");");
            self.dedent();
            self.writeln("}");
            self.output.push('\n');

            self.writeln(
                "pub fn for_parvec<I, T, F>(iterable: I, func: F, options: ParallelForOptions)",
            );
            self.writeln("where I: IntoIterator<Item = T>, T: Send + 'static, F: Fn(T) + Send + Sync + 'static,");
            self.writeln("{");
            self.indent();
            self.writeln("let items: Vec<T> = iterable.into_iter().collect();");
            self.writeln("execute_parallel(items, func, options, \"for parvec\");");
            self.dedent();
            self.writeln("}");
        }

        // Add count operations for sequences
        self.output.push('\n');
        self.writeln("/// Count operations for sequences");
        self.writeln("pub trait SequenceCount {");
        self.indent();
        self.writeln("type Output;");
        self.writeln("fn count(&self) -> Self::Output;");
        self.dedent();
        self.writeln("}");
        self.output.push('\n');

        self.writeln("impl<T> SequenceCount for Vec<T> {");
        self.indent();
        self.writeln("type Output = usize;");
        self.writeln("fn count(&self) -> usize {");
        self.indent();
        self.writeln("self.len()");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.output.push('\n');

        self.writeln("impl<T> SequenceCount for &[T] {");
        self.indent();
        self.writeln("type Output = usize;");
        self.writeln("fn count(&self) -> usize {");
        self.indent();
        self.writeln("self.len()");
        self.dedent();
        self.writeln("}");
        self.dedent();
        self.writeln("}");
        self.output.push('\n');

        self.dedent();
        self.writeln("}");
        self.output.push('\n');
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn writeln(&mut self, line: &str) {
        self.write_indent();
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn generate_function(&mut self, func: &ir::Function) -> Result<()> {
        if matches!(func.async_kind, ir::AsyncKind::Async) && func.name == "main" {
            self.writeln("#[tokio::main]");
        }

        self.write_indent();

        if matches!(func.async_kind, ir::AsyncKind::Async) {
            self.output.push_str("async ");
        }

        write!(self.output, "fn {}(", self.sanitize_name(&func.name)).unwrap();

        for (idx, param) in func.params.iter().enumerate() {
            if idx > 0 {
                self.output.push_str(", ");
            }
            write!(
                self.output,
                "{}: {}",
                self.sanitize_name(&param.name),
                self.type_to_rust(&param.ty)
            )
            .unwrap();
        }

        self.output.push(')');

        if !matches!(func.ret_type, ir::Type::Inferred) {
            write!(self.output, " -> {}", self.type_to_rust(&func.ret_type)).unwrap();
        }

        if let Some(scope) = self.scope_formats.last_mut() {
            scope.clear();
        }
        for param in &func.params {
            self.record_from_type(&param.name, &param.ty);
        }

        self.output.push_str(" {\n");
        self.indent();
        self.push_scope();
        self.generate_block(&func.body)?;
        self.pop_scope();
        self.dedent();
        self.writeln("}");

        if let Some(scope) = self.scope_formats.last_mut() {
            scope.clear();
        }

        Ok(())
    }

    fn generate_test(&mut self, test: &ir::Test) -> Result<()> {
        self.writeln("#[test]");
        self.writeln(&format!(
            "fn test_{}() {{",
            self.sanitize_test_name(&test.name)
        ));
        self.indent();
        self.generate_block(&test.body)?;
        self.dedent();
        self.writeln("}");
        Ok(())
    }

    fn generate_block(&mut self, block: &ir::Block) -> Result<()> {
        for stmt in &block.statements {
            self.generate_stmt(stmt)?;
        }
        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &ir::Stmt) -> Result<()> {
        match stmt {
            ir::Stmt::Let { name, ty, value } => {
                self.write_indent();
                write!(self.output, "let mut {}", self.sanitize_name(name)).unwrap();
                if let Some(ty) = ty {
                    write!(self.output, ": {}", self.type_to_rust(ty)).unwrap();
                    self.record_from_type(name, ty);
                } else {
                    self.record_from_expr(name, value);
                }
                self.output.push_str(" = ");
                // Auto-clone when assigning self.field to a local variable
                let needs_clone = self.expr_is_self_field(value);
                self.generate_expr(value)?;
                if needs_clone {
                    self.output.push_str(".clone()");
                }
                self.output.push_str(";\n");
            }
            ir::Stmt::Const { name, ty, value } => {
                self.write_indent();
                let const_name = name.to_uppercase();
                let ty_str = ty
                    .as_ref()
                    .map(|ty| self.type_to_rust(ty))
                    .unwrap_or_else(|| self.infer_const_type_name(value));
                write!(self.output, "const {}: {} = ", const_name, ty_str).unwrap();
                self.generate_expr(value)?;
                self.output.push_str(";\n");
                if let Some(actual_ty) = ty.as_ref() {
                    self.record_from_type(name, actual_ty);
                } else {
                    self.record_from_expr(name, value);
                }
            }
            ir::Stmt::Assign { target, value } => {
                self.write_indent();
                self.generate_expr(target)?;
                self.output.push_str(" = ");
                self.generate_expr(value)?;
                self.output.push_str(";\n");
                if let ir::Expr::Identifier(ident) = target {
                    self.record_from_expr(ident, value);
                }
            }
            ir::Stmt::Return(expr) => {
                self.write_indent();
                if let Some(expr) = expr {
                    self.output.push_str("return ");
                    self.generate_expr(expr)?;
                    self.output.push_str(";\n");
                } else {
                    self.output.push_str("return;\n");
                }
            }
            ir::Stmt::Throw(expr) => {
                self.write_indent();
                self.output.push_str("return Err(");
                self.generate_expr(expr)?;
                self.output.push_str(".into());\n");
            }
            ir::Stmt::Expr(expr) => {
                self.write_indent();
                self.generate_expr(expr)?;
                self.output.push_str(";\n");
            }
            ir::Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                self.write_indent();
                self.output.push_str("if ");
                self.generate_expr(condition)?;
                self.output.push_str(" {\n");
                self.indent();
                self.push_scope();
                self.generate_block(then_block)?;
                self.pop_scope();
                self.dedent();
                self.write_indent();
                self.output.push('}');
                if let Some(else_block) = else_block {
                    self.output.push_str(" else {\n");
                    self.indent();
                    self.push_scope();
                    self.generate_block(else_block)?;
                    self.pop_scope();
                    self.dedent();
                    self.write_indent();
                    self.output.push('}');
                }
                self.output.push('\n');
            }
            ir::Stmt::While { condition, body } => {
                self.write_indent();
                self.output.push_str("while ");
                self.generate_expr(condition)?;
                self.output.push_str(" {\n");
                self.indent();
                self.push_scope();
                self.generate_block(body)?;
                self.pop_scope();
                self.dedent();
                self.write_indent();
                self.output.push_str("}\n");
            }
            ir::Stmt::For {
                var,
                iterable,
                body,
                policy,
                options,
            } => match policy {
                ir::DataParallelPolicy::Seq => {
                    self.write_indent();
                    write!(self.output, "for {} in ", self.sanitize_name(var)).unwrap();
                    self.generate_expr(iterable)?;
                    self.output.push_str(" {\n");
                    self.indent();
                    self.push_scope();
                    self.set_var_format(var, FormatKind::Display);
                    self.generate_block(body)?;
                    self.pop_scope();
                    self.dedent();
                    self.write_indent();
                    self.output.push_str("}\n");
                }
                ir::DataParallelPolicy::Par => {
                    self.generate_parallel_for(var, iterable, body, options)?;
                }
                ir::DataParallelPolicy::Vec => {
                    self.generate_vector_for(var, iterable, body, options)?;
                }
                ir::DataParallelPolicy::ParVec => {
                    self.generate_parvec_for(var, iterable, body, options)?;
                }
            },
            ir::Stmt::Block(block) => {
                self.write_indent();
                self.output.push_str("{\n");
                self.indent();
                self.push_scope();
                self.generate_block(block)?;
                self.pop_scope();
                self.dedent();
                self.write_indent();
                self.output.push_str("}\n");
            }
            ir::Stmt::TryCatch {
                try_block,
                error_var,
                catch_block,
            } => {
                self.write_indent();
                self.output
                    .push_str("match (|| -> Result<(), Box<dyn std::error::Error>> {\n");
                self.indent();
                self.push_scope();
                self.generate_block(try_block)?;
                self.pop_scope();
                self.write_indent();
                self.output.push_str("Ok(())\n");
                self.dedent();
                self.write_indent();
                self.output.push_str("})() {\n");
                self.indent();
                self.write_indent();
                self.output.push_str("Ok(_) => {},\n");
                self.write_indent();
                write!(
                    self.output,
                    "Err({}) => {{\n",
                    self.sanitize_name(error_var)
                )
                .unwrap();
                self.indent();
                self.push_scope();
                self.set_var_format(error_var, FormatKind::Display);
                self.generate_block(catch_block)?;
                self.pop_scope();
                self.dedent();
                self.write_indent();
                self.output.push_str("}\n");
                self.dedent();
                self.write_indent();
                self.output.push_str("}\n");
            }
            ir::Stmt::Switch {
                discriminant,
                cases,
                default,
            } => {
                self.write_indent();
                self.output.push_str("match ");
                self.generate_expr(discriminant)?;
                self.output.push_str(" {\n");
                self.indent();
                for case in cases {
                    self.write_indent();
                    self.generate_expr(&case.value)?;
                    self.output.push_str(" => {\n");
                    self.indent();
                    self.push_scope();
                    for stmt in &case.body {
                        self.generate_stmt(stmt)?;
                    }
                    self.pop_scope();
                    self.dedent();
                    self.write_indent();
                    self.output.push_str("},\n");
                }
                if let Some(default_body) = default {
                    self.write_indent();
                    self.output.push_str("_ => {\n");
                    self.indent();
                    self.push_scope();
                    for stmt in default_body {
                        self.generate_stmt(stmt)?;
                    }
                    self.pop_scope();
                    self.dedent();
                    self.write_indent();
                    self.output.push_str("},\n");
                }
                self.dedent();
                self.write_indent();
                self.output.push_str("}\n");
            }
            _ => {
                return Err(CompilerError::CodegenError(
                    "Unsupported statement in IR generator".into(),
                ))
            }
        }
        Ok(())
    }

    fn generate_parallel_for(
        &mut self,
        var: &str,
        iterable: &ir::Expr,
        body: &ir::Block,
        options: &ir::ForPolicyOptions,
    ) -> Result<()> {
        self.write_indent();
        self.output.push_str("{\n");
        self.indent();

        self.write_indent();
        self.output.push_str("let __liva_iter = ");
        self.generate_expr(iterable)?;
        self.output.push_str(";\n");

        self.write_indent();
        write!(
            self.output,
            "liva_rt::for_par(__liva_iter, move |{}| {{\n",
            self.sanitize_name(var)
        )
        .unwrap();
        self.indent();
        self.push_scope();
        self.set_var_format(var, FormatKind::Display);
        self.generate_block(body)?;
        self.pop_scope();
        self.dedent();
        self.write_indent();
        self.output.push_str("}, ");
        self.emit_parallel_options(options);
        self.output.push('\n');
        self.write_indent();
        self.output.push_str(");\n");

        self.dedent();
        self.write_indent();
        self.output.push_str("}\n");
        Ok(())
    }

    fn generate_vector_for(
        &mut self,
        var: &str,
        iterable: &ir::Expr,
        body: &ir::Block,
        options: &ir::ForPolicyOptions,
    ) -> Result<()> {
        self.write_indent();
        self.output.push_str("{\n");
        self.indent();

        self.write_indent();
        self.output.push_str("let __liva_iter = ");
        self.generate_expr(iterable)?;
        self.output.push_str(";\n");

        self.write_indent();
        write!(
            self.output,
            "liva_rt::for_vec(__liva_iter, move |{}| {{\n",
            self.sanitize_name(var)
        )
        .unwrap();
        self.indent();
        self.push_scope();
        self.set_var_format(var, FormatKind::Display);
        self.generate_block(body)?;
        self.pop_scope();
        self.dedent();
        self.write_indent();
        self.output.push_str("}, ");
        self.emit_parallel_options(options);
        self.output.push('\n');
        self.write_indent();
        self.output.push_str(");\n");

        self.dedent();
        self.write_indent();
        self.output.push_str("}\n");
        Ok(())
    }

    fn generate_parvec_for(
        &mut self,
        var: &str,
        iterable: &ir::Expr,
        body: &ir::Block,
        options: &ir::ForPolicyOptions,
    ) -> Result<()> {
        self.write_indent();
        self.output.push_str("{\n");
        self.indent();

        self.write_indent();
        self.output.push_str("let __liva_iter = ");
        self.generate_expr(iterable)?;
        self.output.push_str(";\n");

        self.write_indent();
        write!(
            self.output,
            "liva_rt::for_parvec(__liva_iter, move |{}| {{\n",
            self.sanitize_name(var)
        )
        .unwrap();
        self.indent();
        self.push_scope();
        self.set_var_format(var, FormatKind::Display);
        self.generate_block(body)?;
        self.pop_scope();
        self.dedent();
        self.write_indent();
        self.output.push_str("}, ");
        self.emit_parallel_options(options);
        self.output.push('\n');
        self.write_indent();
        self.output.push_str(");\n");

        self.dedent();
        self.write_indent();
        self.output.push_str("}\n");
        Ok(())
    }

    fn emit_parallel_options(&mut self, options: &ir::ForPolicyOptions) {
        self.output.push_str("liva_rt::ParallelForOptions {\n");
        self.indent();

        self.write_indent();
        write!(self.output, "ordered: {},\n", options.ordered).unwrap();

        self.write_indent();
        if let Some(chunk) = options.chunk {
            write!(
                self.output,
                "chunk: Some(liva_rt::normalize_size({}, 1)),\n",
                chunk
            )
            .unwrap();
        } else {
            self.output.push_str("chunk: None,\n");
        }

        self.write_indent();
        match options.threads {
            Some(ir::ThreadOption::Auto) => {
                self.output
                    .push_str("threads: Some(liva_rt::ThreadOption::Auto),\n");
            }
            Some(ir::ThreadOption::Count(value)) => {
                write!(
                    self.output,
                    "threads: Some(liva_rt::ThreadOption::Count(liva_rt::normalize_size({}, 1))),\n",
                    value
                )
                .unwrap();
            }
            None => self.output.push_str("threads: None,\n"),
        }

        self.write_indent();
        match &options.simd_width {
            Some(ir::SimdWidthOption::Auto) => {
                self.output
                    .push_str("simd_width: Some(liva_rt::SimdWidthOption::Auto),\n");
            }
            Some(ir::SimdWidthOption::Width(value)) => {
                write!(
                    self.output,
                    "simd_width: Some(liva_rt::SimdWidthOption::Width(liva_rt::normalize_size({}, 1))),\n",
                    value
                )
                .unwrap();
            }
            None => self.output.push_str("simd_width: None,\n"),
        }

        self.write_indent();
        if let Some(prefetch) = options.prefetch {
            write!(
                self.output,
                "prefetch: Some(liva_rt::normalize_size({}, 1)),\n",
                prefetch
            )
            .unwrap();
        } else {
            self.output.push_str("prefetch: None,\n");
        }

        self.write_indent();
        match options.reduction {
            Some(ir::ReductionOption::Safe) => {
                self.output
                    .push_str("reduction: Some(liva_rt::ReductionOption::Safe),\n");
            }
            Some(ir::ReductionOption::Fast) => {
                self.output
                    .push_str("reduction: Some(liva_rt::ReductionOption::Fast),\n");
            }
            None => self.output.push_str("reduction: None,\n"),
        }

        self.write_indent();
        match options.schedule {
            Some(ir::ScheduleOption::Static) => {
                self.output
                    .push_str("schedule: Some(liva_rt::ScheduleOption::Static),\n");
            }
            Some(ir::ScheduleOption::Dynamic) => {
                self.output
                    .push_str("schedule: Some(liva_rt::ScheduleOption::Dynamic),\n");
            }
            None => self.output.push_str("schedule: None,\n"),
        }

        self.write_indent();
        match options.detect {
            Some(ir::DetectOption::Auto) => {
                self.output
                    .push_str("detect: Some(liva_rt::DetectOption::Auto),\n");
            }
            None => self.output.push_str("detect: None,\n"),
        }

        self.dedent();
        self.write_indent();
        self.output.push_str("}");
    }

    fn generate_expr(&mut self, expr: &ir::Expr) -> Result<()> {
        match expr {
            ir::Expr::Literal(lit) => self.generate_literal(lit),
            ir::Expr::Identifier(name) => {
                if name.chars().all(|c| c.is_uppercase() || c == '_') {
                    write!(self.output, "{}", name).unwrap();
                } else {
                    write!(self.output, "{}", self.sanitize_name(name)).unwrap();
                }
                Ok(())
            }
            ir::Expr::Call { callee, args } => {
                // Check if this is a .count() call on a sequence
                if let ir::Expr::Member { object, property } = callee.as_ref() {
                    if property == "count" {
                        self.generate_expr(object)?;
                        self.output.push_str(".count()");
                        return Ok(());
                    }
                }

                if matches!(callee.as_ref(), ir::Expr::Identifier(id) if id == "print") {
                    if args.is_empty() {
                        self.output.push_str("println!()")
                    } else {
                        self.output.push_str("println!(\"");
                        for _ in args {
                            self.output.push_str("{}");
                        }
                        self.output.push_str("\"");
                        let mut first = true;
                        for arg in args {
                            if first {
                                self.output.push_str(", ");
                                first = false;
                            } else {
                                self.output.push_str(", ");
                            }
                            self.generate_expr(arg)?;
                        }
                        self.output.push(')');
                    }
                    return Ok(());
                }
                self.generate_expr(callee)?;
                self.output.push('(');
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push(')');
                Ok(())
            }
            ir::Expr::Await(expr) => {
                self.generate_expr(expr)?;
                self.output.push_str(".await");
                Ok(())
            }
            ir::Expr::AsyncCall { callee, args } => {
                self.output.push_str("tokio::spawn(async move {");
                self.generate_expr(callee)?;
                self.output.push('(');
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push_str(") }).await.unwrap()");
                Ok(())
            }
            ir::Expr::ParallelCall { callee, args } => {
                self.output
                    .push_str("tokio::task::spawn_blocking(move || { ");
                self.generate_expr(callee)?;
                self.output.push('(');
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push(')');
                self.output.push_str(" })");
                self.output.push_str(".await.unwrap()");
                Ok(())
            }
            ir::Expr::TaskCall { mode, callee, args } => {
                match mode {
                    ir::ConcurrencyMode::Async => {
                        self.output.push_str("tokio::spawn(async move { ");
                        write!(self.output, "{}(", self.sanitize_name(callee)).unwrap();
                        for (idx, arg) in args.iter().enumerate() {
                            if idx > 0 {
                                self.output.push_str(", ");
                            }
                            self.generate_expr(arg)?;
                        }
                        self.output.push(')');
                        self.output.push_str(" })");
                    }
                    ir::ConcurrencyMode::Parallel => {
                        self.output
                            .push_str("tokio::task::spawn_blocking(move || { ");
                        write!(self.output, "{}(", self.sanitize_name(callee)).unwrap();
                        for (idx, arg) in args.iter().enumerate() {
                            if idx > 0 {
                                self.output.push_str(", ");
                            }
                            self.generate_expr(arg)?;
                        }
                        self.output.push(')');
                        self.output.push_str(" })");
                    }
                }
                Ok(())
            }
            ir::Expr::FireCall { mode, callee, args } => {
                match mode {
                    ir::ConcurrencyMode::Async => {
                        self.output.push_str("tokio::spawn(async move { ");
                        write!(self.output, "{}(", self.sanitize_name(callee)).unwrap();
                        for (idx, arg) in args.iter().enumerate() {
                            if idx > 0 {
                                self.output.push_str(", ");
                            }
                            self.generate_expr(arg)?;
                        }
                        self.output.push(')');
                        self.output.push_str(" })");
                    }
                    ir::ConcurrencyMode::Parallel => {
                        self.output
                            .push_str("tokio::task::spawn_blocking(move || { ");
                        write!(self.output, "{}(", self.sanitize_name(callee)).unwrap();
                        for (idx, arg) in args.iter().enumerate() {
                            if idx > 0 {
                                self.output.push_str(", ");
                            }
                            self.generate_expr(arg)?;
                        }
                        self.output.push(')');
                        self.output.push_str(" })");
                    }
                }
                Ok(())
            }
            ir::Expr::Binary { op, left, right } => {
                if matches!(op, ir::BinaryOp::Add)
                    && (self.expr_is_stringy(left) || self.expr_is_stringy(right))
                {
                    self.output.push_str("format!(\"{}{}\", ");
                    self.generate_expr_for_string_concat(left)?;
                    self.output.push_str(", ");
                    self.generate_expr_for_string_concat(right)?;
                    self.output.push(')');
                } else {
                    let needs_paren = !matches!(
                        left.as_ref(),
                        ir::Expr::Literal(_) | ir::Expr::Identifier(_)
                    ) || !matches!(
                        right.as_ref(),
                        ir::Expr::Literal(_) | ir::Expr::Identifier(_)
                    );
                    if needs_paren {
                        self.output.push('(');
                    }
                    
                    // Special handling for string multiplication
                    if matches!(op, ir::BinaryOp::Mul) {
                        // Check if at least one operand is a string literal or template
                        let has_string_literal = matches!(
                            left.as_ref(),
                            ir::Expr::Literal(ir::Literal::String(_)) | ir::Expr::StringTemplate(_)
                        ) || matches!(
                            right.as_ref(),
                            ir::Expr::Literal(ir::Literal::String(_)) | ir::Expr::StringTemplate(_)
                        );
                        
                        if has_string_literal {
                            // Use string_mul helper that handles both String*int and int*String
                            self.output.push_str("liva_rt::string_mul(");
                            self.generate_expr(left)?;
                            self.output.push_str(", ");
                            self.generate_expr(right)?;
                            self.output.push(')');
                        } else {
                            // Pure numeric multiplication
                            self.generate_numeric_operand(left)?;
                            write!(self.output, " {} ", binary_op_str(op)).unwrap();
                            self.generate_numeric_operand(right)?;
                        }
                    } else if matches!(
                        op,
                        ir::BinaryOp::Add
                            | ir::BinaryOp::Sub
                            | ir::BinaryOp::Div
                            | ir::BinaryOp::Mod
                    ) {
                        self.generate_numeric_operand(left)?;
                        write!(self.output, " {} ", binary_op_str(op)).unwrap();
                        self.generate_numeric_operand(right)?;
                    } else {
                        self.generate_expr(left)?;
                        write!(self.output, " {} ", binary_op_str(op)).unwrap();
                        self.generate_expr(right)?;
                    }
                    if needs_paren {
                        self.output.push(')');
                    }
                }
                Ok(())
            }
            ir::Expr::Unary { op, operand } => {
                match op {
                    ir::UnaryOp::Neg => self.output.push('-'),
                    ir::UnaryOp::Not => self.output.push('!'),
                }
                self.generate_expr(operand)
            }
            ir::Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                // Check if either branch is a call to Err (representing fail)
                let then_is_err = matches!(then_expr.as_ref(), ir::Expr::Call { callee, .. }
                    if matches!(callee.as_ref(), ir::Expr::Identifier(id) if id == "Err"));
                let else_is_err = matches!(else_expr.as_ref(), ir::Expr::Call { callee, .. }
                    if matches!(callee.as_ref(), ir::Expr::Identifier(id) if id == "Err"));

                if then_is_err || else_is_err {
                    // Generate Result-returning expression for ternary with fail
                    self.output.push_str("if ");
                    self.generate_expr(condition)?;
                    self.output.push_str(" { ");
                    if then_is_err {
                        self.generate_expr(then_expr)?;
                    } else {
                        self.output.push_str("Ok(");
                        self.generate_expr(then_expr)?;
                        self.output.push_str(")");
                    }
                    self.output.push_str(" } else { ");
                    if else_is_err {
                        self.generate_expr(else_expr)?;
                    } else {
                        self.output.push_str("Ok(");
                        self.generate_expr(else_expr)?;
                        self.output.push_str(")");
                    }
                    self.output.push_str(" }");
                } else {
                    // Normal ternary expression
                    self.output.push_str("if ");
                    self.generate_expr(condition)?;
                    self.output.push_str(" { ");
                    self.generate_expr(then_expr)?;
                    self.output.push_str(" } else { ");
                    self.generate_expr(else_expr)?;
                    self.output.push_str(" }");
                }
                Ok(())
            }
            ir::Expr::StringTemplate(parts) => {
                self.output.push_str("format!(\"");
                let mut args = Vec::new();
                for part in parts {
                    match part {
                        ir::TemplatePart::Text(text) => {
                            for ch in text.chars() {
                                match ch {
                                    '"' => self.output.push_str("\\\""),
                                    '\\' => self.output.push_str("\\\\"),
                                    '\n' => self.output.push_str("\\n"),
                                    '\r' => self.output.push_str("\\r"),
                                    '\t' => self.output.push_str("\\t"),
                                    '{' => self.output.push_str("{{"),
                                    '}' => self.output.push_str("}}"),
                                    _ => self.output.push(ch),
                                }
                            }
                        }
                        ir::TemplatePart::Expr(expr) => {
                            self.output
                                .push_str(self.template_placeholder_for_expr(expr));
                            args.push(expr);
                        }
                    }
                }
                self.output.push('"');
                if !args.is_empty() {
                    for arg in args {
                        self.output.push_str(", ");
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push(')');
                Ok(())
            }
            ir::Expr::Range { start, end } => {
                self.generate_expr(start)?;
                self.output.push_str(" .. ");
                self.generate_expr(end)?;
                Ok(())
            }
            ir::Expr::Member { object, property } => {
                if property == "length" {
                    // Bug #31 fix: Wrap in parens so .toString() works: (x.len() as i32).to_string()
                    self.output.push('(');
                    self.generate_expr(object)?;
                    self.output.push_str(".len() as i32)");
                    return Ok(());
                }
                
                self.generate_expr(object)?;

                // For liva_rt members, use dot notation
                if self.is_liva_rt_member(object) {
                    write!(self.output, ".{}", property).unwrap();
                } else {
                    // For serde_json::Value objects, use bracket notation instead of dot notation
                    // This handles cases where we're accessing properties of object literals in arrays
                    // Automatic conversion to appropriate types happens in binary operations
                    write!(self.output, "[\"{}\"]", property).unwrap();
                }
                Ok(())
            }
            ir::Expr::Index { object, index } => {
                self.generate_expr(object)?;
                self.output.push('[');
                self.generate_expr(index)?;
                self.output.push(']');
                Ok(())
            }
            ir::Expr::ObjectLiteral(fields) => {
                self.output.push_str("serde_json::json!({\n");
                self.indent();
                for (idx, (key, value)) in fields.iter().enumerate() {
                    if idx > 0 {
                        self.output.push_str(",\n");
                    }
                    self.write_indent();
                    write!(self.output, "\"{}\": ", key).unwrap();
                    self.generate_expr(value)?;
                }
                self.output.push('\n');
                self.dedent();
                self.write_indent();
                self.output.push_str("})");
                Ok(())
            }
            ir::Expr::StructLiteral { type_name, fields } => {
                write!(self.output, "{} {{", type_name).unwrap();
                self.indent();
                for (idx, (key, value)) in fields.iter().enumerate() {
                    if idx > 0 {
                        self.output.push_str(",");
                    }
                    self.output.push('\n');
                    self.write_indent();
                    write!(self.output, "{}: ", self.sanitize_name(key)).unwrap();
                    // Add .to_string() for string literals assigned to string fields
                    if key == "name" {
                        if let ir::Expr::Literal(ir::Literal::String(_)) = value {
                            self.generate_expr(value)?;
                            self.output.push_str(".to_string()");
                        } else {
                            self.generate_expr(value)?;
                        }
                    } else {
                        self.generate_expr(value)?;
                    }
                }
                self.output.push('\n');
                self.dedent();
                self.write_indent();
                self.output.push('}');
                Ok(())
            }
            ir::Expr::ArrayLiteral(elements) => {
                self.output.push_str("vec![");
                for (idx, elem) in elements.iter().enumerate() {
                    if idx > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(elem)?;
                }
                self.output.push(']');
                Ok(())
            }
            ir::Expr::TupleLiteral(elements) => {
                self.output.push('(');
                for (idx, elem) in elements.iter().enumerate() {
                    if idx > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(elem)?;
                }
                // Rust requires trailing comma for single-element tuples
                if elements.len() == 1 {
                    self.output.push(',');
                }
                self.output.push(')');
                Ok(())
            }
            ir::Expr::Lambda(lambda) => {
                if lambda.is_move {
                    self.output.push_str("move ");
                }
                self.output.push('|');

                for (idx, param) in lambda.params.iter().enumerate() {
                    if idx > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&self.sanitize_name(&param.name));
                    if let Some(type_ref) = &param.type_ref {
                        self.output.push_str(": ");
                        self.output.push_str(type_ref);
                    }
                }

                self.output.push_str("| ");

                match &lambda.body {
                    ir::LambdaBody::Expr(expr) => {
                        self.generate_expr(expr)?;
                    }
                    ir::LambdaBody::Block(block) => {
                        // For lambdas, we need to generate a block that returns a value
                        // Check if the last statement is a return statement
                        if let Some(last_stmt) = block.last() {
                            if let ir::Stmt::Return(expr) = last_stmt {
                                if let Some(expr) = expr {
                                    // Generate the block with statements except the last return
                                    self.output.push('{');
                                    self.indent();
                                    self.output.push('\n');
                                    self.write_indent();

                                    // Generate all statements except the last return
                                    for stmt in &block[..block.len() - 1] {
                                        self.generate_stmt(stmt)?;
                                        self.output.push('\n');
                                        self.write_indent();
                                    }

                                    // Generate the return expression without the return keyword
                                    self.generate_expr(expr)?;

                                    self.dedent();
                                    self.output.push('\n');
                                    self.write_indent();
                                    self.output.push('}');
                                } else {
                                    // Empty return, generate unit type
                                    self.output.push_str("()");
                                }
                            } else {
                                // No return statement, generate block as-is
                                self.output.push('{');
                                self.indent();
                                self.output.push('\n');
                                self.write_indent();
                                for stmt in block {
                                    self.generate_stmt(stmt)?;
                                    self.output.push('\n');
                                    self.write_indent();
                                }
                                self.dedent();
                                self.output.push('}');
                            }
                        } else {
                            // Empty block
                            self.output.push_str("()");
                        }
                    }
                }
                Ok(())
            }
            ir::Expr::Unsupported(ast_expr) => {
                // Handle unsupported IR expressions that are passed through from AST
                if let Expr::Switch(switch_expr) = ast_expr {
                    self.generate_switch_expr(switch_expr)?;
                    Ok(())
                } else {
                    Err(CompilerError::CodegenError(
                        "Unsupported expression in IR generator".into(),
                    ))
                }
            }
        }
    }

    fn generate_literal(&mut self, lit: &ir::Literal) -> Result<()> {
        match lit {
            ir::Literal::Int(v) => write!(self.output, "{}", v).unwrap(),
            ir::Literal::Float(v) => write!(self.output, "{:?}", v).unwrap(),
            ir::Literal::Bool(v) => write!(self.output, "{}", v).unwrap(),
            ir::Literal::String(s) => {
                // Write string with proper escape sequences interpreted
                // Don't use escape_default() as it would escape the escapes (\\n instead of \n)
                self.output.push('"');
                self.output.push_str(s);
                self.output.push('"');
            }
            ir::Literal::Char(c) => write!(self.output, "'{}'", c.escape_default()).unwrap(),
            ir::Literal::Null => self.output.push_str("None"),
        }
        Ok(())
    }

    /// Generate code for switch expression (pattern matching) from AST
    fn generate_switch_expr(&mut self, switch_expr: &SwitchExpr) -> Result<()> {
        // Generate Rust match expression
        self.output.push_str("match ");
        self.generate_expr_from_ast(&switch_expr.discriminant)?;
        self.output.push_str(" {");
        self.indent();

        for arm in &switch_expr.arms {
            self.output.push('\n');
            self.write_indent();

            // Generate pattern
            self.generate_pattern(&arm.pattern)?;

            // Generate guard if present
            if let Some(guard) = &arm.guard {
                self.output.push_str(" if ");
                self.generate_expr_from_ast(guard)?;
            }

            self.output.push_str(" => ");

            // Generate body
            match &arm.body {
                SwitchBody::Expr(expr) => {
                    self.generate_expr_from_ast(expr)?;
                }
                SwitchBody::Block(stmts) => {
                    self.output.push('{');
                    self.indent();
                    for stmt in stmts {
                        self.output.push('\n');
                        self.write_indent();
                        self.generate_stmt_from_ast(stmt)?;
                    }
                    self.dedent();
                    self.output.push('\n');
                    self.write_indent();
                    self.output.push('}');
                }
            }

            self.output.push(',');
        }

        self.dedent();
        self.output.push('\n');
        self.write_indent();
        self.output.push('}');

        Ok(())
    }

    /// Generate code for a pattern
    fn generate_pattern(&mut self, pattern: &Pattern) -> Result<()> {
        match pattern {
            Pattern::Literal(lit) => {
                self.generate_ast_literal(lit)?;
            }
            Pattern::Wildcard => {
                self.output.push('_');
            }
            Pattern::Binding(name) => {
                self.output.push_str(&self.sanitize_name(name));
            }
            Pattern::Range(range) => {
                match (&range.start, &range.end, range.inclusive) {
                    (Some(start), Some(end), true) => {
                        self.generate_expr_from_ast(start)?;
                        self.output.push_str("..=");
                        self.generate_expr_from_ast(end)?;
                    }
                    (Some(start), Some(end), false) => {
                        self.generate_expr_from_ast(start)?;
                        self.output.push_str("..");
                        self.generate_expr_from_ast(end)?;
                    }
                    (Some(start), None, _) => {
                        self.generate_expr_from_ast(start)?;
                        self.output.push_str("..");
                    }
                    (None, Some(end), true) => {
                        self.output.push_str("..=");
                        self.generate_expr_from_ast(end)?;
                    }
                    (None, Some(end), false) => {
                        self.output.push_str("..");
                        self.generate_expr_from_ast(end)?;
                    }
                    (None, None, _) => {
                        self.output.push_str("..");
                    }
                }
            }
            Pattern::Tuple(patterns) => {
                self.output.push('(');
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_pattern(pat)?;
                }
                self.output.push(')');
            }
            Pattern::Array(patterns) => {
                self.output.push('[');
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_pattern(pat)?;
                }
                self.output.push(']');
            }
            Pattern::Or(patterns) => {
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" | ");
                    }
                    self.generate_pattern(pat)?;
                }
            }
            Pattern::Typed { name, type_ref } => {
                // Type pattern for union narrowing: name: type
                // This will be handled specially in generate_switch_expr
                self.output.push_str(&self.sanitize_name(name));
            }
        }
        Ok(())
    }

    /// Generate code for an AST expression (used for switch patterns)
    fn generate_expr_from_ast(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(lit) => self.generate_ast_literal(lit),
            Expr::Identifier(name) => {
                write!(self.output, "{}", self.sanitize_name(name)).unwrap();
                Ok(())
            }
            Expr::Binary { op, left, right } => {
                self.generate_expr_from_ast(left)?;
                write!(self.output, " {} ", op).unwrap();
                self.generate_expr_from_ast(right)?;
                Ok(())
            }
            _ => {
                // For complex expressions, we'd need full AST generation
                // For now, just handle the simple cases needed for patterns
                Err(CompilerError::CodegenError(
                    "Complex expressions in switch patterns not yet supported".into(),
                ))
            }
        }
    }

    /// Generate code for an AST literal
    fn generate_ast_literal(&mut self, lit: &Literal) -> Result<()> {
        match lit {
            Literal::Int(v) => write!(self.output, "{}", v).unwrap(),
            Literal::Float(v) => write!(self.output, "{:?}", v).unwrap(),
            Literal::Bool(v) => write!(self.output, "{}", v).unwrap(),
            Literal::String(s) => {
                self.output.push('"');
                self.output.push_str(s);
                self.output.push('"');
            }
            Literal::Char(c) => write!(self.output, "'{}'", c.escape_default()).unwrap(),
            Literal::Null => self.output.push_str("None"),
        }
        Ok(())
    }

    /// Generate code for an AST statement (used for switch block bodies)
    fn generate_stmt_from_ast(&mut self, stmt: &Stmt) -> Result<()> {
        // For now, we'll just handle the basic cases needed for switch blocks
        // This is a simplified version - full AST generation would need more
        match stmt {
            Stmt::Expr(expr_stmt) => {
                self.generate_expr_from_ast(&expr_stmt.expr)?;
                self.output.push(';');
                Ok(())
            }
            Stmt::Return(ret_stmt) => {
                self.output.push_str("return");
                if let Some(expr) = &ret_stmt.expr {
                    self.output.push(' ');
                    self.generate_expr_from_ast(expr)?;
                }
                self.output.push(';');
                Ok(())
            }
            _ => {
                Err(CompilerError::CodegenError(
                    "Complex statements in switch blocks not yet fully supported".into(),
                ))
            }
        }
    }


    fn expr_is_stringy(&self, expr: &ir::Expr) -> bool {
        match expr {
            ir::Expr::Literal(ir::Literal::String(_)) => true,
            ir::Expr::StringTemplate(_) => true,
            ir::Expr::Binary {
                op: ir::BinaryOp::Add,
                left,
                right,
            } => self.expr_is_stringy(left) || self.expr_is_stringy(right),
            _ => false,
        }
    }

    /// Generate an expression with special handling for error binding variables in string context
    fn generate_expr_for_string_concat(&mut self, expr: &ir::Expr) -> Result<()> {
        // Check if this is an error binding variable
        if let ir::Expr::Identifier(name) = expr {
            let sanitized = self.sanitize_name(name);
            if self.error_binding_vars.contains(&sanitized) {
                // For error variables, extract the message: err.as_ref().map(|e| e.message.as_str()).unwrap_or("")
                write!(
                    self.output,
                    "{}.as_ref().map(|e| e.message.as_str()).unwrap_or(\"\")",
                    sanitized
                ).unwrap();
                return Ok(());
            }
        }
        // Otherwise, generate normally
        self.generate_expr(expr)
    }

    fn type_to_rust(&self, ty: &ir::Type) -> String {
        match ty {
            ir::Type::Number => "i32".into(),
            ir::Type::Float => "f64".into(),
            ir::Type::Bool => "bool".into(),
            ir::Type::String => "String".into(),
            ir::Type::Bytes => "Vec<u8>".into(),
            ir::Type::Char => "char".into(),
            ir::Type::Array(inner) => format!("Vec<{}>", self.type_to_rust(inner)),
            ir::Type::Optional(inner) => format!("Option<{}>", self.type_to_rust(inner)),
            ir::Type::Custom(name) => name.clone(),
            ir::Type::Unit => "()".into(),
            ir::Type::Inferred => "i32".into(),
        }
    }

    fn sanitize_name(&self, name: &str) -> String {
        let name = name.trim_start_matches('_');
        to_snake_case(name)
    }

    fn sanitize_test_name(&self, name: &str) -> String {
        name.replace(' ', "_").replace('-', "_").to_lowercase()
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_lowercase = false;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && prev_lowercase {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_lowercase = false;
        } else {
            result.push(ch);
            prev_lowercase = ch.is_lowercase();
        }
    }

    if result.is_empty() {
        "_".into()
    } else {
        result
    }
}

fn binary_op_str(op: &ir::BinaryOp) -> &'static str {
    match op {
        ir::BinaryOp::Add => "+",
        ir::BinaryOp::Sub => "-",
        ir::BinaryOp::Mul => "*",
        ir::BinaryOp::Div => "/",
        ir::BinaryOp::Mod => "%",
        ir::BinaryOp::Eq => "==",
        ir::BinaryOp::Ne => "!=",
        ir::BinaryOp::Lt => "<",
        ir::BinaryOp::Le => "<=",
        ir::BinaryOp::Gt => ">",
        ir::BinaryOp::Ge => ">=",
        ir::BinaryOp::And => "&&",
        ir::BinaryOp::Or => "||",
    }
}

fn module_has_unsupported(module: &ir::Module) -> bool {
    module.items.iter().any(|item| match item {
        ir::Item::Unsupported(_) => true,
        ir::Item::Function(func) => function_has_unsupported(func),
        ir::Item::Test(test) => block_has_unsupported(&test.body),
    })
}

fn function_has_unsupported(func: &ir::Function) -> bool {
    block_has_unsupported(&func.body)
}

fn block_has_unsupported(block: &ir::Block) -> bool {
    block.statements.iter().any(stmt_has_unsupported)
}

fn stmt_has_unsupported(stmt: &ir::Stmt) -> bool {
    match stmt {
        ir::Stmt::Unsupported(_) => true,
        ir::Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            expr_has_unsupported(condition)
                || block_has_unsupported(then_block)
                || else_block
                    .as_ref()
                    .map(block_has_unsupported)
                    .unwrap_or(false)
        }
        ir::Stmt::While { condition, body } => {
            expr_has_unsupported(condition) || block_has_unsupported(body)
        }
        ir::Stmt::For { iterable, body, .. } => {
            expr_has_unsupported(iterable) || block_has_unsupported(body)
        }
        ir::Stmt::Block(block) => block_has_unsupported(block),
        ir::Stmt::Let { value, .. } => expr_has_unsupported(value),
        ir::Stmt::Const { value, .. } => expr_has_unsupported(value),
        ir::Stmt::Assign { target, value } => {
            expr_has_unsupported(target) || expr_has_unsupported(value)
        }
        ir::Stmt::Return(expr) => expr.as_ref().map(expr_has_unsupported).unwrap_or(false),
        ir::Stmt::Throw(expr) => expr_has_unsupported(expr),
        ir::Stmt::Expr(expr) => expr_has_unsupported(expr),
        ir::Stmt::TryCatch {
            try_block,
            catch_block,
            ..
        } => block_has_unsupported(try_block) || block_has_unsupported(catch_block),
        ir::Stmt::Switch {
            discriminant,
            cases,
            default,
        } => {
            expr_has_unsupported(discriminant)
                || cases
                    .iter()
                    .any(|case| case.body.iter().any(stmt_has_unsupported))
                || default
                    .as_ref()
                    .map(|body| body.iter().any(stmt_has_unsupported))
                    .unwrap_or(false)
        }
    }
}

fn expr_has_unsupported(expr: &ir::Expr) -> bool {
    match expr {
        ir::Expr::Unsupported(_) => true,
        ir::Expr::Literal(_) | ir::Expr::Identifier(_) => false,
        ir::Expr::Call { callee, args }
        | ir::Expr::AsyncCall { callee, args }
        | ir::Expr::ParallelCall { callee, args } => {
            expr_has_unsupported(callee) || args.iter().any(expr_has_unsupported)
        }
        ir::Expr::TaskCall { args, .. } | ir::Expr::FireCall { args, .. } => {
            args.iter().any(expr_has_unsupported)
        }
        ir::Expr::Await(expr) => expr_has_unsupported(expr),
        ir::Expr::Unary { operand, .. } => expr_has_unsupported(operand),
        ir::Expr::Binary { left, right, .. } => {
            expr_has_unsupported(left) || expr_has_unsupported(right)
        }
        ir::Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_has_unsupported(condition)
                || expr_has_unsupported(then_expr)
                || expr_has_unsupported(else_expr)
        }
        ir::Expr::StringTemplate(parts) => parts.iter().any(|part| match part {
            ir::TemplatePart::Text(_) => false,
            ir::TemplatePart::Expr(expr) => expr_has_unsupported(expr),
        }),
        ir::Expr::Member { object, .. } => expr_has_unsupported(object),
        ir::Expr::Index { object, index } => {
            expr_has_unsupported(object) || expr_has_unsupported(index)
        }
        ir::Expr::Range { start, end } => expr_has_unsupported(start) || expr_has_unsupported(end),
        ir::Expr::ObjectLiteral(fields) => {
            fields.iter().any(|(_, value)| expr_has_unsupported(value))
        }
        ir::Expr::StructLiteral { fields, .. } => {
            fields.iter().any(|(_, value)| expr_has_unsupported(value))
        }
        ir::Expr::ArrayLiteral(elements) => elements.iter().any(expr_has_unsupported),
        ir::Expr::TupleLiteral(elements) => elements.iter().any(expr_has_unsupported),
        ir::Expr::Lambda(lambda) => match &lambda.body {
            ir::LambdaBody::Expr(expr) => expr_has_unsupported(expr),
            ir::LambdaBody::Block(block) => block.iter().any(stmt_has_unsupported),
        },
    }
}

#[allow(dead_code)]
fn module_has_async_concurrency(module: &ir::Module) -> bool {
    module.items.iter().any(|item| match item {
        ir::Item::Function(func) => {
            matches!(func.async_kind, ir::AsyncKind::Async)
                || block_has_async_concurrency(&func.body)
        }
        ir::Item::Test(test) => block_has_async_concurrency(&test.body),
        ir::Item::Unsupported(_) => false,
    })
}

#[allow(dead_code)]
fn module_has_parallel_concurrency(module: &ir::Module) -> bool {
    module.items.iter().any(|item| match item {
        ir::Item::Function(func) => block_has_parallel_concurrency(&func.body),
        ir::Item::Test(test) => block_has_parallel_concurrency(&test.body),
        ir::Item::Unsupported(_) => false,
    })
}

#[allow(dead_code)]
fn block_has_async_concurrency(block: &ir::Block) -> bool {
    block.statements.iter().any(stmt_has_async_concurrency)
}

#[allow(dead_code)]
fn block_has_parallel_concurrency(block: &ir::Block) -> bool {
    block.statements.iter().any(stmt_has_parallel_concurrency)
}

#[allow(dead_code)]
fn stmt_has_async_concurrency(stmt: &ir::Stmt) -> bool {
    match stmt {
        ir::Stmt::Let { value, .. } | ir::Stmt::Const { value, .. } => {
            expr_has_async_concurrency(value)
        }
        ir::Stmt::Assign { target, value } => {
            expr_has_async_concurrency(target) || expr_has_async_concurrency(value)
        }
        ir::Stmt::Return(expr) => expr
            .as_ref()
            .map(expr_has_async_concurrency)
            .unwrap_or(false),
        ir::Stmt::Throw(expr) | ir::Stmt::Expr(expr) => expr_has_async_concurrency(expr),
        ir::Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            expr_has_async_concurrency(condition)
                || block_has_async_concurrency(then_block)
                || else_block
                    .as_ref()
                    .map(block_has_async_concurrency)
                    .unwrap_or(false)
        }
        ir::Stmt::While { condition, body } => {
            expr_has_async_concurrency(condition) || block_has_async_concurrency(body)
        }
        ir::Stmt::For { iterable, body, .. } => {
            expr_has_async_concurrency(iterable) || block_has_async_concurrency(body)
        }
        ir::Stmt::Block(block) => block_has_async_concurrency(block),
        ir::Stmt::TryCatch {
            try_block,
            catch_block,
            ..
        } => block_has_async_concurrency(try_block) || block_has_async_concurrency(catch_block),
        ir::Stmt::Switch {
            discriminant,
            cases,
            default,
        } => {
            expr_has_async_concurrency(discriminant)
                || cases
                    .iter()
                    .any(|case| case.body.iter().any(stmt_has_async_concurrency))
                || default
                    .as_ref()
                    .map(|body| body.iter().any(stmt_has_async_concurrency))
                    .unwrap_or(false)
        }
        ir::Stmt::Unsupported(_) => false,
    }
}

#[allow(dead_code)]
fn stmt_has_parallel_concurrency(stmt: &ir::Stmt) -> bool {
    match stmt {
        ir::Stmt::Let { value, .. } | ir::Stmt::Const { value, .. } => {
            expr_has_parallel_concurrency(value)
        }
        ir::Stmt::Assign { target, value } => {
            expr_has_parallel_concurrency(target) || expr_has_parallel_concurrency(value)
        }
        ir::Stmt::Return(expr) => expr
            .as_ref()
            .map(expr_has_parallel_concurrency)
            .unwrap_or(false),
        ir::Stmt::Throw(expr) | ir::Stmt::Expr(expr) => expr_has_parallel_concurrency(expr),
        ir::Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            expr_has_parallel_concurrency(condition)
                || block_has_parallel_concurrency(then_block)
                || else_block
                    .as_ref()
                    .map(block_has_parallel_concurrency)
                    .unwrap_or(false)
        }
        ir::Stmt::While { condition, body } => {
            expr_has_parallel_concurrency(condition) || block_has_parallel_concurrency(body)
        }
        ir::Stmt::For {
            iterable,
            body,
            policy,
            ..
        } => {
            matches!(
                policy,
                ir::DataParallelPolicy::Par
                    | ir::DataParallelPolicy::Vec
                    | ir::DataParallelPolicy::ParVec
            ) || expr_has_parallel_concurrency(iterable)
                || block_has_parallel_concurrency(body)
        }
        ir::Stmt::Block(block) => block_has_parallel_concurrency(block),
        ir::Stmt::TryCatch {
            try_block,
            catch_block,
            ..
        } => {
            block_has_parallel_concurrency(try_block) || block_has_parallel_concurrency(catch_block)
        }
        ir::Stmt::Switch {
            discriminant,
            cases,
            default,
        } => {
            expr_has_parallel_concurrency(discriminant)
                || cases
                    .iter()
                    .any(|case| case.body.iter().any(stmt_has_parallel_concurrency))
                || default
                    .as_ref()
                    .map(|body| body.iter().any(stmt_has_parallel_concurrency))
                    .unwrap_or(false)
        }
        ir::Stmt::Unsupported(_) => false,
    }
}

#[allow(dead_code)]
fn expr_has_async_concurrency(expr: &ir::Expr) -> bool {
    match expr {
        &ir::Expr::AsyncCall { .. }
        | &ir::Expr::Await(_)
        | &ir::Expr::TaskCall {
            mode: ir::ConcurrencyMode::Async,
            ..
        }
        | &ir::Expr::FireCall {
            mode: ir::ConcurrencyMode::Async,
            ..
        } => true,
        &ir::Expr::Call {
            ref callee,
            ref args,
        }
        | &ir::Expr::ParallelCall {
            ref callee,
            ref args,
        } => {
            expr_has_async_concurrency(callee.as_ref())
                || args.iter().any(expr_has_async_concurrency)
        }
        &ir::Expr::TaskCall { ref args, .. } | &ir::Expr::FireCall { ref args, .. } => {
            args.iter().any(expr_has_async_concurrency)
        }
        &ir::Expr::Unary { ref operand, .. } => expr_has_async_concurrency(operand.as_ref()),
        &ir::Expr::Binary {
            ref left,
            ref right,
            ..
        } => {
            expr_has_async_concurrency(left.as_ref()) || expr_has_async_concurrency(right.as_ref())
        }
        &ir::Expr::Ternary {
            ref condition,
            ref then_expr,
            ref else_expr,
        } => {
            expr_has_async_concurrency(condition.as_ref())
                || expr_has_async_concurrency(then_expr.as_ref())
                || expr_has_async_concurrency(else_expr.as_ref())
        }
        &ir::Expr::StringTemplate(ref parts) => parts.iter().any(|part| match part {
            ir::TemplatePart::Text(_) => false,
            ir::TemplatePart::Expr(expr) => expr_has_async_concurrency(expr),
        }),
        &ir::Expr::Member { ref object, .. } => expr_has_async_concurrency(object.as_ref()),
        &ir::Expr::Index {
            ref object,
            ref index,
        } => {
            expr_has_async_concurrency(object.as_ref())
                || expr_has_async_concurrency(index.as_ref())
        }
        &ir::Expr::Range { ref start, ref end } => {
            expr_has_async_concurrency(start.as_ref()) || expr_has_async_concurrency(end.as_ref())
        }
        &ir::Expr::ObjectLiteral(ref fields) => fields
            .iter()
            .any(|(_, value)| expr_has_async_concurrency(value)),
        &ir::Expr::StructLiteral { ref fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_has_async_concurrency(value)),
        &ir::Expr::ArrayLiteral(ref elements) => elements.iter().any(expr_has_async_concurrency),
        &ir::Expr::TupleLiteral(ref elements) => elements.iter().any(expr_has_async_concurrency),
        ir::Expr::Lambda(lambda) => match &lambda.body {
            ir::LambdaBody::Expr(expr) => expr_has_async_concurrency(expr),
            ir::LambdaBody::Block(block) => block.iter().any(stmt_has_async_concurrency),
        },
        &ir::Expr::Literal(_) | &ir::Expr::Identifier(_) | &ir::Expr::Unsupported(_) => false,
    }
}

#[allow(dead_code)]
fn expr_has_parallel_concurrency(expr: &ir::Expr) -> bool {
    match expr {
        &ir::Expr::ParallelCall { .. }
        | &ir::Expr::TaskCall {
            mode: ir::ConcurrencyMode::Parallel,
            ..
        }
        | &ir::Expr::FireCall {
            mode: ir::ConcurrencyMode::Parallel,
            ..
        } => true,
        &ir::Expr::Call {
            ref callee,
            ref args,
        }
        | &ir::Expr::AsyncCall {
            ref callee,
            ref args,
        } => {
            expr_has_parallel_concurrency(callee.as_ref())
                || args.iter().any(expr_has_parallel_concurrency)
        }
        &ir::Expr::TaskCall { ref args, .. } | &ir::Expr::FireCall { ref args, .. } => {
            args.iter().any(expr_has_parallel_concurrency)
        }
        &ir::Expr::Unary { ref operand, .. } => expr_has_parallel_concurrency(operand.as_ref()),
        &ir::Expr::Binary {
            ref left,
            ref right,
            ..
        } => {
            expr_has_parallel_concurrency(left.as_ref())
                || expr_has_parallel_concurrency(right.as_ref())
        }
        &ir::Expr::Ternary {
            ref condition,
            ref then_expr,
            ref else_expr,
        } => {
            expr_has_parallel_concurrency(condition.as_ref())
                || expr_has_parallel_concurrency(then_expr.as_ref())
                || expr_has_parallel_concurrency(else_expr.as_ref())
        }
        &ir::Expr::StringTemplate(ref parts) => parts.iter().any(|part| match part {
            ir::TemplatePart::Text(_) => false,
            ir::TemplatePart::Expr(expr) => expr_has_parallel_concurrency(expr),
        }),
        &ir::Expr::Member { ref object, .. } => expr_has_parallel_concurrency(object.as_ref()),
        &ir::Expr::Index {
            ref object,
            ref index,
        } => {
            expr_has_parallel_concurrency(object.as_ref())
                || expr_has_parallel_concurrency(index.as_ref())
        }
        &ir::Expr::Range { ref start, ref end } => {
            expr_has_parallel_concurrency(start.as_ref())
                || expr_has_parallel_concurrency(end.as_ref())
        }
        &ir::Expr::ObjectLiteral(ref fields) => fields
            .iter()
            .any(|(_, value)| expr_has_parallel_concurrency(value)),
        &ir::Expr::StructLiteral { ref fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_has_parallel_concurrency(value)),
        &ir::Expr::ArrayLiteral(ref elements) => elements.iter().any(expr_has_parallel_concurrency),
        &ir::Expr::TupleLiteral(ref elements) => elements.iter().any(expr_has_parallel_concurrency),
        ir::Expr::Lambda(lambda) => match &lambda.body {
            ir::LambdaBody::Expr(expr) => expr_has_parallel_concurrency(expr),
            ir::LambdaBody::Block(block) => block.iter().any(stmt_has_parallel_concurrency),
        },
        &ir::Expr::Await(_) => false,
        &ir::Expr::Literal(_) | &ir::Expr::Identifier(_) | &ir::Expr::Unsupported(_) => false,
    }
}

pub fn generate(ctx: DesugarContext) -> Result<(String, String)> {
    let main_rs =
        "// Generated by livac v0.6\n\nfn main() {\n    println!(\"Hello from Liva!\");\n}\n"
            .to_string();

    let cargo_toml = generate_cargo_toml(&ctx)?;

    Ok((main_rs, cargo_toml))
}

pub fn generate_from_ir(
    module: &ir::Module,
    program: &Program,
    ctx: DesugarContext,
) -> Result<(String, String)> {
    if std::env::var("LIVA_DEBUG").is_ok() { println!("DEBUG: Checking if module has unsupported items..."); }
    if module_has_unsupported(module) {
        if std::env::var("LIVA_DEBUG").is_ok() { println!("DEBUG: Module has unsupported items, using AST generator"); }
        return generate_with_ast(program, ctx);
    }

    // For now, always use AST generator since it handles classes and inheritance correctly
    if std::env::var("LIVA_DEBUG").is_ok() { println!("DEBUG: Using AST generator for full compatibility"); }
    return generate_with_ast(program, ctx);
}

/// Generate a multi-file Rust project from multiple Liva modules
pub fn generate_multifile_project(
    modules: &[&crate::module::Module],
    entry_module: &crate::module::Module,
    ctx: DesugarContext,
) -> Result<std::collections::HashMap<std::path::PathBuf, String>> {
    use std::collections::HashMap;
    use std::path::PathBuf;
    
    let mut files = HashMap::new();
    let mut mod_declarations = Vec::new();
    
    // Generate code for each module
    for module in modules {
        let module_name = module.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("module");
        
        // Skip main/entry module - it will be handled separately
        if module.path == entry_module.path {
            continue;
        }
        
        // Generate Rust code for this module
        let rust_code = generate_module_code(module, &ctx)?;
        
        // Determine output path: src/module_name.rs
        let output_path = PathBuf::from("src").join(format!("{}.rs", module_name));
        files.insert(output_path, rust_code);
        
        // Add mod declaration
        mod_declarations.push(format!("mod {};", module_name));
    }
    
    // Generate main.rs (entry point)
    let main_code = generate_entry_point(entry_module, &mod_declarations, &ctx)?;
    files.insert(PathBuf::from("src/main.rs"), main_code);
    
    Ok(files)
}

/// Generate Rust code for a single Liva module
fn generate_module_code(module: &crate::module::Module, ctx: &DesugarContext) -> Result<String> {
    let mut codegen = CodeGenerator::new(ctx.clone());
    let mut output = String::new();
    
    // Modules need access to liva_rt from the crate root
    output.push_str("use crate::liva_rt;\n\n");
    
    // Generate use statements from imports
    for import_decl in &module.imports {
        let use_stmt = generate_use_statement(import_decl, &module.path)?;
        output.push_str(&use_stmt);
        output.push('\n');
    }
    
    if !module.imports.is_empty() {
        output.push('\n'); // Blank line after imports
    }
    
    // Generate code for each top-level item
    for item in &module.ast.items {
        match item {
            TopLevel::Import(_) => {
                // Already handled above
                continue;
            }
            TopLevel::Function(func) => {
                let is_public = !func.name.starts_with('_');
                
                // Reset codegen output for this item
                codegen.output.clear();
                codegen.generate_function(func)?;
                let func_code = codegen.output.clone();
                
                if is_public {
                    output.push_str("pub ");
                }
                output.push_str(&func_code);
                output.push('\n');
            }
            TopLevel::Class(class) => {
                let is_public = !class.name.starts_with('_');
                
                // Reset codegen output for this item
                codegen.output.clear();
                codegen.generate_class(class)?;
                let class_code = codegen.output.clone();
                
                if is_public {
                    // Add pub to struct definition
                    let lines: Vec<&str> = class_code.lines().collect();
                    if let Some(first_line) = lines.first() {
                        if first_line.starts_with("struct") {
                            output.push_str("pub ");
                        }
                    }
                }
                output.push_str(&class_code);
                output.push('\n');
            }
            TopLevel::Type(type_decl) => {
                let is_public = !type_decl.name.starts_with('_');
                
                // Reset codegen output for this item
                codegen.output.clear();
                codegen.generate_type_decl(type_decl)?;
                let type_code = codegen.output.clone();
                
                if is_public {
                    output.push_str("pub ");
                }
                output.push_str(&type_code);
                output.push('\n');
            }
            TopLevel::TypeAlias(_) => {
                // Type aliases are expanded inline, no code generation needed
                continue;
            }
            TopLevel::UseRust(_) | TopLevel::Test(_) => {
                // Reset codegen output for this item
                codegen.output.clear();
                codegen.generate_top_level(item)?;
                let code = codegen.output.clone();
                
                output.push_str(&code);
                output.push('\n');
            }
        }
    }
    
    Ok(output)
}

/// Convert a Liva import to a Rust use statement
/// Examples:
/// - `import { add } from "./math.liva"` → `use crate::math::add;`
/// - `import * as math from "./math.liva"` → `use crate::math;`
fn generate_use_statement(import_decl: &ImportDecl, _current_module_path: &std::path::Path) -> Result<String> {
    use std::path::Path;
    
    // Parse the source path and resolve relative to current module
    let source_path = Path::new(&import_decl.source);
    
    // Remove .liva extension if present
    let module_name = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            crate::CompilerError::CodegenError(crate::error::SemanticErrorInfo::new(
                "E9001",
                &format!("Invalid module path: {}", import_decl.source),
                "",
            ))
        })?;
    
    // Convert relative path to Rust module path
    let rust_module_path = if import_decl.source.starts_with("./") {
        // Same directory: ./math.liva → crate::math
        format!("crate::{}", module_name)
    } else if import_decl.source.starts_with("../") {
        // Parent directory: ../utils/math.liva → crate::utils::math
        // For now, simplify to crate::module_name
        format!("crate::{}", module_name)
    } else {
        // Absolute or other: treat as crate::module_name
        format!("crate::{}", module_name)
    };
    
    if import_decl.is_wildcard {
        // Wildcard import: import * as alias from "..."
        if let Some(alias) = &import_decl.alias {
            // Only use 'as' if alias is different from module name
            if alias != module_name {
                Ok(format!("use {} as {};", rust_module_path, alias))
            } else {
                // If alias == module_name, just import the module itself
                Ok(format!("use {};", rust_module_path))
            }
        } else {
            Ok(format!("use {}::*;", rust_module_path))
        }
    } else if import_decl.imports.len() == 1 {
        // Single import
        let symbol = &import_decl.imports[0];
        // Don't convert if it starts with uppercase (it's a type)
        let rust_symbol = if symbol.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            symbol.clone()
        } else {
            to_snake_case(symbol)
        };
        Ok(format!("use {}::{};", rust_module_path, rust_symbol))
    } else {
        // Multiple imports: use crate::math::{add, subtract};
        let rust_symbols: Vec<String> = import_decl.imports.iter()
            .map(|s| {
                // Don't convert if it starts with uppercase (it's a type)
                if s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    s.clone()
                } else {
                    to_snake_case(s)
                }
            })
            .collect();
        let symbols = rust_symbols.join(", ");
        Ok(format!("use {}::{{{}}};", rust_module_path, symbols))
    }
}

/// Generate the entry point (main.rs) with mod declarations and main function
fn generate_entry_point(
    entry_module: &crate::module::Module,
    mod_declarations: &[String],
    ctx: &DesugarContext,
) -> Result<String> {
    let mut codegen = CodeGenerator::new(ctx.clone());
    
    // Add mod declarations for all other modules
    for mod_decl in mod_declarations {
        codegen.writeln(mod_decl);
    }
    
    if !mod_declarations.is_empty() {
        codegen.output.push('\n'); // Blank line after mod declarations
    }
    
    // Generate use statements from entry module's imports
    // Skip wildcard imports that just reference the whole module (they're already available via mod)
    for import_decl in &entry_module.imports {
        if import_decl.is_wildcard && import_decl.alias.is_some() {
            // Wildcard import with alias like `import * as utils from "./utils.liva"`
            // The module is already available via `mod utils;`, skip the use statement
            continue;
        }
        
        let use_stmt = generate_use_statement(import_decl, &entry_module.path)?;
        codegen.output.push_str(&use_stmt);
        codegen.output.push('\n');
    }
    
    if !entry_module.imports.is_empty() {
        codegen.output.push('\n');
    }
    
    // Generate the entry module using generate_program logic
    codegen.generate_program(&entry_module.ast)?;
    
    Ok(codegen.output.clone())
}

pub fn generate_with_ast(program: &Program, ctx: DesugarContext) -> Result<(String, String)> {
    let mut generator = CodeGenerator::new(ctx);

    // First pass: collect fallible functions
    for item in &program.items {
        if let TopLevel::Function(func) = item {
            if func.contains_fail {
                generator.fallible_functions.insert(func.name.clone());
            }
        }
    }

    generator.generate_program(program)?;

    let cargo_toml = generate_cargo_toml(&generator.ctx)?;

    Ok((generator.output, cargo_toml))
}

pub fn generate_cargo_toml(ctx: &DesugarContext) -> Result<String> {
    let mut cargo_toml = String::from(
        "[package]\n\
         name = \"liva_project\"\n\
         version = \"0.1.0\"\n\
         edition = \"2021\"\n\n\
         [dependencies]\n",
    );

    // Always add tokio since liva_rt uses it
    cargo_toml.push_str("tokio = { version = \"1\", features = [\"full\"] }\n");

    // Add serde and serde_json (serde needed for derive macros in Phase 2)
    cargo_toml.push_str("serde = { version = \"1.0\", features = [\"derive\"] }\n");
    cargo_toml.push_str("serde_json = \"1.0\"\n");

    // Add reqwest for HTTP client
    cargo_toml.push_str("reqwest = { version = \"0.11\", default-features = false, features = [\"json\", \"rustls-tls\"] }\n");

    if ctx.has_parallel {
        cargo_toml.push_str("rayon = \"1.11\"\n");
    }

    if ctx.has_random {
        cargo_toml.push_str("rand = \"0.8\"\n");
    }

    // Add user-specified crates
    for (crate_name, _) in &ctx.rust_crates {
        if crate_name != "tokio" && crate_name != "serde_json" && crate_name != "rand" {
            writeln!(cargo_toml, "{} = \"*\"", crate_name).unwrap();
        }
    }

    Ok(cargo_toml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case() {
        let gen = CodeGenerator::new(DesugarContext {
            rust_crates: vec![],
            has_async: false,
            has_parallel: false,
            has_random: false,
            async_functions: std::collections::BTreeSet::new(),
        });

        assert_eq!(gen.to_snake_case("CamelCase"), "camel_case");
        assert_eq!(gen.to_snake_case("myFunction"), "my_function");
        assert_eq!(gen.to_snake_case("snake_case"), "snake_case");
    }
}
