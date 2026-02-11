/// Liva Code Formatter
///
/// Formats Liva source code according to the language's canonical style rules:
/// - 4-space indentation
/// - Consistent spacing around operators
/// - Blank lines between top-level declarations
/// - Consistent brace positioning (same-line opening braces)
/// - Preserved comments at their original positions
/// - Line wrapping at max_width for long lines
/// - Auto-simplification to Liva idioms (one-liners with =>)
///
/// # Architecture
///
/// The formatter works by:
/// 1. Extracting comments with their line numbers from the original source
/// 2. Parsing the source into an AST (reusing the compiler pipeline)
/// 3. Pretty-printing the AST back to source with canonical formatting
/// 4. Reinserting comments at their original relative positions

use crate::ast::*;
use crate::error::{CompilerError, Result};
use crate::{lexer, parser};

/// Formatting configuration options
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Number of spaces per indentation level (default: 4)
    pub indent_size: usize,
    /// Maximum line width before wrapping (default: 100)
    pub max_width: usize,
    /// Use `and`/`or`/`not` instead of `&&`/`||`/`!` (default: false)
    pub prefer_word_operators: bool,
    /// Trailing newline at end of file (default: true)
    pub trailing_newline: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        FormatOptions {
            indent_size: 4,
            max_width: 100,
            prefer_word_operators: true,
            trailing_newline: true,
        }
    }
}

/// A comment extracted from the original source
#[derive(Debug, Clone)]
struct SourceComment {
    /// The comment text including // prefix
    text: String,
    /// Original line number (0-based)
    line: usize,
    /// Whether this comment is on its own line (standalone) vs after code (inline)
    is_standalone: bool,
    /// The trimmed code on the same line (only for inline comments)
    code_on_line: String,
}

/// Formatter state
struct Formatter {
    options: FormatOptions,
    output: String,
    indent_level: usize,
}

impl Formatter {
    fn new(options: FormatOptions) -> Self {
        Formatter {
            options,
            output: String::new(),
            indent_level: 0,
        }
    }

    /// Get the current indentation string
    fn indent(&self) -> String {
        " ".repeat(self.options.indent_size * self.indent_level)
    }

    /// Current column position at the current indent level
    fn current_indent_width(&self) -> usize {
        self.options.indent_size * self.indent_level
    }

    /// Write a line with current indentation
    fn write_line(&mut self, text: &str) {
        if text.is_empty() {
            self.output.push('\n');
        } else {
            self.output.push_str(&self.indent());
            self.output.push_str(text);
            self.output.push('\n');
        }
    }

    /// Write a blank line (avoiding double blanks)
    fn blank_line(&mut self) {
        if !self.output.ends_with("\n\n") {
            self.output.push('\n');
        }
    }

    /// Check if a string would exceed max_width at current indent
    fn would_exceed_width(&self, text: &str) -> bool {
        self.current_indent_width() + text.len() > self.options.max_width
    }

    // ======================================================================
    // Top-level formatting
    // ======================================================================

    fn format_program(&mut self, program: &Program) {
        let total = program.items.len();
        for (i, item) in program.items.iter().enumerate() {
            self.format_top_level(item);
            // Add blank line between items, but group consecutive imports
            if i + 1 < total {
                let next = &program.items[i + 1];
                let both_imports = matches!(item, TopLevel::Import(_))
                    && matches!(next, TopLevel::Import(_));
                if !both_imports {
                    self.blank_line();
                }
            }
        }

        if self.options.trailing_newline && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn format_top_level(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Import(decl) => self.format_import(decl),
            TopLevel::UseRust(decl) => self.format_use_rust(decl),
            TopLevel::Type(decl) => self.format_type_decl(decl),
            TopLevel::TypeAlias(decl) => self.format_type_alias(decl),
            TopLevel::Class(decl) => self.format_class(decl),
            TopLevel::Function(decl) => self.format_function(decl),
            TopLevel::Test(decl) => self.format_test(decl),
            TopLevel::ConstDecl(decl) => self.format_const_decl_stmt(decl),
            TopLevel::ExprStmt(_) => { /* top-level expressions don't need formatting */ }
        }
    }

    // ======================================================================
    // Import declarations
    // ======================================================================

    fn format_import(&mut self, decl: &ImportDecl) {
        if decl.is_wildcard {
            if let Some(alias) = &decl.alias {
                self.write_line(&format!("import * as {} from \"{}\"", alias, decl.source));
            } else {
                self.write_line(&format!("import * from \"{}\"", decl.source));
            }
        } else if decl.imports.len() == 1 {
            self.write_line(&format!(
                "import {{ {} }} from \"{}\"",
                decl.imports[0], decl.source
            ));
        } else {
            let single_line = format!(
                "import {{ {} }} from \"{}\"",
                decl.imports.join(", "),
                decl.source
            );
            if !self.would_exceed_width(&single_line) {
                self.write_line(&single_line);
            } else {
                // Multi-line imports
                self.write_line("import {");
                self.indent_level += 1;
                for (i, import) in decl.imports.iter().enumerate() {
                    if i + 1 < decl.imports.len() {
                        self.write_line(&format!("{},", import));
                    } else {
                        self.write_line(import);
                    }
                }
                self.indent_level -= 1;
                self.write_line(&format!("}} from \"{}\"", decl.source));
            }
        }
    }

    fn format_use_rust(&mut self, decl: &UseRustDecl) {
        if let Some(alias) = &decl.alias {
            self.write_line(&format!("use rust {} as {}", decl.crate_name, alias));
        } else {
            self.write_line(&format!("use rust {}", decl.crate_name));
        }
    }

    // ======================================================================
    // Type declarations
    // ======================================================================

    fn format_type_decl(&mut self, decl: &TypeDecl) {
        let type_params = self.format_type_params(&decl.type_params);
        self.write_line(&format!("{}{} {{", decl.name, type_params));
        self.indent_level += 1;
        self.format_members(&decl.members);
        self.indent_level -= 1;
        self.write_line("}");
    }

    fn format_type_alias(&mut self, decl: &TypeAliasDecl) {
        let type_params = self.format_type_params(&decl.type_params);
        let target = self.format_type_ref(&decl.target_type);
        self.write_line(&format!("type {}{} = {}", decl.name, type_params, target));
    }

    // ======================================================================
    // Class declarations
    // ======================================================================

    fn format_class(&mut self, decl: &ClassDecl) {
        let type_params = self.format_type_params(&decl.type_params);
        let implements = if decl.implements.is_empty() {
            String::new()
        } else {
            format!(" : {}", decl.implements.join(", "))
        };

        self.write_line(&format!("{}{}{} {{", decl.name, type_params, implements));
        self.indent_level += 1;
        self.format_members(&decl.members);
        self.indent_level -= 1;
        self.write_line("}");
    }

    fn format_members(&mut self, members: &[Member]) {
        let mut last_kind: Option<&str> = None;
        let mut first = true;

        for member in members {
            match member {
                Member::Field(field) => {
                    // Add blank line when transitioning from methods back to fields
                    if !first && last_kind == Some("method") {
                        self.blank_line();
                    }
                    self.format_field(field);
                    last_kind = Some("field");
                }
                Member::Method(method) => {
                    if !first {
                        self.blank_line();
                    }
                    self.format_method(method);
                    last_kind = Some("method");
                }
            }
            first = false;
        }
    }

    fn format_field(&mut self, field: &FieldDecl) {
        let name = &field.name;
        let optional = if field.is_optional { "?" } else { "" };
        if let Some(type_ref) = &field.type_ref {
            let ty = self.format_type_ref(type_ref);
            if let Some(init) = &field.init {
                let expr = self.format_expr(init);
                self.write_line(&format!("{}{}: {} = {}", name, optional, ty, expr));
            } else {
                self.write_line(&format!("{}{}: {}", name, optional, ty));
            }
        } else if let Some(init) = &field.init {
            let expr = self.format_expr(init);
            self.write_line(&format!("{}{} = {}", name, optional, expr));
        } else {
            self.write_line(&format!("{}{}", name, optional));
        }
    }

    fn format_method(&mut self, method: &MethodDecl) {
        let type_params = self.format_type_params(&method.type_params);
        let ret_type = method
            .return_type
            .as_ref()
            .map(|t| format!(": {}", self.format_type_ref(t)))
            .unwrap_or_default();

        // Try to simplify { return expr } to => expr
        if let Some(block) = &method.body {
            if let Some(expr) = self.try_extract_single_return(block) {
                let body = self.format_expr(&expr);
                let params_str = self.format_params_simple(&method.params);
                let line = format!(
                    "{}{}({}){} => {}",
                    method.name, type_params, params_str, ret_type, body
                );
                if self.would_exceed_width(&line) {
                    self.write_method_multiline_params(
                        &method.name,
                        &type_params,
                        &method.params,
                        &ret_type,
                        Some(&body),
                        None,
                    );
                } else {
                    self.write_line(&line);
                }
                return;
            }
        }

        if let Some(expr) = &method.expr_body {
            let body = self.format_expr(expr);
            let params_str = self.format_params_simple(&method.params);
            let line = format!(
                "{}{}({}){} => {}",
                method.name, type_params, params_str, ret_type, body
            );
            if self.would_exceed_width(&line) {
                self.write_method_multiline_params(
                    &method.name,
                    &type_params,
                    &method.params,
                    &ret_type,
                    Some(&body),
                    None,
                );
            } else {
                self.write_line(&line);
            }
        } else if let Some(block) = &method.body {
            let params_str = self.format_params_simple(&method.params);
            let header = format!(
                "{}{}({}){} {{",
                method.name, type_params, params_str, ret_type
            );
            if self.would_exceed_width(&header) {
                self.write_method_multiline_params(
                    &method.name,
                    &type_params,
                    &method.params,
                    &ret_type,
                    None,
                    Some(block),
                );
            } else {
                self.write_line(&header);
                self.indent_level += 1;
                self.format_block(block);
                self.indent_level -= 1;
                self.write_line("}");
            }
        } else {
            // Interface method (no body)
            let params_str = self.format_params_simple(&method.params);
            self.write_line(&format!(
                "{}{}({}){}",
                method.name, type_params, params_str, ret_type
            ));
        }
    }

    /// Write a method/function with multiline params when the header exceeds max_width
    fn write_method_multiline_params(
        &mut self,
        name: &str,
        type_params: &str,
        params: &[Param],
        ret_type: &str,
        expr_body: Option<&str>,
        block_body: Option<&BlockStmt>,
    ) {
        self.write_line(&format!("{}{}(", name, type_params));
        self.indent_level += 1;
        for (i, p) in params.iter().enumerate() {
            let param_str = self.format_param(p);
            if i + 1 < params.len() {
                self.write_line(&format!("{},", param_str));
            } else {
                self.write_line(&param_str);
            }
        }
        self.indent_level -= 1;
        if let Some(body) = expr_body {
            self.write_line(&format!("){} => {}", ret_type, body));
        } else if let Some(block) = block_body {
            self.write_line(&format!("){} {{", ret_type));
            self.indent_level += 1;
            self.format_block(block);
            self.indent_level -= 1;
            self.write_line("}");
        }
    }

    // ======================================================================
    // Function declarations
    // ======================================================================

    fn format_function(&mut self, decl: &FunctionDecl) {
        let type_params = self.format_type_params(&decl.type_params);
        let ret_type = decl
            .return_type
            .as_ref()
            .map(|t| format!(": {}", self.format_type_ref(t)))
            .unwrap_or_default();

        // Try to simplify { return expr } to => expr
        if let Some(block) = &decl.body {
            if let Some(expr) = self.try_extract_single_return(block) {
                let body = self.format_expr(&expr);
                let params_str = self.format_params_simple(&decl.params);
                let line = format!(
                    "{}{}({}){} => {}",
                    decl.name, type_params, params_str, ret_type, body
                );
                if self.would_exceed_width(&line) {
                    self.write_method_multiline_params(
                        &decl.name,
                        &type_params,
                        &decl.params,
                        &ret_type,
                        Some(&body),
                        None,
                    );
                } else {
                    self.write_line(&line);
                }
                return;
            }
        }

        if let Some(expr) = &decl.expr_body {
            let body = self.format_expr(expr);
            let params_str = self.format_params_simple(&decl.params);
            let line = format!(
                "{}{}({}){} => {}",
                decl.name, type_params, params_str, ret_type, body
            );
            if self.would_exceed_width(&line) {
                self.write_method_multiline_params(
                    &decl.name,
                    &type_params,
                    &decl.params,
                    &ret_type,
                    Some(&body),
                    None,
                );
            } else {
                self.write_line(&line);
            }
        } else if let Some(block) = &decl.body {
            let params_str = self.format_params_simple(&decl.params);
            let header = format!(
                "{}{}({}){} {{",
                decl.name, type_params, params_str, ret_type
            );
            if self.would_exceed_width(&header) {
                self.write_method_multiline_params(
                    &decl.name,
                    &type_params,
                    &decl.params,
                    &ret_type,
                    None,
                    Some(block),
                );
            } else {
                self.write_line(&header);
                self.indent_level += 1;
                self.format_block(block);
                self.indent_level -= 1;
                self.write_line("}");
            }
        }
    }

    fn format_test(&mut self, decl: &TestDecl) {
        self.write_line(&format!("test \"{}\" {{", decl.name));
        self.indent_level += 1;
        self.format_block(&decl.body);
        self.indent_level -= 1;
        self.write_line("}");
    }

    // ======================================================================
    // Simplification helpers
    // ======================================================================

    /// If a block is { return expr }, extract the expr for one-liner simplification
    fn try_extract_single_return(&self, block: &BlockStmt) -> Option<Expr> {
        if block.stmts.len() != 1 {
            return None;
        }
        match &block.stmts[0] {
            Stmt::Return(ret) => ret.expr.clone(),
            _ => None,
        }
    }

    /// Simplify condition expressions to idiomatic Liva:
    /// - `expr != ""` → `expr` (truthy check)
    /// - `expr == ""` → `not expr` / `!expr` (falsy check)
    fn try_simplify_condition(&self, expr: &Expr) -> Option<String> {
        if let Expr::Binary { op, left, right } = expr {
            // Check if right side is an empty string literal
            if let Expr::Literal(Literal::String(s)) = right.as_ref() {
                if s.is_empty() {
                    match op {
                        BinOp::Ne => {
                            // expr != "" → expr
                            return Some(self.format_expr_readonly(left));
                        }
                        BinOp::Eq => {
                            // expr == "" → not expr / !expr
                            let inner = self.format_expr_readonly(left);
                            if self.options.prefer_word_operators {
                                return Some(format!("not {}", inner));
                            } else {
                                return Some(format!("!{}", inner));
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Also check left side ("" != expr, "" == expr)
            if let Expr::Literal(Literal::String(s)) = left.as_ref() {
                if s.is_empty() {
                    match op {
                        BinOp::Ne => {
                            return Some(self.format_expr_readonly(right));
                        }
                        BinOp::Eq => {
                            let inner = self.format_expr_readonly(right);
                            if self.options.prefer_word_operators {
                                return Some(format!("not {}", inner));
                            } else {
                                return Some(format!("!{}", inner));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    /// Format a condition expression with idiomatic simplifications applied
    fn format_condition(&mut self, expr: &Expr) -> String {
        if let Some(simplified) = self.try_simplify_condition(expr) {
            return simplified;
        }
        self.format_expr(expr)
    }

    /// Format an expression without mutating self (for use in try_simplify_condition)
    fn format_expr_readonly(&self, expr: &Expr) -> String {
        // We need a temporary formatter to avoid borrow issues
        let mut temp = Formatter::new(self.options.clone());
        temp.indent_level = self.indent_level;
        temp.format_expr(expr)
    }

    // ======================================================================
    // Statements
    // ======================================================================

    fn format_block(&mut self, block: &BlockStmt) {
        for stmt in &block.stmts {
            self.format_stmt(stmt);
        }
    }

    fn format_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl(decl) => self.format_var_decl(decl),
            Stmt::ConstDecl(decl) => self.format_const_decl_stmt(decl),
            Stmt::Assign(assign) => self.format_assign(assign),
            Stmt::If(if_stmt) => self.format_if(if_stmt),
            Stmt::While(while_stmt) => self.format_while(while_stmt),
            Stmt::For(for_stmt) => self.format_for(for_stmt),
            Stmt::Switch(switch_stmt) => self.format_switch(switch_stmt),
            Stmt::TryCatch(tc) => self.format_try_catch(tc),
            Stmt::Throw(throw) => {
                let expr = self.format_expr(&throw.expr);
                self.write_line(&format!("throw {}", expr));
            }
            Stmt::Fail(fail) => {
                let expr = self.format_expr(&fail.expr);
                self.write_line(&format!("fail {}", expr));
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    let e = self.format_expr(expr);
                    self.write_line(&format!("return {}", e));
                } else {
                    self.write_line("return");
                }
            }
            Stmt::Expr(expr_stmt) => {
                let e = self.format_expr(&expr_stmt.expr);
                // If the expression has embedded newlines (multiline call), handle it
                if e.contains('\n') {
                    self.write_multiline_expr(&e);
                } else {
                    self.write_line(&e);
                }
            }
            Stmt::Block(block) => {
                self.write_line("{");
                self.indent_level += 1;
                self.format_block(block);
                self.indent_level -= 1;
                self.write_line("}");
            }
        }
    }

    /// Write an expression that already contains newlines (multiline calls etc.)
    fn write_multiline_expr(&mut self, expr: &str) {
        let lines: Vec<&str> = expr.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if i == 0 {
                self.write_line(line);
            } else {
                // Inner lines already have their own indentation from format_call
                self.output.push_str(line);
                self.output.push('\n');
            }
        }
    }

    fn format_var_decl(&mut self, decl: &VarDecl) {
        let init = self.format_expr(&decl.init);
        let or_fail_suffix = if let Some(msg) = &decl.or_fail_msg {
            format!(" or fail {}", self.format_expr(msg))
        } else {
            String::new()
        };
        if decl.bindings.len() == 1 {
            let binding = &decl.bindings[0];
            let pattern = self.format_binding_pattern(&binding.pattern);
            let type_ann = binding
                .type_ref
                .as_ref()
                .map(|t| format!(": {}", self.format_type_ref(t)))
                .unwrap_or_default();
            let line = format!("let {}{} = {}{}", pattern, type_ann, init, or_fail_suffix);
            if init.contains('\n') {
                // Multiline init (e.g., multiline call)
                let init_lines: Vec<&str> = init.lines().collect();
                self.write_line(&format!("let {}{} = {}", pattern, type_ann, init_lines[0]));
                for il in &init_lines[1..init_lines.len()-1] {
                    self.output.push_str(il);
                    self.output.push('\n');
                }
                // Append or_fail to last line of multiline init
                let last = init_lines.last().unwrap_or(&"");
                self.output.push_str(last);
                if !or_fail_suffix.is_empty() {
                    self.output.push_str(&or_fail_suffix);
                }
                self.output.push('\n');
            } else if self.would_exceed_width(&line) {
                // Re-format the init at the indented level so calls can wrap properly
                self.indent_level += 1;
                let init_reformat = self.format_expr(&decl.init);
                self.indent_level -= 1;
                if init_reformat.contains('\n') {
                    let init_lines: Vec<&str> = init_reformat.lines().collect();
                    self.write_line(&format!("let {}{} = {}", pattern, type_ann, init_lines[0]));
                    for il in &init_lines[1..init_lines.len()-1] {
                        self.output.push_str(il);
                        self.output.push('\n');
                    }
                    let last = init_lines.last().unwrap_or(&"");
                    self.output.push_str(last);
                    if !or_fail_suffix.is_empty() {
                        self.output.push_str(&or_fail_suffix);
                    }
                    self.output.push('\n');
                } else {
                    self.write_line(&format!("let {}{} =", pattern, type_ann));
                    self.indent_level += 1;
                    self.write_line(&format!("{}{}", init_reformat, or_fail_suffix));
                    self.indent_level -= 1;
                }
            } else {
                self.write_line(&line);
            }
        } else {
            let patterns: Vec<String> = decl
                .bindings
                .iter()
                .map(|b| {
                    let pat = self.format_binding_pattern(&b.pattern);
                    if let Some(t) = &b.type_ref {
                        format!("{}: {}", pat, self.format_type_ref(t))
                    } else {
                        pat
                    }
                })
                .collect();
            let line = format!("let {} = {}", patterns.join(", "), init);
            if init.contains('\n') {
                let init_lines: Vec<&str> = init.lines().collect();
                self.write_line(&format!("let {} = {}", patterns.join(", "), init_lines[0]));
                for il in &init_lines[1..] {
                    self.output.push_str(il);
                    self.output.push('\n');
                }
            } else {
                self.write_line(&line);
            }
        }
    }

    fn format_const_decl_stmt(&mut self, decl: &ConstDecl) {
        let init = self.format_expr(&decl.init);
        let type_ann = decl
            .type_ref
            .as_ref()
            .map(|t| format!(": {}", self.format_type_ref(t)))
            .unwrap_or_default();
        self.write_line(&format!("const {}{} = {}", decl.name, type_ann, init));
    }

    fn format_assign(&mut self, assign: &AssignStmt) {
        let target = self.format_expr(&assign.target);
        let value = self.format_expr(&assign.value);
        let line = format!("{} = {}", target, value);
        if value.contains('\n') {
            let value_lines: Vec<&str> = value.lines().collect();
            self.write_line(&format!("{} = {}", target, value_lines[0]));
            for vl in &value_lines[1..] {
                self.output.push_str(vl);
                self.output.push('\n');
            }
        } else {
            self.write_line(&line);
        }
    }

    fn format_if(&mut self, if_stmt: &IfStmt) {
        let cond = self.format_condition(&if_stmt.condition);
        match &if_stmt.then_branch {
            IfBody::Block(block) => {
                self.write_line(&format!("if {} {{", cond));
                self.indent_level += 1;
                self.format_block(block);
                self.indent_level -= 1;
            }
            IfBody::Stmt(stmt) => {
                self.write_line(&format!("if {} {{", cond));
                self.indent_level += 1;
                self.format_stmt(stmt);
                self.indent_level -= 1;
            }
        }

        if let Some(else_branch) = &if_stmt.else_branch {
            self.format_else_branch(else_branch);
        } else {
            self.write_line("}");
        }
    }

    fn format_else_branch(&mut self, branch: &IfBody) {
        match branch {
            IfBody::Block(block) => {
                self.write_line("} else {");
                self.indent_level += 1;
                self.format_block(block);
                self.indent_level -= 1;
                self.write_line("}");
            }
            IfBody::Stmt(stmt) => {
                if let Stmt::If(inner_if) = stmt.as_ref() {
                    let cond = self.format_condition(&inner_if.condition);
                    match &inner_if.then_branch {
                        IfBody::Block(block) => {
                            self.write_line(&format!("}} else if {} {{", cond));
                            self.indent_level += 1;
                            self.format_block(block);
                            self.indent_level -= 1;
                        }
                        IfBody::Stmt(s) => {
                            self.write_line(&format!("}} else if {} {{", cond));
                            self.indent_level += 1;
                            self.format_stmt(s);
                            self.indent_level -= 1;
                        }
                    }
                    if let Some(inner_else) = &inner_if.else_branch {
                        self.format_else_branch(inner_else);
                    } else {
                        self.write_line("}");
                    }
                } else {
                    self.write_line("} else {");
                    self.indent_level += 1;
                    self.format_stmt(stmt);
                    self.indent_level -= 1;
                    self.write_line("}");
                }
            }
        }
    }

    fn format_while(&mut self, while_stmt: &WhileStmt) {
        let cond = self.format_condition(&while_stmt.condition);
        self.write_line(&format!("while {} {{", cond));
        self.indent_level += 1;
        self.format_block(&while_stmt.body);
        self.indent_level -= 1;
        self.write_line("}");
    }

    fn format_for(&mut self, for_stmt: &ForStmt) {
        let iterable = self.format_expr(&for_stmt.iterable);
        
        // Phase 11.3/11.4: Detect point-free body (single bare identifier or method ref as body)
        // for item in items => print  →  for item in items { print(item) }
        // for item in items => Utils::log  →  for item in items { Utils::log(item) }
        let is_point_free = for_stmt.body.stmts.len() == 1
            && matches!(&for_stmt.body.stmts[0], Stmt::Expr(expr_stmt) 
                if matches!(&expr_stmt.expr, Expr::Identifier(_) | Expr::MethodRef { .. }));
        
        if is_point_free {
            if let Stmt::Expr(expr_stmt) = &for_stmt.body.stmts[0] {
                match &expr_stmt.expr {
                    Expr::Identifier(func_name) => {
                        self.write_line(&format!("for {} in {} {{", for_stmt.var, iterable));
                        self.indent_level += 1;
                        self.write_line(&format!("{}({})", func_name, for_stmt.var));
                        self.indent_level -= 1;
                        self.write_line("}");
                        return;
                    }
                    Expr::MethodRef { object, method } => {
                        self.write_line(&format!("for {} in {} {{", for_stmt.var, iterable));
                        self.indent_level += 1;
                        self.write_line(&format!("{}::{}({})", object, method, for_stmt.var));
                        self.indent_level -= 1;
                        self.write_line("}");
                        return;
                    }
                    _ => {}
                }
            }
        }
        
        self.write_line(&format!("for {} in {} {{", for_stmt.var, iterable));
        self.indent_level += 1;
        self.format_block(&for_stmt.body);
        self.indent_level -= 1;
        self.write_line("}");
    }

    fn format_switch(&mut self, switch_stmt: &SwitchStmt) {
        let disc = self.format_expr(&switch_stmt.discriminant);
        self.write_line(&format!("switch {} {{", disc));
        self.indent_level += 1;

        for case in &switch_stmt.cases {
            let val = self.format_expr(&case.value);
            if case.body.len() == 1 {
                if let Stmt::Expr(expr_stmt) = &case.body[0] {
                    let body = self.format_expr(&expr_stmt.expr);
                    self.write_line(&format!("case {}: {}", val, body));
                    continue;
                }
            }
            self.write_line(&format!("case {}:", val));
            self.indent_level += 1;
            for stmt in &case.body {
                self.format_stmt(stmt);
            }
            self.indent_level -= 1;
        }

        if let Some(default_body) = &switch_stmt.default {
            if default_body.len() == 1 {
                if let Stmt::Expr(expr_stmt) = &default_body[0] {
                    let body = self.format_expr(&expr_stmt.expr);
                    self.write_line(&format!("default: {}", body));
                } else {
                    self.write_line("default:");
                    self.indent_level += 1;
                    for stmt in default_body {
                        self.format_stmt(stmt);
                    }
                    self.indent_level -= 1;
                }
            } else {
                self.write_line("default:");
                self.indent_level += 1;
                for stmt in default_body {
                    self.format_stmt(stmt);
                }
                self.indent_level -= 1;
            }
        }

        self.indent_level -= 1;
        self.write_line("}");
    }

    fn format_try_catch(&mut self, tc: &TryCatchStmt) {
        self.write_line("try {");
        self.indent_level += 1;
        self.format_block(&tc.try_block);
        self.indent_level -= 1;
        self.write_line(&format!("}} catch {} {{", tc.catch_var));
        self.indent_level += 1;
        self.format_block(&tc.catch_block);
        self.indent_level -= 1;
        self.write_line("}");
    }

    // ======================================================================
    // Expressions
    // ======================================================================

    fn format_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => self.format_literal(lit),
            Expr::Identifier(name) => name.clone(),
            Expr::Binary { op, left, right } => {
                let l = self.format_expr(left);
                let r = self.format_expr(right);
                let op_str = self.format_binop(op);
                format!("{} {} {}", l, op_str, r)
            }
            Expr::Unary { op, operand } => {
                let operand_str = self.format_expr(operand);
                match op {
                    UnOp::Neg => format!("-{}", operand_str),
                    UnOp::Not => {
                        if self.options.prefer_word_operators {
                            format!("not {}", operand_str)
                        } else {
                            format!("!{}", operand_str)
                        }
                    }
                    UnOp::Await => format!("await {}", operand_str),
                }
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond = self.format_expr(condition);
                let then_e = self.format_expr(then_expr);
                let else_e = self.format_expr(else_expr);
                format!("{} ? {} : {}", cond, then_e, else_e)
            }
            Expr::Call(call) => self.format_call(call),
            Expr::Member { object, property } => {
                let obj = self.format_expr(object);
                format!("{}.{}", obj, property)
            }
            Expr::Index { object, index } => {
                let obj = self.format_expr(object);
                let idx = self.format_expr(index);
                format!("{}[{}]", obj, idx)
            }
            Expr::ObjectLiteral(fields) => self.format_object_literal(fields),
            Expr::StructLiteral { type_name, fields } => {
                self.format_struct_literal(type_name, fields)
            }
            Expr::ArrayLiteral(elements) => self.format_array_literal(elements),
            Expr::Tuple(elements) => {
                let elems: Vec<String> = elements.iter().map(|e| self.format_expr(e)).collect();
                format!("({})", elems.join(", "))
            }
            Expr::Lambda(lambda) => self.format_lambda(lambda),
            Expr::StringTemplate { parts } => self.format_string_template(parts),
            Expr::Fail(expr) => {
                let e = self.format_expr(expr);
                format!("fail {}", e)
            }
            Expr::MethodCall(mc) => self.format_method_call(mc),
            Expr::Switch(switch_expr) => self.format_switch_expr(switch_expr),
            Expr::MethodRef { object, method } => {
                format!("{}::{}", object, method)
            }
        }
    }

    fn format_literal(&mut self, lit: &Literal) -> String {
        match lit {
            Literal::Int(n) => n.to_string(),
            Literal::Float(f) => {
                let s = f.to_string();
                if s.contains('.') {
                    s
                } else {
                    format!("{}.0", s)
                }
            }
            Literal::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Literal::Char(c) => format!("'{}'", c),
            Literal::Bool(b) => b.to_string(),
            Literal::Null => "null".to_string(),
        }
    }

    fn format_binop(&mut self, op: &BinOp) -> String {
        if self.options.prefer_word_operators {
            match op {
                BinOp::And => "and".to_string(),
                BinOp::Or => "or".to_string(),
                _ => op.to_string(),
            }
        } else {
            op.to_string()
        }
    }

    fn format_call(&mut self, call: &CallExpr) -> String {
        let callee = self.format_expr(&call.callee);
        let args: Vec<String> = call.args.iter().map(|a| self.format_expr(a)).collect();

        let type_args = if call.type_args.is_empty() {
            String::new()
        } else {
            let tas: Vec<String> = call
                .type_args
                .iter()
                .map(|t| self.format_type_ref(t))
                .collect();
            format!("<{}>", tas.join(", "))
        };

        let policy_prefix = match call.exec_policy {
            ExecPolicy::Async => "async ",
            ExecPolicy::Par => "par ",
            ExecPolicy::TaskAsync => "task async ",
            ExecPolicy::TaskPar => "task par ",
            ExecPolicy::FireAsync => "fire async ",
            ExecPolicy::FirePar => "fire par ",
            ExecPolicy::Normal => "",
        };

        let single_line = format!(
            "{}{}{}({})",
            policy_prefix,
            callee,
            type_args,
            args.join(", ")
        );

        // If it fits on one line at current indent, use it
        let total_width = self.current_indent_width() + single_line.len();
        if total_width <= self.options.max_width {
            return single_line;
        }

        // Multi-line: one arg per line
        let inner_indent = " ".repeat(self.options.indent_size * (self.indent_level + 1));
        let outer_indent = " ".repeat(self.options.indent_size * self.indent_level);
        let mut result = format!("{}{}{}(\n", policy_prefix, callee, type_args);
        for (i, arg) in args.iter().enumerate() {
            result.push_str(&inner_indent);
            result.push_str(arg);
            if i + 1 < args.len() {
                result.push(',');
            }
            result.push('\n');
        }
        result.push_str(&outer_indent);
        result.push(')');
        result
    }

    fn format_method_call(&mut self, mc: &MethodCallExpr) -> String {
        let obj = self.format_expr(&mc.object);
        let args: Vec<String> = mc.args.iter().map(|a| self.format_expr(a)).collect();

        let adapter = match mc.adapter {
            ArrayAdapter::Par => ".par()",
            ArrayAdapter::Vec => ".vec()",
            ArrayAdapter::ParVec => ".parvec()",
            ArrayAdapter::Seq => "",
        };

        let single_line = format!(
            "{}{}.{}({})",
            obj, adapter, mc.method, args.join(", ")
        );

        let total_width = self.current_indent_width() + single_line.len();
        if total_width <= self.options.max_width {
            return single_line;
        }

        // For method chains (obj is a method call or another call), break at the dot
        let is_chain = matches!(&*mc.object, Expr::MethodCall(_) | Expr::Call(_));
        let inner_indent = " ".repeat(self.options.indent_size * (self.indent_level + 1));

        if is_chain {
            let chain_call = format!(
                ".{}({})",
                mc.method, args.join(", ")
            );
            // If the chain part itself is short, just put it on new line
            if inner_indent.len() + chain_call.len() <= self.options.max_width {
                return format!("{}{}\n{}{}", obj, adapter, inner_indent, chain_call);
            }
        }

        // Wrap args multi-line
        if args.len() > 1 {
            let outer_indent = " ".repeat(self.options.indent_size * self.indent_level);
            let mut result = format!("{}{}.{}(\n", obj, adapter, mc.method);
            for (i, arg) in args.iter().enumerate() {
                result.push_str(&inner_indent);
                result.push_str(arg);
                if i + 1 < args.len() {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&outer_indent);
            result.push(')');
            return result;
        }

        // Fallback: keep single line even if long (for single-arg lambdas etc.)
        single_line
    }

    fn format_object_literal(&mut self, fields: &[(String, Expr)]) -> String {
        if fields.is_empty() {
            return "{}".to_string();
        }
        let items: Vec<String> = fields
            .iter()
            .map(|(k, v)| {
                let val = self.format_expr(v);
                format!("{}: {}", k, val)
            })
            .collect();
        let single_line = format!("{{ {} }}", items.join(", "));
        if self.current_indent_width() + single_line.len() <= self.options.max_width {
            single_line
        } else {
            let inner_indent = " ".repeat(self.options.indent_size * (self.indent_level + 1));
            let outer_indent = " ".repeat(self.options.indent_size * self.indent_level);
            let mut result = "{\n".to_string();
            for (i, item) in items.iter().enumerate() {
                result.push_str(&inner_indent);
                result.push_str(item);
                if i + 1 < items.len() {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&outer_indent);
            result.push('}');
            result
        }
    }

    fn format_struct_literal(&mut self, type_name: &str, fields: &[(String, Expr)]) -> String {
        if fields.is_empty() {
            return format!("{} {{}}", type_name);
        }
        let items: Vec<String> = fields
            .iter()
            .map(|(k, v)| {
                let val = self.format_expr(v);
                format!("{}: {}", k, val)
            })
            .collect();
        let single_line = format!("{} {{ {} }}", type_name, items.join(", "));
        if self.current_indent_width() + single_line.len() <= self.options.max_width {
            single_line
        } else {
            let inner_indent = " ".repeat(self.options.indent_size * (self.indent_level + 1));
            let outer_indent = " ".repeat(self.options.indent_size * self.indent_level);
            let mut result = format!("{} {{\n", type_name);
            for (i, item) in items.iter().enumerate() {
                result.push_str(&inner_indent);
                result.push_str(item);
                if i + 1 < items.len() {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&outer_indent);
            result.push('}');
            result
        }
    }

    fn format_array_literal(&mut self, elements: &[Expr]) -> String {
        if elements.is_empty() {
            return "[]".to_string();
        }
        let items: Vec<String> = elements.iter().map(|e| self.format_expr(e)).collect();
        let single_line = format!("[{}]", items.join(", "));

        if self.current_indent_width() + single_line.len() <= self.options.max_width {
            single_line
        } else {
            let inner_indent = " ".repeat(self.options.indent_size * (self.indent_level + 1));
            let outer_indent = " ".repeat(self.options.indent_size * self.indent_level);
            let mut result = "[\n".to_string();
            for (i, item) in items.iter().enumerate() {
                result.push_str(&inner_indent);
                result.push_str(item);
                if i + 1 < items.len() {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&outer_indent);
            result.push(']');
            result
        }
    }

    fn format_lambda(&mut self, lambda: &LambdaExpr) -> String {
        let params: Vec<String> = lambda
            .params
            .iter()
            .map(|p| {
                let name = self.format_binding_pattern(&p.pattern);
                if let Some(t) = &p.type_ref {
                    format!("{}: {}", name, self.format_type_ref(t))
                } else {
                    name
                }
            })
            .collect();

        let params_str = if params.len() == 1 && lambda.params[0].type_ref.is_none() {
            params[0].clone()
        } else {
            format!("({})", params.join(", "))
        };

        match &lambda.body {
            LambdaBody::Expr(expr) => {
                let body = self.format_expr(expr);
                format!("{} => {}", params_str, body)
            }
            LambdaBody::Block(block) => {
                // Try to simplify { return expr } to => expr
                if let Some(expr) = self.try_extract_single_return(block) {
                    let body = self.format_expr(&expr);
                    return format!("{} => {}", params_str, body);
                }

                let saved_output = std::mem::take(&mut self.output);
                let saved_indent = self.indent_level;
                self.indent_level += 1;
                self.format_block(block);
                self.indent_level -= 1;
                let inner = std::mem::replace(&mut self.output, saved_output);
                self.indent_level = saved_indent;
                let outer_indent = " ".repeat(self.options.indent_size * self.indent_level);
                format!("{} => {{\n{}{}}}", params_str, inner, outer_indent)
            }
        }
    }

    fn format_string_template(&mut self, parts: &[StringTemplatePart]) -> String {
        let mut result = "$\"".to_string();
        for part in parts {
            match part {
                StringTemplatePart::Text(text) => {
                    result.push_str(&text.replace('\\', "\\\\").replace('"', "\\\""));
                }
                StringTemplatePart::Expr(expr) => {
                    result.push('{');
                    result.push_str(&self.format_expr(expr));
                    result.push('}');
                }
            }
        }
        result.push('"');
        result
    }

    fn format_switch_expr(&mut self, switch_expr: &SwitchExpr) -> String {
        let disc = self.format_expr(&switch_expr.discriminant);
        let indent = " ".repeat(self.options.indent_size * (self.indent_level + 1));
        let outer_indent = " ".repeat(self.options.indent_size * self.indent_level);

        let mut result = format!("switch {} {{\n", disc);
        for arm in &switch_expr.arms {
            let pattern = self.format_pattern(&arm.pattern);
            let guard = arm
                .guard
                .as_ref()
                .map(|g| format!(" if {}", self.format_expr(g)))
                .unwrap_or_default();
            let body = match &arm.body {
                SwitchBody::Expr(e) => self.format_expr(e),
                SwitchBody::Block(stmts) => {
                    let inner: Vec<String> =
                        stmts.iter().map(|s| self.format_stmt_inline(s)).collect();
                    format!("{{ {} }}", inner.join("; "))
                }
            };
            result.push_str(&indent);
            result.push_str(&format!("{}{} => {},\n", pattern, guard, body));
        }
        result.push_str(&outer_indent);
        result.push('}');
        result
    }

    fn format_pattern(&mut self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Literal(lit) => self.format_literal(lit),
            Pattern::Wildcard => "_".to_string(),
            Pattern::Binding(name) => name.clone(),
            Pattern::Typed { name, type_ref } => {
                format!("{}: {}", name, self.format_type_ref(type_ref))
            }
            Pattern::Range(range) => {
                let start = range
                    .start
                    .as_ref()
                    .map(|e| self.format_expr(e))
                    .unwrap_or_default();
                let end = range
                    .end
                    .as_ref()
                    .map(|e| self.format_expr(e))
                    .unwrap_or_default();
                let op = if range.inclusive { "..=" } else { ".." };
                format!("{}{}{}", start, op, end)
            }
            Pattern::Tuple(patterns) => {
                let pats: Vec<String> = patterns.iter().map(|p| self.format_pattern(p)).collect();
                format!("({})", pats.join(", "))
            }
            Pattern::Array(patterns) => {
                let pats: Vec<String> = patterns.iter().map(|p| self.format_pattern(p)).collect();
                format!("[{}]", pats.join(", "))
            }
            Pattern::Or(patterns) => {
                let pats: Vec<String> = patterns.iter().map(|p| self.format_pattern(p)).collect();
                pats.join(" | ")
            }
        }
    }

    /// Format a statement inline (for switch expression bodies)
    fn format_stmt_inline(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::Return(ret) => {
                if let Some(e) = &ret.expr {
                    format!("return {}", self.format_expr(e))
                } else {
                    "return".to_string()
                }
            }
            Stmt::Expr(es) => self.format_expr(&es.expr),
            Stmt::Fail(f) => format!("fail {}", self.format_expr(&f.expr)),
            _ => "/* ... */".to_string(),
        }
    }

    // ======================================================================
    // Type references
    // ======================================================================

    fn format_type_ref(&mut self, type_ref: &TypeRef) -> String {
        match type_ref {
            TypeRef::Simple(name) => name.clone(),
            TypeRef::Generic { base, args } => {
                let args_str: Vec<String> = args.iter().map(|a| self.format_type_ref(a)).collect();
                format!("{}<{}>", base, args_str.join(", "))
            }
            TypeRef::Array(inner) => format!("[{}]", self.format_type_ref(inner)),
            TypeRef::Optional(inner) => format!("{}?", self.format_type_ref(inner)),
            TypeRef::Fallible(inner) => format!("{}!", self.format_type_ref(inner)),
            TypeRef::Tuple(types) => {
                let ts: Vec<String> = types.iter().map(|t| self.format_type_ref(t)).collect();
                format!("({})", ts.join(", "))
            }
            TypeRef::Union(types) => {
                let ts: Vec<String> = types.iter().map(|t| self.format_type_ref(t)).collect();
                ts.join(" | ")
            }
        }
    }

    // ======================================================================
    // Helper formatters
    // ======================================================================

    fn format_type_params(&mut self, params: &[TypeParameter]) -> String {
        if params.is_empty() {
            return String::new();
        }
        let ps: Vec<String> = params
            .iter()
            .map(|p| {
                if p.constraints.is_empty() {
                    p.name.clone()
                } else {
                    format!("{}: {}", p.name, p.constraints.join(" + "))
                }
            })
            .collect();
        format!("<{}>", ps.join(", "))
    }

    /// Format a single param
    fn format_param(&mut self, p: &Param) -> String {
        let pattern = self.format_binding_pattern(&p.pattern);
        let type_ann = p
            .type_ref
            .as_ref()
            .map(|t| format!(": {}", self.format_type_ref(t)))
            .unwrap_or_default();
        let default = p
            .default
            .as_ref()
            .map(|d| format!(" = {}", self.format_expr(d)))
            .unwrap_or_default();
        format!("{}{}{}", pattern, type_ann, default)
    }

    /// Format params on a single line
    fn format_params_simple(&mut self, params: &[Param]) -> String {
        let ps: Vec<String> = params.iter().map(|p| self.format_param(p)).collect();
        ps.join(", ")
    }

    fn format_binding_pattern(&mut self, pattern: &BindingPattern) -> String {
        match pattern {
            BindingPattern::Identifier(name) => name.clone(),
            BindingPattern::Object(obj) => {
                let fields: Vec<String> = obj
                    .fields
                    .iter()
                    .map(|f| {
                        if f.key == f.binding {
                            f.key.clone()
                        } else {
                            format!("{}: {}", f.key, f.binding)
                        }
                    })
                    .collect();
                format!("{{ {} }}", fields.join(", "))
            }
            BindingPattern::Array(arr) => {
                let mut elems: Vec<String> = arr
                    .elements
                    .iter()
                    .map(|e| e.as_deref().unwrap_or("_").to_string())
                    .collect();
                if let Some(rest) = &arr.rest {
                    elems.push(format!("...{}", rest));
                }
                format!("[{}]", elems.join(", "))
            }
            BindingPattern::Tuple(tup) => {
                format!("({})", tup.elements.join(", "))
            }
        }
    }
}

// ======================================================================
// Comment handling
// ======================================================================

/// Find the position of a line comment (//), ignoring those inside strings
fn find_line_comment(line: &str) -> Option<usize> {
    let mut in_string = false;
    let mut in_template = false;
    let mut escape_next = false;
    let chars: Vec<char> = line.chars().collect();

    for i in 0..chars.len() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if chars[i] == '\\' && (in_string || in_template) {
            escape_next = true;
            continue;
        }
        if !in_template && chars[i] == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string && !in_template && chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '"' {
            in_template = true;
            continue;
        }
        if in_template && chars[i] == '"' {
            in_template = false;
            continue;
        }
        if !in_string
            && !in_template
            && i + 1 < chars.len()
            && chars[i] == '/'
            && chars[i + 1] == '/'
        {
            return Some(i);
        }
    }

    None
}

/// Extract comments from source, keeping their line numbers
fn extract_comments(source: &str) -> Vec<SourceComment> {
    let mut comments = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if let Some(comment_pos) = find_line_comment(line) {
            let before = line[..comment_pos].trim_end();
            let comment_text = line[comment_pos..].trim_end().to_string();

            comments.push(SourceComment {
                text: comment_text,
                line: line_num,
                is_standalone: before.is_empty(),
                code_on_line: before.to_string(),
            });
        }
    }

    comments
}

/// Normalize code for comparison: collapse whitespace, trim
fn normalize_code(code: &str) -> String {
    code.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Get leading whitespace of a line
fn get_line_indent(line: &str) -> String {
    let trimmed = line.trim_start();
    line[..line.len() - trimmed.len()].to_string()
}

/// Find the next non-empty, non-comment code line after line_num in the source
fn find_next_code_line(source_lines: &[&str], line_num: usize) -> Option<String> {
    for i in (line_num + 1)..source_lines.len() {
        let trimmed = source_lines[i].trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        let code = if let Some(pos) = find_line_comment(trimmed) {
            trimmed[..pos].trim_end().to_string()
        } else {
            trimmed.to_string()
        };
        if !code.is_empty() {
            return Some(code);
        }
    }
    None
}

/// Find the previous non-empty, non-comment code line before line_num in the source
fn find_prev_code_line(source_lines: &[&str], line_num: usize) -> Option<String> {
    if line_num == 0 {
        return None;
    }
    for i in (0..line_num).rev() {
        let trimmed = source_lines[i].trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        let code = if let Some(pos) = find_line_comment(trimmed) {
            trimmed[..pos].trim_end().to_string()
        } else {
            trimmed.to_string()
        };
        if !code.is_empty() {
            return Some(code);
        }
    }
    None
}

/// Reinsert comments into formatted code.
///
/// Strategy:
/// - For inline comments: match the code on the line and append the comment
/// - For standalone comments: find the closest code anchor (the next code line
///   in the original source) and insert the comment before that anchor in the
///   formatted output. Uses both next and prev anchors plus sequential ordering
///   to disambiguate duplicate code lines.
fn reinsert_comments(source: &str, formatted: &str, _options: &FormatOptions) -> String {
    let comments = extract_comments(source);
    if comments.is_empty() {
        return formatted.to_string();
    }

    let source_lines: Vec<&str> = source.lines().collect();
    let mut result_lines: Vec<String> = formatted.lines().map(|l| l.to_string()).collect();

    // Track which formatted lines have been used for anchoring
    let mut used_lines: Vec<bool> = vec![false; result_lines.len()];

    // Phase 1: Process inline comments (comments on the same line as code)
    for comment in &comments {
        if comment.is_standalone {
            continue;
        }
        let normalized_code = normalize_code(&comment.code_on_line);
        if normalized_code.is_empty() {
            continue;
        }
        // Find matching code line in formatted output
        for (i, line) in result_lines.iter_mut().enumerate() {
            if used_lines[i] {
                continue;
            }
            let norm_formatted = normalize_code(line.trim());
            if norm_formatted == normalized_code {
                *line = format!("{}  {}", line, comment.text);
                used_lines[i] = true;
                break;
            }
        }
    }

    // Phase 2: Process standalone comments
    // Group consecutive standalone comments
    let standalone: Vec<&SourceComment> = comments.iter().filter(|c| c.is_standalone).collect();
    if standalone.is_empty() {
        return finalize_lines(&result_lines, formatted);
    }

    let mut groups: Vec<Vec<&SourceComment>> = Vec::new();
    let mut current_group: Vec<&SourceComment> = Vec::new();

    for comment in &standalone {
        if current_group.is_empty() {
            current_group.push(comment);
        } else {
            let last = current_group.last().unwrap();
            if comment.line == last.line + 1 {
                current_group.push(comment);
            } else {
                groups.push(current_group);
                current_group = vec![comment];
            }
        }
    }
    if !current_group.is_empty() {
        groups.push(current_group);
    }

    // For each comment group, find anchor and insert position.
    // Process groups in source order and track used formatted lines to avoid
    // multiple comment groups anchoring to the same duplicate code line.
    let mut insertions: Vec<(usize, Vec<String>)> = Vec::new();
    // Track minimum search position: comments later in source should match
    // formatted lines at or after previously matched positions
    let mut min_search_pos: usize = 0;

    for group in &groups {
        let last_comment_line = group.last().unwrap().line;
        let first_comment_line = group.first().unwrap().line;

        // Primary anchor: the next code line after the comment group
        let next_code = find_next_code_line(&source_lines, last_comment_line);
        // Secondary anchor: the previous code line before the comment group
        let prev_code = find_prev_code_line(&source_lines, first_comment_line);

        let insert_pos = if let Some(ref code) = next_code {
            let normalized = normalize_code(code);
            // Search starting from min_search_pos to handle duplicates
            find_line_in_formatted_from(&result_lines, &normalized, min_search_pos, &used_lines)
                .or_else(|| {
                    // Fallback: search from beginning
                    find_line_in_formatted_from(&result_lines, &normalized, 0, &used_lines)
                })
                .or_else(|| {
                    // Prefix match fallback: when the line got reformatted (e.g., multiline params),
                    // try matching by the first few tokens
                    find_line_by_prefix(&result_lines, &normalized, min_search_pos, &used_lines)
                })
                .or_else(|| {
                    find_line_by_prefix(&result_lines, &normalized, 0, &used_lines)
                })
                .unwrap_or(result_lines.len())
        } else if let Some(ref code) = prev_code {
            let normalized = normalize_code(code);
            find_line_in_formatted_from(&result_lines, &normalized, min_search_pos, &used_lines)
                .map(|pos| pos + 1)
                .unwrap_or_else(|| {
                    find_line_in_formatted_from(&result_lines, &normalized, 0, &used_lines)
                        .map(|pos| pos + 1)
                        .unwrap_or(result_lines.len())
                })
        } else {
            // No code context — put at beginning
            0
        };

        // Update min_search_pos for next group
        if insert_pos < result_lines.len() {
            min_search_pos = insert_pos;
        }

        // Determine indent from the target position
        let indent = if insert_pos < result_lines.len() {
            get_line_indent(&result_lines[insert_pos])
        } else if !result_lines.is_empty() {
            get_line_indent(result_lines.last().unwrap())
        } else {
            String::new()
        };

        let comment_lines: Vec<String> = group
            .iter()
            .map(|c| format!("{}{}", indent, c.text))
            .collect();

        insertions.push((insert_pos, comment_lines));
    }

    // Apply insertions in reverse order so positions don't shift
    insertions.sort_by(|a, b| b.0.cmp(&a.0));
    for (pos, lines) in insertions {
        let insert_at = pos.min(result_lines.len());
        for line in lines.into_iter().rev() {
            result_lines.insert(insert_at, line);
        }
    }

    finalize_lines(&result_lines, formatted)
}

/// Find the first line in formatted output matching the normalized code,
/// starting from a given position and skipping already-used lines.
fn find_line_in_formatted_from(
    lines: &[String],
    normalized_code: &str,
    start_from: usize,
    used: &[bool],
) -> Option<usize> {
    if normalized_code.is_empty() {
        return None;
    }
    for i in start_from..lines.len() {
        if used[i] {
            continue;
        }
        let norm_line = normalize_code(lines[i].trim());
        if norm_line == *normalized_code {
            return Some(i);
        }
    }
    None
}

/// Find a line by prefix match when exact match fails.
/// This handles cases where a function header got reformatted to multi-line:
/// Original: `fetchRepoPullRequests(owner: string, ...)`
/// Formatted: `fetchRepoPullRequests(`
/// We match by the first significant token (identifier before '(' or '{').
fn find_line_by_prefix(
    lines: &[String],
    normalized_code: &str,
    start_from: usize,
    used: &[bool],
) -> Option<usize> {
    // Extract the first "word" up to a delimiter like '(' or '{'
    let prefix = extract_code_prefix(normalized_code);
    if prefix.is_empty() || prefix.len() < 3 {
        return None;
    }
    for i in start_from..lines.len() {
        if used[i] {
            continue;
        }
        let norm_line = normalize_code(lines[i].trim());
        let line_prefix = extract_code_prefix(&norm_line);
        if !line_prefix.is_empty() && line_prefix == prefix {
            return Some(i);
        }
    }
    None
}

/// Extract a meaningful prefix from a code line for fuzzy matching.
/// Returns the content up to the first '(' or '{', normalized.
fn extract_code_prefix(code: &str) -> String {
    let end = code.find(|c: char| c == '(' || c == '{').unwrap_or(code.len());
    code[..end].trim().to_string()
}

/// Find the first line in formatted output matching the normalized code
fn find_line_in_formatted(lines: &[String], normalized_code: &str) -> Option<usize> {
    let unused = vec![false; lines.len()];
    find_line_in_formatted_from(lines, normalized_code, 0, &unused)
}

/// Finalize lines back into a string
fn finalize_lines(lines: &[String], original_formatted: &str) -> String {
    let mut result = lines.join("\n");
    if original_formatted.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

// ======================================================================
// Public API
// ======================================================================

/// Format Liva source code according to canonical style rules.
pub fn format_source(source: &str, options: &FormatOptions) -> Result<String> {
    // 1. Tokenize
    let tokens = lexer::tokenize(source)?;

    // 2. Parse to AST
    let ast = parser::parse(tokens, source)?;

    // 3. Pretty-print from AST
    let mut formatter = Formatter::new(options.clone());
    formatter.format_program(&ast);
    let formatted_code = formatter.output;

    // 4. Reinsert comments from original source
    let result = reinsert_comments(source, &formatted_code, options);

    Ok(result)
}

/// Check if the source code is already formatted correctly.
pub fn check_format(source: &str, options: &FormatOptions) -> Result<bool> {
    let formatted = format_source(source, options)?;
    Ok(formatted == source)
}

/// Format a Liva source file in place.
pub fn format_file(path: &std::path::Path, options: &FormatOptions) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| CompilerError::IoError(format!("Failed to read file: {}", e)))?;

    let formatted = format_source(&source, options)?;

    std::fs::write(path, &formatted)
        .map_err(|e| CompilerError::IoError(format!("Failed to write file: {}", e)))?;

    Ok(())
}

/// Check if a file is already formatted correctly.
pub fn check_file(path: &std::path::Path, options: &FormatOptions) -> Result<bool> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| CompilerError::IoError(format!("Failed to read file: {}", e)))?;

    check_format(&source, options)
}

// ======================================================================
// Tests
// ======================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(source: &str) -> String {
        format_source(source, &FormatOptions::default()).unwrap()
    }

    #[test]
    fn test_format_simple_function() {
        let input = "main(){\nprint(\"Hello\")\n}";
        let output = fmt(input);
        assert_eq!(output, "main() {\n    print(\"Hello\")\n}\n");
    }

    #[test]
    fn test_format_variable_declaration() {
        let input = "main(){let x=10\nlet y = 20\n}";
        let output = fmt(input);
        assert!(output.contains("let x = 10"));
        assert!(output.contains("let y = 20"));
    }

    #[test]
    fn test_format_class() {
        let input = "Person{name:string\nage:number\ngreet()=>print(\"hi\")\n}";
        let output = fmt(input);
        assert!(output.contains("Person {"));
        assert!(output.contains("    name: string"));
        assert!(output.contains("    age: number"));
    }

    #[test]
    fn test_format_if_else() {
        let input = "main(){if x > 0{print(\"pos\")} else{print(\"neg\")}}";
        let output = fmt(input);
        assert!(output.contains("if x > 0 {"));
        assert!(output.contains("} else {"));
    }

    #[test]
    fn test_format_import() {
        let input = "import{add,multiply}from\"./math.liva\"";
        let output = fmt(input);
        assert!(output.contains("import { add, multiply } from \"./math.liva\""));
    }

    #[test]
    fn test_format_one_liner_function() {
        let input = "add(a,b)=>a+b";
        let output = fmt(input);
        assert_eq!(output, "add(a, b) => a + b\n");
    }

    #[test]
    fn test_format_for_loop() {
        let input = "main(){for i in 0..10{print(i)}}";
        let output = fmt(input);
        assert!(output.contains("for i in 0 .. 10 {"));
        assert!(output.contains("    print(i)"));
    }

    #[test]
    fn test_format_const() {
        let input = "main(){const PI=3.14159}";
        let output = fmt(input);
        assert!(output.contains("const PI = 3.14159"));
    }

    #[test]
    fn test_format_method_chain() {
        let input = "main(){nums.map(x => x * 2).filter(x => x > 5)}";
        let output = fmt(input);
        assert!(output.contains(".map(x => x * 2)"));
    }

    #[test]
    fn test_format_string_template() {
        let input = "main(){let msg = $\"Hello, {name}!\"}";
        let output = fmt(input);
        assert!(output.contains("$\"Hello, {name}!\""));
    }

    #[test]
    fn test_format_idempotent() {
        let input = r#"main() {
    let x = 10
    let y = 20
    print(x + y)
}
"#;
        let formatted = fmt(input);
        let reformatted = fmt(&formatted);
        assert_eq!(formatted, reformatted, "Formatter should be idempotent");
    }

    #[test]
    fn test_format_error_binding() {
        let input = "main(){let data, err = File.read(\"test.txt\")}";
        let output = fmt(input);
        assert!(output.contains("let data, err = File.read(\"test.txt\")"));
    }

    #[test]
    fn test_format_blank_lines_between_functions() {
        let input = "add(a,b)=>a+b\nsub(a,b)=>a-b";
        let output = fmt(input);
        assert!(
            output.contains("add(a, b) => a + b\n\nsub(a, b) => a - b"),
            "Should have blank line between functions. Got: {}",
            output
        );
    }

    #[test]
    fn test_check_format() {
        let well_formatted = "main() {\n    print(\"Hello\")\n}\n";
        assert!(check_format(well_formatted, &FormatOptions::default()).unwrap());
    }

    #[test]
    fn test_format_while_loop() {
        let input = "main(){while x<10{x=x+1}}";
        let output = fmt(input);
        assert!(output.contains("while x < 10 {"));
        assert!(output.contains("    x = x + 1"));
    }

    #[test]
    fn test_format_return_simplification() {
        let input = "add(a: number, b: number): number {\n    return a + b\n}";
        let output = fmt(input);
        assert!(
            output.contains("=> a + b"),
            "Should simplify single return to =>. Got: {}",
            output
        );
    }

    #[test]
    fn test_format_typed_params() {
        let input = "greet(name:string,age:number)=>print(name)";
        let output = fmt(input);
        assert_eq!(output, "greet(name: string, age: number) => print(name)\n");
    }

    #[test]
    fn test_format_lambda() {
        let input = "main(){let fn1 = (x, y) => x + y}";
        let output = fmt(input);
        assert!(output.contains("(x, y) => x + y"));
    }

    #[test]
    fn test_format_array() {
        let input = "main(){let arr=[1,2,3,4,5]}";
        let output = fmt(input);
        assert!(output.contains("[1, 2, 3, 4, 5]"));
    }

    #[test]
    fn test_format_switch_stmt() {
        let input = "main(){switch level{\ncase \"INFO\": print(\"info\")\ncase \"ERROR\": print(\"error\")\ndefault: print(\"unknown\")\n}}";
        let output = fmt(input);
        assert!(output.contains("switch level {"));
        assert!(output.contains("    case \"INFO\": print(\"info\")"));
    }

    #[test]
    fn test_format_ternary() {
        let input = "main(){let status=age>=18?\"adult\":\"minor\"}";
        let output = fmt(input);
        assert!(output.contains("age >= 18 ? \"adult\" : \"minor\""));
    }

    #[test]
    fn test_format_interface() {
        let input = "Animal{makeSound():string\ngetName():string}";
        let output = fmt(input);
        assert!(output.contains("Animal {"));
        assert!(output.contains("    makeSound(): string"));
        assert!(output.contains("    getName(): string"));
    }

    #[test]
    fn test_format_preserves_standalone_comments() {
        let input = "// Header comment\nmain() {\n// inner comment\nprint(\"hi\")\n}";
        let output = fmt(input);
        assert!(
            output.contains("// Header comment"),
            "header comment should be preserved. Got: {}",
            output
        );
        assert!(
            output.contains("// inner comment"),
            "inner comment should be preserved. Got: {}",
            output
        );
    }

    #[test]
    fn test_format_preserves_inline_comments() {
        let input = "main() {\nlet x = 10 // the value\n}";
        let output = fmt(input);
        assert!(
            output.contains("// the value"),
            "inline comment should be preserved. Got: {}",
            output
        );
    }

    #[test]
    fn test_format_simplify_single_return_to_oneliner() {
        let input = "add(a: number, b: number): number {\n    return a + b\n}";
        let output = fmt(input);
        assert!(
            output.contains("=> a + b"),
            "Should simplify {{ return expr }} to => expr. Got: {}",
            output
        );
    }

    #[test]
    fn test_format_long_params_multiline() {
        let input = "fetchRepoIssues(owner: string, repo: string, token: string, state: string, limit: number): [string] {\n    return []\n}";
        let output = fmt(input);
        assert!(
            output.lines().all(|l| l.len() <= 100),
            "All lines should be <= 100 chars. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_format_long_constructor_multiline() {
        let input = "IssueStats {\n    constructor(total: number, open: number, closed: number, avgLabelsPerIssue: float, highPriorityCount: number) {\n        this.total = total\n    }\n    total: number\n}";
        let output = fmt(input);
        assert!(
            output.lines().all(|l| l.len() <= 100),
            "Constructor params should wrap. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_format_call_multiline_wrap() {
        let input = "main() {\n    let x = PullRequest(item.id, item.number, item.title, item.body, item.state, item.draft, false, 0, 0, 0)\n}";
        let output = fmt(input);
        assert!(
            output.lines().all(|l| l.len() <= 100),
            "Long calls should wrap. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_format_groups_consecutive_imports() {
        let input = "import { add } from \"./math.liva\"\nimport { sub } from \"./ops.liva\"\n\nmain() {\n    print(\"hi\")\n}";
        let output = fmt(input);
        // Imports should NOT have a blank line between them
        assert!(
            output.contains("import { add } from \"./math.liva\"\nimport { sub } from \"./ops.liva\""),
            "Consecutive imports should be grouped. Got:\n{}",
            output
        );
    }

    #[test]
    fn test_format_top_level_const() {
        let input = "const API_BASE = \"https://api.github.com\"";
        let output = fmt(input);
        assert!(
            output.contains("const API_BASE = \"https://api.github.com\""),
            "Top-level const should be formatted. Got: {}",
            output
        );
    }

    #[test]
    fn test_format_simplify_ne_empty_string() {
        let input = "main() {\n    if err != \"\" {\n        fail err\n    }\n}";
        let output = fmt(input);
        assert!(
            output.contains("if err {"),
            "Should simplify `err != \"\"` to `err`. Got:\n{}",
            output
        );
        assert!(
            !output.contains("!= \"\""),
            "Should not contain != \"\". Got:\n{}",
            output
        );
    }

    #[test]
    fn test_format_simplify_eq_empty_string() {
        let input = "main() {\n    if name == \"\" {\n        fail \"empty\"\n    }\n}";
        let output = fmt(input);
        assert!(
            output.contains("if not name {") || output.contains("if !name {"),
            "Should simplify `name == \"\"` to `not name` or `!name`. Got:\n{}",
            output
        );
    }
}
