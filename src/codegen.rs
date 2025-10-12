use crate::ast::*;
use crate::desugaring::DesugarContext;
use crate::error::Result;
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
        let has_methods = type_decl.members.iter().any(|m| matches!(m, Member::Method(_)));
        
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

        self.writeln(&format!("{}{}: {},", vis, self.sanitize_name(&field.name), type_str));
        Ok(())
    }

    fn generate_method(&mut self, method: &MethodDecl) -> Result<()> {
        let vis = match method.visibility {
            Visibility::Public => "pub ",
            Visibility::Protected => "pub(super) ",
            Visibility::Private => "",
        };

        let async_kw = if method.is_async_inferred { "async " } else { "" };

        let type_params = if !method.type_params.is_empty() {
            // Add basic trait bounds for generic types
            let bounded_params: Vec<String> = method.type_params.iter()
                .map(|param| format!("{}: std::cmp::PartialOrd", param))
                .collect();
            format!("<{}>", bounded_params.join(", "))
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
            vis, async_kw, self.sanitize_name(&method.name), type_params, params_str, return_type
        ).unwrap();

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

        let _type_params = if !func.type_params.is_empty() {
            // Add basic trait bounds for generic types
            let bounded_params: Vec<String> = func.type_params.iter()
                .map(|param| format!("{}: std::cmp::PartialOrd", param))
                .collect();
            format!("<{}>", bounded_params.join(", "))
        } else {
            String::new()
        };
        let params_str = self.generate_params(&func.params, false)?;
        let return_type = if let Some(ret) = &func.return_type {
            format!(" -> {}", ret.to_rust_type())
        } else if func.expr_body.is_some() {
            // For expression-bodied functions without explicit return type, infer i32
            " -> i32".to_string()
        } else {
            String::new()
        };

        write!(
            self.output,
            "{}{}fn {}({})",
            tokio_attr, async_kw, self.sanitize_name(&func.name), params_str
        ).unwrap();
        
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
        self.writeln(&format!("fn test_{}() {{", self.sanitize_test_name(&test.name)));
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
                write!(self.output, "let {}", self.sanitize_name(&var.name)).unwrap();
                
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
                // Try to infer type from literal
                let type_str = self.infer_const_type(&const_decl.init);
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
                write!(self.output, "Err({}) => {{\n", self.sanitize_name(&try_catch.catch_var)).unwrap();
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
                write!(self.output, "{}", self.sanitize_name(name)).unwrap();
            }
            Expr::Binary { op, left, right } => {
                // Only add parentheses for complex expressions
                let needs_parens = !matches!(left.as_ref(), Expr::Literal(_) | Expr::Identifier(_)) ||
                                  !matches!(right.as_ref(), Expr::Literal(_) | Expr::Identifier(_));
                
                if needs_parens {
                    self.output.push('(');
                }
                self.generate_expr(left)?;
                write!(self.output, " {} ", op).unwrap();
                self.generate_expr(right)?;
                if needs_parens {
                    self.output.push(')');
                }
            }
            Expr::Unary { op, operand } => {
                match op {
                    crate::ast::UnOp::Await => {
                        self.generate_expr(operand)?;
                        self.output.push_str(".await");
                    }
                    _ => {
                        write!(self.output, "{}", op).unwrap();
                        self.generate_expr(operand)?;
                    }
                }
            }
            Expr::Ternary { condition, then_expr, else_expr } => {
                self.output.push_str("if ");
                self.generate_expr(condition)?;
                self.output.push_str(" { ");
                self.generate_expr(then_expr)?;
                self.output.push_str(" } else { ");
                self.generate_expr(else_expr)?;
                self.output.push_str(" }");
            }
            Expr::Call { callee, args } => {
                // Special handling for print function
                if let Expr::Identifier(name) = callee.as_ref() {
                    if name == "print" {
                        if args.is_empty() {
                            self.output.push_str("println!()");
                        } else {
                            self.output.push_str("println!(\"");
                            for _arg in args.iter() {
                                self.output.push_str("{}");
                            }
                            self.output.push_str("\", ");
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    self.output.push_str(", ");
                                }
                                self.generate_expr(arg)?;
                            }
                            self.output.push_str(")");
                        }
                        return Ok(());
                    }
                }
                
                // Regular function call
                self.generate_expr(callee)?;
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push(')');
            }
            Expr::Member { object, property } => {
                self.generate_expr(object)?;
                write!(self.output, ".{}", self.sanitize_name(property)).unwrap();
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
            Expr::AsyncCall { callee, args } => {
                self.output.push_str("tokio::spawn(async move { ");
                self.generate_expr(callee)?;
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push_str(") }).await.unwrap()");
            }
            Expr::ParallelCall { callee, args } => {
                self.output.push_str("std::thread::spawn(move || ");
                self.generate_expr(callee)?;
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push_str(").join().unwrap()");
            }
            Expr::TaskCall { mode, callee, args } => {
                match mode {
                    ConcurrencyMode::Async => self.output.push_str("tokio::spawn("),
                    ConcurrencyMode::Parallel => self.output.push_str("std::thread::spawn(|| "),
                }
                write!(self.output, "{}(", callee).unwrap();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push(')');
                if matches!(mode, ConcurrencyMode::Parallel) {
                    self.output.push(')');
                }
                self.output.push(')');
            }
            Expr::FireCall { mode, callee, args } => {
                match mode {
                    ConcurrencyMode::Async => self.output.push_str("tokio::spawn("),
                    ConcurrencyMode::Parallel => self.output.push_str("std::thread::spawn(|| "),
                }
                write!(self.output, "{}(", callee).unwrap();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg)?;
                }
                self.output.push(')');
                if matches!(mode, ConcurrencyMode::Parallel) {
                    self.output.push(')');
                }
                self.output.push(')');
            }
            Expr::StringTemplate { parts } => {
                self.output.push_str("format!(\"");
                let mut format_args = Vec::new();
                
                for part in parts {
                    match part {
                        StringTemplatePart::Text(text) => {
                            self.output.push_str(text);
                        }
                        StringTemplatePart::Expr(expr) => {
                            self.output.push_str("{}");
                            format_args.push(expr);
                        }
                    }
                }
                
                self.output.push('"');
                for arg in format_args {
                    self.output.push_str(", ");
                    self.generate_expr(arg)?;
                }
                self.output.push(')');
            }
        }
        Ok(())
    }

    fn generate_literal(&mut self, lit: &Literal) -> Result<()> {
        match lit {
            Literal::Int(n) => write!(self.output, "{}", n).unwrap(),
            Literal::Float(f) => write!(self.output, "{}", f).unwrap(),
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
        name.replace(' ', "_")
            .replace('-', "_")
            .to_lowercase()
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

pub fn generate(ctx: DesugarContext) -> Result<(String, String)> {
    // This function should receive both the desugaring context and the AST
    // For now, return placeholder content
    let main_rs = "// Generated by livac v0.6\n\nfn main() {\n    println!(\"Hello from Liva!\");\n}\n".to_string();
    
    let cargo_toml = generate_cargo_toml(&ctx)?;
    
    Ok((main_rs, cargo_toml))
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
         [dependencies]\n"
    );

    // Add tokio if async is used
    if ctx.has_async {
        cargo_toml.push_str("tokio = { version = \"1\", features = [\"full\"] }\n");
    }

    // Add serde_json for object literals
    cargo_toml.push_str("serde_json = \"1.0\"\n");

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