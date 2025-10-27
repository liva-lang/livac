use crate::ast::*;
use crate::error::{CompilerError, ErrorLocation, Result, SemanticErrorInfo};
use crate::suggestions;
use crate::traits::TraitRegistry;
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
    // Imported symbols: map from module path to (public_symbols, private_symbols)
    imported_modules: HashMap<std::path::PathBuf, (HashSet<String>, HashSet<String>)>,
    // Imported symbol names in current module (for collision detection)
    imported_symbols: HashSet<String>,
    // Track if we're currently in an error binding context (allows fallible calls)
    in_error_binding: bool,
    // Track type parameters in current scope (for generics)
    type_parameters: Vec<HashSet<String>>,
    // Track type parameter constraints (T -> [Add, Sub, ...])
    type_constraints: Vec<HashMap<String, Vec<String>>>,
    // Trait registry for constraint validation
    trait_registry: TraitRegistry,
    // Track classes used with JSON.parse (need serde derive) - Phase 2
    json_classes: HashSet<String>,
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
            imported_modules: HashMap::new(),
            imported_symbols: HashSet::new(),
            in_error_binding: false,
            type_parameters: vec![HashSet::new()],
            type_constraints: vec![HashMap::new()],
            trait_registry: TraitRegistry::new(),
            json_classes: HashSet::new(),
        }
    }

    /// Create a semantic error with location information from a span
    fn error_with_span(
        &self,
        code: &str,
        title: &str,
        message: &str,
        span: Option<crate::span::Span>,
    ) -> SemanticErrorInfo {
        let mut error = SemanticErrorInfo::new(code, title, message);

        if let (Some(span), Some(source_map)) = (span, &self.source_map) {
            let (line, column) = span.start_position(source_map);
            let source_line = self.get_source_line(line);
            let token_length = span.len();

            // Get context lines (2 before and 2 after)
            let (context_before, context_after) = self.get_context_lines(line, 2);

            error = error
                .with_location(&self.source_file, line)
                .with_column(column)
                .with_length(token_length)
                .with_context(context_before, context_after);

            if let Some(source_line) = source_line {
                error = error.with_source_line(source_line);
            }
        }

        error
    }

    fn analyze_program(&mut self, mut program: Program) -> Result<Program> {
        // Phase 0: Validate imports if module context is available
        if !self.imported_modules.is_empty() {
            self.validate_imports(&program)?;
        }
        
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

        // Fourth pass: Mark classes that need serde (Phase 2: JSON Typed Parsing)
        self.mark_json_classes(&mut program);

        Ok(program)
    }

    /// Validate all import statements in the program
    fn validate_imports(&mut self, program: &Program) -> Result<()> {
        use crate::ast::TopLevel;
        
        for item in &program.items {
            if let TopLevel::Import(import) = item {
                self.validate_import(import)?;
            }
        }
        
        Ok(())
    }
    
    /// Validate a single import declaration
    fn validate_import(&mut self, import: &crate::ast::ImportDecl) -> Result<()> {
        use std::path::Path;
        
        // Resolve the import path relative to the current file
        let current_file = Path::new(&self.source_file);
        let current_dir = current_file.parent().unwrap_or_else(|| Path::new("."));
        let import_path = current_dir.join(&import.source);
        
        // Canonicalize to match how modules are stored
        let canonical_path = import_path.canonicalize().ok();
        
        // Try to find the module by matching against all known modules
        let module_info = canonical_path
            .as_ref()
            .and_then(|p| self.imported_modules.get(p))
            .or_else(|| {
                // Fallback: try to find by comparing file names
                self.imported_modules.iter()
                    .find(|(path, _)| {
                        path.file_name() == import_path.file_name()
                    })
                    .map(|(_, info)| info)
            });
        
        // Check if we have information about this module
        let (public_symbols, private_symbols) = module_info
            .ok_or_else(|| {
                CompilerError::SemanticError(SemanticErrorInfo::new(
                    "E4004",
                    "Cannot find module",
                    &format!("Module not found: {}\nHint: Make sure the module file exists in the same directory or provide the correct relative path.", import.source),
                ))
            })?;
        
        if import.is_wildcard {
            // Wildcard import: import * as name
            if let Some(alias) = &import.alias {
                // All public symbols are available via alias.symbol
                // We'll handle this in expression validation
                // For now, just record that we have this namespace
                self.imported_symbols.insert(alias.clone());
            }
        } else {
            // Named imports: validate each symbol
            for symbol in &import.imports {
                // Check if symbol exists in module
                if !public_symbols.contains(symbol) && !private_symbols.contains(symbol) {
                    // Generate suggestion for similar symbol names
                    let all_symbols: Vec<String> = public_symbols
                        .iter()
                        .chain(private_symbols.iter())
                        .cloned()
                        .collect();
                    let suggestion = suggestions::find_suggestion(symbol, &all_symbols, 2);
                    
                    let message = format!(
                        "Symbol '{}' not found in module '{}'.",
                        symbol, import.source
                    );
                    
                    let mut error = SemanticErrorInfo::new(
                        "E4006",
                        "Imported symbol not found",
                        &message,
                    );
                    
                    if let Some(suggested) = suggestion {
                        error = error.with_suggestion(&format!("Did you mean '{}'?", suggested));
                    } else {
                        error = error.with_hint("Check the spelling and make sure the symbol is defined in the module.");
                    }
                    
                    return Err(CompilerError::SemanticError(error));
                }
                
                // Check if symbol is private (starts with _)
                if private_symbols.contains(symbol) {
                    return Err(CompilerError::SemanticError(
                        SemanticErrorInfo::new(
                            "E4007",
                            "Cannot import private symbol",
                            &format!(
                                "Symbol '{}' is private (starts with '_') and cannot be imported from '{}'.\nHint: Only symbols without '_' prefix can be imported. Either remove the import or make the symbol public by removing the '_' prefix.",
                                symbol, import.source
                            ),
                        )
                    ));
                }
                
                // Check for name collision with existing symbols
                if self.functions.contains_key(symbol) || self.types.contains_key(symbol) {
                    return Err(CompilerError::SemanticError(
                        SemanticErrorInfo::new(
                            "E4008",
                            "Import conflicts with local definition",
                            &format!(
                                "Cannot import '{}': a {} with this name is already defined in this module.\nHint: Use an alias for the import: 'import {{ {} as new_name }} from \"{}\"'",
                                symbol,
                                if self.functions.contains_key(symbol) { "function" } else { "type" },
                                symbol,
                                import.source
                            ),
                        )
                    ));
                }
                
                // Check for collision with another import
                if self.imported_symbols.contains(symbol) {
                    return Err(CompilerError::SemanticError(
                        SemanticErrorInfo::new(
                            "E4009",
                            "Import conflicts with another import",
                            &format!(
                                "Symbol '{}' is imported multiple times.\nHint: Use aliases to distinguish between them: 'import {{ {} as name1 }} from \"module1\"' and 'import {{ {} as name2 }} from \"module2\"'",
                                symbol, symbol, symbol
                            ),
                        )
                    ));
                }
                
                // Record this symbol as imported
                self.imported_symbols.insert(symbol.clone());
                
                // Add to function registry so it can be called
                // (We don't know the signature, so we'll be permissive)
                self.functions.insert(
                    symbol.clone(),
                    FunctionSignature {
                        params: vec![],  // Unknown params
                        return_type: None,  // Unknown return type
                        is_async: false,  // Assume sync
                        defaults: vec![],
                    },
                );
            }
        }
        
        Ok(())
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
            Stmt::VarDecl(var) => self.expr_contains_async(&var.init),
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
                    || if_stmt
                        .else_branch
                        .as_ref()
                        .map_or(false, |eb| self.if_body_contains_fail(eb))
            }
            Stmt::While(while_stmt) => self.stmt_list_contains_fail(&while_stmt.body.stmts),
            Stmt::For(for_stmt) => self.stmt_list_contains_fail(&for_stmt.body.stmts),
            Stmt::Return(ret) => ret
                .expr
                .as_ref()
                .map_or(false, |e| self.expr_contains_fail(e)),
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
            Expr::StringTemplate { parts } => parts.iter().any(|part| {
                if let StringTemplatePart::Expr(e) = part {
                    self.expr_contains_fail(e)
                } else {
                    false
                }
            }),
            Expr::ArrayLiteral(elements) => elements.iter().any(|e| self.expr_contains_fail(e)),
            Expr::Index { object, index } => {
                self.expr_contains_fail(object) || self.expr_contains_fail(index)
            }
            Expr::Member { object, .. } => self.expr_contains_fail(object),
            Expr::StructLiteral { fields, .. } => {
                fields.iter().any(|(_, expr)| self.expr_contains_fail(expr))
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.expr_contains_fail(condition)
                    || self.expr_contains_fail(then_expr)
                    || self.expr_contains_fail(else_expr)
            }
            Expr::Fail(_) => true,
            _ => false,
        }
    }

    /// Check if an expression is a direct call to a fallible function
    /// Get a specific line from the source code
    fn get_source_line(&self, line_num: usize) -> Option<String> {
        self.source_code
            .lines()
            .nth(line_num.saturating_sub(1))
            .map(|s| s.to_string())
    }

    /// Get context lines before and after a specific line
    fn get_context_lines(&self, line_num: usize, context_size: usize) -> (Vec<String>, Vec<String>) {
        let lines: Vec<&str> = self.source_code.lines().collect();
        let total_lines = lines.len();

        // Lines before (up to context_size)
        let start_before = line_num.saturating_sub(context_size + 1);
        let end_before = line_num.saturating_sub(1);
        let before: Vec<String> = if start_before < end_before && end_before <= total_lines {
            lines[start_before..end_before]
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        // Lines after (up to context_size)
        let start_after = line_num; // line_num is 1-indexed, array is 0-indexed
        let end_after = (line_num + context_size).min(total_lines);
        let after: Vec<String> = if start_after < end_after {
            lines[start_after..end_after]
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        (before, after)
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

            // Skip if it's a function declaration
            // In Liva, function declarations look like: "funcname(params) {" at the start of a line
            // or "funcname(params): returnType {" for typed functions
            if trimmed.contains(&format!("{}(", func_name)) {
                // Check if it looks like a declaration (line starts with function name)
                if trimmed.starts_with(&format!("{}(", func_name)) {
                    // This is a function declaration, skip it
                    continue;
                }

                // Check if it looks like a declaration
                let after_func = trimmed
                    .split(&format!("{}(", func_name))
                    .nth(1)
                    .unwrap_or("");

                // If there's a colon before the closing paren or opening brace, it's likely a declaration
                if after_func.contains("): ")
                    || (after_func.contains(':') && after_func.contains('{'))
                    || (after_func.contains(':') && !after_func.contains(')'))
                {
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
        // Enter type parameter scope and register type parameters with constraints
        self.enter_type_param_scope();
        
        for param in &func.type_params {
            if !param.constraints.is_empty() {
                // Validate that all constraints are known traits or aliases
                for constraint in &param.constraints {
                    if !self.trait_registry.is_valid_constraint(constraint) {
                        let suggestions = self.trait_registry.all_trait_names();
                        let similar = suggestions::find_multiple_suggestions(&constraint, &suggestions, 3, 3);
                        
                        self.exit_type_param_scope();
                        return Err(CompilerError::SemanticError(
                            format!(
                                "E5001: Unknown trait constraint '{}'. {}Available traits: {}",
                                constraint,
                                if !similar.is_empty() {
                                    format!("Did you mean '{}'? ", similar.join("', '"))
                                } else {
                                    String::new()
                                },
                                suggestions.join(", ")
                            ).into(),
                        ));
                    }
                    
                    // Expand aliases to underlying traits
                    if self.trait_registry.is_alias(constraint) {
                        let underlying = self.trait_registry.expand_alias(constraint);
                        for trait_name in underlying {
                            self.declare_type_param_with_constraint(&param.name, &trait_name);
                        }
                    } else {
                        self.declare_type_param_with_constraint(&param.name, constraint);
                    }
                }
            } else {
                self.declare_type_param(&param.name);
            }
        }
        
        let type_params: HashSet<String> = func.type_params.iter().map(|tp| tp.name.clone()).collect();

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
            // Declare variables from parameter pattern
            self.declare_param_pattern(&param.pattern, param.type_ref.clone(), None)?;
        }

        if let Some(body) = &func.body {
            self.validate_block_stmt(body)?;
        }

        if let Some(expr) = &func.expr_body {
            self.validate_expr(expr)?;
        }

        self.exit_scope()?;
        self.exit_type_param_scope();
        Ok(())
    }

    fn validate_class(&mut self, class: &ClassDecl) -> Result<()> {
        // Enter type parameter scope and register class type parameters with constraints
        self.enter_type_param_scope();
        
        for param in &class.type_params {
            if !param.constraints.is_empty() {
                // Validate that all constraints are known traits or aliases
                for constraint in &param.constraints {
                    if !self.trait_registry.is_valid_constraint(constraint) {
                        let suggestions = self.trait_registry.all_trait_names();
                        let similar = suggestions::find_multiple_suggestions(&constraint, &suggestions, 3, 3);
                        
                        self.exit_type_param_scope();
                        return Err(CompilerError::SemanticError(
                            format!(
                                "E5001: Unknown trait constraint '{}'. {}Available traits: {}",
                                constraint,
                                if !similar.is_empty() {
                                    format!("Did you mean '{}'? ", similar.join("', '"))
                                } else {
                                    String::new()
                                },
                                suggestions.join(", ")
                            ).into(),
                        ));
                    }
                    
                    // Expand aliases to underlying traits
                    if self.trait_registry.is_alias(constraint) {
                        let underlying = self.trait_registry.expand_alias(constraint);
                        for trait_name in underlying {
                            self.declare_type_param_with_constraint(&param.name, &trait_name);
                        }
                    } else {
                        self.declare_type_param_with_constraint(&param.name, constraint);
                    }
                }
            } else {
                self.declare_type_param(&param.name);
            }
        }
        
        // Collect type parameters from class declaration
        let type_params: HashSet<String> = class
            .type_params
            .iter()
            .map(|tp| tp.name.clone())
            .collect();

        for member in &class.members {
            match member {
                Member::Field(field) => {
                    if let Some(type_ref) = &field.type_ref {
                        self.validate_type_ref(type_ref, &type_params)?;
                    }
                }
                Member::Method(method) => {
                    self.validate_method_with_params(method, &class.name, &type_params)?;
                }
            }
        }

        // Check base class exists if specified
        if let Some(base) = &class.base {
            if !self.types.contains_key(base) {
                // Generate suggestion for similar type names
                let available_types = self.get_all_types();
                let suggestion = suggestions::find_suggestion(base, &available_types, 2);
                
                let mut error = SemanticErrorInfo::new(
                    "E2004",
                    "Undefined base class",
                    &format!("Base class '{}' not found", base)
                );
                
                if let Some(suggested) = suggestion {
                    error = error.with_suggestion(&format!("Did you mean '{}'?", suggested));
                }
                
                self.exit_type_param_scope();
                return Err(CompilerError::SemanticError(error));
            }
        }

        self.exit_type_param_scope();
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
        let empty = HashSet::new();
        self.validate_method_with_params(method, owner, &empty)
    }

    fn validate_method_with_params(
        &mut self,
        method: &MethodDecl,
        owner: &str,
        class_type_params: &HashSet<String>,
    ) -> Result<()> {
        // Register method's own type parameters with constraints
        // Note: Class type parameters are already in scope from validate_class
        for param in &method.type_params {
            if !param.constraints.is_empty() {
                // Validate that all constraints are known traits or aliases
                for constraint in &param.constraints {
                    if !self.trait_registry.is_valid_constraint(constraint) {
                        let suggestions = self.trait_registry.all_trait_names();
                        let similar = suggestions::find_multiple_suggestions(&constraint, &suggestions, 3, 3);
                        
                        return Err(CompilerError::SemanticError(
                            format!(
                                "E5001: Unknown trait constraint '{}'. {}Available traits: {}",
                                constraint,
                                if !similar.is_empty() {
                                    format!("Did you mean '{}'? ", similar.join("', '"))
                                } else {
                                    String::new()
                                },
                                suggestions.join(", ")
                            ).into(),
                        ));
                    }
                    
                    // Expand aliases to underlying traits
                    if self.trait_registry.is_alias(constraint) {
                        let underlying = self.trait_registry.expand_alias(constraint);
                        for trait_name in underlying {
                            self.declare_type_param_with_constraint(&param.name, &trait_name);
                        }
                    } else {
                        self.declare_type_param_with_constraint(&param.name, constraint);
                    }
                }
            } else {
                self.declare_type_param(&param.name);
            }
        }
        
        // Combine class type parameters with method's own type parameters
        let mut all_type_params = class_type_params.clone();
        for tp in &method.type_params {
            all_type_params.insert(tp.name.clone());
        }

        for param in &method.params {
            if let Some(type_ref) = &param.type_ref {
                self.validate_type_ref(type_ref, &all_type_params)?;
            }
        }

        if let Some(return_type) = &method.return_type {
            self.validate_type_ref(return_type, &all_type_params)?;
        }

        self.enter_scope();

        let owner_type = TypeRef::Simple(owner.to_string());
        self.declare_symbol("self", Some(owner_type.clone()));
        self.declare_symbol("this", Some(owner_type));

        for param in &method.params {
            // Declare variables from parameter pattern
            self.declare_param_pattern(&param.pattern, param.type_ref.clone(), None)?;
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
                // Validate the init expression (will check fallibility in validate_call_expr)
                // Note: is_fallible=true means error binding pattern is used, so fallible calls are allowed
                let previous_error_binding = self.in_error_binding;
                if var.is_fallible {
                    self.in_error_binding = true;
                }
                
                // Check if this is a JSON.parse or response.json() call with type hint (Phase 1: JSON Typed Parsing)
                if let Some(type_hint) = var.bindings.first().and_then(|b| b.type_ref.as_ref()) {
                    if let Expr::MethodCall(method_call) = &var.init {
                        // Check for JSON.parse() or any .json() method
                        let is_json_parse = method_call.method == "parse" && 
                            matches!(method_call.object.as_ref(), Expr::Identifier(id) if id == "JSON");
                        let is_json_method = method_call.method == "json";
                        
                        if is_json_parse || is_json_method {
                            // This is JSON.parse or response.json() with a type hint
                            self.validate_json_parse_type_hint(type_hint)?;
                        }
                    }
                }
                
                self.validate_expr(&var.init)?;
                self.in_error_binding = previous_error_binding;

                for binding in &var.bindings {
                    if let Some(type_ref) = &binding.type_ref {
                        self.validate_type_ref(type_ref, &empty)?;
                    }

                    let declared_type = if var.is_fallible {
                        // For fallible bindings, the type is inferred from context
                        // The actual type will be Result<T, Error> but we handle this in lowering
                        binding.type_ref.clone()
                    } else {
                        binding
                            .type_ref
                            .clone()
                            .or_else(|| self.infer_expr_type(&var.init))
                    };

                    // Validate and declare the binding pattern (supports destructuring)
                    self.validate_and_declare_pattern(
                        &binding.pattern,
                        &var.init,
                        declared_type,
                        var.is_fallible,
                        binding.span
                    )?;
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
                    return Err(CompilerError::SemanticError(
                        format!("Loop variable '{}' already defined", for_stmt.var).into(),
                    ));
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

                // Fallible function call validation is now done in validate_call_expr
                // which is called from validate_expr

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
            Expr::Binary { left, right, op } => {
                self.validate_expr(left)?;
                self.validate_expr(right)?;
                
                // Check constraints for binary operators on generic types
                self.validate_binary_op_constraints(left, right, op)
            }
            Expr::Unary { op, operand } => {
                self.validate_expr(operand)?;
                
                if *op == UnOp::Await {
                    self.validate_await_expr(operand)
                } else {
                    // Check constraints for unary operators on generic types
                    self.validate_unary_op_constraints(operand, op)
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
            Expr::Tuple(elements) => {
                // Validate all tuple elements
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
            Expr::StructLiteral {
                type_name: _,
                fields,
            } => {
                // TODO: Validate that type_name exists and is a struct/class
                // TODO: Validate that fields match the struct definition
                for (_, value) in fields {
                    self.validate_expr(value)?;
                }
                Ok(())
            }
            Expr::Lambda(lambda) => self.validate_lambda(lambda),
            Expr::MethodCall(method_call) => {
                // Validate the object expression
                self.validate_expr(&method_call.object)?;
                
                // Check if this is response.json() - mark as fallible
                if method_call.method == "json" {
                    // This is a fallible method that returns (JsonValue?, Error?)
                    // Mark it so error binding validation knows it needs error handling
                    // We don't need to store it anywhere, the compiler will handle it
                }
                
                // Validate method arguments
                for arg in &method_call.args {
                    self.validate_expr(arg)?;
                }
                
                // TODO: Phase 2 - validate method exists for the object type
                // TODO: Phase 2 - validate adapter usage (par, vec, parvec)
                Ok(())
            }
            Expr::Switch(switch_expr) => {
                self.validate_expr(&switch_expr.discriminant)?;
                
                // Validate all arms
                for arm in &switch_expr.arms {
                    // Validate guard if present
                    if let Some(guard) = &arm.guard {
                        self.validate_expr(guard)?;
                    }
                    
                    // Validate body
                    match &arm.body {
                        SwitchBody::Expr(expr) => self.validate_expr(expr)?,
                        SwitchBody::Block(stmts) => {
                            for stmt in stmts {
                                self.validate_stmt(stmt)?;
                            }
                        }
                    }
                }
                
                // Check exhaustiveness
                self.check_switch_exhaustiveness(switch_expr)?;
                
                Ok(())
            }
        }
    }

    fn validate_call_expr(&mut self, call: &CallExpr) -> Result<()> {
        if let Some((first, second)) = Self::extract_modifier_chain(&call.callee) {
            return Err(CompilerError::SemanticError(
                format!(
                    "E0602: duplicate execution modifiers '{}' and '{}' on the same call",
                    first, second
                )
                .into(),
            ));
        }

        // Detect and mark HTTP.* calls as async and fallible
        if let Expr::Member { object, property: member } = &*call.callee {
            if let Expr::Identifier(name) = &**object {
                if name == "HTTP" {
                    match member.as_str() {
                        "get" | "post" | "put" | "delete" => {
                            // Mark as async and fallible
                            let http_fn = format!("HTTP.{}", member);
                            self.async_functions.insert(http_fn.clone());
                            self.fallible_functions.insert(http_fn);
                            
                            // Validate arguments
                            match member.as_str() {
                                "get" | "delete" => {
                                    // GET and DELETE: require 1 argument (url)
                                    if call.args.len() != 1 {
                                        let line = 0; // Placeholder for now
                                        return Err(CompilerError::SemanticError(
                                            SemanticErrorInfo {
                                                location: Some(ErrorLocation {
                                                    file: self.source_file.clone(),
                                                    line,
                                                    column: None,
                                                    source_line: None,
                                                    length: None,
                                                    context_before: None,
                                                    context_after: None,
                                                }),
                                                code: "E0902".to_string(),
                                                title: format!("Invalid HTTP.{} call", member),
                                                message: format!(
                                                    "HTTP.{} requires exactly 1 argument (url: string), found {}",
                                                    member, call.args.len()
                                                ),
                                                help: Some(format!("Usage: HTTP.{}(\"https://api.example.com\")", member)),
                                                suggestion: None,
                                                hint: None,
                                                example: None,
                                                doc_link: None,
                                                category: None,
                                            }
                                        ));
                                    }
                                }
                                "post" | "put" => {
                                    // POST and PUT: require 2 arguments (url, body)
                                    if call.args.len() != 2 {
                                        let line = 0; // Placeholder for now
                                        return Err(CompilerError::SemanticError(
                                            SemanticErrorInfo {
                                                location: Some(ErrorLocation {
                                                    file: self.source_file.clone(),
                                                    line,
                                                    column: None,
                                                    source_line: None,
                                                    length: None,
                                                    context_before: None,
                                                    context_after: None,
                                                }),
                                                code: "E0902".to_string(),
                                                title: format!("Invalid HTTP.{} call", member),
                                                message: format!(
                                                    "HTTP.{} requires exactly 2 arguments (url: string, body: string), found {}",
                                                    member, call.args.len()
                                                ),
                                                help: Some(format!("Usage: HTTP.{}(\"https://api.example.com\", body)", member)),
                                                suggestion: None,
                                                hint: None,
                                                example: None,
                                                doc_link: None,
                                                category: None,
                                            }
                                        ));
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            // Unknown HTTP method
                            let line = 0; // Placeholder for now
                            return Err(CompilerError::SemanticError(
                                SemanticErrorInfo {
                                    location: Some(ErrorLocation {
                                        file: self.source_file.clone(),
                                        line,
                                        column: None,
                                        source_line: None,
                                        length: None,
                                        context_before: None,
                                        context_after: None,
                                    }),
                                    code: "E0902".to_string(),
                                    title: "Unknown HTTP method".to_string(),
                                    message: format!(
                                        "HTTP.{} is not a valid HTTP method. Available methods: get, post, put, delete",
                                        member
                                    ),
                                    help: Some("Use one of: HTTP.get(), HTTP.post(), HTTP.put(), HTTP.delete()".to_string()),
                                    suggestion: None,
                                    hint: None,
                                    example: None,
                                    doc_link: None,
                                    category: None,
                                }
                            ));
                        }
                    }
                }
            }
        }

        // E0701: Check if calling fallible function without error binding
        // This validation applies to ALL call expressions, including those nested in other expressions
        // Exception: if we're in an error binding context (let result, err = ...), allow fallible calls
        if !self.in_error_binding {
            let func_name = match &*call.callee {
                Expr::Identifier(name) => Some(name.clone()),
                Expr::Member { object, property: member } => {
                    if let Expr::Identifier(name) = &**object {
                        if name == "HTTP" {
                            Some(format!("HTTP.{}", member))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            };
            
            if let Some(func_name) = func_name {
                if self.fallible_functions.contains(&func_name) {
                    let line = self.find_line_for_function_call(&func_name).unwrap_or(0);
                    let source_line = self.get_source_line(line);

                    let error = SemanticErrorInfo {
                        location: Some(ErrorLocation {
                            file: self.source_file.clone(),
                            line,
                            column: None,
                            source_line,
                            length: None,
                            context_before: None,
                            context_after: None,
                        }),
                        code: "E0701".to_string(),
                        title: "Fallible function must be called with error binding".to_string(),
                        message: format!(
                            "Function '{}' can fail but is not being called with error binding.\n       The function contains 'fail' statements and must be handled properly.",
                            func_name
                        ),
                        help: Some(format!("Change to: let result, err = async {}(...)", func_name)),
                        suggestion: None,
                        hint: None,
                        example: None,
                        doc_link: None,
                        category: None,
                    };

                    return Err(CompilerError::SemanticError(error));
                }
            }
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
        if matches!(
            call.exec_policy,
            ExecPolicy::Par | ExecPolicy::TaskPar | ExecPolicy::FirePar
        ) {
            // Placeholder: In a full implementation, we'd check if the call accesses shared mutable state
            // For now, we'll just note that this is where the check would go
        }

        // W0403: Warn about potentially inefficient concurrency patterns
        // This would detect patterns like spawning too many tasks or using parallel execution for trivial operations
        // For now, this is a placeholder implementation
        // TODO: Implement proper efficiency analysis
        if matches!(
            call.exec_policy,
            ExecPolicy::Par | ExecPolicy::TaskPar | ExecPolicy::FirePar
        ) {
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
                        Err(CompilerError::SemanticError(
                            format!("E0603: cannot await call using '{}' policy.", policy).into(),
                        ))
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

            // Handle both simple and destructured parameters
            match &param.pattern {
                BindingPattern::Identifier(name) => {
                    if self.declare_symbol(name, param.type_ref.clone()) {
                        self.exit_scope()?;
                        return Err(CompilerError::SemanticError(
                            format!("Parameter '{}' defined multiple times", name).into(),
                        ));
                    }
                }
                BindingPattern::Object(obj_pattern) => {
                    // Validate and declare all bindings from object pattern
                    for field in &obj_pattern.fields {
                        if self.declare_symbol(&field.binding, None) {
                            self.exit_scope()?;
                            return Err(CompilerError::SemanticError(
                                format!("Binding '{}' defined multiple times", field.binding).into(),
                            ));
                        }
                    }
                }
                BindingPattern::Array(arr_pattern) => {
                    // Validate and declare all bindings from array pattern
                    for element in arr_pattern.elements.iter().flatten() {
                        if self.declare_symbol(element, None) {
                            self.exit_scope()?;
                            return Err(CompilerError::SemanticError(
                                format!("Binding '{}' defined multiple times", element).into(),
                            ));
                        }
                    }
                    
                    // Handle rest pattern
                    if let Some(rest_name) = &arr_pattern.rest {
                        if self.declare_symbol(rest_name, None) {
                            self.exit_scope()?;
                            return Err(CompilerError::SemanticError(
                                format!("Binding '{}' defined multiple times", rest_name).into(),
                            ));
                        }
                    }
                }
                BindingPattern::Tuple(tuple_pattern) => {
                    // Validate and declare all bindings from tuple pattern
                    for element in &tuple_pattern.elements {
                        if self.declare_symbol(element, None) {
                            self.exit_scope()?;
                            return Err(CompilerError::SemanticError(
                                format!("Binding '{}' defined multiple times", element).into(),
                            ));
                        }
                    }
                }
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
            Expr::Tuple(elements) => elements
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
            Expr::MethodCall(method_call) => {
                Self::expr_contains_await(&method_call.object)
                    || method_call.args.iter().any(Self::expr_contains_await)
            }
            Expr::Switch(switch_expr) => {
                Self::expr_contains_await(&switch_expr.discriminant)
                    || switch_expr.arms.iter().any(|arm| {
                        arm.guard.as_ref().map_or(false, |g| Self::expr_contains_await(g))
                            || match &arm.body {
                                SwitchBody::Expr(expr) => Self::expr_contains_await(expr),
                                SwitchBody::Block(stmts) => {
                                    stmts.iter().any(Self::stmt_contains_await)
                                }
                            }
                    })
            }
        }
    }

    fn validate_known_function(&self, name: &str, arity: usize) -> Result<()> {
        if let Some(signature) = self.functions.get(name) {
            let total = signature.params.len();
            
            // Skip validation for imported functions (they have empty params)
            // This is indicated by params being empty AND not being in async/fallible sets
            // (local functions with no params would still be in those sets)
            if total == 0 && self.imported_symbols.contains(name) {
                // Imported function - skip arity validation
                return Ok(());
            }
            
            let optional = signature
                .defaults
                .iter()
                .filter(|is_default| **is_default)
                .count();
            let required = total.saturating_sub(optional);

            if arity < required || arity > total {
                return Err(CompilerError::SemanticError(
                    format!(
                        "Function '{}' expects between {} and {} arguments but {} were provided",
                        name, required, total, arity
                    )
                    .into(),
                ));
            }
        }

        Ok(())
    }

    fn validate_assignment_target(&mut self, target: &Expr) -> Result<()> {
        match target {
            Expr::Identifier(name) => {
                if self.lookup_symbol(name).is_none() {
                    // Generate suggestion for similar variable names
                    let available_vars = self.get_all_variables();
                    let suggestion = suggestions::find_suggestion(name, &available_vars, 2);
                    
                    let mut error = SemanticErrorInfo::new(
                        "E2003",
                        "Undefined variable",
                        &format!("Cannot assign to undefined variable '{}'", name)
                    );
                    
                    if let Some(suggested) = suggestion {
                        error = error.with_suggestion(&format!("Did you mean '{}'?", suggested));
                    }
                    
                    return Err(CompilerError::SemanticError(error));
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
            return Err(CompilerError::SemanticError(
                format!("W0601: task handle '{}' is never awaited.", name).into(),
            ));
        }

        Ok(())
    }

    // Type parameter scope management for generics
    fn enter_type_param_scope(&mut self) {
        self.type_parameters.push(HashSet::new());
        self.type_constraints.push(HashMap::new());
    }

    fn exit_type_param_scope(&mut self) {
        self.type_parameters.pop();
        self.type_constraints.pop();
    }

    fn declare_type_param(&mut self, name: &str) {
        if let Some(scope) = self.type_parameters.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn declare_type_param_with_constraint(&mut self, name: &str, constraint: &str) {
        self.declare_type_param(name);
        
        if let Some(scope) = self.type_constraints.last_mut() {
            let constraints = scope.entry(name.to_string()).or_insert_with(Vec::new);
            if !constraints.contains(&constraint.to_string()) {
                constraints.push(constraint.to_string());
            }
        }
    }

    fn is_type_param(&self, name: &str) -> bool {
        self.type_parameters
            .iter()
            .any(|scope| scope.contains(name))
    }

    fn get_type_param_constraints(&self, name: &str) -> Vec<String> {
        for scope in self.type_constraints.iter().rev() {
            if let Some(constraints) = scope.get(name) {
                return constraints.clone();
            }
        }
        vec![]
    }

    /// Check if a type parameter has a specific constraint
    fn has_constraint(&self, type_param: &str, required_trait: &str) -> bool {
        let constraints = self.get_type_param_constraints(type_param);
        
        // Check if the constraint is directly present
        if constraints.contains(&required_trait.to_string()) {
            return true;
        }
        
        // Check if any constraint implies the required trait (e.g., Ord implies Eq)
        for constraint in &constraints {
            let required_traits = self.trait_registry.get_required_traits(constraint);
            if required_traits.contains(required_trait) {
                return true;
            }
        }
        
        false
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

    /// Get all variable names currently in scope (for suggestions)
    fn get_all_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        for scope in &self.current_scope {
            vars.extend(scope.keys().cloned());
        }
        vars
    }

    /// Get all function names currently defined (for suggestions)
    fn get_all_functions(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Get all type names currently defined (for suggestions)
    fn get_all_types(&self) -> Vec<String> {
        self.types.keys().cloned().collect()
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
                return Err(CompilerError::SemanticError(
                    format!("E0604: handle '{}' awaited more than once.", name).into(),
                ));
            }
            return Err(CompilerError::SemanticError(
                format!("E0603: expression '{}' is not awaitable.", name).into(),
            ));
        }

        Err(CompilerError::SemanticError(
            format!("E0603: expression '{}' is not awaitable.", name).into(),
        ))
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
            Expr::Tuple(elements) => {
                // Infer tuple type from element types
                let mut types = Vec::new();
                for elem in elements {
                    if let Some(ty) = self.infer_expr_type(elem) {
                        types.push(ty);
                    } else {
                        // If we can't infer an element, we can't infer the tuple
                        return None;
                    }
                }
                Some(TypeRef::Tuple(types))
            }
            Expr::Identifier(name) => self.lookup_symbol(name).cloned().flatten(),
            Expr::Member { object, property } => {
                if property == "length" {
                    return Some(TypeRef::Simple("number".into()));
                }

                let base_type = self.infer_expr_type(object)?;
                let base_type = Self::strip_optional(base_type);
                
                // Handle tuple member access (.0, .1, .2, etc.)
                if let TypeRef::Tuple(types) = &base_type {
                    if let Ok(index) = property.parse::<usize>() {
                        if index < types.len() {
                            return Some(types[index].clone());
                        }
                    }
                    // Invalid tuple index - will error later
                    return None;
                }
                
                // Handle struct/class member access
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
            // Allow .length on identifiers - will be validated at codegen
            Expr::Identifier(_) => true,
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
            TypeRef::Tuple(_) => false,  // Tuples don't have .length
        }
    }

    /// Validate a destructuring binding pattern and declare all bound variables
    fn validate_and_declare_pattern(
        &mut self,
        pattern: &BindingPattern,
        init_expr: &Expr,
        declared_type: Option<TypeRef>,
        is_fallible: bool,
        span: Option<crate::span::Span>
    ) -> Result<()> {
        match pattern {
            BindingPattern::Identifier(name) => {
                // Simple identifier binding - existing behavior
                if self.declare_symbol(name, declared_type.clone()) {
                    let error = self.error_with_span(
                        "E0001",
                        &format!("Variable '{}' already defined in this scope", name),
                        &format!("Variable '{}' already defined in this scope", name),
                        span
                    )
                    .with_help(&format!("Consider using a different name or removing the previous declaration of '{}'", name));
                    return Err(CompilerError::SemanticError(error));
                }

                if is_fallible {
                    self.clear_awaitable(name);
                } else {
                    self.update_awaitable_from_expr(name, init_expr)?;
                }
            }
            BindingPattern::Object(obj_pattern) => {
                // Validate that we're destructuring from an object type
                let inferred_type = declared_type.or_else(|| self.infer_expr_type(init_expr));
                
                // Extract type name for validation
                let type_name = match inferred_type.as_ref() {
                    Some(TypeRef::Simple(name)) => Some(name.as_str()),
                    Some(TypeRef::Optional(inner)) => {
                        if let TypeRef::Simple(name) = inner.as_ref() {
                            Some(name.as_str())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                // If we have a concrete type, validate that fields exist
                if let Some(type_name) = type_name {
                    if let Some(type_info) = self.types.get(type_name) {
                        for field in &obj_pattern.fields {
                            if !type_info.fields.contains_key(&field.key) {
                                let error = self.error_with_span(
                                    "E0301",
                                    &format!("Field '{}' does not exist on type '{}'", field.key, type_name),
                                    &format!("Field '{}' does not exist on type '{}'", field.key, type_name),
                                    span
                                )
                                .with_help(&format!("Available fields: {}", type_info.fields.keys().map(|k| format!("'{}'", k)).collect::<Vec<_>>().join(", ")));
                                return Err(CompilerError::SemanticError(error));
                            }
                        }
                    }
                }

                // Check for duplicate bindings
                let mut seen_bindings = HashSet::new();
                for field in &obj_pattern.fields {
                    if !seen_bindings.insert(&field.binding) {
                        let error = self.error_with_span(
                            "E0302",
                            &format!("Duplicate binding '{}' in destructuring pattern", field.binding),
                            &format!("Duplicate binding '{}' in destructuring pattern", field.binding),
                            span
                        )
                        .with_help("Each binding in a destructuring pattern must be unique");
                        return Err(CompilerError::SemanticError(error));
                    }
                }

                // Declare all bound variables
                for field in &obj_pattern.fields {
                    // Infer field type if possible
                    let field_type = if let Some(type_name) = type_name {
                        self.types.get(type_name)
                            .and_then(|info| info.fields.get(&field.key))
                            .map(|(_, ty)| ty.clone())
                    } else {
                        None
                    };

                    if self.declare_symbol(&field.binding, field_type) {
                        let error = self.error_with_span(
                            "E0001",
                            &format!("Variable '{}' already defined in this scope", field.binding),
                            &format!("Variable '{}' already defined in this scope", field.binding),
                            span
                        )
                        .with_help(&format!("Consider using a different name or removing the previous declaration of '{}'", field.binding));
                        return Err(CompilerError::SemanticError(error));
                    }

                    if !is_fallible {
                        self.update_awaitable_from_expr(&field.binding, init_expr)?;
                    }
                }
            }
            BindingPattern::Array(arr_pattern) => {
                // Validate that we're destructuring from an array type
                let inferred_type = declared_type.or_else(|| self.infer_expr_type(init_expr));
                
                // Check if it's an array type
                let is_array = match inferred_type.as_ref() {
                    Some(TypeRef::Array(_)) => true,
                    Some(TypeRef::Optional(inner)) => matches!(inner.as_ref(), TypeRef::Array(_)),
                    _ => false,
                };

                if !is_array && inferred_type.is_some() {
                    let error = self.error_with_span(
                        "E0303",
                        "Cannot destructure non-array type with array pattern",
                        "Cannot destructure non-array type with array pattern",
                        span
                    )
                    .with_help("Array destructuring can only be used with array types");
                    return Err(CompilerError::SemanticError(error));
                }

                // Check for duplicate bindings
                let mut seen_bindings = HashSet::new();
                for element in &arr_pattern.elements {
                    if let Some(name) = element {
                        if !seen_bindings.insert(name) {
                            let error = self.error_with_span(
                                "E0302",
                                &format!("Duplicate binding '{}' in destructuring pattern", name),
                                &format!("Duplicate binding '{}' in destructuring pattern", name),
                                span
                            )
                            .with_help("Each binding in a destructuring pattern must be unique");
                            return Err(CompilerError::SemanticError(error));
                        }
                    }
                }
                
                if let Some(rest) = &arr_pattern.rest {
                    if !seen_bindings.insert(rest) {
                        let error = self.error_with_span(
                            "E0302",
                            &format!("Duplicate binding '{}' in destructuring pattern", rest),
                            &format!("Duplicate binding '{}' in destructuring pattern", rest),
                            span
                        )
                        .with_help("Each binding in a destructuring pattern must be unique");
                        return Err(CompilerError::SemanticError(error));
                    }
                }

                // Infer element type from array
                let element_type = match inferred_type {
                    Some(TypeRef::Array(inner)) => Some(*inner),
                    Some(TypeRef::Optional(inner)) => {
                        if let TypeRef::Array(elem) = *inner {
                            Some(*elem)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                // Declare all bound variables
                for element in &arr_pattern.elements {
                    if let Some(name) = element {
                        if self.declare_symbol(name, element_type.clone()) {
                            let error = self.error_with_span(
                                "E0001",
                                &format!("Variable '{}' already defined in this scope", name),
                                &format!("Variable '{}' already defined in this scope", name),
                                span
                            )
                            .with_help(&format!("Consider using a different name or removing the previous declaration of '{}'", name));
                            return Err(CompilerError::SemanticError(error));
                        }

                        if !is_fallible {
                            self.update_awaitable_from_expr(name, init_expr)?;
                        }
                    }
                }

                // Declare rest binding (as an array of the element type)
                if let Some(rest) = &arr_pattern.rest {
                    let rest_type = element_type.map(|t| TypeRef::Array(Box::new(t)));
                    if self.declare_symbol(rest, rest_type) {
                        let error = self.error_with_span(
                            "E0001",
                            &format!("Variable '{}' already defined in this scope", rest),
                            &format!("Variable '{}' already defined in this scope", rest),
                            span
                        )
                        .with_help(&format!("Consider using a different name or removing the previous declaration of '{}'", rest));
                        return Err(CompilerError::SemanticError(error));
                    }

                    if !is_fallible {
                        self.update_awaitable_from_expr(rest, init_expr)?;
                    }
                }
            }
            BindingPattern::Tuple(tuple_pattern) => {
                // Validate that we're destructuring from a tuple type
                let inferred_type = declared_type.or_else(|| self.infer_expr_type(init_expr));
                
                // Check if it's a tuple type
                let is_tuple = matches!(inferred_type.as_ref(), Some(TypeRef::Tuple(_)));

                if !is_tuple && inferred_type.is_some() {
                    let error = self.error_with_span(
                        "E0304",
                        "Cannot destructure non-tuple type with tuple pattern",
                        "Cannot destructure non-tuple type with tuple pattern",
                        span
                    )
                    .with_help("Tuple destructuring can only be used with tuple types");
                    return Err(CompilerError::SemanticError(error));
                }

                // Check for duplicate bindings
                let mut seen_bindings = HashSet::new();
                for element in &tuple_pattern.elements {
                    if !seen_bindings.insert(element) {
                        let error = self.error_with_span(
                            "E0302",
                            &format!("Duplicate binding '{}' in destructuring pattern", element),
                            &format!("Duplicate binding '{}' in destructuring pattern", element),
                            span
                        )
                        .with_help("Each binding in a destructuring pattern must be unique");
                        return Err(CompilerError::SemanticError(error));
                    }
                }

                // Extract element types from tuple
                let element_types: Vec<Option<TypeRef>> = match inferred_type {
                    Some(TypeRef::Tuple(types)) => types.iter().map(|t| Some(t.clone())).collect(),
                    _ => vec![None; tuple_pattern.elements.len()],
                };

                // Declare all bound variables with their corresponding types
                for (i, name) in tuple_pattern.elements.iter().enumerate() {
                    let element_type = element_types.get(i).and_then(|t| t.clone());
                    
                    if self.declare_symbol(name, element_type) {
                        let error = self.error_with_span(
                            "E0001",
                            &format!("Variable '{}' already defined in this scope", name),
                            &format!("Variable '{}' already defined in this scope", name),
                            span
                        )
                        .with_help(&format!("Consider using a different name or removing the previous declaration of '{}'", name));
                        return Err(CompilerError::SemanticError(error));
                    }

                    if !is_fallible {
                        self.update_awaitable_from_expr(name, init_expr)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Declare variables from a parameter pattern (for function parameters)
    fn declare_param_pattern(
        &mut self,
        pattern: &BindingPattern,
        param_type: Option<TypeRef>,
        span: Option<crate::span::Span>
    ) -> Result<()> {
        match pattern {
            BindingPattern::Identifier(name) => {
                // Simple parameter
                if self.declare_symbol(name, param_type) {
                    let error = self.error_with_span(
                        "E0310",
                        &format!("Parameter '{}' already declared", name),
                        &format!("Parameter '{}' already declared", name),
                        span
                    )
                    .with_help(&format!("Each parameter must have a unique name"));
                    return Err(CompilerError::SemanticError(error));
                }
            }
            BindingPattern::Object(obj_pattern) => {
                // Validate field existence if type is known
                if let Some(TypeRef::Simple(type_name)) = &param_type {
                    if let Some(type_info) = self.types.get(type_name) {
                        for field in &obj_pattern.fields {
                            if !type_info.fields.contains_key(&field.key) {
                                let error = self.error_with_span(
                                    "E0311",
                                    &format!("Field '{}' not found on type '{}'", field.key, type_name),
                                    &format!("Field '{}' not found on type '{}'", field.key, type_name),
                                    span
                                )
                                .with_help(&format!("Available fields: {}", type_info.fields.keys().map(|k| format!("'{}'", k)).collect::<Vec<_>>().join(", ")));
                                return Err(CompilerError::SemanticError(error));
                            }
                        }
                    }
                }
                
                // Check for duplicates
                let mut seen = HashSet::new();
                for field in &obj_pattern.fields {
                    if !seen.insert(&field.binding) {
                        let error = self.error_with_span(
                            "E0312",
                            &format!("Binding '{}' appears multiple times in pattern", field.binding),
                            &format!("Binding '{}' appears multiple times in pattern", field.binding),
                            span
                        )
                        .with_help("Each binding in a destructuring pattern must be unique");
                        return Err(CompilerError::SemanticError(error));
                    }
                }
                
                // Declare all bindings
                for field in &obj_pattern.fields {
                    let field_type = if let Some(TypeRef::Simple(type_name)) = &param_type {
                        self.types.get(type_name)
                            .and_then(|info| info.fields.get(&field.key))
                            .map(|(_, ty)| ty.clone())
                    } else {
                        None
                    };
                    
                    if self.declare_symbol(&field.binding, field_type) {
                        let error = self.error_with_span(
                            "E0312",
                            &format!("Binding '{}' already declared", field.binding),
                            &format!("Binding '{}' already declared", field.binding),
                            span
                        )
                        .with_help(&format!("Consider using a different name"));
                        return Err(CompilerError::SemanticError(error));
                    }
                }
            }
            BindingPattern::Array(arr_pattern) => {
                // Infer element type from array type
                let element_type = match &param_type {
                    Some(TypeRef::Array(inner)) => Some((**inner).clone()),
                    _ => None,
                };
                
                // Check for duplicates
                let mut seen = HashSet::new();
                for element in &arr_pattern.elements {
                    if let Some(name) = element {
                        if !seen.insert(name) {
                            let error = self.error_with_span(
                                "E0312",
                                &format!("Binding '{}' appears multiple times in pattern", name),
                                &format!("Binding '{}' appears multiple times in pattern", name),
                                span
                            )
                            .with_help("Each binding in a destructuring pattern must be unique");
                            return Err(CompilerError::SemanticError(error));
                        }
                    }
                }
                if let Some(rest) = &arr_pattern.rest {
                    if !seen.insert(rest) {
                        let error = self.error_with_span(
                            "E0312",
                            &format!("Binding '{}' appears multiple times in pattern", rest),
                            &format!("Binding '{}' appears multiple times in pattern", rest),
                            span
                        )
                        .with_help("Each binding in a destructuring pattern must be unique");
                        return Err(CompilerError::SemanticError(error));
                    }
                }
                
                // Declare element bindings
                for element in &arr_pattern.elements {
                    if let Some(name) = element {
                        if self.declare_symbol(name, element_type.clone()) {
                            let error = self.error_with_span(
                                "E0312",
                                &format!("Binding '{}' already declared", name),
                                &format!("Binding '{}' already declared", name),
                                span
                            )
                            .with_help(&format!("Consider using a different name"));
                            return Err(CompilerError::SemanticError(error));
                        }
                    }
                }
                
                // Declare rest binding
                if let Some(rest) = &arr_pattern.rest {
                    let rest_type = element_type.map(|t| TypeRef::Array(Box::new(t)));
                    if self.declare_symbol(rest, rest_type) {
                        let error = self.error_with_span(
                            "E0312",
                            &format!("Binding '{}' already declared", rest),
                            &format!("Binding '{}' already declared", rest),
                            span
                        )
                        .with_help(&format!("Consider using a different name"));
                        return Err(CompilerError::SemanticError(error));
                    }
                }
            }
            BindingPattern::Tuple(tuple_pattern) => {
                // Infer element types from tuple type
                let element_types: Vec<Option<TypeRef>> = match &param_type {
                    Some(TypeRef::Tuple(types)) => types.iter().map(|t| Some(t.clone())).collect(),
                    _ => vec![None; tuple_pattern.elements.len()],
                };
                
                // Check for duplicates
                let mut seen = HashSet::new();
                for name in &tuple_pattern.elements {
                    if !seen.insert(name) {
                        let error = self.error_with_span(
                            "E0312",
                            &format!("Binding '{}' appears multiple times in pattern", name),
                            &format!("Binding '{}' appears multiple times in pattern", name),
                            span
                        )
                        .with_help("Each binding in a destructuring pattern must be unique");
                        return Err(CompilerError::SemanticError(error));
                    }
                }
                
                // Declare element bindings with their corresponding types
                for (i, name) in tuple_pattern.elements.iter().enumerate() {
                    let element_type = element_types.get(i).and_then(|t| t.clone());
                    
                    if self.declare_symbol(name, element_type) {
                        let error = self.error_with_span(
                            "E0312",
                            &format!("Binding '{}' already declared", name),
                            &format!("Binding '{}' already declared", name),
                            span
                        )
                        .with_help(&format!("Consider using a different name"));
                        return Err(CompilerError::SemanticError(error));
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate that a binary operator can be used with the given operands
    fn validate_binary_op_constraints(&self, left: &Expr, right: &Expr, op: &BinOp) -> Result<()> {
        // Get the operator string for trait lookup
        let op_str = match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            _ => return Ok(()), // Logical operators (&&, ||) don't need constraints
        };
        
        // Find required trait for this operator
        let required_trait = self.trait_registry.trait_for_operator(op_str);
        if required_trait.is_none() {
            return Ok(()); // No trait constraint needed
        }
        
        let trait_name = &required_trait.unwrap().name;
        
        // Check if left operand is a type parameter
        if let Some(type_param) = self.extract_type_parameter(left) {
            if !self.has_constraint(&type_param, trait_name) {
                let mut error = SemanticErrorInfo::new(
                    "E5002",
                    "Missing trait constraint",
                    &format!("Cannot use operator '{}' with generic type '{}'", op_str, type_param)
                );
                error = error.with_hint(&format!("The type parameter '{}' must implement the '{}' trait to use this operator", type_param, trait_name));
                error = error.with_suggestion(&format!("Add constraint: <{}: {}>", type_param, trait_name));
                return Err(CompilerError::SemanticError(error));
            }
        }
        
        // Check if right operand is a type parameter
        if let Some(type_param) = self.extract_type_parameter(right) {
            if !self.has_constraint(&type_param, trait_name) {
                let mut error = SemanticErrorInfo::new(
                    "E5002",
                    "Missing trait constraint",
                    &format!("Cannot use operator '{}' with generic type '{}'", op_str, type_param)
                );
                error = error.with_hint(&format!("The type parameter '{}' must implement the '{}' trait to use this operator", type_param, trait_name));
                error = error.with_suggestion(&format!("Add constraint: <{}: {}>", type_param, trait_name));
                return Err(CompilerError::SemanticError(error));
            }
        }
        
        Ok(())
    }
    
    /// Validate that a unary operator can be used with the given operand
    fn validate_unary_op_constraints(&self, operand: &Expr, op: &UnOp) -> Result<()> {
        // Get the operator string for trait lookup
        let op_str = match op {
            UnOp::Neg => "unary-",
            UnOp::Not => "!",
            _ => return Ok(()), // Await doesn't need trait constraints
        };
        
        // Find required trait for this operator
        let required_trait = self.trait_registry.trait_for_operator(op_str);
        if required_trait.is_none() {
            return Ok(()); // No trait constraint needed
        }
        
        let trait_name = &required_trait.unwrap().name;
        
        // Check if operand is a type parameter
        if let Some(type_param) = self.extract_type_parameter(operand) {
            if !self.has_constraint(&type_param, trait_name) {
                let mut error = SemanticErrorInfo::new(
                    "E5002",
                    "Missing trait constraint",
                    &format!("Cannot use operator '{}' with generic type '{}'", op_str, type_param)
                );
                error = error.with_hint(&format!("The type parameter '{}' must implement the '{}' trait to use this operator", type_param, trait_name));
                error = error.with_suggestion(&format!("Add constraint: <{}: {}>", type_param, trait_name));
                return Err(CompilerError::SemanticError(error));
            }
        }
        
        Ok(())
    }
    
    /// Extract type parameter name from an expression, if it's a type parameter
    fn extract_type_parameter(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Identifier(name) => {
                // Check if this identifier is a type parameter
                if self.is_type_param(name) {
                    Some(name.clone())
                } else {
                    // Check if the identifier has a type annotation that's a type parameter
                    if let Some(Some(type_ref)) = self.lookup_symbol(name) {
                        self.extract_type_param_from_type_ref(type_ref)
                    } else {
                        None
                    }
                }
            }
            _ => None,
        }
    }
    
    /// Extract type parameter name from a TypeRef
    fn extract_type_param_from_type_ref(&self, type_ref: &TypeRef) -> Option<String> {
        match type_ref {
            TypeRef::Simple(name) if self.is_type_param(name) => Some(name.clone()),
            TypeRef::Optional(inner) => self.extract_type_param_from_type_ref(inner),
            TypeRef::Fallible(inner) => self.extract_type_param_from_type_ref(inner),
            _ => None,
        }
    }

    fn validate_type_ref(
        &self,
        type_ref: &TypeRef,
        available_type_params: &std::collections::HashSet<String>,
    ) -> Result<()> {
        match type_ref {
            TypeRef::Simple(name) => {
                // Check if it's a type parameter
                if available_type_params.contains(name) {
                    return Ok(());
                }
                
                // Check if it's a known type (class, interface, primitive)
                let primitives = ["int", "float", "bool", "string"];
                if primitives.contains(&name.as_str()) || self.types.contains_key(name) {
                    return Ok(());
                }
                
                // If not found, it might be undefined
                // For now we allow it (could be from external module or stdlib)
                Ok(())
            }
            TypeRef::Generic { base, args } => {
                // Validate base type (it's a String, so wrap it as Simple TypeRef)
                let base_ref = TypeRef::Simple(base.clone());
                self.validate_type_ref(&base_ref, available_type_params)?;
                
                // Validate all type arguments
                for arg in args {
                    self.validate_type_ref(arg, available_type_params)?;
                }
                Ok(())
            }
            TypeRef::Array(inner) => self.validate_type_ref(inner, available_type_params),
            TypeRef::Optional(inner) => self.validate_type_ref(inner, available_type_params),
            TypeRef::Fallible(inner) => self.validate_type_ref(inner, available_type_params),
            TypeRef::Tuple(types) => {
                // Validate all element types in the tuple
                for ty in types {
                    self.validate_type_ref(ty, available_type_params)?;
                }
                Ok(())
            }
        }
    }

    /// Extract all binding names from a pattern
    fn extract_pattern_bindings(&self, pattern: &Pattern, bindings: &mut Vec<String>) {
        match pattern {
            Pattern::Binding(name) => {
                bindings.push(name.clone());
            }
            Pattern::Tuple(patterns) | Pattern::Array(patterns) | Pattern::Or(patterns) => {
                for p in patterns {
                    self.extract_pattern_bindings(p, bindings);
                }
            }
            Pattern::Literal(_) | Pattern::Wildcard | Pattern::Range(_) => {
                // No bindings
            }
        }
    }

    /// Validate or-patterns have consistent bindings across all alternatives
    fn validate_or_pattern(&self, or_patterns: &[Pattern]) -> Result<()> {
        if or_patterns.is_empty() {
            return Ok(());
        }

        // Extract bindings from first pattern
        let mut first_bindings = Vec::new();
        self.extract_pattern_bindings(&or_patterns[0], &mut first_bindings);
        first_bindings.sort();

        // Check all other patterns have same bindings
        for (i, pattern) in or_patterns.iter().enumerate().skip(1) {
            let mut bindings = Vec::new();
            self.extract_pattern_bindings(pattern, &mut bindings);
            bindings.sort();

            if bindings != first_bindings {
                let mut error = SemanticErrorInfo::new(
                    "E0906",
                    "Incompatible Or-Pattern Bindings",
                    &format!(
                        "All alternatives in an or-pattern must bind the same variables. \
                        First pattern binds: {:?}, but pattern {} binds: {:?}",
                        first_bindings,
                        i + 1,
                        bindings
                    ),
                );

                error.category = Some("Pattern Matching".to_string());
                error.hint = Some("Ensure all alternatives in the or-pattern bind the same variable names".to_string());
                error.example = Some("// ✅ Good:\n1 | 2 | 3 => \"small\"\n\n// ✅ Good with bindings:\nx | y => x  // Both bind one variable\n\n// ❌ Bad:\nSome(x) | None => x  // Inconsistent bindings".to_string());
                error.doc_link = Some("https://liva-lang.org/docs/pattern-matching#or-patterns".to_string());

                return Err(CompilerError::SemanticError(error));
            }
        }

        Ok(())
    }

    /// Validate all patterns in a switch expression
    fn validate_switch_patterns(&self, switch_expr: &SwitchExpr) -> Result<()> {
        for arm in &switch_expr.arms {
            self.validate_pattern(&arm.pattern)?;
        }
        Ok(())
    }

    /// Recursively validate a pattern
    fn validate_pattern(&self, pattern: &Pattern) -> Result<()> {
        match pattern {
            Pattern::Or(patterns) => {
                self.validate_or_pattern(patterns)?;
                for p in patterns {
                    self.validate_pattern(p)?;
                }
            }
            Pattern::Tuple(patterns) | Pattern::Array(patterns) => {
                for p in patterns {
                    self.validate_pattern(p)?;
                }
            }
            Pattern::Literal(_) | Pattern::Wildcard | Pattern::Binding(_) | Pattern::Range(_) => {
                // No additional validation needed
            }
        }
        Ok(())
    }

    /// Check if switch expression patterns are exhaustive
    fn check_switch_exhaustiveness(&self, switch_expr: &SwitchExpr) -> Result<()> {
        // First validate all patterns (or-patterns, nested patterns, etc.)
        self.validate_switch_patterns(switch_expr)?;

        // Check if there's a wildcard or binding pattern (catches all cases)
        let has_catch_all = switch_expr.arms.iter().any(|arm| {
            matches!(arm.pattern, Pattern::Wildcard | Pattern::Binding(_))
        });

        if has_catch_all {
            return Ok(()); // Exhaustive with wildcard/binding
        }

        // Try to infer discriminant type from literal patterns
        let discriminant_type = self.infer_switch_discriminant_type(switch_expr);

        // Check exhaustiveness based on type
        match discriminant_type.as_deref() {
            Some("bool") => {
                // Check if both true and false are covered
                let mut has_true = false;
                let mut has_false = false;

                for arm in &switch_expr.arms {
                    if let Pattern::Literal(Literal::Bool(val)) = &arm.pattern {
                        if *val {
                            has_true = true;
                        } else {
                            has_false = true;
                        }
                    }
                }

                if !has_true || !has_false {
                    let missing = if !has_true && !has_false {
                        "true and false"
                    } else if !has_true {
                        "true"
                    } else {
                        "false"
                    };

                    let mut error = SemanticErrorInfo::new(
                        "E0901",
                        "Non-exhaustive Pattern Matching",
                        &format!("Pattern matching on bool is not exhaustive - missing case(s): {}", missing),
                    );
                    
                    error.category = Some("Pattern Matching".to_string());
                    error.hint = Some(format!("Add pattern `{}` or use wildcard `_` to catch remaining cases", missing));
                    error.example = Some(format!(
                        "switch value {{\n    true => \"yes\",\n    false => \"no\"\n}}\n// or\nswitch value {{\n    {} => \"...\",\n    _ => \"...\"\n}}",
                        if !has_true { "true" } else { "false" }
                    ));
                    error.doc_link = Some("https://liva-lang.org/docs/pattern-matching#exhaustiveness".to_string());

                    return Err(CompilerError::SemanticError(error));
                }

                Ok(())
            }
            Some("int") | Some("i8") | Some("i16") | Some("i32") | Some("i64") | Some("i128") |
            Some("u8") | Some("u16") | Some("u32") | Some("u64") | Some("u128") => {
                self.check_int_exhaustiveness(switch_expr, discriminant_type.as_deref().unwrap())
            }
            Some("string") | Some("String") => {
                self.check_string_exhaustiveness(switch_expr)
            }
            _ => {
                // For other types (float, char, custom types), we can't easily determine exhaustiveness
                // without advanced type analysis. Suggest using a wildcard pattern.
                // This is a soft warning - we don't enforce it for now.
                Ok(())
            }
        }
    }

    /// Try to infer the type of the switch discriminant from its patterns
    fn infer_switch_discriminant_type(&self, switch_expr: &SwitchExpr) -> Option<String> {
        // Check first literal pattern to infer type, including inside or-patterns
        for arm in &switch_expr.arms {
            if let Some(typ) = self.infer_pattern_type(&arm.pattern) {
                return Some(typ);
            }
        }

        None
    }

    /// Helper to infer type from a pattern recursively
    fn infer_pattern_type(&self, pattern: &Pattern) -> Option<String> {
        match pattern {
            Pattern::Literal(lit) => Some(match lit {
                Literal::Int(_) => "int".to_string(),
                Literal::Float(_) => "float".to_string(),
                Literal::String(_) => "string".to_string(),
                Literal::Bool(_) => "bool".to_string(),
                Literal::Char(_) => "char".to_string(),
            }),
            Pattern::Or(patterns) => {
                // Check first sub-pattern in or-pattern
                patterns.first().and_then(|p| self.infer_pattern_type(p))
            }
            Pattern::Tuple(patterns) | Pattern::Array(patterns) => {
                // For tuple/array, we'd need more complex type inference
                // For now, we don't infer types from these
                None
            }
            Pattern::Wildcard | Pattern::Binding(_) | Pattern::Range(_) => {
                // These don't give us type info directly
                None
            }
        }
    }

    /// Extract integer literals from a pattern recursively
    fn extract_int_literals(&self, pattern: &Pattern, values: &mut HashSet<i64>, has_ranges: &mut bool) {
        match pattern {
            Pattern::Literal(Literal::Int(val)) => {
                values.insert(*val);
            }
            Pattern::Literal(_) => {
                // Other literal types don't contribute to integer exhaustiveness
            }
            Pattern::Range(range_pattern) => {
                *has_ranges = true;
                // Try to extract integer bounds if both are literals
                let start_val = range_pattern.start.as_ref().and_then(|expr| {
                    if let Expr::Literal(Literal::Int(v)) = expr.as_ref() {
                        Some(*v)
                    } else {
                        None
                    }
                });
                
                let end_val = range_pattern.end.as_ref().and_then(|expr| {
                    if let Expr::Literal(Literal::Int(v)) = expr.as_ref() {
                        Some(*v)
                    } else {
                        None
                    }
                });
                
                // Only enumerate small, bounded ranges
                if let (Some(s), Some(e)) = (start_val, end_val) {
                    let range_size = (e - s + 1).abs();
                    if range_size <= 1000 && range_size > 0 {
                        for i in s..=e {
                            values.insert(i);
                        }
                    }
                }
            }
            Pattern::Or(patterns) => {
                for p in patterns {
                    self.extract_int_literals(p, values, has_ranges);
                }
            }
            Pattern::Tuple(patterns) | Pattern::Array(patterns) => {
                // Don't extract from nested structures for now
            }
            Pattern::Wildcard | Pattern::Binding(_) => {
                // These don't contribute to coverage
            }
        }
    }

    /// Check exhaustiveness for integer patterns
    fn check_int_exhaustiveness(&self, switch_expr: &SwitchExpr, _int_type: &str) -> Result<()> {
        use std::collections::HashSet;
        
        let mut covered_values: HashSet<i64> = HashSet::new();
        let mut has_ranges = false;
        
        // Collect all explicitly covered values and check for ranges
        for arm in &switch_expr.arms {
            self.extract_int_literals(&arm.pattern, &mut covered_values, &mut has_ranges);
        }
        
        // For integers with only literal patterns (no ranges), we can check if all reasonable values are covered
        // But since integers are infinite, we require a wildcard unless it's a very small set
        if !has_ranges && !covered_values.is_empty() && covered_values.len() <= 20 {
            // For small sets of literals without wildcard, suggest adding wildcard
            let mut error = SemanticErrorInfo::new(
                "E0902",
                "Non-exhaustive Pattern Matching",
                &format!("Pattern matching on integers is not exhaustive - {} value(s) explicitly covered, but no wildcard for other integers", covered_values.len()),
            );
            
            error.category = Some("Pattern Matching".to_string());
            error.hint = Some("Add wildcard pattern `_` to catch all other integer values".to_string());
            error.example = Some("switch num {\n    0 => \"zero\",\n    1 => \"one\",\n    _ => \"other\"  // Required\n}".to_string());
            error.doc_link = Some("https://liva-lang.org/docs/pattern-matching#exhaustiveness".to_string());
            
            return Err(CompilerError::SemanticError(error));
        }
        
        // For ranges or large sets, we always require a wildcard (already checked at start)
        if has_ranges || covered_values.len() > 20 {
            let mut error = SemanticErrorInfo::new(
                "E0902",
                "Non-exhaustive Pattern Matching",
                "Pattern matching on integers with ranges requires a wildcard pattern",
            );
            
            error.category = Some("Pattern Matching".to_string());
            error.hint = Some("Add wildcard pattern `_` to catch all values not covered by explicit patterns or ranges".to_string());
            error.example = Some("switch num {\n    0..=10 => \"small\",\n    11..=100 => \"medium\",\n    _ => \"large\"  // Required\n}".to_string());
            error.doc_link = Some("https://liva-lang.org/docs/pattern-matching#exhaustiveness".to_string());
            
            return Err(CompilerError::SemanticError(error));
        }
        
        Ok(())
    }

    /// Check exhaustiveness for string patterns
    fn check_string_exhaustiveness(&self, _switch_expr: &SwitchExpr) -> Result<()> {
        // Strings are infinite, so we always require a wildcard or binding pattern
        // This is already checked by has_catch_all at the start of check_switch_exhaustiveness
        // If we reach here, it means no wildcard was found
        
        let mut error = SemanticErrorInfo::new(
            "E0903",
            "Non-exhaustive Pattern Matching",
            "Pattern matching on strings requires a wildcard or binding pattern",
        );
        
        error.category = Some("Pattern Matching".to_string());
        error.hint = Some("Add wildcard pattern `_` or binding pattern to catch all string values not explicitly matched".to_string());
        error.example = Some("switch text {\n    \"active\" => 1,\n    \"inactive\" => 2,\n    _ => 0  // Required\n}".to_string());
        error.doc_link = Some("https://liva-lang.org/docs/pattern-matching#exhaustiveness".to_string());
        
        return Err(CompilerError::SemanticError(error));
    }
    
    /// Validate that a type hint for JSON.parse is serializable (Phase 1: JSON Typed Parsing)
    fn validate_json_parse_type_hint(&mut self, type_ref: &TypeRef) -> Result<()> {
        match type_ref {
            // Primitive types are always serializable
            TypeRef::Simple(name) => {
                let valid_primitives = ["int", "i8", "i16", "i32", "i64", "i128", 
                                       "u8", "u16", "u32", "u64", "u128", "usize", "isize",
                                       "float", "f32", "f64", "bool", "string", "String"];
                
                if !valid_primitives.contains(&name.as_str()) {
                    // Check if it's a defined class
                    if !self.types.contains_key(name) {
                        return Err(CompilerError::SemanticError(
                            format!("Type '{}' is not defined or not serializable for JSON parsing", name).into()
                        ));
                    }
                    // Phase 2: Mark this class as needing serde derive
                    self.json_classes.insert(name.clone());
                    // TODO: Recursive validation of all class fields
                }
                Ok(())
            }
            // Arrays are serializable if their element type is
            TypeRef::Array(inner) => {
                self.validate_json_parse_type_hint(inner)
            }
            // Optional types are serializable if their inner type is
            TypeRef::Optional(inner) => {
                self.validate_json_parse_type_hint(inner)
            }
            // Fallible types (Result) - validate inner type
            TypeRef::Fallible(inner) => {
                self.validate_json_parse_type_hint(inner)
            }
            // Generic types - basic validation
            TypeRef::Generic { base, args } => {
                // Validate base type
                if !self.types.contains_key(base) {
                    return Err(CompilerError::SemanticError(
                        format!("Generic type '{}' is not defined", base).into()
                    ));
                }
                // Validate all type arguments
                for arg in args {
                    self.validate_json_parse_type_hint(arg)?;
                }
                Ok(())
            }
            // Tuples are serializable if all their element types are
            TypeRef::Tuple(types) => {
                for ty in types {
                    self.validate_json_parse_type_hint(ty)?;
                }
                Ok(())
            }
        }
    }

    /// Mark classes that need serde derives (Phase 2: JSON Typed Parsing)
    fn mark_json_classes(&self, program: &mut Program) {
        // Phase 4: Collect all classes transitively (including nested dependencies)
        let mut all_json_classes = std::collections::HashSet::new();
        
        // Start with direct JSON.parse classes
        for class_name in &self.json_classes {
            self.collect_class_dependencies(class_name, program, &mut all_json_classes);
        }
        
        // Mark all collected classes
        for item in &mut program.items {
            if let TopLevel::Class(class) = item {
                if all_json_classes.contains(&class.name) {
                    class.needs_serde = true;
                }
            }
        }
    }
    
    /// Recursively collect all class dependencies for JSON serialization
    fn collect_class_dependencies(
        &self,
        class_name: &str,
        program: &Program,
        collected: &mut std::collections::HashSet<String>,
    ) {
        // Avoid infinite recursion
        if collected.contains(class_name) {
            return;
        }
        
        collected.insert(class_name.to_string());
        
        // Find the class definition
        for item in &program.items {
            if let TopLevel::Class(class) = item {
                if class.name == class_name {
                    // Check all fields for class types
                    for member in &class.members {
                        if let Member::Field(field) = member {
                            if let Some(type_ref) = &field.type_ref {
                                self.collect_type_dependencies(type_ref, program, collected);
                            }
                        }
                    }
                    break;
                }
            }
        }
    }
    
    /// Collect dependencies from a type reference (handles arrays and nested types)
    fn collect_type_dependencies(
        &self,
        type_ref: &TypeRef,
        program: &Program,
        collected: &mut std::collections::HashSet<String>,
    ) {
        match type_ref {
            TypeRef::Simple(name) => {
                // Check if this is a class type (not a primitive)
                if self.is_class_type(name, program) {
                    self.collect_class_dependencies(name, program, collected);
                }
            }
            TypeRef::Array(elem_type) => {
                // Recursively check array element type
                self.collect_type_dependencies(elem_type, program, collected);
            }
            TypeRef::Optional(inner) => {
                // Recursively check optional inner type
                self.collect_type_dependencies(inner, program, collected);
            }
            _ => {}
        }
    }
    
    /// Check if a type name is a class (not a primitive)
    fn is_class_type(&self, type_name: &str, program: &Program) -> bool {
        // Primitives are not classes
        match type_name {
            "int" | "i8" | "i16" | "i32" | "i64" | "i128" |
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "isize" |
            "float" | "f32" | "f64" | "bool" | "string" | "String" => false,
            _ => {
                // Check if it's actually defined as a class
                program.items.iter().any(|item| {
                    if let TopLevel::Class(class) = item {
                        class.name == type_name
                    } else {
                        false
                    }
                })
            }
        }
    }
}

pub fn analyze(program: Program) -> Result<Program> {
    let mut analyzer = SemanticAnalyzer::new(String::new(), String::new());
    analyzer.analyze_program(program)
}

pub fn analyze_with_source(
    program: Program,
    source_file: String,
    source_code: String,
) -> Result<Program> {
    let mut analyzer = SemanticAnalyzer::new(source_file, source_code);
    analyzer.analyze_program(program)
}

/// Analyze a program with import context from resolved modules
pub fn analyze_with_modules(
    program: Program,
    source_file: String,
    source_code: String,
    modules: &HashMap<std::path::PathBuf, (HashSet<String>, HashSet<String>)>,
) -> Result<Program> {
    let mut analyzer = SemanticAnalyzer::new(source_file, source_code);
    analyzer.imported_modules = modules.clone();
    analyzer.analyze_program(program)
}
