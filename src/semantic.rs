use crate::ast::*;
use crate::error::{CompilerError, Result, SemanticErrorInfo, ErrorLocation};
use std::collections::{HashMap, HashSet};

pub struct SemanticAnalyzer {
    // Track which functions are async
    async_functions: HashSet<String>,
    // Track which functions are fallible (contain fail)
    fallible_functions: HashSet<String>,
    // Track defined types
    types: HashMap<String, TypeInfo>,
    // Track function signatures (arity, optional type information)
    functions: HashMap<String, FunctionSignature>,
    // Track external modules brought via `use rust`
    external_modules: HashSet<String>,
    // Current scope for variables
    current_scope: Vec<HashMap<String, Option<TypeRef>>>,
    awaitable_scopes: Vec<HashMap<String, AwaitableInfo>>,
    // Source file name for error reporting
    source_file: String,
    // Source code for line tracking
    source_code: String,
    // Source map for precise line/column tracking
    source_map: Option<crate::span::SourceMap>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TypeInfo {
    name: String,
    fields: HashMap<String, (Visibility, TypeRef)>,
    methods: HashMap<String, (Visibility, bool)>, // (visibility, is_async)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FunctionSignature {
    params: Vec<Option<TypeRef>>,
    return_type: Option<TypeRef>,
    is_async: bool,
    defaults: Vec<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AwaitableKind {
    Async,
    Task,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AwaitState {
    Pending,
    Awaited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AwaitableInfo {
    kind: AwaitableKind,
    state: AwaitState,
}

impl SemanticAnalyzer {
    fn new(source_file: String, source_code: String) -> Self {
        let source_map = if !source_code.is_empty() {
            Some(crate::span::SourceMap::new(&source_code))
        } else {
            None
        };
        
        Self {
            async_functions: HashSet::new(),
            fallible_functions: HashSet::new(),
            types: HashMap::new(),
            functions: HashMap::new(),
            external_modules: HashSet::new(),
            current_scope: vec![HashMap::new()],
            awaitable_scopes: vec![HashMap::new()],
            source_file,
            source_code,
            source_map,
        }
    }

    /// Create a semantic error with location information from a span
    fn error_with_span(&self, code: &str, title: &str, message: &str, span: Option<crate::span::Span>) -> SemanticErrorInfo {
        let mut error = SemanticErrorInfo::new(code, title, message);
        
        if let (Some(span), Some(source_map)) = (span, &self.source_map) {
            let (line, column) = span.start_position(source_map);
            let source_line = self.get_source_line(line);
            
            error = error.with_location(&self.source_file, line).with_column(column);
            
            if let Some(source_line) = source_line {
                error = error.with_source_line(source_line);
            }
        }
        
        error
    }

    fn analyze_program(&mut self, mut program: Program) -> Result<Program> {
        // First pass: collect type definitions and function signatures
        self.collect_definitions(&program)?;

        // Second pass: infer async functions
        let mut changed = true;
        while changed {
            changed = false;
            for item in &mut program.items {
                if self.infer_async(item)? {
                    changed = true;
                }
            }
        }

        // Detect fallible functions (those containing 'fail')
        self.detect_fallible_functions(&program);

        // Third pass: type checking and validation
        for item in &program.items {
            self.validate_item(item)?;
        }

        Ok(program)
    }

    fn collect_definitions(&mut self, program: &Program) -> Result<()> {
        for item in &program.items {
            match item {
                TopLevel::Function(func) => {
                    self.functions.insert(
                        func.name.clone(),
                        FunctionSignature {
                            params: func.params.iter().map(|p| p.type_ref.clone()).collect(),
                            return_type: func.return_type.clone(),
                            is_async: func.is_async_inferred,
                            defaults: func.params.iter().map(|p| p.default.is_some()).collect(),
                        },
                    );
                    if func.is_async_inferred {
                        self.async_functions.insert(func.name.clone());
                    }
                }
                TopLevel::UseRust(use_rust) => {
                    if let Some(alias) = &use_rust.alias {
                        self.external_modules.insert(alias.clone());
                    } else {
                        self.external_modules.insert(use_rust.crate_name.clone());
                    }
                }
                TopLevel::Class(class) => {
                    let mut fields = HashMap::new();
                    let mut methods = HashMap::new();

                    for member in &class.members {
                        match member {
                            Member::Field(field) => {
                                if let Some(type_ref) = &field.type_ref {
                                    fields.insert(
                                        field.name.clone(),
                                        (field.visibility, type_ref.clone()),
                                    );
                                }
                            }
                            Member::Method(method) => {
                                methods.insert(
                                    method.name.clone(),
                                    (method.visibility, method.is_async_inferred),
                                );
                            }
                        }
                    }

                    self.types.insert(
                        class.name.clone(),
                        TypeInfo {
                            name: class.name.clone(),
                            fields,
                            methods,
                        },
                    );
                }
                TopLevel::Type(type_decl) => {
                    let mut fields = HashMap::new();
                    let mut methods = HashMap::new();

                    for member in &type_decl.members {
                        match member {
                            Member::Field(field) => {
                                if let Some(type_ref) = &field.type_ref {
                                    fields.insert(
                                        field.name.clone(),
                                        (field.visibility, type_ref.clone()),
                                    );
                                }
                            }
                            Member::Method(method) => {
                                methods.insert(
                                    method.name.clone(),
                                    (method.visibility, method.is_async_inferred),
                                );
                            }
                        }
                    }

                    self.types.insert(
                        type_decl.name.clone(),
                        TypeInfo {
                            name: type_decl.name.clone(),
                            fields,
                            methods,
                        },
                    );
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn infer_async(&mut self, item: &mut TopLevel) -> Result<bool> {
        let mut mutated = false;
        match item {
            TopLevel::Function(func) => {
                let newly_async = if let Some(body) = &func.body {
                    self.contains_async_call_stmt(body)
                } else if let Some(expr) = &func.expr_body {
                    self.expr_contains_async(expr)
                } else {
                    false
                };

                if newly_async && !func.is_async_inferred {
                    func.is_async_inferred = true;
                    mutated = true;
                }

                if func.is_async_inferred {
                    self.async_functions.insert(func.name.clone());
                    if let Some(sig) = self.functions.get_mut(&func.name) {
                        if !sig.is_async {
                            sig.is_async = true;
                            mutated = true;
                        }
                    }
                }
            }
            TopLevel::Class(class) => {
                for member in &mut class.members {
                    if let Member::Method(method) = member {
                        let is_async = if let Some(body) = &method.body {
                            self.contains_async_call_stmt(body)
                        } else if let Some(expr) = &method.expr_body {
                            self.expr_contains_async(expr)
                        } else {
                            false
                        };

                        if is_async && !method.is_async_inferred {
                            method.is_async_inferred = true;
                            mutated = true;
                        }

                        if method.is_async_inferred {
                            if let Some(type_info) = self.types.get_mut(&class.name) {
                                if let Some(method_info) = type_info.methods.get_mut(&method.name) {
                                    if !method_info.1 {
                                        method_info.1 = true;
                                        mutated = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            TopLevel::Type(type_decl) => {
                for member in &mut type_decl.members {
                    if let Member::Method(method) = member {
                        let is_async = if let Some(body) = &method.body {
                            self.contains_async_call_stmt(body)
                        } else if let Some(expr) = &method.expr_body {
                            self.expr_contains_async(expr)
                        } else {
                            false
                        };

                        if is_async && !method.is_async_inferred {
                            method.is_async_inferred = true;
                            mutated = true;
                        }

                        if method.is_async_inferred {
                            if let Some(type_info) = self.types.get_mut(&type_decl.name) {
                                if let Some(method_info) = type_info.methods.get_mut(&method.name) {
                                    if !method_info.1 {
                                        method_info.1 = true;
                                        mutated = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(mutated)
    }

    fn contains_async_call(&self, body: &IfBody) -> bool {
        match body {
            IfBody::Block(block) => {
                for stmt in &block.stmts {
                    if self.stmt_contains_async(stmt) {
                        return true;
                    }
                }
                false
            }
            IfBody::Stmt(stmt) => self.stmt_contains_async(stmt),
        }
    }

    fn contains_async_call_stmt(&self, block: &BlockStmt) -> bool {
        for stmt in &block.stmts {
            if self.stmt_contains_async(stmt) {
                return true;
            }
        }
        false
    }

    fn stmt_contains_async(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::VarDecl(var) => {
                self.expr_contains_async(&var.init)
            }
            Stmt::ConstDecl(const_decl) => self.expr_contains_async(&const_decl.init),
            Stmt::Assign(assign) => {
                self.expr_contains_async(&assign.target) || self.expr_contains_async(&assign.value)
            }
            Stmt::If(if_stmt) => {
                self.expr_contains_async(&if_stmt.condition)
                    || self.contains_async_call(&if_stmt.then_branch)
                    || if_stmt
                        .else_branch
                        .as_ref()
                        .map(|b| self.contains_async_call(b))
                        .unwrap_or(false)
            }
            Stmt::While(while_stmt) => {
                self.expr_contains_async(&while_stmt.condition)
                    || self.contains_async_call_stmt(&while_stmt.body)
            }
            Stmt::For(for_stmt) => {
                self.expr_contains_async(&for_stmt.iterable)
                    || self.contains_async_call_stmt(&for_stmt.body)
            }
            Stmt::Switch(switch_stmt) => {
                self.expr_contains_async(&switch_stmt.discriminant)
                    || switch_stmt
                        .cases
                        .iter()
                        .any(|case| case.body.iter().any(|s| self.stmt_contains_async(s)))
            }
            Stmt::TryCatch(try_catch) => {
                self.contains_async_call_stmt(&try_catch.try_block)
                    || self.contains_async_call_stmt(&try_catch.catch_block)
            }
            Stmt::Throw(throw_stmt) => self.expr_contains_async(&throw_stmt.expr),
            Stmt::Fail(fail_stmt) => self.expr_contains_async(&fail_stmt.expr),
            Stmt::Return(ret) => ret
                .expr
                .as_ref()
                .map(|e| self.expr_contains_async(e))
                .unwrap_or(false),
            Stmt::Expr(expr_stmt) => self.expr_contains_async(&expr_stmt.expr),
            Stmt::Block(block) => self.contains_async_call_stmt(block),
        }
    }

    fn expr_contains_async(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                match call.exec_policy {
                    ExecPolicy::Async
                    | ExecPolicy::Par  // Par also needs await for JoinHandle
                    | ExecPolicy::TaskAsync
                    | ExecPolicy::TaskPar
                    | ExecPolicy::FireAsync
                    | ExecPolicy::FirePar => return true,
                    ExecPolicy::Normal => {}
                }

                if let Expr::Identifier(name) = call.callee.as_ref() {
                    if self.async_functions.contains(name) {
                        return true;
                    }
                }

                if self.expr_contains_async(&call.callee) {
                    return true;
                }

                call.args.iter().any(|arg| self.expr_contains_async(arg))
            }
            Expr::Lambda(lambda) => match &lambda.body {
                LambdaBody::Expr(expr) => self.expr_contains_async(expr),
                LambdaBody::Block(block) => self.contains_async_call_stmt(block),
            },
            Expr::Binary { left, right, .. } => {
                self.expr_contains_async(left) || self.expr_contains_async(right)
            }
            Expr::Unary { operand, .. } => self.expr_contains_async(operand),
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.expr_contains_async(condition)
                    || self.expr_contains_async(then_expr)
                    || self.expr_contains_async(else_expr)
            }
            Expr::Member { object, .. } => self.expr_contains_async(object),
            Expr::Index { object, index } => {
                self.expr_contains_async(object) || self.expr_contains_async(index)
            }
            Expr::ObjectLiteral(fields) => fields.iter().any(|(_, v)| self.expr_contains_async(v)),
            Expr::ArrayLiteral(elements) => elements.iter().any(|e| self.expr_contains_async(e)),
            Expr::StringTemplate { parts } => parts.iter().any(|part| match part {
                StringTemplatePart::Expr(e) => self.expr_contains_async(e),
                _ => false,
            }),
            _ => false,
        }
    }

    // ============================================
    // Fallibility detection and validation
    // ============================================

    /// Detect which functions are fallible (contain 'fail' statements)
    fn detect_fallible_functions(&mut self, program: &Program) {
        for item in &program.items {
            if let TopLevel::Function(func) = item {
                if self.function_contains_fail(&func.body, &func.expr_body) {
                    self.fallible_functions.insert(func.name.clone());
                }
            }
        }
    }

    /// Check if a function contains any 'fail' statements
    fn function_contains_fail(&self, body: &Option<BlockStmt>, expr_body: &Option<Expr>) -> bool {
        if let Some(block) = body {
            for stmt in &block.stmts {
                if self.stmt_contains_fail(stmt) {
                    return true;
                }
            }
        }
        if let Some(expr) = expr_body {
            if self.expr_contains_fail(expr) {
                return true;
            }
        }
        false
    }

    /// Recursively check if a statement contains 'fail'
    fn stmt_contains_fail(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Fail(_) => true,
            Stmt::If(if_stmt) => {
                self.if_body_contains_fail(&if_stmt.then_branch)
                    || if_stmt.else_branch.as_ref().map_or(false, |eb| self.if_body_contains_fail(eb))
            }
            Stmt::While(while_stmt) => self.stmt_list_contains_fail(&while_stmt.body.stmts),
            Stmt::For(for_stmt) => self.stmt_list_contains_fail(&for_stmt.body.stmts),
            Stmt::Return(ret) => ret.expr.as_ref().map_or(false, |e| self.expr_contains_fail(e)),
            Stmt::Expr(expr_stmt) => self.expr_contains_fail(&expr_stmt.expr),
            Stmt::VarDecl(var) => self.expr_contains_fail(&var.init),
            _ => false,
        }
    }

    fn if_body_contains_fail(&self, body: &IfBody) -> bool {
        match body {
            IfBody::Block(block_stmt) => self.stmt_list_contains_fail(&block_stmt.stmts),
            IfBody::Stmt(stmt) => self.stmt_contains_fail(stmt),
        }
    }

    fn stmt_list_contains_fail(&self, stmts: &[Stmt]) -> bool {
        stmts.iter().any(|s| self.stmt_contains_fail(s))
    }

    /// Recursively check if an expression contains calls to fallible functions
    fn expr_contains_fail(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                if let Expr::Identifier(name) = &*call.callee {
                    if self.fallible_functions.contains(name) {
                        return true;
                    }
                }
                call.args.iter().any(|arg| self.expr_contains_fail(arg))
            }
            Expr::Binary { left, right, .. } => {
                self.expr_contains_fail(left) || self.expr_contains_fail(right)
            }
            Expr::Unary { operand, .. } => self.expr_contains_fail(operand),
            Expr::StringTemplate { parts } => {
                parts.iter().any(|part| {
                    if let StringTemplatePart::Expr(e) = part {
                        self.expr_contains_fail(e)
                    } else {
                        false
                    }
                })
            }
            Expr::ArrayLiteral(elements) => elements.iter().any(|e| self.expr_contains_fail(e)),
            Expr::Index { object, index } => {
                self.expr_contains_fail(object) || self.expr_contains_fail(index)
            }
            Expr::Member { object, .. } => self.expr_contains_fail(object),
            Expr::StructLiteral { fields, .. } => {
                fields.iter().any(|(_, expr)| self.expr_contains_fail(expr))
            }
            Expr::Ternary { condition, then_expr, else_expr } => {
                self.expr_contains_fail(condition)
                    || self.expr_contains_fail(then_expr)
                    || self.expr_contains_fail(else_expr)
            }
            Expr::Fail(_) => true,
            _ => false,
        }
    }

    /// Check if an expression is a direct call to a fallible function
    fn is_expr_fallible(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                if let Expr::Identifier(name) = &*call.callee {
                    self.fallible_functions.contains(name)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Get a specific line from the source code
    fn get_source_line(&self, line_num: usize) -> Option<String> {
        self.source_code
            .lines()
            .nth(line_num.saturating_sub(1))
            .map(|s| s.to_string())
    }

    /// Find the line number where a function is called WITHOUT error binding (heuristic)
    /// Tries to find calls that don't use the pattern "let result, err = func(...)"
    fn find_line_for_function_call(&self, func_name: &str) -> Option<usize> {
        let mut first_call = None;
        
        for (line_num, line) in self.source_code.lines().enumerate() {
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }
            
            // Check if line contains the function name
            if !trimmed.contains(func_name) {
                continue;
            }
            
            // Skip if it's a function declaration (has parameter types or return type declaration)
            // Pattern: "functionName(param: type" or "functionName(...): returnType"
            if trimmed.contains(&format!("{}(", func_name)) {
                // Check if it looks like a declaration
                let after_func = trimmed.split(&format!("{}(", func_name)).nth(1).unwrap_or("");
                
                // If there's a colon before the closing paren or opening brace, it's likely a declaration
                if after_func.contains("): ") || 
                   (after_func.contains(':') && after_func.contains('{')) ||
                   (after_func.contains(':') && !after_func.contains(')')) {
                    continue;
                }
                
                // If the line starts with "function" or "fn", it's a declaration
                if trimmed.starts_with("function ") || trimmed.starts_with("fn ") {
                    continue;
                }
                
                // Check if this call has error binding (pattern: "let x, err =")
                // The pattern should have a comma in the left side of assignment before the '='
                let has_error_binding = if trimmed.starts_with("let ") {
                    // Split by '=' to get the left side
                    if let Some(left_side) = trimmed.split('=').next() {
                        // Check if the left side has a comma (indicating multiple bindings)
                        left_side.contains(',')
                    } else {
                        false
                    }
                } else {
                    false
                };
                
                // If we found a call without error binding, return it immediately
                if !has_error_binding {
                    return Some(line_num + 1);
                }
                
                // Otherwise, save it as potential fallback
                if first_call.is_none() {
                    first_call = Some(line_num + 1);
                }
            }
        }
        
        // If we didn't find a call without error binding, return the first call we found
        first_call
    }

    fn validate_item(&mut self, item: &TopLevel) -> Result<()> {
        match item {
            TopLevel::Function(func) => self.validate_function(func),
            TopLevel::Class(class) => self.validate_class(class),
            TopLevel::Type(type_decl) => self.validate_type_decl(type_decl),
            _ => Ok(()),
        }
    }

    fn validate_function(&mut self, func: &FunctionDecl) -> Result<()> {
        let type_params: HashSet<String> = func.type_params.iter().cloned().collect();

        // Check parameter types
        for param in &func.params {
            if let Some(type_ref) = &param.type_ref {
                self.validate_type_ref(type_ref, &type_params)?;
            }
        }

        // Check return type
        if let Some(return_type) = &func.return_type {
            self.validate_type_ref(return_type, &type_params)?;
        }

        // Note: Fallibility detection is now handled in lowering.rs
        // The AST is immutable, so we can't mark functions as fallible here

        self.enter_scope();

        for param in &func.params {
            if self.declare_symbol(&param.name, param.type_ref.clone()) {
                self.exit_scope()?;
                return Err(CompilerError::SemanticError(format!(
                    "Parameter '{}' defined multiple times",
                    param.name).into()));
            }
        }

        if let Some(body) = &func.body {
            self.validate_block_stmt(body)?;
        }

        if let Some(expr) = &func.expr_body {
            self.validate_expr(expr)?;
        }

        self.exit_scope()?;
        Ok(())
    }

    fn validate_class(&mut self, class: &ClassDecl) -> Result<()> {
        let empty: HashSet<String> = HashSet::new();

        for member in &class.members {
            match member {
                Member::Field(field) => {
                    if let Some(type_ref) = &field.type_ref {
                        self.validate_type_ref(type_ref, &empty)?;
                    }
                }
                Member::Method(method) => {
                    self.validate_method(method, &class.name)?;
                }
            }
        }

        // Check base class exists if specified
        if let Some(base) = &class.base {
            if !self.types.contains_key(base) {
                return Err(CompilerError::SemanticError(format!(
                    "Base class '{}' not found",
                    base).into()));
            }
        }

        Ok(())
    }

    fn validate_type_decl(&mut self, type_decl: &TypeDecl) -> Result<()> {
        let empty: HashSet<String> = HashSet::new();

        for member in &type_decl.members {
            match member {
                Member::Field(field) => {
                    if let Some(type_ref) = &field.type_ref {
                        self.validate_type_ref(type_ref, &empty)?;
                    }
                }
                Member::Method(method) => {
                    self.validate_method(method, &type_decl.name)?;
                }
            }
        }

        Ok(())
    }

    fn validate_method(&mut self, method: &MethodDecl, owner: &str) -> Result<()> {
        let type_params: HashSet<String> = method.type_params.iter().cloned().collect();

        for param in &method.params {
            if let Some(type_ref) = &param.type_ref {
                self.validate_type_ref(type_ref, &type_params)?;
            }
        }

        if let Some(return_type) = &method.return_type {
            self.validate_type_ref(return_type, &type_params)?;
        }

        self.enter_scope();

        let owner_type = TypeRef::Simple(owner.to_string());
        self.declare_symbol("self", Some(owner_type.clone()));
        self.declare_symbol("this", Some(owner_type));

        for param in &method.params {
            if self.declare_symbol(&param.name, param.type_ref.clone()) {
                self.exit_scope()?;
                return Err(CompilerError::SemanticError(format!(
                    "Parameter '{}' defined multiple times",
                    param.name).into()));
            }
        }

        if let Some(body) = &method.body {
            self.validate_block_stmt(body)?;
        }

        if let Some(expr) = &method.expr_body {
            self.validate_expr(expr)?;
        }

        self.exit_scope()?;
        Ok(())
    }

    fn validate_block(&mut self, body: &IfBody) -> Result<()> {
        self.enter_scope();
        match body {
            IfBody::Block(block) => {
                for stmt in &block.stmts {
                    self.validate_stmt(stmt)?;
                }
            }
            IfBody::Stmt(stmt) => {
                self.validate_stmt(stmt)?;
            }
        }
        self.exit_scope()?;
        Ok(())
    }

    fn validate_block_stmt(&mut self, block: &BlockStmt) -> Result<()> {
        self.enter_scope();
        for stmt in &block.stmts {
            self.validate_stmt(stmt)?;
        }
        self.exit_scope()?;
        Ok(())
    }

    fn validate_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        let empty: HashSet<String> = HashSet::new();

        match stmt {
            Stmt::VarDecl(var) => {
                // Validate the init expression
                self.validate_expr(&var.init)?;

                // Check fallibility: if init expression calls a fallible function, var must have is_fallible=true
                if !var.is_fallible && self.is_expr_fallible(&var.init) {
                    if let Expr::Call(call) = &var.init {
                        if let Expr::Identifier(func_name) = &*call.callee {
                            let line = self.find_line_for_function_call(func_name).unwrap_or(0);
                            let source_line = self.get_source_line(line);
                            
                            let error = SemanticErrorInfo {
                                location: Some(ErrorLocation {
                                    file: self.source_file.clone(),
                                    line,
                                    column: None,
                                    source_line,
                                }),
                                code: "E0701".to_string(),
                                title: "Fallible function must be called with error binding".to_string(),
                                message: format!(
                                    "Function '{}' can fail but is not being called with error binding.\n       The function contains 'fail' statements and must be handled properly.",
                                    func_name
                                ),
                                help: Some(format!("Change to: let result, err = {}(...)", func_name)),
                            };
                            
                            return Err(CompilerError::SemanticError(error));
                        }
                    }
                }

                for binding in &var.bindings {
                    if let Some(type_ref) = &binding.type_ref {
                        self.validate_type_ref(type_ref, &empty)?;
                    }

                    let declared_type = if var.is_fallible {
                        // For fallible bindings, the type is inferred from context
                        // The actual type will be Result<T, Error> but we handle this in lowering
                        binding.type_ref.clone()
                    } else {
                        binding.type_ref.clone().or_else(|| self.infer_expr_type(&var.init))
                    };

                    if self.declare_symbol(&binding.name, declared_type.clone()) {
                        let error = self.error_with_span(
                            "E0001",
                            &format!("Variable '{}' already defined in this scope", binding.name),
                            &format!("Variable '{}' already defined in this scope", binding.name),
                            binding.span
                        )
                        .with_help(&format!("Consider using a different name or removing the previous declaration of '{}'", binding.name));
                        
                        return Err(CompilerError::SemanticError(error));
                    }

                    if var.is_fallible {
                        // For fallible bindings, we don't update awaitable status in the same way
                        self.clear_awaitable(&binding.name);
                    } else {
                        self.update_awaitable_from_expr(&binding.name, &var.init)?;
                    }
                }
            }
            Stmt::ConstDecl(const_decl) => {
                if let Some(type_ref) = &const_decl.type_ref {
                    self.validate_type_ref(type_ref, &empty)?;
                }
                self.validate_expr(&const_decl.init)?;
                let inferred = const_decl
                    .type_ref
                    .clone()
                    .or_else(|| self.infer_expr_type(&const_decl.init));
                if self.declare_symbol(&const_decl.name, inferred) {
                    let error = self.error_with_span(
                        "E0002",
                        &format!("Constant '{}' already defined in this scope", const_decl.name),
                        &format!("Constant '{}' already defined in this scope", const_decl.name),
                        const_decl.span
                    )
                    .with_help(&format!("Consider using a different name or removing the previous declaration of '{}'", const_decl.name));
                    
                    return Err(CompilerError::SemanticError(error));
                }
                self.update_awaitable_from_expr(&const_decl.name, &const_decl.init)?;
            }
            Stmt::Assign(assign) => {
                self.validate_assignment_target(&assign.target)?;
                self.validate_expr(&assign.value)?;
                self.handle_assignment(&assign.target, &assign.value)?;
            }
            Stmt::If(if_stmt) => {
                self.validate_expr(&if_stmt.condition)?;
                self.validate_block(&if_stmt.then_branch)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.validate_block(else_branch)?;
                }
            }
            Stmt::While(while_stmt) => {
                self.validate_expr(&while_stmt.condition)?;
                self.validate_block_stmt(&while_stmt.body)?;
            }
            Stmt::For(for_stmt) => {
                self.validate_expr(&for_stmt.iterable)?;
                self.enter_scope();
                if self.declare_symbol(&for_stmt.var, None) {
                    self.exit_scope()?;
                    return Err(CompilerError::SemanticError(format!(
                        "Loop variable '{}' already defined",
                        for_stmt.var).into()));
                }
                self.validate_block_stmt(&for_stmt.body)?;
                let validation = self.validate_for_loop(for_stmt);
                self.exit_scope()?;
                validation?;
            }
            Stmt::Switch(switch_stmt) => {
                self.validate_expr(&switch_stmt.discriminant)?;
                for case in &switch_stmt.cases {
                    self.enter_scope();
                    for stmt in &case.body {
                        self.validate_stmt(stmt)?;
                    }
                    self.exit_scope()?;
                }
                if let Some(default) = &switch_stmt.default {
                    self.enter_scope();
                    for stmt in default {
                        self.validate_stmt(stmt)?;
                    }
                    self.exit_scope()?;
                }
            }
            Stmt::TryCatch(try_catch) => {
                self.validate_block_stmt(&try_catch.try_block)?;
                self.enter_scope();
                self.declare_symbol(&try_catch.catch_var, None);
                self.validate_block_stmt(&try_catch.catch_block)?;
                self.exit_scope()?;
            }
            Stmt::Throw(throw_stmt) => {
                self.validate_expr(&throw_stmt.expr)?;
            }
            Stmt::Fail(fail_stmt) => {
                self.validate_expr(&fail_stmt.expr)?;
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.validate_expr(expr)?;
                    self.handle_return(expr);
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.validate_expr(&expr_stmt.expr)?;
                
                // Check if expression is a fallible function call without error binding
                if self.is_expr_fallible(&expr_stmt.expr) {
                    if let Expr::Call(call) = &expr_stmt.expr {
                        if let Expr::Identifier(func_name) = &*call.callee {
                            let line = self.find_line_for_function_call(func_name).unwrap_or(0);
                            let source_line = self.get_source_line(line);
                            
                            let error = SemanticErrorInfo {
                                location: Some(ErrorLocation {
                                    file: self.source_file.clone(),
                                    line,
                                    column: None,
                                    source_line,
                                }),
                                code: "E0701".to_string(),
                                title: "Fallible function must be called with error binding".to_string(),
                                message: format!(
                                    "Function '{}' can fail but is not being called with error binding.\n       The function contains 'fail' statements and must be handled properly.",
                                    func_name
                                ),
                                help: Some(format!("Change to: let result, err = {}(...)", func_name)),
                            };
                            
                            return Err(CompilerError::SemanticError(error));
                        }
                    }
                }
                
                if let Expr::Call(call) = &expr_stmt.expr {
                    if matches!(
                        call.exec_policy,
                        ExecPolicy::TaskAsync | ExecPolicy::TaskPar
                    ) {
                        return Err(CompilerError::SemanticError(
                            "W0601: task call result is never awaited.".into(),
                        ));
                    }
                }
            }
            Stmt::Block(block) => {
                self.validate_block_stmt(block)?;
            }
        }

        Ok(())
    }

    fn validate_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(_) => Ok(()),
            Expr::Identifier(_name) => Ok(()),
            Expr::Fail(expr) => self.validate_expr(expr),
            Expr::Binary { left, right, .. } => {
                self.validate_expr(left)?;
                self.validate_expr(right)
            }
            Expr::Unary { op, operand } => {
                if *op == UnOp::Await {
                    self.validate_await_expr(operand)
                } else {
                    self.validate_expr(operand)
                }
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.validate_expr(condition)?;
                self.validate_expr(then_expr)?;
                self.validate_expr(else_expr)
            }
            Expr::Call(call) => self.validate_call_expr(call),
            Expr::Member { object, property } => {
                self.validate_expr(object)?;
                if property == "length" && !self.expr_supports_length(object) {
                    return Err(CompilerError::SemanticError(
                        "E0701: `.length` is only available on strings, bytes, and arrays. Consider `.count()` for iterables."
                            .into(),
                    ));
                }
                Ok(())
            }
            Expr::Index { object, index } => {
                self.validate_expr(object)?;
                self.validate_expr(index)
            }
            Expr::ObjectLiteral(fields) => {
                for (_, value) in fields {
                    self.validate_expr(value)?;
                }
                Ok(())
            }
            Expr::ArrayLiteral(elements) => {
                for elem in elements {
                    self.validate_expr(elem)?;
                }
                Ok(())
            }
            Expr::StringTemplate { parts } => {
                for part in parts {
                    if let StringTemplatePart::Expr(expr) = part {
                        self.validate_expr(expr)?;
                    }
                }
                Ok(())
            }
            Expr::StructLiteral { type_name, fields } => {
                // TODO: Validate that type_name exists and is a struct/class
                // TODO: Validate that fields match the struct definition
                for (_, value) in fields {
                    self.validate_expr(value)?;
                }
                Ok(())
            }
            Expr::Lambda(lambda) => self.validate_lambda(lambda),
        }
    }

    fn validate_call_expr(&mut self, call: &CallExpr) -> Result<()> {
        if let Some((first, second)) = Self::extract_modifier_chain(&call.callee) {
            return Err(CompilerError::SemanticError(format!(
                "E0602: duplicate execution modifiers '{}' and '{}' on the same call",
                first, second).into()));
        }

        // E0401: Check for invalid concurrent execution combinations
        match call.exec_policy {
            ExecPolicy::Async | ExecPolicy::TaskAsync => {
                // Check if async call is used in a context that doesn't support async
                // For now, this is a placeholder - in a full implementation we'd track execution context
                // TODO: Implement proper async context validation
            }
            ExecPolicy::Par | ExecPolicy::TaskPar | ExecPolicy::FirePar => {
                // Check if parallel call is used in a context that doesn't support parallelism
                // For now, this is a placeholder - in a full implementation we'd track execution context
                // TODO: Implement proper parallel context validation
            }
            _ => {}
        }

        // E0402: Check for unsafe concurrent access patterns
        // This would detect patterns like accessing shared mutable state from parallel contexts
        // For now, this is a placeholder implementation
        // TODO: Implement proper shared state access validation
        if matches!(call.exec_policy, ExecPolicy::Par | ExecPolicy::TaskPar | ExecPolicy::FirePar) {
            // Placeholder: In a full implementation, we'd check if the call accesses shared mutable state
            // For now, we'll just note that this is where the check would go
        }

        // W0403: Warn about potentially inefficient concurrency patterns
        // This would detect patterns like spawning too many tasks or using parallel execution for trivial operations
        // For now, this is a placeholder implementation
        // TODO: Implement proper efficiency analysis
        if matches!(call.exec_policy, ExecPolicy::Par | ExecPolicy::TaskPar | ExecPolicy::FirePar) {
            // Placeholder: In a full implementation, we'd analyze the complexity of the operation
            // and warn if parallel execution might be inefficient
            // For now, we'll just note that this is where the check would go
        }

        self.validate_call(&call.callee, &call.args)?;
        Ok(())
    }

    fn validate_await_expr(&mut self, operand: &Expr) -> Result<()> {
        self.validate_expr(operand)?;

        match operand {
            Expr::Identifier(name) => self.mark_identifier_awaited(name),
            Expr::Call(call) => {
                if self.classify_call_awaitable(call).is_some() {
                    return Ok(());
                }

                match call.exec_policy {
                    ExecPolicy::FireAsync | ExecPolicy::FirePar => {
                        let policy =
                            Self::policy_name(call.exec_policy.clone()).unwrap_or("fire call");
                        Err(CompilerError::SemanticError(format!(
                            "E0603: cannot await call using '{}' policy.",
                            policy).into()))
                    }
                    ExecPolicy::Par => Err(CompilerError::SemanticError(
                        "E0603: `par` calls complete eagerly and cannot be awaited.".into(),
                    )),
                    ExecPolicy::Normal => Ok(()),
                    _ => Err(CompilerError::SemanticError(
                        "E0603: expression is not awaitable.".into(),
                    )),
                }
            }
            Expr::Unary {
                op: UnOp::Await, ..
            } => Err(CompilerError::SemanticError(
                "E0604: expression awaited more than once.".into(),
            )),
            Expr::Literal(_) | Expr::StringTemplate { .. } => Err(CompilerError::SemanticError(
                "E0603: cannot await a literal value.".into(),
            )),
            _ => Ok(()),
        }
    }

    fn extract_modifier_chain(expr: &Expr) -> Option<(&'static str, &'static str)> {
        if let Expr::Call(inner) = expr {
            if let Expr::Call(outer) = inner.callee.as_ref() {
                let first = Self::policy_name(outer.exec_policy.clone())?;
                let second = Self::policy_name(inner.exec_policy.clone())?;
                return Some((first, second));
            }
        }
        None
    }

    fn policy_name(policy: ExecPolicy) -> Option<&'static str> {
        match policy {
            ExecPolicy::Async => Some("async"),
            ExecPolicy::Par => Some("par"),
            ExecPolicy::TaskAsync => Some("task async"),
            ExecPolicy::TaskPar => Some("task par"),
            ExecPolicy::FireAsync => Some("fire async"),
            ExecPolicy::FirePar => Some("fire par"),
            ExecPolicy::Normal => None,
        }
    }

    fn validate_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<()> {
        match callee {
            Expr::Identifier(name) => {
                if name == "len" {
                    return Err(CompilerError::SemanticError(
                        "W0700: `len(expr)` is deprecated. Use `expr.length` instead.".into(),
                    ));
                }
                if self.lookup_symbol(name).is_none() {
                    self.validate_known_function(name, args.len())?;
                }
            }
            _ => {
                self.validate_expr(callee)?;
            }
        }

        for arg in args {
            self.validate_expr(arg)?;
        }

        Ok(())
    }

    fn validate_lambda(&mut self, lambda: &LambdaExpr) -> Result<()> {
        if let Some(ret_type) = &lambda.return_type {
            let empty: HashSet<String> = HashSet::new();
            self.validate_type_ref(ret_type, &empty)?;
        }

        // E0510: Check for non-Send captures in move lambdas used in parallel contexts
        if lambda.is_move && !lambda.captures.is_empty() {
            // For now, we'll emit a warning for any captures in move lambdas
            // In a full implementation, we'd check if the captured variables are Send
            // This is a placeholder implementation
            for capture in &lambda.captures {
                // Check if this lambda is used in a parallel context
                // For now, we'll emit a warning for any move lambda with captures
                // TODO: Implement proper Send trait checking
                println!("Warning: E0510: Move lambda captures '{}' which may not be Send-safe for parallel execution", capture);
            }
        }

        // E0511: Check for non-Sync captures in lambdas used in parallel contexts
        if !lambda.captures.is_empty() {
            // For now, we'll emit a warning for any captures in lambdas that might be used in parallel contexts
            // In a full implementation, we'd check if the captured variables are Sync
            // This is a placeholder implementation
            for capture in &lambda.captures {
                // TODO: Implement proper Sync trait checking and context detection
                println!("Warning: E0511: Lambda captures '{}' which may not be Sync-safe for parallel execution", capture);
            }
        }

        self.enter_scope();

        for param in &lambda.params {
            if let Some(type_ref) = &param.type_ref {
                let empty: HashSet<String> = HashSet::new();
                self.validate_type_ref(type_ref, &empty)?;
            }

            if self.declare_symbol(&param.name, param.type_ref.clone()) {
                self.exit_scope()?;
                return Err(CompilerError::SemanticError(format!(
                    "Parameter '{}' defined multiple times",
                    param.name).into()));
            }
        }

        let result = match &lambda.body {
            LambdaBody::Expr(expr) => self.validate_expr(expr),
            LambdaBody::Block(block) => {
                self.validate_block_stmt(block)?;
                Ok(())
            }
        };

        self.exit_scope()?;
        result
    }

    fn validate_for_loop(&self, for_stmt: &ForStmt) -> Result<()> {
        self.validate_for_loop_options(for_stmt.policy.clone(), &for_stmt.options)?;

        if matches!(
            for_stmt.policy,
            DataParallelPolicy::Par | DataParallelPolicy::ParVec
        ) && Self::block_contains_await_stmt(&for_stmt.body)
        {
            return Err(CompilerError::SemanticError(
                "E0605: `await` is not allowed inside `for par` or `for parvec` loops.".into(),
            ));
        }

        Ok(())
    }

    fn validate_for_loop_options(
        &self,
        policy: DataParallelPolicy,
        options: &ForPolicyOptions,
    ) -> Result<()> {
        if let Some(chunk) = options.chunk {
            if chunk <= 0 {
                return Err(CompilerError::SemanticError(
                    "E0702: `chunk` option must be a positive integer.".into(),
                ));
            }
        }

        if let Some(prefetch) = options.prefetch {
            if prefetch <= 0 {
                return Err(CompilerError::SemanticError(
                    "E0703: `prefetch` option must be a positive integer.".into(),
                ));
            }
        }

        if let Some(thread_option) = &options.threads {
            if let ThreadOption::Count(count) = thread_option {
                if *count <= 0 {
                    return Err(CompilerError::SemanticError(
                        "E0704: `threads` option must be a positive integer when specified.".into(),
                    ));
                }
            }
        }

        if let Some(simd) = &options.simd_width {
            if !matches!(policy, DataParallelPolicy::Vec | DataParallelPolicy::ParVec) {
                return Err(CompilerError::SemanticError(
                    "E0705: `simdWidth` option requires `for vec` or `for parvec` policy.".into(),
                ));
            }

            if let SimdWidthOption::Width(width) = simd {
                if *width <= 0 {
                    return Err(CompilerError::SemanticError(
                        "E0706: `simdWidth` value must be a positive integer.".into(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn block_contains_await(body: &IfBody) -> bool {
        match body {
            IfBody::Block(block) => block
                .stmts
                .iter()
                .any(|stmt| Self::stmt_contains_await(stmt)),
            IfBody::Stmt(stmt) => Self::stmt_contains_await(stmt),
        }
    }

    fn block_contains_await_stmt(block: &BlockStmt) -> bool {
        block
            .stmts
            .iter()
            .any(|stmt| Self::stmt_contains_await(stmt))
    }

    fn stmt_contains_await(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::VarDecl(var) => Self::expr_contains_await(&var.init),
            Stmt::ConstDecl(const_decl) => Self::expr_contains_await(&const_decl.init),
            Stmt::Assign(assign) => {
                Self::expr_contains_await(&assign.target)
                    || Self::expr_contains_await(&assign.value)
            }
            Stmt::If(if_stmt) => {
                Self::expr_contains_await(&if_stmt.condition)
                    || Self::block_contains_await(&if_stmt.then_branch)
                    || if_stmt
                        .else_branch
                        .as_ref()
                        .map_or(false, |block| Self::block_contains_await(block))
            }
            Stmt::While(while_stmt) => {
                Self::expr_contains_await(&while_stmt.condition)
                    || Self::block_contains_await_stmt(&while_stmt.body)
            }
            Stmt::For(for_stmt) => {
                Self::expr_contains_await(&for_stmt.iterable)
                    || Self::block_contains_await_stmt(&for_stmt.body)
            }
            Stmt::Switch(switch_stmt) => {
                if Self::expr_contains_await(&switch_stmt.discriminant) {
                    return true;
                }
                for case in &switch_stmt.cases {
                    if case.body.iter().any(Self::stmt_contains_await) {
                        return true;
                    }
                }
                if let Some(default) = &switch_stmt.default {
                    if default.iter().any(Self::stmt_contains_await) {
                        return true;
                    }
                }
                false
            }
            Stmt::TryCatch(try_catch) => {
                Self::block_contains_await_stmt(&try_catch.try_block)
                    || Self::block_contains_await_stmt(&try_catch.catch_block)
            }
            Stmt::Throw(throw_stmt) => Self::expr_contains_await(&throw_stmt.expr),
            Stmt::Fail(fail_stmt) => Self::expr_contains_await(&fail_stmt.expr),
            Stmt::Return(ret) => ret
                .expr
                .as_ref()
                .map_or(false, |expr| Self::expr_contains_await(expr)),
            Stmt::Expr(expr_stmt) => Self::expr_contains_await(&expr_stmt.expr),
            Stmt::Block(block) => Self::block_contains_await_stmt(block),
        }
    }

    fn expr_contains_await(expr: &Expr) -> bool {
        match expr {
            Expr::Unary {
                op: UnOp::Await, ..
            } => true,
            Expr::Unary { operand, .. } => Self::expr_contains_await(operand),
            Expr::Binary { left, right, .. } => {
                Self::expr_contains_await(left) || Self::expr_contains_await(right)
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                Self::expr_contains_await(condition)
                    || Self::expr_contains_await(then_expr)
                    || Self::expr_contains_await(else_expr)
            }
            Expr::Call(call) => {
                Self::expr_contains_await(&call.callee)
                    || call.args.iter().any(Self::expr_contains_await)
            }
            Expr::Member { object, .. } => Self::expr_contains_await(object),
            Expr::Index { object, index } => {
                Self::expr_contains_await(object) || Self::expr_contains_await(index)
            }
            Expr::ObjectLiteral(fields) => fields
                .iter()
                .any(|(_, value)| Self::expr_contains_await(value)),
            Expr::StructLiteral { fields, .. } => fields
                .iter()
                .any(|(_, value)| Self::expr_contains_await(value)),
            Expr::ArrayLiteral(elements) => elements
                .iter()
                .any(|value| Self::expr_contains_await(value)),
            Expr::Lambda(lambda) => match &lambda.body {
                LambdaBody::Expr(body) => Self::expr_contains_await(body),
                LambdaBody::Block(block) => Self::block_contains_await_stmt(block),
            },
            Expr::StringTemplate { parts } => parts.iter().any(|part| match part {
                StringTemplatePart::Expr(expr) => Self::expr_contains_await(expr),
                _ => false,
            }),
            Expr::Literal(_) | Expr::Identifier(_) => false,
            Expr::Fail(expr) => Self::expr_contains_await(expr),
        }
    }

    fn validate_known_function(&self, name: &str, arity: usize) -> Result<()> {
        if let Some(signature) = self.functions.get(name) {
            let total = signature.params.len();
            let optional = signature
                .defaults
                .iter()
                .filter(|is_default| **is_default)
                .count();
            let required = total.saturating_sub(optional);

            if arity < required || arity > total {
                return Err(CompilerError::SemanticError(format!(
                    "Function '{}' expects between {} and {} arguments but {} were provided",
                    name, required, total, arity).into()));
            }
        }

        Ok(())
    }

    fn validate_assignment_target(&mut self, target: &Expr) -> Result<()> {
        match target {
            Expr::Identifier(name) => {
                if self.lookup_symbol(name).is_none() {
                    return Err(CompilerError::SemanticError(format!(
                        "Cannot assign to undefined variable '{}'",
                        name).into()));
                }
            }
            Expr::Member { object, .. } => {
                self.validate_expr(object)?;
            }
            Expr::Index { object, index } => {
                self.validate_expr(object)?;
                self.validate_expr(index)?;
            }
            _ => {
                return Err(CompilerError::SemanticError(
                    "Invalid assignment target".into(),
                ));
            }
        }
        Ok(())
    }

    fn enter_scope(&mut self) {
        self.current_scope.push(HashMap::new());
        self.awaitable_scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) -> Result<()> {
        let awaitables = self.awaitable_scopes.pop().unwrap_or_default();
        let mut unawaited_task: Option<String> = None;

        for (name, info) in awaitables.into_iter() {
            if info.kind == AwaitableKind::Task && info.state == AwaitState::Pending {
                unawaited_task = Some(name);
                break;
            }
        }

        self.current_scope.pop();

        if let Some(name) = unawaited_task {
            return Err(CompilerError::SemanticError(format!(
                "W0601: task handle '{}' is never awaited.",
                name).into()));
        }

        Ok(())
    }

    fn declare_symbol(&mut self, name: &str, ty: Option<TypeRef>) -> bool {
        if let Some(scope) = self.current_scope.last_mut() {
            let existed = scope.contains_key(name);
            scope.insert(name.to_string(), ty);
            existed
        } else {
            false
        }
    }

    fn lookup_symbol(&self, name: &str) -> Option<&Option<TypeRef>> {
        for scope in self.current_scope.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry);
            }
        }
        None
    }

    fn find_symbol_scope(&self, name: &str) -> Option<usize> {
        for index in (0..self.current_scope.len()).rev() {
            if self.current_scope[index].contains_key(name) {
                return Some(index);
            }
        }
        None
    }

    fn set_awaitable(&mut self, name: &str, info: AwaitableInfo) {
        if let Some(index) = self.find_symbol_scope(name) {
            if let Some(scope) = self.awaitable_scopes.get_mut(index) {
                scope.insert(name.to_string(), info);
            }
        }
    }

    fn clear_awaitable(&mut self, name: &str) {
        if let Some(index) = self.find_symbol_scope(name) {
            if let Some(scope) = self.awaitable_scopes.get_mut(index) {
                scope.remove(name);
            }
        }
    }

    fn move_awaitable(&mut self, source: &str, target: &str) -> bool {
        if let Some(index) = self.find_symbol_scope(source) {
            if let Some(info) = self.awaitable_scopes[index].remove(source) {
                self.set_awaitable(target, info);
                return true;
            }
        }
        false
    }

    fn classify_call_awaitable(&self, call: &CallExpr) -> Option<AwaitableKind> {
        match call.exec_policy {
            ExecPolicy::Async => Some(AwaitableKind::Async),
            ExecPolicy::TaskAsync | ExecPolicy::TaskPar => Some(AwaitableKind::Task),
            ExecPolicy::Normal => {
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    if self.async_functions.contains(name) {
                        Some(AwaitableKind::Async)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn classify_awaitable_expr(&self, expr: &Expr) -> Option<AwaitableKind> {
        if let Expr::Call(call) = expr {
            self.classify_call_awaitable(call)
        } else {
            None
        }
    }

    fn update_awaitable_from_expr(&mut self, name: &str, expr: &Expr) -> Result<()> {
        if let Expr::Identifier(source) = expr {
            if self.move_awaitable(source, name) {
                return Ok(());
            }
        }

        if let Some(kind) = self.classify_awaitable_expr(expr) {
            self.set_awaitable(
                name,
                AwaitableInfo {
                    kind,
                    state: AwaitState::Pending,
                },
            );
        } else {
            self.clear_awaitable(name);
        }

        Ok(())
    }

    fn mark_identifier_awaited(&mut self, name: &str) -> Result<()> {
        if let Some(index) = self.find_symbol_scope(name) {
            if let Some(info) = self.awaitable_scopes[index].get_mut(name) {
                if info.state == AwaitState::Pending {
                    info.state = AwaitState::Awaited;
                    return Ok(());
                }
                return Err(CompilerError::SemanticError(format!(
                    "E0604: handle '{}' awaited more than once.",
                    name).into()));
            }
            return Err(CompilerError::SemanticError(format!(
                "E0603: expression '{}' is not awaitable.",
                name).into()));
        }

        Err(CompilerError::SemanticError(format!(
            "E0603: expression '{}' is not awaitable.",
            name).into()))
    }

    fn handle_assignment(&mut self, target: &Expr, value: &Expr) -> Result<()> {
        if let Expr::Identifier(name) = target {
            self.update_awaitable_from_expr(name, value)?;
        }
        Ok(())
    }

    fn handle_return(&mut self, expr: &Expr) {
        if let Expr::Identifier(name) = expr {
            self.clear_awaitable(name);
        }
    }

    fn infer_expr_type(&self, expr: &Expr) -> Option<TypeRef> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::String(_) => Some(TypeRef::Simple("string".into())),
                Literal::Int(_) => Some(TypeRef::Simple("number".into())),
                Literal::Float(_) => Some(TypeRef::Simple("float".into())),
                Literal::Bool(_) => Some(TypeRef::Simple("bool".into())),
                Literal::Char(_) => Some(TypeRef::Simple("char".into())),
            },
            Expr::StringTemplate { .. } => Some(TypeRef::Simple("string".into())),
            Expr::ArrayLiteral(elements) => {
                let inner = elements
                    .first()
                    .and_then(|expr| self.infer_expr_type(expr))
                    .unwrap_or_else(|| TypeRef::Simple("unknown".into()));
                Some(TypeRef::Array(Box::new(inner)))
            }
            Expr::Identifier(name) => self.lookup_symbol(name).cloned().flatten(),
            Expr::Member { object, property } => {
                if property == "length" {
                    return Some(TypeRef::Simple("number".into()));
                }

                let base_type = self.infer_expr_type(object)?;
                let base_type = Self::strip_optional(base_type);
                if let TypeRef::Simple(type_name) = base_type {
                    if let Some(info) = self.types.get(&type_name) {
                        if let Some((_, field_ty)) = info.fields.get(property) {
                            return Some(field_ty.clone());
                        }
                    }
                }
                None
            }
            Expr::Index { object, .. } => {
                let base = self.infer_expr_type(object)?;
                if let TypeRef::Array(inner) = Self::strip_optional(base) {
                    Some(*inner)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn strip_optional(ty: TypeRef) -> TypeRef {
        match ty {
            TypeRef::Optional(inner) => Self::strip_optional(*inner),
            other => other,
        }
    }

    fn expr_supports_length(&self, object: &Expr) -> bool {
        match object {
            Expr::ArrayLiteral(_) => true,
            Expr::Literal(Literal::String(_)) => true,
            Expr::StringTemplate { .. } => true,
            _ => self
                .infer_expr_type(object)
                .map(|ty| self.type_supports_length(&Self::strip_optional(ty)))
                .unwrap_or(false),
        }
    }

    fn type_supports_length(&self, ty: &TypeRef) -> bool {
        match ty {
            TypeRef::Array(_) => true,
            TypeRef::Simple(name) => matches!(
                name.as_str(),
                "string" | "bytes" | "Vec" | "Array" | "String"
            ),
            TypeRef::Generic { base, .. } => matches!(base.as_str(), "Vec" | "Array"),
            TypeRef::Optional(inner) => self.type_supports_length(inner),
            TypeRef::Fallible(_) => false,
        }
    }

    fn validate_type_ref(
        &self,
        type_ref: &TypeRef,
        available_type_params: &std::collections::HashSet<String>,
    ) -> Result<()> {
        match type_ref {
            TypeRef::Simple(_name) => Ok(()),
            TypeRef::Generic { args, .. } => {
                for arg in args {
                    self.validate_type_ref(arg, available_type_params)?;
                }
                Ok(())
            }
            TypeRef::Array(inner) => self.validate_type_ref(inner, available_type_params),
            TypeRef::Optional(inner) => self.validate_type_ref(inner, available_type_params),
            TypeRef::Fallible(inner) => self.validate_type_ref(inner, available_type_params),
        }
    }
}

pub fn analyze(program: Program) -> Result<Program> {
    let mut analyzer = SemanticAnalyzer::new(String::new(), String::new());
    analyzer.analyze_program(program)
}

pub fn analyze_with_source(program: Program, source_file: String, source_code: String) -> Result<Program> {
    let mut analyzer = SemanticAnalyzer::new(source_file, source_code);
    analyzer.analyze_program(program)
}

