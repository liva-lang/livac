/// Liva Code Formatter
///
/// Formats Liva source code according to the language's canonical style rules:
/// - 4-space indentation
/// - Consistent spacing around operators
/// - Blank lines between top-level declarations
/// - Consistent brace positioning (same-line opening braces)
/// - Preserved comments
///
/// # Architecture
///
/// The formatter works by:
/// 1. Scanning the source to extract comments with their positions
/// 2. Parsing the source into an AST (reusing the compiler pipeline)
/// 3. Pretty-printing the AST back to source with canonical formatting
/// 4. Reinserting comments at appropriate locations
///
/// # Usage
///
/// ```rust,no_run
/// use livac::formatter::{format_source, FormatOptions};
///
/// let source = "  let   x=10";
/// let options = FormatOptions::default();
/// let formatted = format_source(source, &options).unwrap();
/// assert_eq!(formatted, "let x = 10\n");
/// ```

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
            prefer_word_operators: false,
            trailing_newline: true,
        }
    }
}

/// Formatter state for tracking indentation and output
struct Formatter {
    options: FormatOptions,
    output: String,
    indent_level: usize,
    /// Current output line number (0-indexed)
    current_line: usize,
}

impl Formatter {
    fn new(options: FormatOptions, _source: &str) -> Self {
        Formatter {
            options,
            output: String::new(),
            indent_level: 0,
            current_line: 0,
        }
    }

    /// Get the current indentation string
    fn indent(&self) -> String {
        " ".repeat(self.options.indent_size * self.indent_level)
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
        self.current_line += 1;
    }

    /// Write a blank line
    fn blank_line(&mut self) {
        // Avoid double blank lines
        if !self.output.ends_with("\n\n") {
            self.output.push('\n');
            self.current_line += 1;
        }
    }

    // ======================================================================
    // Top-level formatting
    // ======================================================================

    fn format_program(&mut self, program: &Program) {
        let total = program.items.len();
        for (i, item) in program.items.iter().enumerate() {
            self.format_top_level(item);
            // Blank line between top-level items (but not after the last one)
            if i + 1 < total {
                self.blank_line();
            }
        }

        // Ensure trailing newline
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
            // Multiple imports - check if they fit on one line
            let single_line = format!(
                "import {{ {} }} from \"{}\"",
                decl.imports.join(", "),
                decl.source
            );
            if single_line.len() <= self.options.max_width {
                self.write_line(&single_line);
            } else {
                // Multi-line imports
                self.write_line(&format!("import {{"));
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
        let mut last_was_field = false;
        let mut first = true;

        for member in members {
            match member {
                Member::Field(field) => {
                    // Fields grouped together, no extra blank line between fields
                    if !first && !last_was_field {
                        self.blank_line();
                    }
                    self.format_field(field);
                    last_was_field = true;
                }
                Member::Method(method) => {
                    if !first {
                        self.blank_line();
                    }
                    self.format_method(method);
                    last_was_field = false;
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
        let params = self.format_params(&method.params);
        let ret_type = method
            .return_type
            .as_ref()
            .map(|t| format!(": {}", self.format_type_ref(t)))
            .unwrap_or_default();

        if let Some(expr) = &method.expr_body {
            let body = self.format_expr(expr);
            self.write_line(&format!(
                "{}{}({}) => {}",
                method.name, type_params, params, body
            ));
        } else if let Some(block) = &method.body {
            self.write_line(&format!(
                "{}{}({}){} {{",
                method.name, type_params, params, ret_type
            ));
            self.indent_level += 1;
            self.format_block(block);
            self.indent_level -= 1;
            self.write_line("}");
        } else {
            // Interface method (no body)
            self.write_line(&format!(
                "{}{}({}){}", 
                method.name, type_params, params, ret_type
            ));
        }
    }

    // ======================================================================
    // Function declarations
    // ======================================================================

    fn format_function(&mut self, decl: &FunctionDecl) {
        let type_params = self.format_type_params(&decl.type_params);
        let params = self.format_params(&decl.params);
        let ret_type = decl
            .return_type
            .as_ref()
            .map(|t| format!(": {}", self.format_type_ref(t)))
            .unwrap_or_default();

        if let Some(expr) = &decl.expr_body {
            let body = self.format_expr(expr);
            self.write_line(&format!(
                "{}{}({}){} => {}",
                decl.name, type_params, params, ret_type, body
            ));
        } else if let Some(block) = &decl.body {
            self.write_line(&format!(
                "{}{}({}){} {{",
                decl.name, type_params, params, ret_type
            ));
            self.indent_level += 1;
            self.format_block(block);
            self.indent_level -= 1;
            self.write_line("}");
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
            Stmt::ConstDecl(decl) => self.format_const_decl(decl),
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
                self.write_line(&e);
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

    fn format_var_decl(&mut self, decl: &VarDecl) {
        let init = self.format_expr(&decl.init);
        if decl.bindings.len() == 1 {
            let binding = &decl.bindings[0];
            let pattern = self.format_binding_pattern(&binding.pattern);
            let type_ann = binding
                .type_ref
                .as_ref()
                .map(|t| format!(": {}", self.format_type_ref(t)))
                .unwrap_or_default();
            self.write_line(&format!("let {}{} = {}", pattern, type_ann, init));
        } else {
            // Multiple bindings (error binding): let value, err = expr
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
            self.write_line(&format!("let {} = {}", patterns.join(", "), init));
        }
    }

    fn format_const_decl(&mut self, decl: &ConstDecl) {
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
        self.write_line(&format!("{} = {}", target, value));
    }

    fn format_if(&mut self, if_stmt: &IfStmt) {
        let cond = self.format_expr(&if_stmt.condition);
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
            match else_branch {
                IfBody::Block(block) => {
                    self.write_line("} else {");
                    self.indent_level += 1;
                    self.format_block(block);
                    self.indent_level -= 1;
                    self.write_line("}");
                }
                IfBody::Stmt(stmt) => {
                    // Check if it's an else-if chain
                    if let Stmt::If(inner_if) = stmt.as_ref() {
                        let inner_cond = self.format_expr(&inner_if.condition);
                        match &inner_if.then_branch {
                            IfBody::Block(block) => {
                                self.write_line(&format!("}} else if {} {{", inner_cond));
                                self.indent_level += 1;
                                self.format_block(block);
                                self.indent_level -= 1;
                            }
                            IfBody::Stmt(s) => {
                                self.write_line(&format!("}} else if {} {{", inner_cond));
                                self.indent_level += 1;
                                self.format_stmt(s);
                                self.indent_level -= 1;
                            }
                        }
                        // Recurse for the else branch of the inner if
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
                    let cond = self.format_expr(&inner_if.condition);
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
        let cond = self.format_expr(&while_stmt.condition);
        self.write_line(&format!("while {} {{", cond));
        self.indent_level += 1;
        self.format_block(&while_stmt.body);
        self.indent_level -= 1;
        self.write_line("}");
    }

    fn format_for(&mut self, for_stmt: &ForStmt) {
        let iterable = self.format_expr(&for_stmt.iterable);
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
        }
    }

    fn format_literal(&mut self, lit: &Literal) -> String {
        match lit {
            Literal::Int(n) => n.to_string(),
            Literal::Float(f) => {
                let s = f.to_string();
                // Ensure we always have a decimal point
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
            let tas: Vec<String> = call.type_args.iter().map(|t| self.format_type_ref(t)).collect();
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

        format!("{}{}{}({})", policy_prefix, callee, type_args, args.join(", "))
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

        format!("{}{}.{}({})", obj, adapter, mc.method, args.join(", "))
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
        format!("{{ {} }}", items.join(", "))
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
        format!("{} {{ {} }}", type_name, items.join(", "))
    }

    fn format_array_literal(&mut self, elements: &[Expr]) -> String {
        if elements.is_empty() {
            return "[]".to_string();
        }
        let items: Vec<String> = elements.iter().map(|e| self.format_expr(e)).collect();
        let single_line = format!("[{}]", items.join(", "));

        // If it fits on one line, use single-line format
        if single_line.len() + self.options.indent_size * self.indent_level <= self.options.max_width
        {
            single_line
        } else {
            // Multi-line array
            let indent = " ".repeat(self.options.indent_size * (self.indent_level + 1));
            let mut result = "[\n".to_string();
            for (i, item) in items.iter().enumerate() {
                result.push_str(&indent);
                result.push_str(item);
                if i + 1 < items.len() {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&" ".repeat(self.options.indent_size * self.indent_level));
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
                // For block lambdas, format the body at an increased indent level
                let saved_output = std::mem::take(&mut self.output);
                let saved_line = self.current_line;
                self.indent_level += 1;
                self.format_block(block);
                self.indent_level -= 1;
                let inner = std::mem::replace(&mut self.output, saved_output);
                self.current_line = saved_line;
                let outer_indent = " ".repeat(self.options.indent_size * self.indent_level);
                format!("{} => {{\n{}{}}}", params_str, inner, outer_indent)
            }
        }
    }

    /// Format a statement as a string (for use inside expressions like lambdas)
    fn format_stmt_inline(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::VarDecl(decl) => {
                let init = self.format_expr(&decl.init);
                if decl.bindings.len() == 1 {
                    let b = &decl.bindings[0];
                    let pat = self.format_binding_pattern(&b.pattern);
                    let ty = b.type_ref.as_ref()
                        .map(|t| format!(": {}", self.format_type_ref(t)))
                        .unwrap_or_default();
                    format!("let {}{} = {}", pat, ty, init)
                } else {
                    let pats: Vec<String> = decl.bindings.iter().map(|b| {
                        let pat = self.format_binding_pattern(&b.pattern);
                        if let Some(t) = &b.type_ref {
                            format!("{}: {}", pat, self.format_type_ref(t))
                        } else {
                            pat
                        }
                    }).collect();
                    format!("let {} = {}", pats.join(", "), init)
                }
            }
            Stmt::ConstDecl(decl) => {
                let init = self.format_expr(&decl.init);
                format!("const {} = {}", decl.name, init)
            }
            Stmt::Assign(a) => {
                let target = self.format_expr(&a.target);
                let value = self.format_expr(&a.value);
                format!("{} = {}", target, value)
            }
            Stmt::Return(ret) => {
                if let Some(e) = &ret.expr {
                    format!("return {}", self.format_expr(e))
                } else {
                    "return".to_string()
                }
            }
            Stmt::Expr(es) => self.format_expr(&es.expr),
            Stmt::Fail(f) => format!("fail {}", self.format_expr(&f.expr)),
            Stmt::If(if_stmt) => {
                let cond = self.format_expr(&if_stmt.condition);
                // Simplified inline if - just the condition line
                format!("if {} {{ ... }}", cond)
            }
            _ => "/* complex stmt */".to_string(),
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
                    let inner: Vec<String> = stmts
                        .iter()
                        .map(|s| self.format_stmt_inline(s))
                        .collect();
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

    fn format_params(&mut self, params: &[Param]) -> String {
        let ps: Vec<String> = params
            .iter()
            .map(|p| {
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
            })
            .collect();
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
// Comment handling utilities
// ======================================================================

/// Find the position of a line comment (//), ignoring those inside strings
fn find_line_comment(line: &str) -> Option<usize> {
    let mut in_string = false;
    let mut escape_next = false;
    let chars: Vec<char> = line.chars().collect();

    for i in 0..chars.len() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if chars[i] == '\\' && in_string {
            escape_next = true;
            continue;
        }
        if chars[i] == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string && i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            return Some(i);
        }
    }

    None
}

// ======================================================================
// Public API
// ======================================================================

/// Format Liva source code according to canonical style rules.
///
/// Parses the source code into an AST and re-emits it with consistent formatting.
/// Comments are preserved and reinserted at their original relative positions.
///
/// # Arguments
///
/// * `source` - The Liva source code to format
/// * `options` - Formatting options (indentation, line width, etc.)
///
/// # Returns
///
/// * `Ok(String)` - The formatted source code
/// * `Err(CompilerError)` - If the source code has syntax errors
pub fn format_source(source: &str, options: &FormatOptions) -> Result<String> {
    // 1. Tokenize
    let tokens = lexer::tokenize(source)?;

    // 2. Parse to AST
    let ast = parser::parse(tokens, source)?;

    // 3. Pretty-print from AST (without comments)
    let mut formatter = Formatter::new(options.clone(), source);
    formatter.format_program(&ast);
    let formatted_code = formatter.output;

    // 4. Reinsert comments from original source
    let result = reinsert_comments(source, &formatted_code, options);

    Ok(result)
}

/// Check if the source code is already formatted correctly.
///
/// Returns `Ok(true)` if the source is already formatted,
/// `Ok(false)` if it needs formatting, or an error if the source has syntax errors.
pub fn check_format(source: &str, options: &FormatOptions) -> Result<bool> {
    let formatted = format_source(source, options)?;
    Ok(formatted == source)
}

/// Format a Liva source file in place.
///
/// Reads the file, formats it, and writes it back.
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
// Comment reinsertion
// ======================================================================

/// Represents a comment with its context in the original source
#[derive(Debug)]
struct SourceComment {
    /// The comment text (including // prefix)
    text: String,
    /// The code that preceded this comment on the same line (if any)
    preceding_code: String,
    /// Lines of code immediately before this comment (for matching position)
    context_before: Vec<String>,
    /// Whether this comment is standalone (own line) or inline (after code)
    is_standalone: bool,
}

/// Extract comments with their surrounding context from the original source
fn extract_source_comments(source: &str) -> Vec<SourceComment> {
    let lines: Vec<&str> = source.lines().collect();
    let mut comments = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        if let Some(comment_pos) = find_line_comment(line) {
            let before_comment = line[..comment_pos].trim_end();
            let comment_text = line[comment_pos..].trim_end();
            let is_standalone = before_comment.is_empty();
            
            // Get preceding code lines for context matching
            let mut context_before = Vec::new();
            let start = if i >= 3 { i - 3 } else { 0 };
            for j in start..i {
                let ctx_line = lines[j].trim();
                if !ctx_line.is_empty() && find_line_comment(ctx_line).map_or(true, |p| p > 0) {
                    // Get code part only (without comments)
                    let code = if let Some(cp) = find_line_comment(ctx_line) {
                        ctx_line[..cp].trim().to_string()
                    } else {
                        ctx_line.to_string()
                    };
                    if !code.is_empty() {
                        context_before.push(code);
                    }
                }
            }
            
            comments.push(SourceComment {
                text: comment_text.to_string(),
                preceding_code: before_comment.to_string(),
                context_before,
                is_standalone,
            });
        }
    }
    
    comments
}

/// Reinsert comments from the original source into the formatted code.
///
/// Strategy:
/// - Standalone comments (on their own line): Find the best position by matching
///   surrounding code context, then insert at the same relative position.
/// - Inline comments (after code): Find the matching code line in formatted output
///   and append the comment.
fn reinsert_comments(original: &str, formatted: &str, _options: &FormatOptions) -> String {
    let source_comments = extract_source_comments(original);
    
    if source_comments.is_empty() {
        return formatted.to_string();
    }
    
    let mut result_lines: Vec<String> = formatted.lines().map(|l| l.to_string()).collect();
    
    // Collect all insertions first
    let mut standalone_insertions: Vec<(usize, String)> = Vec::new();
    
    // Track which formatted lines have been used for inline comments
    let mut used_for_inline: Vec<bool> = vec![false; result_lines.len()];
    
    for comment in &source_comments {
        if comment.is_standalone {
            let best_pos = find_comment_position(&result_lines, comment);
            
            // Determine indentation from surrounding code
            let indent = if best_pos < result_lines.len() {
                get_line_indent(&result_lines[best_pos])
            } else if best_pos > 0 {
                get_line_indent(&result_lines[best_pos - 1])
            } else {
                String::new()
            };
            
            standalone_insertions.push((best_pos, format!("{}{}", indent, comment.text)));
        } else {
            // Inline comment: find the matching code line
            if let Some(pos) = find_matching_code_line_excluding(&result_lines, &comment.preceding_code, &used_for_inline) {
                result_lines[pos] = format!("{}  {}", result_lines[pos], comment.text);
                used_for_inline[pos] = true;
            }
        }
    }
    
    // Apply standalone insertions in reverse order (to preserve correct positions)
    standalone_insertions.sort_by(|a, b| b.0.cmp(&a.0));
    for (pos, text) in standalone_insertions {
        let insert_at = pos.min(result_lines.len());
        result_lines.insert(insert_at, text);
    }
    
    let mut result = result_lines.join("\n");
    if formatted.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Find the best position to insert a standalone comment in the formatted code
fn find_comment_position(lines: &[String], comment: &SourceComment) -> usize {
    if comment.context_before.is_empty() {
        // Comment at the very beginning of the file
        return 0;
    }
    
    // Try to match the last context line
    let last_context = comment.context_before.last().unwrap();
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == last_context || code_matches(trimmed, last_context) {
            return i + 1; // Insert after the matching line
        }
    }
    
    // Try partial matching with earlier context
    for ctx in comment.context_before.iter().rev() {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if code_matches(trimmed, ctx) {
                return i + 1;
            }
        }
    }
    
    // Fallback: append at end
    lines.len()
}

/// Check if two code lines match (allowing for formatting differences)
fn code_matches(formatted: &str, original: &str) -> bool {
    // Normalize whitespace for comparison
    let norm_f: String = formatted.split_whitespace().collect::<Vec<_>>().join(" ");
    let norm_o: String = original.split_whitespace().collect::<Vec<_>>().join(" ");
    norm_f == norm_o
}

/// Find a matching code line, excluding already-used positions
fn find_matching_code_line_excluding(lines: &[String], code: &str, used: &[bool]) -> Option<usize> {
    let norm_code: String = code.split_whitespace().collect::<Vec<_>>().join(" ");
    
    for (i, line) in lines.iter().enumerate() {
        if used[i] {
            continue;
        }
        let norm_line: String = line.trim().split_whitespace().collect::<Vec<_>>().join(" ");
        if norm_line == norm_code {
            return Some(i);
        }
    }
    None
}

/// Get the leading whitespace of a line
fn get_line_indent(line: &str) -> String {
    let trimmed = line.trim_start();
    line[..line.len() - trimmed.len()].to_string()
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
        assert!(output.contains("add(a, b) => a + b\n\nsub(a, b) => a - b"));
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
    fn test_format_return() {
        let input = "add(a: number, b: number): number{return a + b}";
        let output = fmt(input);
        assert!(output.contains("return a + b"));
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
        assert!(output.contains("// Header comment"), "header comment should be preserved. Got: {}", output);
        assert!(output.contains("// inner comment"), "inner comment should be preserved. Got: {}", output);
    }

    #[test]
    fn test_format_preserves_inline_comments() {
        let input = "main() {\nlet x = 10 // the value\n}";
        let output = fmt(input);
        assert!(output.contains("// the value"), "inline comment should be preserved. Got: {}", output);
    }
}
