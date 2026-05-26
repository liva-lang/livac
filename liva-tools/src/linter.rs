use colored::Colorize;
/// Linter module for Liva
///
/// Runs static analysis on the parsed AST to detect code smells and warnings.
/// Warnings use W-codes (W001-W008) and are non-blocking — compilation proceeds.
///
/// ## Warning Codes
///
/// - **W001**: Variable declared but never used
/// - **W002**: Import declared but never used
/// - **W003**: Unreachable code after `return` or `fail`
/// - **W004**: Comparison is always true or always false
/// - **W005**: Variable shadows an outer-scope binding
/// - **W006**: Empty block (if / else / while / for body)
/// - **W007**: Function parameter declared but never used
/// - **W008**: Unnecessary `else` after a diverging branch (`return`/`throw`/`fail`/`break`/`continue`)
use livac::ast::*;
use livac::span::SourceMap;
use std::collections::{HashMap, HashSet};

/// A single lint warning emitted by the linter.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LintWarning {
    /// Warning code (e.g., "W001")
    pub code: String,
    /// Short title
    pub title: String,
    /// Detailed message
    pub message: String,
    /// Source file name
    pub file: String,
    /// 1-based line number
    pub line: usize,
    /// 1-based column (optional)
    pub column: Option<usize>,
    /// The source line text
    pub source_line: Option<String>,
    /// Suggested fix
    pub help: Option<String>,
}

impl LintWarning {
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "{} {}: {}\n",
            "warning".yellow().bold(),
            format!("[{}]", self.code).yellow(),
            self.title.bold()
        ));

        // Location
        output.push_str(&format!(
            "  {} {}:{}\n",
            "-->".blue(),
            self.file.cyan(),
            self.line.to_string().yellow()
        ));

        // Source line
        if let Some(src) = &self.source_line {
            output.push_str(&format!(
                "   {} {} {}\n",
                format!("{:>4}", self.line).bright_black(),
                "|".bright_black(),
                src
            ));
        }

        // Message
        output.push_str(&format!("   {} {}\n", "=".blue(), self.message));

        // Help
        if let Some(help) = &self.help {
            output.push_str(&format!("   {} {}\n", "help:".green(), help));
        }

        output
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Linter context that accumulates warnings while walking the AST.
pub struct Linter {
    warnings: Vec<LintWarning>,
    source_file: String,
    source_code: String,
    #[allow(dead_code)]
    source_map: Option<SourceMap>,
}

/// Tracks a variable declaration for unused-variable analysis.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct VarInfo {
    /// The variable name
    name: String,
    /// 1-based line where declared
    line: usize,
    /// Whether this variable has been read/used
    used: bool,
}

impl Linter {
    pub fn new(source_file: String, source_code: String) -> Self {
        let source_map = if !source_code.is_empty() {
            Some(SourceMap::new(&source_code))
        } else {
            None
        };
        Self {
            warnings: Vec::new(),
            source_file,
            source_code,
            source_map,
        }
    }

    /// Run all lint checks on the program and return warnings.
    pub fn lint(&mut self, program: &Program) -> Vec<LintWarning> {
        self.check_unused_imports(program);
        self.check_unused_variables(program);
        self.check_unreachable_code(program);
        self.check_always_true_false(program);
        self.check_shadowed_variables(program);
        self.check_empty_blocks(program);
        self.check_unused_parameters(program);
        self.check_redundant_else(program);
        self.warnings.clone()
    }

    /// Get the source line text at a 1-based line number.
    fn source_line_at(&self, line: usize) -> Option<String> {
        self.source_code
            .lines()
            .nth(line.saturating_sub(1))
            .map(|s| s.to_string())
    }

    /// Estimate the line number of a statement by searching the source.
    /// This is a heuristic since not all AST nodes carry span info.
    fn estimate_var_line(&self, name: &str, after_line: usize) -> usize {
        for (idx, line) in self.source_code.lines().enumerate() {
            let line_num = idx + 1;
            if line_num <= after_line {
                continue;
            }
            let trimmed = line.trim();
            // Match patterns like: let name = ..., let name: type = ..., const name = ...
            if (trimmed.starts_with("let ") || trimmed.starts_with("const "))
                && trimmed.contains(name)
            {
                return line_num;
            }
        }
        // Fallback: search from beginning
        for (idx, line) in self.source_code.lines().enumerate() {
            let trimmed = line.trim();
            if (trimmed.starts_with("let ") || trimmed.starts_with("const "))
                && trimmed.contains(name)
            {
                return idx + 1;
            }
        }
        1
    }

    /// Search source for an import line containing the given symbol.
    fn find_import_line(&self, symbol: &str) -> usize {
        for (idx, line) in self.source_code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") && trimmed.contains(symbol) {
                return idx + 1;
            }
        }
        1
    }

    /// Search source for a statement text pattern, starting after a given line.
    fn find_line_containing(&self, pattern: &str, after_line: usize) -> usize {
        for (idx, line) in self.source_code.lines().enumerate() {
            let line_num = idx + 1;
            if line_num <= after_line {
                continue;
            }
            if line.contains(pattern) {
                return line_num;
            }
        }
        after_line + 1
    }

    // ───────────────────────────────────────────────────────────
    // W001: Variable declared but never used
    // ───────────────────────────────────────────────────────────

    fn check_unused_variables(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevel::Function(f) => {
                    let mut vars: HashMap<String, VarInfo> = HashMap::new();
                    let mut used_names: HashSet<String> = HashSet::new();

                    // Register parameters as used (they're part of the interface)
                    // But still collect them for scope
                    for param in &f.params {
                        if let Some(name) = param.name() {
                            used_names.insert(name.to_string());
                        }
                    }

                    // Collect declarations and usages from the body
                    if let Some(body) = &f.body {
                        self.collect_var_decls_block(body, &mut vars, 0);
                        self.collect_var_usages_block(body, &mut used_names);
                    }
                    if let Some(expr) = &f.expr_body {
                        self.collect_var_usages_expr(expr, &mut used_names);
                    }

                    // Report unused
                    for (name, info) in &vars {
                        if !used_names.contains(name) && !name.starts_with('_') {
                            let line = info.line;
                            self.warnings.push(LintWarning {
                                code: "W001".to_string(),
                                title: "Unused variable".to_string(),
                                message: format!("Variable '{}' is declared but never used", name),
                                file: self.source_file.clone(),
                                line,
                                column: None,
                                source_line: self.source_line_at(line),
                                help: Some(format!("Prefix with underscore to suppress: _{name}")),
                            });
                        }
                    }
                }
                TopLevel::Class(class) => {
                    for member in &class.members {
                        if let Member::Method(method) = member {
                            let mut vars: HashMap<String, VarInfo> = HashMap::new();
                            let mut used_names: HashSet<String> = HashSet::new();

                            // Parameters are used
                            for param in &method.params {
                                if let Some(name) = param.name() {
                                    used_names.insert(name.to_string());
                                }
                            }

                            if let Some(body) = &method.body {
                                self.collect_var_decls_block(body, &mut vars, 0);
                                self.collect_var_usages_block(body, &mut used_names);
                            }
                            if let Some(expr) = &method.expr_body {
                                self.collect_var_usages_expr(expr, &mut used_names);
                            }

                            for (name, info) in &vars {
                                if !used_names.contains(name) && !name.starts_with('_') {
                                    let line = info.line;
                                    self.warnings.push(LintWarning {
                                        code: "W001".to_string(),
                                        title: "Unused variable".to_string(),
                                        message: format!(
                                            "Variable '{}' is declared but never used",
                                            name
                                        ),
                                        file: self.source_file.clone(),
                                        line,
                                        column: None,
                                        source_line: self.source_line_at(line),
                                        help: Some(format!(
                                            "Prefix with underscore to suppress: _{name}"
                                        )),
                                    });
                                }
                            }
                        }
                    }
                }
                TopLevel::Test(test) => {
                    let mut vars: HashMap<String, VarInfo> = HashMap::new();
                    let mut used_names: HashSet<String> = HashSet::new();

                    self.collect_var_decls_block(&test.body, &mut vars, 0);
                    self.collect_var_usages_block(&test.body, &mut used_names);

                    for (name, info) in &vars {
                        if !used_names.contains(name) && !name.starts_with('_') {
                            let line = info.line;
                            self.warnings.push(LintWarning {
                                code: "W001".to_string(),
                                title: "Unused variable".to_string(),
                                message: format!("Variable '{}' is declared but never used", name),
                                file: self.source_file.clone(),
                                line,
                                column: None,
                                source_line: self.source_line_at(line),
                                help: Some(format!("Prefix with underscore to suppress: _{name}")),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Collect variable declarations from a block.
    fn collect_var_decls_block(
        &self,
        block: &BlockStmt,
        vars: &mut HashMap<String, VarInfo>,
        mut last_line: usize,
    ) {
        for stmt in &block.stmts {
            match stmt {
                Stmt::VarDecl(decl) => {
                    for binding in &decl.bindings {
                        self.collect_binding_names(&binding.pattern, vars, &mut last_line);
                    }
                }
                Stmt::ConstDecl(decl) => {
                    let line = self.estimate_var_line(&decl.name, last_line);
                    last_line = line;
                    vars.insert(
                        decl.name.clone(),
                        VarInfo {
                            name: decl.name.clone(),
                            line,
                            used: false,
                        },
                    );
                }
                Stmt::For(for_stmt) => {
                    let line = self.find_line_containing(
                        &format!("for {}", for_stmt.var),
                        last_line.saturating_sub(1),
                    );
                    last_line = line;
                    vars.insert(
                        for_stmt.var.clone(),
                        VarInfo {
                            name: for_stmt.var.clone(),
                            line,
                            used: false,
                        },
                    );
                    if let Some(v2) = &for_stmt.var2 {
                        vars.insert(
                            v2.clone(),
                            VarInfo {
                                name: v2.clone(),
                                line,
                                used: false,
                            },
                        );
                    }
                    self.collect_var_decls_block(&for_stmt.body, vars, last_line);
                }
                Stmt::If(if_stmt) => {
                    if let IfBody::Block(b) = &if_stmt.then_branch {
                        self.collect_var_decls_block(b, vars, last_line);
                    }
                    if let Some(IfBody::Block(b)) = &if_stmt.else_branch {
                        self.collect_var_decls_block(b, vars, last_line);
                    }
                }
                Stmt::While(w) => {
                    self.collect_var_decls_block(&w.body, vars, last_line);
                }
                Stmt::Block(b) => {
                    self.collect_var_decls_block(b, vars, last_line);
                }
                Stmt::TryCatch(tc) => {
                    self.collect_var_decls_block(&tc.try_block, vars, last_line);
                    self.collect_var_decls_block(&tc.catch_block, vars, last_line);
                }
                _ => {}
            }
        }
    }

    /// Extract variable names from binding patterns.
    fn collect_binding_names(
        &self,
        pattern: &BindingPattern,
        vars: &mut HashMap<String, VarInfo>,
        last_line: &mut usize,
    ) {
        match pattern {
            BindingPattern::Identifier(name) => {
                let line = self.estimate_var_line(name, *last_line);
                *last_line = line;
                vars.insert(
                    name.clone(),
                    VarInfo {
                        name: name.clone(),
                        line,
                        used: false,
                    },
                );
            }
            BindingPattern::Object(obj) => {
                for field in &obj.fields {
                    let line = self.estimate_var_line(&field.binding, *last_line);
                    *last_line = line;
                    vars.insert(
                        field.binding.clone(),
                        VarInfo {
                            name: field.binding.clone(),
                            line,
                            used: false,
                        },
                    );
                }
            }
            BindingPattern::Array(arr) => {
                for elem in &arr.elements {
                    if let Some(name) = elem {
                        let line = self.estimate_var_line(name, *last_line);
                        *last_line = line;
                        vars.insert(
                            name.clone(),
                            VarInfo {
                                name: name.clone(),
                                line,
                                used: false,
                            },
                        );
                    }
                }
                if let Some(rest) = &arr.rest {
                    let line = self.estimate_var_line(rest, *last_line);
                    *last_line = line;
                    vars.insert(
                        rest.clone(),
                        VarInfo {
                            name: rest.clone(),
                            line,
                            used: false,
                        },
                    );
                }
            }
            BindingPattern::Tuple(tup) => {
                for name in &tup.elements {
                    let line = self.estimate_var_line(name, *last_line);
                    *last_line = line;
                    vars.insert(
                        name.clone(),
                        VarInfo {
                            name: name.clone(),
                            line,
                            used: false,
                        },
                    );
                }
            }
        }
    }

    /// Collect variable usages from a block.
    fn collect_var_usages_block(&self, block: &BlockStmt, used: &mut HashSet<String>) {
        for stmt in &block.stmts {
            self.collect_var_usages_stmt(stmt, used);
        }
    }

    fn collect_var_usages_stmt(&self, stmt: &Stmt, used: &mut HashSet<String>) {
        match stmt {
            Stmt::VarDecl(decl) => {
                self.collect_var_usages_expr(&decl.init, used);
                if let Some(or_fail) = &decl.or_fail_msg {
                    self.collect_var_usages_expr(or_fail, used);
                }
                if let Some(or_val) = &decl.or_value {
                    self.collect_var_usages_expr(or_val, used);
                }
            }
            Stmt::ConstDecl(decl) => {
                self.collect_var_usages_expr(&decl.init, used);
            }
            Stmt::Assign(assign) => {
                self.collect_var_usages_expr(&assign.target, used);
                self.collect_var_usages_expr(&assign.value, used);
            }
            Stmt::If(if_stmt) => {
                self.collect_var_usages_expr(&if_stmt.condition, used);
                match &if_stmt.then_branch {
                    IfBody::Block(b) => self.collect_var_usages_block(b, used),
                    IfBody::Stmt(s) => self.collect_var_usages_stmt(s, used),
                }
                if let Some(else_branch) = &if_stmt.else_branch {
                    match else_branch {
                        IfBody::Block(b) => self.collect_var_usages_block(b, used),
                        IfBody::Stmt(s) => self.collect_var_usages_stmt(s, used),
                    }
                }
            }
            Stmt::While(w) => {
                self.collect_var_usages_expr(&w.condition, used);
                self.collect_var_usages_block(&w.body, used);
            }
            Stmt::For(f) => {
                self.collect_var_usages_expr(&f.iterable, used);
                // The loop variable IS used if it appears in the body
                // We do NOT mark it as used here — the body usages will do that
                self.collect_var_usages_block(&f.body, used);
            }
            Stmt::Switch(sw) => {
                self.collect_var_usages_expr(&sw.discriminant, used);
                for case in &sw.cases {
                    self.collect_var_usages_expr(&case.value, used);
                    for s in &case.body {
                        self.collect_var_usages_stmt(s, used);
                    }
                }
                if let Some(default_stmts) = &sw.default {
                    for s in default_stmts {
                        self.collect_var_usages_stmt(s, used);
                    }
                }
            }
            Stmt::TryCatch(tc) => {
                self.collect_var_usages_block(&tc.try_block, used);
                used.insert(tc.catch_var.clone());
                self.collect_var_usages_block(&tc.catch_block, used);
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.collect_var_usages_expr(expr, used);
                }
            }
            Stmt::Fail(fail) => {
                self.collect_var_usages_expr(&fail.expr, used);
            }
            Stmt::Throw(throw) => {
                self.collect_var_usages_expr(&throw.expr, used);
            }
            Stmt::Expr(expr_stmt) => {
                self.collect_var_usages_expr(&expr_stmt.expr, used);
            }
            Stmt::Block(b) => {
                self.collect_var_usages_block(b, used);
            }
            Stmt::Defer(defer_stmt) => {
                self.collect_var_usages_stmt(&defer_stmt.body, used);
            }
            Stmt::Break | Stmt::Continue => {}
        }
    }

    fn collect_var_usages_expr(&self, expr: &Expr, used: &mut HashSet<String>) {
        match expr {
            Expr::Identifier(name) => {
                used.insert(name.clone());
            }
            Expr::Binary { left, right, .. } => {
                self.collect_var_usages_expr(left, used);
                self.collect_var_usages_expr(right, used);
            }
            Expr::Unary { operand, .. } => {
                self.collect_var_usages_expr(operand, used);
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.collect_var_usages_expr(condition, used);
                self.collect_var_usages_expr(then_expr, used);
                self.collect_var_usages_expr(else_expr, used);
            }
            Expr::Call(call) => {
                self.collect_var_usages_expr(&call.callee, used);
                for arg in &call.args {
                    self.collect_var_usages_expr(arg, used);
                }
            }
            Expr::MethodCall(mc) => {
                self.collect_var_usages_expr(&mc.object, used);
                for arg in &mc.args {
                    self.collect_var_usages_expr(arg, used);
                }
            }
            Expr::Member { object, .. } => {
                self.collect_var_usages_expr(object, used);
            }
            Expr::Index { object, index } => {
                self.collect_var_usages_expr(object, used);
                self.collect_var_usages_expr(index, used);
            }
            Expr::ObjectLiteral(fields) => {
                for (_, val) in fields {
                    self.collect_var_usages_expr(val, used);
                }
            }
            Expr::StructLiteral { fields, .. } => {
                for (_, val) in fields {
                    self.collect_var_usages_expr(val, used);
                }
            }
            Expr::ArrayLiteral(elems) => {
                for e in elems {
                    self.collect_var_usages_expr(e, used);
                }
            }
            Expr::MapLiteral(pairs) => {
                for (k, v) in pairs {
                    self.collect_var_usages_expr(k, used);
                    self.collect_var_usages_expr(v, used);
                }
            }
            Expr::SetLiteral(elems) => {
                for e in elems {
                    self.collect_var_usages_expr(e, used);
                }
            }
            Expr::Tuple(elems) => {
                for e in elems {
                    self.collect_var_usages_expr(e, used);
                }
            }
            Expr::Lambda(lambda) => match &lambda.body {
                LambdaBody::Expr(e) => self.collect_var_usages_expr(e, used),
                LambdaBody::Block(b) => self.collect_var_usages_block(b, used),
            },
            Expr::StringTemplate { parts } => {
                for part in parts {
                    if let StringTemplatePart::Expr(e) = part {
                        self.collect_var_usages_expr(e, used);
                    }
                }
            }
            Expr::Fail(e) => {
                self.collect_var_usages_expr(e, used);
            }
            Expr::Switch(sw) => {
                self.collect_var_usages_expr(&sw.discriminant, used);
                for arm in &sw.arms {
                    if let Some(guard) = &arm.guard {
                        self.collect_var_usages_expr(guard, used);
                    }
                    match &arm.body {
                        SwitchBody::Expr(e) => self.collect_var_usages_expr(e, used),
                        SwitchBody::Block(stmts) => {
                            for s in stmts {
                                self.collect_var_usages_stmt(s, used);
                            }
                        }
                    }
                }
            }
            Expr::MethodRef { object, .. } => {
                used.insert(object.clone());
            }
            Expr::Unwrap(inner) => {
                self.collect_var_usages_expr(inner, used);
            }
            Expr::Try(inner) => {
                self.collect_var_usages_expr(inner, used);
            }
            Expr::OptionalChain { object, .. } => {
                self.collect_var_usages_expr(object, used);
            }
            Expr::Literal(_) | Expr::RustBlock { .. } => {}
        }
    }

    // ───────────────────────────────────────────────────────────
    // W002: Unused imports
    // ───────────────────────────────────────────────────────────

    fn check_unused_imports(&mut self, program: &Program) {
        // Collect all imported symbols
        let mut imported_symbols: Vec<(String, String)> = Vec::new(); // (symbol, source)

        for item in &program.items {
            if let TopLevel::Import(import) = item {
                if import.is_wildcard {
                    // Can't track individual usage of wildcard imports
                    continue;
                }
                for sym in &import.imports {
                    imported_symbols.push((sym.clone(), import.source.clone()));
                }
            }
        }

        if imported_symbols.is_empty() {
            return;
        }

        // Collect all used identifiers in the program
        let mut used_names: HashSet<String> = HashSet::new();
        for item in &program.items {
            match item {
                TopLevel::Function(f) => {
                    if let Some(body) = &f.body {
                        self.collect_var_usages_block(body, &mut used_names);
                    }
                    if let Some(expr) = &f.expr_body {
                        self.collect_var_usages_expr(expr, &mut used_names);
                    }
                    // Check param types and return type for type references
                    self.collect_type_usages_function(f, &mut used_names);
                }
                TopLevel::Class(class) => {
                    // Class might reference imported types
                    used_names.insert(class.name.clone());
                    for member in &class.members {
                        match member {
                            Member::Method(method) => {
                                if let Some(body) = &method.body {
                                    self.collect_var_usages_block(body, &mut used_names);
                                }
                                if let Some(expr) = &method.expr_body {
                                    self.collect_var_usages_expr(expr, &mut used_names);
                                }
                            }
                            Member::Field(field) => {
                                if let Some(init) = &field.init {
                                    self.collect_var_usages_expr(init, &mut used_names);
                                }
                                if let Some(type_ref) = &field.type_ref {
                                    self.collect_type_ref_usages(type_ref, &mut used_names);
                                }
                            }
                        }
                    }
                }
                TopLevel::Test(test) => {
                    self.collect_var_usages_block(&test.body, &mut used_names);
                }
                TopLevel::ExprStmt(expr) => {
                    self.collect_var_usages_expr(expr, &mut used_names);
                }
                _ => {}
            }
        }

        // Report unused imports
        for (symbol, source) in &imported_symbols {
            if !used_names.contains(symbol) {
                let line = self.find_import_line(symbol);
                self.warnings.push(LintWarning {
                    code: "W002".to_string(),
                    title: "Unused import".to_string(),
                    message: format!("Import '{}' from \"{}\" is never used", symbol, source),
                    file: self.source_file.clone(),
                    line,
                    column: None,
                    source_line: self.source_line_at(line),
                    help: Some("Remove the unused import".to_string()),
                });
            }
        }
    }

    /// Collect type references used in function signatures (params + return type).
    fn collect_type_usages_function(&self, f: &FunctionDecl, used: &mut HashSet<String>) {
        for param in &f.params {
            if let Some(type_ref) = &param.type_ref {
                self.collect_type_ref_usages(type_ref, used);
            }
        }
        if let Some(ret) = &f.return_type {
            self.collect_type_ref_usages(ret, used);
        }
    }

    /// Collect type names referenced in a TypeRef.
    fn collect_type_ref_usages(&self, type_ref: &TypeRef, used: &mut HashSet<String>) {
        match type_ref {
            TypeRef::Simple(name) => {
                used.insert(name.clone());
            }
            TypeRef::Generic { base, args } => {
                used.insert(base.clone());
                for arg in args {
                    self.collect_type_ref_usages(arg, used);
                }
            }
            TypeRef::Array(inner) => self.collect_type_ref_usages(inner, used),
            TypeRef::Map(k, v) => {
                self.collect_type_ref_usages(k, used);
                self.collect_type_ref_usages(v, used);
            }
            TypeRef::Set(inner) => self.collect_type_ref_usages(inner, used),
            TypeRef::Optional(inner) => self.collect_type_ref_usages(inner, used),
            TypeRef::Fallible(inner) => self.collect_type_ref_usages(inner, used),
            TypeRef::Tuple(types) => {
                for t in types {
                    self.collect_type_ref_usages(t, used);
                }
            }
            TypeRef::Union(types) => {
                for t in types {
                    self.collect_type_ref_usages(t, used);
                }
            }
            TypeRef::Fn(args, ret) => {
                for a in args {
                    self.collect_type_ref_usages(a, used);
                }
                self.collect_type_ref_usages(ret, used);
            }
        }
    }

    // ───────────────────────────────────────────────────────────
    // W003: Unreachable code after return/fail
    // ───────────────────────────────────────────────────────────

    fn check_unreachable_code(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevel::Function(f) => {
                    if let Some(body) = &f.body {
                        self.check_unreachable_block(body, 0);
                    }
                }
                TopLevel::Class(class) => {
                    for member in &class.members {
                        if let Member::Method(method) = member {
                            if let Some(body) = &method.body {
                                self.check_unreachable_block(body, 0);
                            }
                        }
                    }
                }
                TopLevel::Test(test) => {
                    self.check_unreachable_block(&test.body, 0);
                }
                _ => {}
            }
        }
    }

    /// Check a block for statements after return/fail/break/continue.
    fn check_unreachable_block(&mut self, block: &BlockStmt, start_line: usize) {
        let mut found_terminator = false;
        let mut terminator_line = 0usize;
        let mut terminator_kind = "";

        for stmt in &block.stmts {
            if found_terminator {
                // This statement is unreachable
                let line = self.estimate_stmt_line(stmt, terminator_line);
                self.warnings.push(LintWarning {
                    code: "W003".to_string(),
                    title: "Unreachable code".to_string(),
                    message: format!("Code after '{}' will never be executed", terminator_kind),
                    file: self.source_file.clone(),
                    line,
                    column: None,
                    source_line: self.source_line_at(line),
                    help: Some("Remove unreachable code or restructure the logic".to_string()),
                });
                // Only report the first unreachable statement per block
                return;
            }

            match stmt {
                Stmt::Return(_) => {
                    found_terminator = true;
                    terminator_kind = "return";
                    terminator_line = self.estimate_stmt_line(stmt, start_line);
                }
                Stmt::Fail(_) => {
                    found_terminator = true;
                    terminator_kind = "fail";
                    terminator_line = self.estimate_stmt_line(stmt, start_line);
                }
                Stmt::Break => {
                    found_terminator = true;
                    terminator_kind = "break";
                    terminator_line = self.estimate_stmt_line(stmt, start_line);
                }
                Stmt::Continue => {
                    found_terminator = true;
                    terminator_kind = "continue";
                    terminator_line = self.estimate_stmt_line(stmt, start_line);
                }
                // Recurse into sub-blocks
                Stmt::If(if_stmt) => {
                    if let IfBody::Block(b) = &if_stmt.then_branch {
                        self.check_unreachable_block(b, start_line);
                    }
                    if let Some(IfBody::Block(b)) = &if_stmt.else_branch {
                        self.check_unreachable_block(b, start_line);
                    }
                }
                Stmt::While(w) => {
                    self.check_unreachable_block(&w.body, start_line);
                }
                Stmt::For(f) => {
                    self.check_unreachable_block(&f.body, start_line);
                }
                Stmt::TryCatch(tc) => {
                    self.check_unreachable_block(&tc.try_block, start_line);
                    self.check_unreachable_block(&tc.catch_block, start_line);
                }
                Stmt::Block(b) => {
                    self.check_unreachable_block(b, start_line);
                }
                Stmt::Defer(defer_stmt) => {
                    // defer is NOT a terminator, but recurse into its body
                    if let Stmt::Block(b) = defer_stmt.body.as_ref() {
                        self.check_unreachable_block(b, start_line);
                    }
                }
                _ => {}
            }
        }
    }

    /// Estimate the line number of a statement by looking at its content.
    fn estimate_stmt_line(&self, stmt: &Stmt, after_line: usize) -> usize {
        match stmt {
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    if let Some(pattern) = self.expr_search_pattern(expr) {
                        // Search for "return <pattern>"
                        return self.find_line_containing(
                            &format!("return {}", pattern),
                            after_line.saturating_sub(1),
                        );
                    }
                }
                self.find_line_containing("return", after_line.saturating_sub(1))
            }
            Stmt::Fail(fail) => {
                if fail.line > 0 {
                    return fail.line as usize;
                }
                self.find_line_containing("fail", after_line.saturating_sub(1))
            }
            Stmt::Break => self.find_line_containing("break", after_line.saturating_sub(1)),
            Stmt::Continue => self.find_line_containing("continue", after_line.saturating_sub(1)),
            Stmt::VarDecl(decl) => {
                if let Some(name) = decl.bindings.first().and_then(|b| b.name()) {
                    self.estimate_var_line(name, after_line.saturating_sub(1))
                } else {
                    after_line + 1
                }
            }
            Stmt::Expr(expr_stmt) => {
                if let Some(pattern) = self.expr_search_pattern(&expr_stmt.expr) {
                    self.find_line_containing(&pattern, after_line.saturating_sub(1))
                } else {
                    after_line + 1
                }
            }
            _ => after_line + 1,
        }
    }

    /// Get a search pattern from an expression for line-finding.
    fn expr_search_pattern(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Identifier(name) => Some(name.clone()),
            Expr::Literal(Literal::String(s)) => Some(format!("\"{}\"", s)),
            Expr::Literal(Literal::Int(n)) => Some(n.to_string()),
            Expr::Call(call) => self.expr_search_pattern(&call.callee),
            Expr::Member { object, property } => self
                .expr_search_pattern(object)
                .map(|o| format!("{}.{}", o, property)),
            _ => None,
        }
    }

    // ───────────────────────────────────────────────────────────
    // W004: Comparison is always true / always false
    // ───────────────────────────────────────────────────────────

    fn check_always_true_false(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevel::Function(f) => {
                    if let Some(body) = &f.body {
                        self.check_always_tf_block(body, 0);
                    }
                    if let Some(expr) = &f.expr_body {
                        self.check_always_tf_expr(expr, 0);
                    }
                }
                TopLevel::Class(class) => {
                    for member in &class.members {
                        if let Member::Method(method) = member {
                            if let Some(body) = &method.body {
                                self.check_always_tf_block(body, 0);
                            }
                            if let Some(expr) = &method.expr_body {
                                self.check_always_tf_expr(expr, 0);
                            }
                        }
                    }
                }
                TopLevel::Test(test) => {
                    self.check_always_tf_block(&test.body, 0);
                }
                _ => {}
            }
        }
    }

    fn check_always_tf_block(&mut self, block: &BlockStmt, start_line: usize) {
        for stmt in &block.stmts {
            self.check_always_tf_stmt(stmt, start_line);
        }
    }

    fn check_always_tf_stmt(&mut self, stmt: &Stmt, start_line: usize) {
        match stmt {
            Stmt::If(if_stmt) => {
                self.check_always_tf_expr(&if_stmt.condition, start_line);
                if let IfBody::Block(b) = &if_stmt.then_branch {
                    self.check_always_tf_block(b, start_line);
                }
                if let Some(IfBody::Block(b)) = &if_stmt.else_branch {
                    self.check_always_tf_block(b, start_line);
                }
            }
            Stmt::While(w) => {
                self.check_always_tf_expr(&w.condition, start_line);
                self.check_always_tf_block(&w.body, start_line);
            }
            Stmt::For(f) => {
                self.check_always_tf_block(&f.body, start_line);
            }
            Stmt::VarDecl(decl) => {
                self.check_always_tf_expr(&decl.init, start_line);
            }
            Stmt::Expr(expr_stmt) => {
                self.check_always_tf_expr(&expr_stmt.expr, start_line);
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.check_always_tf_expr(expr, start_line);
                }
            }
            Stmt::Block(b) => {
                self.check_always_tf_block(b, start_line);
            }
            Stmt::TryCatch(tc) => {
                self.check_always_tf_block(&tc.try_block, start_line);
                self.check_always_tf_block(&tc.catch_block, start_line);
            }
            Stmt::Defer(defer_stmt) => {
                self.check_always_tf_stmt(&defer_stmt.body, start_line);
            }
            _ => {}
        }
    }

    fn check_always_tf_expr(&mut self, expr: &Expr, start_line: usize) {
        if let Expr::Binary { op, left, right } = expr {
            // Check for comparisons between identical expressions
            if matches!(
                op,
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
            ) {
                // Case 1: Same identifier on both sides (x == x)
                if left == right {
                    let (always_result, description) = match op {
                        BinOp::Eq | BinOp::Le | BinOp::Ge => ("true", "always true"),
                        BinOp::Ne | BinOp::Lt | BinOp::Gt => ("false", "always false"),
                        _ => unreachable!(),
                    };
                    let expr_str = self.expr_display(left);
                    let search = format!("{} {} {}", &expr_str, op, &expr_str);
                    let line = self.find_line_containing(&search, start_line.saturating_sub(1));
                    self.warnings.push(LintWarning {
                        code: "W004".to_string(),
                        title: format!("Comparison is {}", description),
                        message: format!(
                            "Comparing '{}' with itself using '{}' is {}",
                            expr_str, op, description
                        ),
                        file: self.source_file.clone(),
                        line,
                        column: None,
                        source_line: self.source_line_at(line),
                        help: Some(format!(
                            "This comparison always evaluates to {}",
                            always_result
                        )),
                    });
                }

                // Case 2: Comparing two literals (42 == 42, "a" != "a")
                if let (Expr::Literal(left_lit), Expr::Literal(right_lit)) =
                    (left.as_ref(), right.as_ref())
                {
                    if left != right {
                        // Different literals compared — we can determine the result
                        let (always_result, description) = match op {
                            BinOp::Eq => ("false", "always false"),
                            BinOp::Ne => ("true", "always true"),
                            _ => return, // Don't flag < > <= >= for different literals
                        };
                        let left_str = self.literal_display(left_lit);
                        let right_str = self.literal_display(right_lit);
                        let search = format!("{} {} {}", &left_str, op, &right_str);
                        let line = self.find_line_containing(&search, start_line.saturating_sub(1));
                        self.warnings.push(LintWarning {
                            code: "W004".to_string(),
                            title: format!("Comparison is {}", description),
                            message: format!(
                                "Comparing literal {} with {} is {}",
                                left_str, right_str, description
                            ),
                            file: self.source_file.clone(),
                            line,
                            column: None,
                            source_line: self.source_line_at(line),
                            help: Some(format!(
                                "This comparison always evaluates to {}",
                                always_result
                            )),
                        });
                    }
                }

                // Case 3: true == true, false == false (bool literal comparisons)
                if let (Expr::Literal(Literal::Bool(l)), Expr::Literal(Literal::Bool(r))) =
                    (left.as_ref(), right.as_ref())
                {
                    // Already caught by left == right above, but also catch true == false etc.
                    // This case is fully covered by the literal comparison above
                    let _ = (l, r); // suppress unused warning
                }
            }

            // Recurse into sub-expressions
            self.check_always_tf_expr(left, start_line);
            self.check_always_tf_expr(right, start_line);
        }

        // Recurse into other compound expressions
        match expr {
            Expr::Unary { operand, .. } => self.check_always_tf_expr(operand, start_line),
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.check_always_tf_expr(condition, start_line);
                self.check_always_tf_expr(then_expr, start_line);
                self.check_always_tf_expr(else_expr, start_line);
            }
            Expr::Call(call) => {
                for arg in &call.args {
                    self.check_always_tf_expr(arg, start_line);
                }
            }
            Expr::MethodCall(mc) => {
                for arg in &mc.args {
                    self.check_always_tf_expr(arg, start_line);
                }
            }
            _ => {}
        }
    }

    /// Get a displayable string for an expression (for warning messages).
    fn expr_display(&self, expr: &Expr) -> String {
        match expr {
            Expr::Identifier(name) => name.clone(),
            Expr::Literal(lit) => self.literal_display(lit),
            Expr::Member { object, property } => {
                format!("{}.{}", self.expr_display(object), property)
            }
            _ => "<expr>".to_string(),
        }
    }

    fn literal_display(&self, lit: &Literal) -> String {
        match lit {
            Literal::Int(n) => n.to_string(),
            Literal::Float(f) => format!("{}", f),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Bool(b) => b.to_string(),
            Literal::Char(c) => format!("'{}'", c),
            Literal::Null => "null".to_string(),
        }
    }
}

// ──────────────────────────────────────────────────────────────────
// W005 / W006 / W007 — additions
// ──────────────────────────────────────────────────────────────────

impl Linter {
    // ───────────────────────────────────────────────────────────
    // W005: Variable shadows an outer-scope binding
    // ───────────────────────────────────────────────────────────

    fn check_shadowed_variables(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevel::Function(f) => {
                    let mut scopes: Vec<HashSet<String>> = vec![HashSet::new()];
                    for param in &f.params {
                        if let Some(name) = param.name() {
                            scopes[0].insert(name.to_string());
                        }
                    }
                    if let Some(body) = &f.body {
                        self.shadow_walk_block(body, &mut scopes, 0);
                    }
                }
                TopLevel::Class(class) => {
                    for member in &class.members {
                        if let Member::Method(method) = member {
                            let mut scopes: Vec<HashSet<String>> = vec![HashSet::new()];
                            for param in &method.params {
                                if let Some(name) = param.name() {
                                    scopes[0].insert(name.to_string());
                                }
                            }
                            if let Some(body) = &method.body {
                                self.shadow_walk_block(body, &mut scopes, 0);
                            }
                        }
                    }
                }
                TopLevel::Test(test) => {
                    let mut scopes: Vec<HashSet<String>> = vec![HashSet::new()];
                    self.shadow_walk_block(&test.body, &mut scopes, 0);
                }
                _ => {}
            }
        }
    }

    fn shadow_walk_block(
        &mut self,
        block: &BlockStmt,
        scopes: &mut Vec<HashSet<String>>,
        last_line: usize,
    ) {
        scopes.push(HashSet::new());
        let mut last_line = last_line;
        for stmt in &block.stmts {
            self.shadow_walk_stmt(stmt, scopes, &mut last_line);
        }
        scopes.pop();
    }

    fn shadow_walk_stmt(
        &mut self,
        stmt: &Stmt,
        scopes: &mut Vec<HashSet<String>>,
        last_line: &mut usize,
    ) {
        match stmt {
            Stmt::VarDecl(decl) => {
                for binding in &decl.bindings {
                    self.shadow_check_pattern(&binding.pattern, scopes, last_line);
                }
            }
            Stmt::ConstDecl(decl) => {
                let name = &decl.name;
                self.shadow_report_if_outer(name, scopes, last_line);
                if let Some(top) = scopes.last_mut() {
                    top.insert(name.clone());
                }
            }
            Stmt::If(if_stmt) => {
                if let IfBody::Block(b) = &if_stmt.then_branch {
                    self.shadow_walk_block(b, scopes, *last_line);
                }
                if let Some(else_branch) = &if_stmt.else_branch {
                    if let IfBody::Block(b) = else_branch {
                        self.shadow_walk_block(b, scopes, *last_line);
                    }
                }
            }
            Stmt::While(w) => {
                self.shadow_walk_block(&w.body, scopes, *last_line);
            }
            Stmt::For(for_stmt) => {
                // for-bound variables introduce a new scope; check shadowing
                scopes.push(HashSet::new());
                self.shadow_report_if_outer(&for_stmt.var, scopes, last_line);
                if let Some(top) = scopes.last_mut() {
                    top.insert(for_stmt.var.clone());
                }
                if let Some(v2) = &for_stmt.var2 {
                    self.shadow_report_if_outer(v2, scopes, last_line);
                    if let Some(top) = scopes.last_mut() {
                        top.insert(v2.clone());
                    }
                }
                self.shadow_walk_block(&for_stmt.body, scopes, *last_line);
                scopes.pop();
            }
            Stmt::Block(b) => {
                self.shadow_walk_block(b, scopes, *last_line);
            }
            Stmt::TryCatch(tc) => {
                self.shadow_walk_block(&tc.try_block, scopes, *last_line);
                self.shadow_walk_block(&tc.catch_block, scopes, *last_line);
            }
            _ => {}
        }
    }

    fn shadow_check_pattern(
        &mut self,
        pattern: &BindingPattern,
        scopes: &mut Vec<HashSet<String>>,
        last_line: &mut usize,
    ) {
        let names = collect_pattern_names(pattern);
        for name in names {
            self.shadow_report_if_outer(&name, scopes, last_line);
            if let Some(top) = scopes.last_mut() {
                top.insert(name);
            }
        }
    }

    fn shadow_report_if_outer(
        &mut self,
        name: &str,
        scopes: &Vec<HashSet<String>>,
        last_line: &mut usize,
    ) {
        if name.starts_with('_') {
            return;
        }
        // Look in all outer scopes (everything except the current top)
        let depth = scopes.len();
        if depth < 2 {
            return;
        }
        let shadowed = scopes[..depth - 1].iter().any(|s| s.contains(name));
        if !shadowed {
            return;
        }

        let line = self.estimate_var_line(name, *last_line);
        *last_line = line;
        self.warnings.push(LintWarning {
            code: "W005".to_string(),
            title: "Shadowed variable".to_string(),
            message: format!("'{}' shadows a binding from an outer scope", name),
            file: self.source_file.clone(),
            line,
            column: None,
            source_line: self.source_line_at(line),
            help: Some(format!(
                "Rename this binding (e.g. '{}_inner') or prefix with '_' to suppress.",
                name
            )),
        });
    }

    // ───────────────────────────────────────────────────────────
    // W006: Empty block
    // ───────────────────────────────────────────────────────────

    fn check_empty_blocks(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevel::Function(f) => {
                    if let Some(body) = &f.body {
                        self.empty_walk_block(body, 0);
                    }
                }
                TopLevel::Class(class) => {
                    for member in &class.members {
                        if let Member::Method(method) = member {
                            if let Some(body) = &method.body {
                                self.empty_walk_block(body, 0);
                            }
                        }
                    }
                }
                TopLevel::Test(test) => {
                    self.empty_walk_block(&test.body, 0);
                }
                _ => {}
            }
        }
    }

    fn empty_walk_block(&mut self, block: &BlockStmt, last_line: usize) {
        let mut last_line = last_line;
        for stmt in &block.stmts {
            self.empty_check_stmt(stmt, &mut last_line);
        }
    }

    fn empty_check_stmt(&mut self, stmt: &Stmt, last_line: &mut usize) {
        match stmt {
            Stmt::If(if_stmt) => {
                if let IfBody::Block(b) = &if_stmt.then_branch {
                    if b.stmts.is_empty() {
                        let line = self.find_line_containing("if ", *last_line);
                        *last_line = line;
                        self.warn_empty("if", line);
                    } else {
                        self.empty_walk_block(b, *last_line);
                    }
                }
                if let Some(IfBody::Block(b)) = &if_stmt.else_branch {
                    if b.stmts.is_empty() {
                        let line = self.find_line_containing("else", *last_line);
                        *last_line = line;
                        self.warn_empty("else", line);
                    } else {
                        self.empty_walk_block(b, *last_line);
                    }
                }
            }
            Stmt::While(w) => {
                if w.body.stmts.is_empty() {
                    let line = self.find_line_containing("while ", *last_line);
                    *last_line = line;
                    self.warn_empty("while", line);
                } else {
                    self.empty_walk_block(&w.body, *last_line);
                }
            }
            Stmt::For(for_stmt) => {
                if for_stmt.body.stmts.is_empty() {
                    let line = self.find_line_containing(
                        &format!("for {}", for_stmt.var),
                        *last_line,
                    );
                    *last_line = line;
                    self.warn_empty("for", line);
                } else {
                    self.empty_walk_block(&for_stmt.body, *last_line);
                }
            }
            Stmt::Block(b) => {
                if !b.stmts.is_empty() {
                    self.empty_walk_block(b, *last_line);
                }
            }
            Stmt::TryCatch(tc) => {
                self.empty_walk_block(&tc.try_block, *last_line);
                self.empty_walk_block(&tc.catch_block, *last_line);
            }
            _ => {}
        }
    }

    fn warn_empty(&mut self, kind: &str, line: usize) {
        self.warnings.push(LintWarning {
            code: "W006".to_string(),
            title: "Empty block".to_string(),
            message: format!("Empty '{}' block", kind),
            file: self.source_file.clone(),
            line,
            column: None,
            source_line: self.source_line_at(line),
            help: Some(
                "Remove the block, or add a comment explaining why it is intentionally empty."
                    .to_string(),
            ),
        });
    }

    // ───────────────────────────────────────────────────────────
    // W007: Function parameter declared but never used
    // ───────────────────────────────────────────────────────────

    fn check_unused_parameters(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevel::Function(f) => {
                    self.check_params(&f.name, &f.params, f.body.as_ref(), f.expr_body.as_ref());
                }
                TopLevel::Class(class) => {
                    for member in &class.members {
                        if let Member::Method(method) = member {
                            // Skip interface impl candidates if the body is empty —
                            // they're stubs by design. We still warn on real bodies.
                            self.check_params(
                                &method.name,
                                &method.params,
                                method.body.as_ref(),
                                method.expr_body.as_ref(),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn check_params(
        &mut self,
        fn_name: &str,
        params: &[Param],
        body: Option<&BlockStmt>,
        expr_body: Option<&Expr>,
    ) {
        if params.is_empty() {
            return;
        }
        let mut used: HashSet<String> = HashSet::new();
        if let Some(b) = body {
            self.collect_var_usages_block(b, &mut used);
        }
        if let Some(e) = expr_body {
            self.collect_var_usages_expr(e, &mut used);
        }

        for param in params {
            let Some(name) = param.name() else { continue };
            if name.starts_with('_') || name == "self" {
                continue;
            }
            if used.contains(name) {
                continue;
            }
            let line = self.find_line_containing(&format!("fn {}", fn_name), 0);
            let line = if line == 1 {
                self.find_line_containing(fn_name, 0)
            } else {
                line
            };
            self.warnings.push(LintWarning {
                code: "W007".to_string(),
                title: "Unused parameter".to_string(),
                message: format!("Parameter '{}' of '{}' is never used", name, fn_name),
                file: self.source_file.clone(),
                line,
                column: None,
                source_line: self.source_line_at(line),
                help: Some(format!(
                    "Prefix with underscore to suppress: _{}",
                    name
                )),
            });
        }
    }
}

// ───────────────────────────────────────────────────────────
// W008: Unnecessary `else` after a diverging branch
// ───────────────────────────────────────────────────────────

impl Linter {
    fn check_redundant_else(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevel::Function(f) => {
                    if let Some(body) = &f.body {
                        self.redundant_else_walk_block(body);
                    }
                }
                TopLevel::Class(class) => {
                    for member in &class.members {
                        if let Member::Method(m) = member {
                            if let Some(body) = &m.body {
                                self.redundant_else_walk_block(body);
                            }
                        }
                    }
                }
                TopLevel::Test(t) => self.redundant_else_walk_block(&t.body),
                _ => {}
            }
        }
    }

    fn redundant_else_walk_block(&mut self, block: &BlockStmt) {
        for stmt in &block.stmts {
            self.redundant_else_walk_stmt(stmt);
        }
    }

    fn redundant_else_walk_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::If(if_stmt) => {
                // Recurse first so nested if/else are also checked.
                Self::walk_if_body(self, &if_stmt.then_branch);
                if let Some(else_branch) = &if_stmt.else_branch {
                    Self::walk_if_body(self, else_branch);

                    // The diverging-then check: if the then-branch is a block
                    // that ends with a diverging statement, the else is
                    // redundant (the else body can be dedented one level).
                    if let IfBody::Block(b) = &if_stmt.then_branch {
                        if block_diverges(b) {
                            let line = self.find_line_containing("else", 0);
                            self.warn_redundant_else(line);
                        }
                    } else if let IfBody::Stmt(s) = &if_stmt.then_branch {
                        if stmt_diverges(s) {
                            let line = self.find_line_containing("else", 0);
                            self.warn_redundant_else(line);
                        }
                    }
                }
            }
            Stmt::While(w) => self.redundant_else_walk_block(&w.body),
            Stmt::For(f) => self.redundant_else_walk_block(&f.body),
            Stmt::Block(b) => self.redundant_else_walk_block(b),
            Stmt::TryCatch(tc) => {
                self.redundant_else_walk_block(&tc.try_block);
                self.redundant_else_walk_block(&tc.catch_block);
            }
            _ => {}
        }
    }

    fn walk_if_body(&mut self, body: &IfBody) {
        match body {
            IfBody::Block(b) => self.redundant_else_walk_block(b),
            IfBody::Stmt(s) => self.redundant_else_walk_stmt(s),
        }
    }

    fn warn_redundant_else(&mut self, line: usize) {
        self.warnings.push(LintWarning {
            code: "W008".to_string(),
            title: "Unnecessary else".to_string(),
            message: "Else branch is unnecessary because the then-branch always diverges"
                .to_string(),
            file: self.source_file.clone(),
            line,
            column: None,
            source_line: self.source_line_at(line),
            help: Some(
                "Drop the `else` and dedent its body — execution can only reach it when \
                 the `if` condition was false."
                    .to_string(),
            ),
        });
    }
}

/// Returns true if a block's execution always leaves the enclosing function /
/// loop (i.e. its last statement is `return`, `throw`, `fail`, `break`, or
/// `continue`). Used by W008 to detect a redundant `else`.
fn block_diverges(block: &BlockStmt) -> bool {
    block.stmts.last().map(stmt_diverges).unwrap_or(false)
}

fn stmt_diverges(stmt: &Stmt) -> bool {
    matches!(
        stmt,
        Stmt::Return(_) | Stmt::Throw(_) | Stmt::Fail(_) | Stmt::Break | Stmt::Continue
    )
}

/// Collect all identifier names introduced by a binding pattern.
fn collect_pattern_names(pattern: &BindingPattern) -> Vec<String> {
    let mut out = Vec::new();
    match pattern {
        BindingPattern::Identifier(name) => out.push(name.clone()),
        BindingPattern::Object(obj) => {
            for field in &obj.fields {
                out.push(field.binding.clone());
            }
        }
        BindingPattern::Array(arr) => {
            for elem in &arr.elements {
                if let Some(name) = elem {
                    out.push(name.clone());
                }
            }
            if let Some(rest) = &arr.rest {
                out.push(rest.clone());
            }
        }
        BindingPattern::Tuple(tup) => {
            for name in &tup.elements {
                out.push(name.clone());
            }
        }
    }
    out
}

// ──────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────

/// Run the linter on a parsed program and return warnings.
pub fn lint(program: &Program, source_file: &str, source_code: &str) -> Vec<LintWarning> {
    let mut linter = Linter::new(source_file.to_string(), source_code.to_string());
    linter.lint(program)
}

/// Format lint warnings for terminal display.
pub fn format_warnings(warnings: &[LintWarning]) -> String {
    if warnings.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    for w in warnings {
        output.push_str(&w.format());
        output.push('\n');
    }
    output.push_str(&format!(
        "{}\n",
        format!(
            "{} warning{} emitted",
            warnings.len(),
            if warnings.len() == 1 { "" } else { "s" }
        )
        .yellow()
        .bold()
    ));
    output
}

/// Format lint warnings as JSON array.
pub fn format_warnings_json(warnings: &[LintWarning]) -> String {
    serde_json::to_string_pretty(warnings).unwrap_or_else(|_| "[]".to_string())
}
