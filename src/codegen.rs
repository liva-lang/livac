use crate::ast::*;
use crate::desugaring::DesugarContext;
use crate::error::{CompilerError, Result};
use crate::ir;
use std::collections::HashMap;
use std::fmt::Write;

pub struct CodeGenerator {
    output: String,
    indent_level: usize,
    ctx: DesugarContext,
}

impl CodeGenerator {
    fn new(ctx: DesugarContext) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            ctx,
        }
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

        // Generate top-level items
        for item in &program.items {
            self.generate_top_level(item)?;
            self.output.push('\n');
        }

        Ok(())
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
            TopLevel::Class(class) => self.generate_class(class),
            TopLevel::Function(func) => self.generate_function(func),
            TopLevel::Test(test) => self.generate_test(test),
        }
    }

    fn generate_type_decl(&mut self, type_decl: &TypeDecl) -> Result<()> {
        // Generate struct
        self.writeln(&format!("pub struct {} {{", type_decl.name));
        self.indent();

        for member in &type_decl.members {
            if let Member::Field(field) = member {
                self.generate_field(field)?;
            }
        }

        self.dedent();
        self.writeln("}");
        self.output.push('\n');

        // Generate impl block for methods
        let has_methods = type_decl
            .members
            .iter()
            .any(|m| matches!(m, Member::Method(_)));

        if has_methods {
            self.writeln(&format!("impl {} {{", type_decl.name));
            self.indent();

            for member in &type_decl.members {
                if let Member::Method(method) = member {
                    self.generate_method(method)?;
                    self.output.push('\n');
                }
            }

            self.dedent();
            self.writeln("}");
        }

        Ok(())
    }

    fn generate_class(&mut self, class: &ClassDecl) -> Result<()> {
        // Handle inheritance with composition
        if let Some(base) = &class.base {
            self.writeln(&format!("// Class {} extends {}", class.name, base));
            self.writeln(&format!("pub struct {} {{", class.name));
            self.indent();
            self.writeln(&format!("pub base: {},", base));
        } else {
            self.writeln(&format!("pub struct {} {{", class.name));
            self.indent();
        }

        for member in &class.members {
            if let Member::Field(field) = member {
                self.generate_field(field)?;
            }
        }

        self.dedent();
        self.writeln("}");
        self.output.push('\n');

        // Generate impl block
        let has_methods = class.members.iter().any(|m| matches!(m, Member::Method(_)));

        if has_methods {
            self.writeln(&format!("impl {} {{", class.name));
            self.indent();

            for member in &class.members {
                if let Member::Method(method) = member {
                    self.generate_method(method)?;
                    self.output.push('\n');
                }
            }

            self.dedent();
            self.writeln("}");
        }

        Ok(())
    }

    fn generate_field(&mut self, field: &FieldDecl) -> Result<()> {
        let vis = match field.visibility {
            Visibility::Public => "pub ",
            Visibility::Protected => "pub(super) ",
            Visibility::Private => "",
        };

        let type_str = if let Some(type_ref) = &field.type_ref {
            type_ref.to_rust_type()
        } else {
            "()".to_string()
        };

        self.writeln(&format!(
            "{}{}: {},",
            vis,
            self.sanitize_name(&field.name),
            type_str
        ));
        Ok(())
    }

    fn generate_method(&mut self, method: &MethodDecl) -> Result<()> {
        let vis = match method.visibility {
            Visibility::Public => "pub ",
            Visibility::Protected => "pub(super) ",
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
                .map(|param| format!("{}: std::cmp::PartialOrd", param))
                .collect();
            format!("<{}>", bounded.join(", "))
        } else {
            String::new()
        };

        let params_str = self.generate_params(&method.params, true)?;

        let return_type = if let Some(ret) = &method.return_type {
            format!(" -> {}", ret.to_rust_type())
        } else {
            String::new()
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

        if let Some(expr) = &method.expr_body {
            self.output.push_str(" { ");
            self.generate_expr(expr)?;
            self.output.push_str(" }\n");
        } else if let Some(body) = &method.body {
            self.output.push_str(" {\n");
            self.indent();
            self.generate_block_inner(body)?;
            self.dedent();
            self.writeln("}");
        }

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
                .map(|param| format!("{}: std::cmp::PartialOrd", param))
                .collect();
            format!("<{}>", bounded.join(", "))
        } else {
            String::new()
        };
        let params_str = self.generate_params(&func.params, false)?;
        let return_type = if let Some(ret) = &func.return_type {
            format!(" -> {}", ret.to_rust_type())
        } else if func.expr_body.is_some() {
            // For expression-bodied functions without explicit return type, default to i32 for arithmetic
            " -> i32".to_string()
        } else if func.body.is_some() {
            // For block-bodied functions, check if there's a return statement
            if let Some(body) = &func.body {
                if self.block_has_return(body) {
                    " -> f64".to_string() // Default to f64 for functions that return values
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
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
            self.write_indent();
            self.generate_expr(expr)?;
            self.output.push('\n');
            self.dedent();
            self.writeln("}");
        } else if let Some(body) = &func.body {
            self.output.push_str(" {\n");
            self.indent();
            self.generate_block_inner(body)?;
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

    fn generate_params(&mut self, params: &[Param], is_method: bool) -> Result<String> {
        let mut result = String::new();

        if is_method {
            result.push_str("&self");
            if !params.is_empty() {
                result.push_str(", ");
            }
        }

        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }

            let param_name = self.sanitize_name(&param.name);
            let type_str = if let Some(type_ref) = &param.type_ref {
                type_ref.to_rust_type()
            } else {
                "i32".to_string() // Default to i32
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

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::VarDecl(var) => {
                self.write_indent();
                write!(self.output, "let mut {}", self.sanitize_name(&var.name)).unwrap();

                if let Some(type_ref) = &var.type_ref {
                    write!(self.output, ": {}", type_ref.to_rust_type()).unwrap();
                }

                if let Some(init) = &var.init {
                    self.output.push_str(" = ");
                    self.generate_expr(init)?;
                }

                self.output.push_str(";\n");
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
                self.write_indent();
                self.generate_expr(&assign.target)?;
                self.output.push_str(" = ");
                self.generate_expr(&assign.value)?;
                self.output.push_str(";\n");
            }
            Stmt::If(if_stmt) => {
                self.write_indent();
                self.output.push_str("if ");
                self.generate_expr(&if_stmt.condition)?;
                self.output.push_str(" {\n");
                self.indent();
                self.generate_block_inner(&if_stmt.then_branch)?;
                self.dedent();
                self.write_indent();
                self.output.push('}');

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.output.push_str(" else {\n");
                    self.indent();
                    self.generate_block_inner(else_branch)?;
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
                self.write_indent();
                write!(self.output, "for {} in ", self.sanitize_name(&for_stmt.var)).unwrap();
                self.generate_expr(&for_stmt.iterable)?;
                self.output.push_str(" {\n");
                self.indent();
                self.generate_block_inner(&for_stmt.body)?;
                self.dedent();
                self.writeln("}");
            }
            Stmt::Switch(switch_stmt) => {
                self.write_indent();
                self.output.push_str("match ");
                self.generate_expr(&switch_stmt.discriminant)?;
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
                    self.generate_expr(expr)?;
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
        }
        Ok(())
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(lit) => self.generate_literal(lit)?,
            Expr::Identifier(name) => {
                // Check if this is a constant (uppercase identifier)
                if name.chars().all(|c| c.is_uppercase() || c == '_') {
                    write!(self.output, "{}", name).unwrap();
                } else {
                    write!(self.output, "{}", self.sanitize_name(name)).unwrap();
                }
            }
            Expr::Binary { op, left, right } => {
                if matches!(op, BinOp::Add)
                    && (self.expr_is_stringy(left) || self.expr_is_stringy(right))
                {
                    self.output.push_str("format!(\"{}{}\", ");
                    self.generate_expr(left)?;
                    self.output.push_str(", ");
                    self.generate_expr(right)?;
                    self.output.push(')');
                } else {
                    self.generate_binary_operation(op, left, right)?;
                }
            }
            Expr::Unary { op, operand } => match op {
                crate::ast::UnOp::Await => {
                    self.generate_expr(operand)?;
                    self.output.push_str(".await");
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
                self.output.push_str("if ");
                self.generate_expr(condition)?;
                self.output.push_str(" { ");
                self.generate_expr(then_expr)?;
                self.output.push_str(" } else { ");
                self.generate_expr(else_expr)?;
                self.output.push_str(" }");
            }
            Expr::Call(call) => {
                self.generate_call_expr(call)?;
            }
            Expr::Member { object, property } => {
                self.generate_expr(object)?;

                if property == "length" {
                    self.output.push_str(".len()");
                } else {
                    // For serde_json::Value objects, use bracket notation instead of dot notation
                    match object.as_ref() {
                        Expr::Identifier(_) => {
                            // Check if this is likely a serde_json::Value (from object literals in arrays)
                            // For now, use bracket notation for any identifier access
                            write!(self.output, "[\"{}\"]", property).unwrap();
                        }
                        _ => {
                            write!(self.output, ".{}", self.sanitize_name(property)).unwrap();
                        }
                    }
                }
            }
            Expr::Index { object, index } => {
                self.generate_expr(object)?;
                self.output.push('[');
                self.generate_expr(index)?;
                self.output.push(']');
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
                                    _ => self.output.push(ch),
                                }
                            }
                        }
                        StringTemplatePart::Expr(expr) => match expr.as_ref() {
                            Expr::Literal(_) => {
                                self.output.push_str("{}");
                            }
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
                    for (idx, expr) in exprs.iter().enumerate() {
                        if idx > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(expr)?;
                    }
                }

                self.output.push(')');
            }
            Expr::Lambda(_) => {
                return Err(CompilerError::CodegenError(
                    "Lambda expressions are not yet supported in code generation".into(),
                ));
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
            if name == "print" {
                if call.args.is_empty() {
                    self.output.push_str("println!()");
                } else {
                    self.output.push_str("println!(\"");
                    for arg in call.args.iter() {
                        match arg {
                            Expr::ArrayLiteral(_) | Expr::ObjectLiteral(_) => {
                                self.output.push_str("{:?}");
                            }
                            _ => {
                                self.output.push_str("{}");
                            }
                        }
                    }
                    self.output.push_str("\", ");
                    for (i, arg) in call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg)?;
                    }
                    self.output.push(')');
                }
                return Ok(());
            }
        }

        self.generate_expr(&call.callee)?;
        self.output.push('(');
        for (i, arg) in call.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.generate_expr(arg)?;
        }
        self.output.push(')');
        Ok(())
    }

    fn generate_async_call(&mut self, call: &CallExpr) -> Result<()> {
        self.output.push_str("liva_rt::spawn_async(async move { ");
        self.generate_expr(&call.callee)?;
        self.output.push('(');
        for (i, arg) in call.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.generate_expr(arg)?;
        }
        self.output.push_str(") }).await.unwrap()");
        Ok(())
    }

    fn generate_parallel_call(&mut self, call: &CallExpr) -> Result<()> {
        self.output.push_str("liva_rt::spawn_parallel(move || ");
        self.generate_expr(&call.callee)?;
        self.output.push('(');
        for (i, arg) in call.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.generate_expr(arg)?;
        }
        self.output.push(')');
        self.output.push_str(").join().unwrap()");
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
                self.output.push_str("); });");
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
                self.output.push_str("); });");
            }
        }
        Ok(())
    }

    fn generate_binary_operation(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Result<()> {
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

    fn expr_is_stringy(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Literal(Literal::String(_)) => true,
            Expr::StringTemplate { .. } => true,
            Expr::Binary {
                op: BinOp::Add,
                left,
                right,
            } => self.expr_is_stringy(left) || self.expr_is_stringy(right),
            _ => false,
        }
    }

    fn generate_literal(&mut self, lit: &Literal) -> Result<()> {
        match lit {
            Literal::Int(n) => write!(self.output, "{}", n).unwrap(),
            Literal::Float(f) => write!(self.output, "{:?}", f).unwrap(),
            Literal::String(s) => write!(self.output, "\"{}\"", s.escape_default()).unwrap(),
            Literal::Char(c) => write!(self.output, "'{}'", c.escape_default()).unwrap(),
            Literal::Bool(b) => write!(self.output, "{}", b).unwrap(),
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
            _ => "i32".to_string(),
        }
    }

    fn sanitize_name(&self, name: &str) -> String {
        // Convert to snake_case and remove visibility prefixes
        let name = name.trim_start_matches('_');
        self.to_snake_case(name)
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
}

struct IrCodeGenerator<'a> {
    output: String,
    indent_level: usize,
    ctx: &'a DesugarContext,
    scope_formats: Vec<HashMap<String, FormatKind>>,
}

#[derive(Copy, Clone, PartialEq)]
enum FormatKind {
    Display,
    Debug,
}

impl FormatKind {
    fn placeholder(self) -> &'static str {
        match self {
            FormatKind::Display => "{}",
            FormatKind::Debug => "{:?}",
        }
    }
}

impl<'a> IrCodeGenerator<'a> {
    fn new(ctx: &'a DesugarContext) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            ctx,
            scope_formats: vec![HashMap::new()],
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
            ir::Expr::ObjectLiteral(_) => FormatKind::Debug,
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
            ir::Expr::ObjectLiteral(_) => true,
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

        let uses_async_helpers = self.ctx.has_async || module_has_async_concurrency(module);
        let uses_parallel_helpers = module_has_parallel_concurrency(module);

        if uses_async_helpers || uses_parallel_helpers {
            self.emit_runtime_module(uses_async_helpers, uses_parallel_helpers);
        }

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

        for (crate_name, alias) in emitted {
            if let Some(alias_name) = alias {
                writeln!(self.output, "use {} as {};", crate_name, alias_name).unwrap();
            } else {
                writeln!(self.output, "use {};", crate_name).unwrap();
            }
        }

        if !self.output.trim().is_empty() {
            self.output.push('\n');
        }
    }

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
                "pub fn for_boost<I, T, F>(iterable: I, func: F, options: ParallelForOptions)",
            );
            self.writeln("where I: IntoIterator<Item = T>, T: Send + 'static, F: Fn(T) + Send + Sync + 'static,");
            self.writeln("{");
            self.indent();
            self.writeln("let items: Vec<T> = iterable.into_iter().collect();");
            self.writeln("execute_parallel(items, func, options, \"for boost\");");
            self.dedent();
            self.writeln("}");
        }

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
                self.generate_expr(value)?;
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
                ir::DataParallelPolicy::Boost => {
                    self.generate_boost_for(var, iterable, body, options)?;
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

    fn generate_boost_for(
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
            "liva_rt::for_boost(__liva_iter, move |{}| {{\n",
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
                    self.generate_expr(left)?;
                    self.output.push_str(", ");
                    self.generate_expr(right)?;
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
                    if matches!(
                        op,
                        ir::BinaryOp::Add
                            | ir::BinaryOp::Sub
                            | ir::BinaryOp::Mul
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
                self.output.push_str("if ");
                self.generate_expr(condition)?;
                self.output.push_str(" { ");
                self.generate_expr(then_expr)?;
                self.output.push_str(" } else { ");
                self.generate_expr(else_expr)?;
                self.output.push_str(" }");
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
                self.generate_expr(object)?;

                if property == "length" {
                    self.output.push_str(".len()");
                    return Ok(());
                }

                // For serde_json::Value objects, use bracket notation instead of dot notation
                // This handles cases where we're accessing properties of object literals in arrays
                match object.as_ref() {
                    ir::Expr::Identifier(name) if name == "self" => {
                        write!(self.output, ".{}", self.sanitize_name(property)).unwrap();
                    }
                    ir::Expr::Identifier(_)
                    | ir::Expr::Member { .. }
                    | ir::Expr::Index { .. }
                    | ir::Expr::Call { .. } => {
                        write!(self.output, "[\"{}\"]", property).unwrap();
                    }
                    _ => {
                        write!(self.output, "[\"{}\"]", property).unwrap();
                    }
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
            ir::Expr::Unsupported(_) => Err(CompilerError::CodegenError(
                "Unsupported expression in IR generator".into(),
            )),
        }
    }

    fn generate_literal(&mut self, lit: &ir::Literal) -> Result<()> {
        match lit {
            ir::Literal::Int(v) => write!(self.output, "{}", v).unwrap(),
            ir::Literal::Float(v) => write!(self.output, "{:?}", v).unwrap(),
            ir::Literal::Bool(v) => write!(self.output, "{}", v).unwrap(),
            ir::Literal::String(s) => write!(self.output, "\"{}\"", s.escape_default()).unwrap(),
            ir::Literal::Char(c) => write!(self.output, "'{}'", c.escape_default()).unwrap(),
            ir::Literal::Null => self.output.push_str("()"),
        }
        Ok(())
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
        ir::Expr::ArrayLiteral(elements) => elements.iter().any(expr_has_unsupported),
    }
}

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

fn module_has_parallel_concurrency(module: &ir::Module) -> bool {
    module.items.iter().any(|item| match item {
        ir::Item::Function(func) => block_has_parallel_concurrency(&func.body),
        ir::Item::Test(test) => block_has_parallel_concurrency(&test.body),
        ir::Item::Unsupported(_) => false,
    })
}

fn block_has_async_concurrency(block: &ir::Block) -> bool {
    block.statements.iter().any(stmt_has_async_concurrency)
}

fn block_has_parallel_concurrency(block: &ir::Block) -> bool {
    block.statements.iter().any(stmt_has_parallel_concurrency)
}

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
                    | ir::DataParallelPolicy::Boost
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
        &ir::Expr::ArrayLiteral(ref elements) => elements.iter().any(expr_has_async_concurrency),
        &ir::Expr::Literal(_) | &ir::Expr::Identifier(_) | &ir::Expr::Unsupported(_) => false,
    }
}

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
        &ir::Expr::ArrayLiteral(ref elements) => elements.iter().any(expr_has_parallel_concurrency),
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
    if module_has_unsupported(module) {
        return generate_with_ast(program, ctx);
    }

    let ir_gen = IrCodeGenerator::new(&ctx);
    let rust_code = ir_gen.generate(module)?;
    let cargo_toml = generate_cargo_toml(&ctx)?;
    Ok((rust_code, cargo_toml))
}

pub fn generate_with_ast(program: &Program, ctx: DesugarContext) -> Result<(String, String)> {
    let mut generator = CodeGenerator::new(ctx);
    generator.generate_program(program)?;

    let cargo_toml = generate_cargo_toml(&generator.ctx)?;

    Ok((generator.output, cargo_toml))
}

fn generate_cargo_toml(ctx: &DesugarContext) -> Result<String> {
    let mut cargo_toml = String::from(
        "[package]\n\
         name = \"liva_project\"\n\
         version = \"0.1.0\"\n\
         edition = \"2021\"\n\n\
         [dependencies]\n",
    );

    // Add tokio if async is used
    if ctx.has_async {
        cargo_toml.push_str("tokio = { version = \"1\", features = [\"full\"] }\n");
    }

    // Add serde_json for object literals
    cargo_toml.push_str("serde_json = \"1.0\"\n");

    if ctx.has_parallel {
        cargo_toml.push_str("rayon = \"1.11\"\n");
    }

    // Add user-specified crates
    for (crate_name, _) in &ctx.rust_crates {
        if crate_name != "tokio" && crate_name != "serde_json" {
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
        });

        assert_eq!(gen.to_snake_case("CamelCase"), "camel_case");
        assert_eq!(gen.to_snake_case("myFunction"), "my_function");
        assert_eq!(gen.to_snake_case("snake_case"), "snake_case");
    }
}
