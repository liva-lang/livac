use crate::ast::*;
use crate::desugaring::DesugarContext;
use crate::error::{CompilerError, Result, SemanticErrorInfo};
use crate::ir;
use std::collections::HashMap;
use std::fmt::Write;

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
}

pub struct CodeGenerator {
    output: String,
    indent_level: usize,
    ctx: DesugarContext,
    in_method: bool,
    in_assignment_target: bool,
    in_fallible_function: bool,
    in_string_template: bool, // Track if we're inside a string template
    bracket_notation_vars: std::collections::HashSet<String>,
    class_instance_vars: std::collections::HashSet<String>,
    array_vars: std::collections::HashSet<String>, // Track which variables are arrays
    // --- Class/type metadata (for inheritance and field resolution)
    class_fields: std::collections::HashMap<String, std::collections::HashSet<String>>,
    class_base: std::collections::HashMap<String, Option<String>>,
    var_types: std::collections::HashMap<String, String>, // var -> ClassName
    fallible_functions: std::collections::HashSet<String>, // Track which functions are fallible
    // --- Phase 2: Lazy await/join tracking
    pending_tasks: std::collections::HashMap<String, TaskInfo>, // Variables that hold unawaited Tasks
    // --- Phase 3: Error binding variables (Option<String> type)
    error_binding_vars: std::collections::HashSet<String>, // Variables from error binding (second variable in let x, err = ...)
    // --- Phase 4: Join combining optimization
    #[allow(dead_code)]
    awaitable_tasks: Vec<String>, // Tasks that can be combined with tokio::join!
}

impl CodeGenerator {
    fn new(ctx: DesugarContext) -> Self {
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
            class_fields: std::collections::HashMap::new(),
            class_base: std::collections::HashMap::new(),
            var_types: std::collections::HashMap::new(),
            fallible_functions: std::collections::HashSet::new(),
            pending_tasks: std::collections::HashMap::new(),
            error_binding_vars: std::collections::HashSet::new(),
            awaitable_tasks: Vec::new(),
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
        self.class_base.clear();
        for item in &program.items {
            if let TopLevel::Class(cls) = item {
                let mut fields = std::collections::HashSet::new();
                for m in &cls.members {
                    if let Member::Field(f) = m {
                        fields.insert(f.name.clone());
                    }
                }
                self.class_fields.insert(cls.name.clone(), fields);
                self.class_base.insert(cls.name.clone(), cls.base.clone());
            }
        }

        // Always include concurrency runtime for now
        println!("DEBUG: Including liva_rt module");
        self.writeln("mod liva_rt {");
        self.indent();
        self.writeln("use std::future::Future;");
        self.writeln("use tokio::task::JoinHandle;");
        self.writeln("");

        // Add Error type for fallibility system
        self.writeln("/// Runtime error type for fallible operations");
        self.writeln("#[derive(Debug, Clone)]");
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

        self.dedent();
        self.writeln("}");
        self.writeln("");

        // Generate top-level items
        for item in &program.items {
            println!("DEBUG: Processing top-level item: {:?}", item);
            match item {
                TopLevel::Class(cls) => println!("DEBUG: Found class: {}", cls.name),
                TopLevel::Function(func) => println!("DEBUG: Found function: {}", func.name),
                _ => println!("DEBUG: Found other item: {:?}", item),
            }
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
            TopLevel::Class(class) => {
                println!("DEBUG: Generating class {}", class.name);
                self.generate_class(class)
            }
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
                    self.generate_method(method, None)?;
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
            self.writeln("#[derive(Debug, Clone, Default)]");
            self.writeln(&format!("pub struct {} {{", class.name));
            self.indent();
            self.writeln(&format!("pub base: {},", base));
        } else {
            self.writeln("#[derive(Debug, Clone, Default)]");
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

        self.writeln(&format!("impl {} {{", class.name));
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
            )?;
            write!(self.output, "{}) -> Self {{\n", params_str).unwrap();
            self.indent();
            self.write_indent();
            self.output.push_str("Self {\n");
            self.indent();

            // Generate field assignments based on parameters
            if let Some(base_name) = &class.base {
                // Initialize base with matching parameters in base field order (by field names)
                self.write_indent();
                self.output.push_str("base: ");
                write!(self.output, "{}::new(", base_name).unwrap();

                // Pass params that coinciden por nombre con campos de la base
                let mut first = true;
                if let Some(base_fields) = self.class_fields.get(base_name) {
                    for bf in base_fields {
                        if let Some(p) = constructor_method.params.iter().find(|p| &p.name == bf) {
                            if !first {
                                self.output.push_str(", ");
                            } else {
                                first = false;
                            }
                            if p.name == "name" {
                                // Common name-as-String convenience
                                self.output.push_str(&format!(
                                    "{}.to_string()",
                                    self.sanitize_name(&p.name)
                                ));
                            } else {
                                self.output.push_str(&self.sanitize_name(&p.name));
                            }
                        }
                    }
                }
                self.output.push_str("),\n");

                // Then handle own fields (excluding base fields)
                for param in &constructor_method.params {
                    if let Some(base_fields) = self.class_fields.get(base_name) {
                        if base_fields.contains(&param.name) {
                            continue;
                        }
                    }
                    self.write_indent();
                    let field_name = self.sanitize_name(&param.name);
                    if param.name == "name" {
                        self.output
                            .push_str(&format!("{}: {}.to_string(),\n", field_name, field_name));
                    } else {
                        self.output
                            .push_str(&format!("{}: {},\n", field_name, field_name));
                    }
                }
            } else {
                for param in &constructor_method.params {
                    self.write_indent();
                    let field_name = self.sanitize_name(&param.name);
                    if param.name == "name" {
                        self.output
                            .push_str(&format!("{}: {}.to_string(),\n", field_name, field_name));
                    } else {
                        self.output
                            .push_str(&format!("{}: {},\n", field_name, field_name));
                    }
                }
            }

            // Add default values for fields not covered by parameters
            for member in &class.members {
                if let Member::Field(field) = member {
                    if !constructor_method
                        .params
                        .iter()
                        .any(|p| p.name == field.name)
                    {
                        let default_value = match field.type_ref.as_ref() {
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
                        };
                        self.write_indent();
                        self.writeln(&format!(
                            "{}: {},",
                            self.sanitize_name(&field.name),
                            default_value
                        ));
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
                    let default_value = match field.type_ref.as_ref() {
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
                    };

                    self.writeln(&format!(
                        "{}: {},",
                        self.sanitize_name(&field.name),
                        default_value
                    ));
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

    fn generate_field(&mut self, field: &FieldDecl) -> Result<()> {
        let vis = match field.visibility {
            Visibility::Public => "pub ",
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

    fn infer_expr_type(&self, expr: &Expr, class: Option<&ClassDecl>) -> Option<String> {
        match expr {
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
                Literal::String(_) => Some(" -> String".to_string()),
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

        // Walk current class and its bases to find a matching field
        let mut cls_name = class.name.clone();
        loop {
            if let Some(fields) = self.class_fields.get(&cls_name) {
                if fields.contains(&field_name) || fields.contains(&format!("_{}", field_name)) {
                    // Busca el tipo en los miembros de la clase actual
                    for m in &class.members {
                        if let Member::Field(f) = m {
                            if f.name == field_name || f.name == format!("_{}", field_name) {
                                return f.type_ref.as_ref().map(|t| t.to_rust_type());
                            }
                        }
                    }
                    // Fallback simple
                    return Some(match field_name.as_str() {
                        "name" => "String".to_string(),
                        "age" => "i32".to_string(),
                        _ => "i32".to_string(),
                    });
                }
            }
            // Move to base
            if let Some(Some(base)) = self.class_base.get(&cls_name) {
                cls_name = base.clone();
            } else {
                break;
            }
        }
        None
    }

    #[allow(dead_code)]
    fn generate_constructor_method(&mut self, method: &MethodDecl) -> Result<()> {
        let vis = "pub";
        let _async_kw = "";
        let _type_params = String::new();

        let params_str = self.generate_params(&method.params, false, None, None)?; // false because constructor is not a method

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
            let field_name = self.sanitize_name(&param.name);
            // Add conversion for string fields
            if param.name == "name" {
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
                .map(|param| format!("{}: std::cmp::PartialOrd", param))
                .collect();
            format!("<{}>", bounded.join(", "))
        } else {
            String::new()
        };

        let params_str = self.generate_params(&method.params, true, class, Some(&method.name))?;

        let return_type = if let Some(ret) = &method.return_type {
            format!(" -> {}", ret.to_rust_type())
        } else if let Some(expr) = &method.expr_body {
            // Try to infer return type from expression
            self.infer_expr_type(expr, class)
                .unwrap_or_else(|| " -> ()".to_string())
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

        self.in_method = true;
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
                .map(|param| format!("{}: std::cmp::PartialOrd", param))
                .collect();
            format!("<{}>", bounded.join(", "))
        } else {
            String::new()
        };
        let params_str = self.generate_params(&func.params, false, None, None)?;

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
    ) -> Result<String> {
        let mut result = String::new();

        if is_method {
            // Use &mut self for methods that modify fields (setters)
            let is_setter = method_name.map_or(false, |name| name.starts_with("set"));
            if is_setter {
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

            let param_name = self.sanitize_name(&param.name);
            let type_str = if let Some(type_ref) = &param.type_ref {
                type_ref.to_rust_type()
            } else if let Some(cls) = class {
                // Try to infer from field types in the class
                self.infer_param_type_from_class(&param.name, cls, method_name)
                    .unwrap_or_else(|| "i32".to_string())
            } else {
                // Infer type based on parameter name (hack for constructor)
                match param.name.as_str() {
                    "name" => "String".to_string(),
                    "age" => "i32".to_string(),
                    "items" => "Vec<serde_json::Value>".to_string(),
                    _ => "i32".to_string(),
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
                        .map(|b| self.sanitize_name(&b.name))
                        .collect();

                    // Phase 3: Track the error variable (second binding) as Option<String>
                    if binding_names.len() == 2 {
                        self.error_binding_vars.insert(binding_names[1].clone());
                    }

                    if let Some(exec_policy) = task_exec_policy {
                        // Phase 2: Error binding with Task - store task without awaiting
                        // Generate: let task_name = async/par call();
                        let task_var_name = format!("{}_task", binding_names[0]);
                        write!(self.output, "let {} = ", task_var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str(";\n");

                        // Register as pending task with error binding
                        self.pending_tasks.insert(
                            binding_names[0].clone(),
                            TaskInfo {
                                is_error_binding: true,
                                binding_names: binding_names.clone(),
                                awaited: false,
                                exec_policy,
                            },
                        );
                    } else {
                        // Non-Task error binding (original behavior)
                        write!(self.output, "let (").unwrap();
                        for (i, binding) in var.bindings.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            write!(self.output, "{}", self.sanitize_name(&binding.name)).unwrap();
                        }

                        if is_fallible_call {
                            // Generate: let (value, err) = match expr { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };
                            self.output.push_str(") = match ");
                            self.generate_expr(&var.init)?;
                            self.output.push_str(" { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };\n");
                        } else {
                            // Check if the expression is a built-in conversion function that returns a tuple
                            let returns_tuple = self.is_builtin_conversion_call(&var.init);
                            
                            if returns_tuple {
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
                    let var_name = self.sanitize_name(&binding.name);

                    // Phase 2: Check if this is a Task assignment
                    if let Some(exec_policy) = task_exec_policy {
                        // Simple task binding (no error handling)
                        // Generate: let var_name_task = async/par call();
                        let task_var_name = format!("{}_task", var_name);
                        write!(self.output, "let {} = ", task_var_name).unwrap();
                        self.generate_expr(&var.init)?;
                        self.output.push_str(";\n");

                        // Register as pending task
                        self.pending_tasks.insert(
                            var_name.clone(),
                            TaskInfo {
                                is_error_binding: false,
                                binding_names: vec![var_name.clone()],
                                awaited: false,
                                exec_policy,
                            },
                        );
                    } else {
                        // Non-Task normal binding (original behavior)

                        // Check if initializing with an object literal - mark variable for bracket notation
                        if let Expr::ObjectLiteral(_) = &var.init {
                            self.bracket_notation_vars.insert(binding.name.clone());
                        }

                        // Check if initializing with an array literal - mark variable as array
                        if let Expr::ArrayLiteral(_) = &var.init {
                            self.array_vars.insert(binding.name.clone());
                        }
                        // Check if initializing with a method call that returns an array (map, filter, etc.)
                        else if let Expr::MethodCall(method_call) = &var.init {
                            if matches!(method_call.method.as_str(), "map" | "filter") {
                                self.array_vars.insert(binding.name.clone());
                            }
                        }
                        // Mark instances created via constructor call: let x = ClassName(...)
                        else if let Expr::Call(call) = &var.init {
                            if let Expr::Identifier(class_name) = &*call.callee {
                                self.class_instance_vars.insert(binding.name.clone());
                                self.var_types
                                    .insert(binding.name.clone(), class_name.clone());
                            }
                        }
                        // Mark instances created via struct literal: let x = ClassName { ... }
                        else if let Expr::StructLiteral { type_name, .. } = &var.init {
                            self.class_instance_vars.insert(binding.name.clone());
                            self.var_types
                                .insert(binding.name.clone(), type_name.clone());
                        }

                        write!(self.output, "let mut {}", var_name).unwrap();

                        if let Some(type_ref) = &binding.type_ref {
                            write!(self.output, ": {}", type_ref.to_rust_type()).unwrap();
                        }

                        self.output.push_str(" = ");
                        self.generate_expr(&var.init)?;
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
                self.generate_expr(&if_stmt.condition)?;
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
                    if self.in_fallible_function {
                        self.output.push_str("Ok(");
                        self.generate_expr(expr)?;
                        self.output.push(')');
                    } else {
                        self.generate_expr(expr)?;
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
                }

                self.generate_expr(object)?;

                if property == "length" {
                    self.output.push_str(".len()");
                } else {
                    // Use bracket notation for JSON objects, dot notation for structs
                    match object.as_ref() {
                        Expr::Identifier(var_name) => {
                            // For class instances, use dot notation. For everything else (JSON), use bracket notation
                            if self.is_class_instance(var_name)
                                || var_name.contains("person")
                                || var_name.contains("user")
                            {
                                // Prefer struct field access, but if the field lives in a base class, delegate via `.base`
                                let prop = self.sanitize_name(property);
                                if let Some(class_name) = self.var_types.get(var_name) {
                                    // Walk up inheritance chain to find declaring class
                                    let mut current = Some(class_name.clone());
                                    let mut path = String::new();
                                    let mut found = false;
                                    while let Some(cls) = current.clone() {
                                        if let Some(fields) = self.class_fields.get(&cls) {
                                            if fields.contains(&prop) {
                                                // Emit accumulated `.base` hops then `.prop`
                                                self.output.push_str(&path);
                                                write!(self.output, ".{}", prop).unwrap();
                                                found = true;
                                                break;
                                            }
                                        }
                                        // Hop to base
                                        if let Some(base_opt) = self.class_base.get(&cls) {
                                            if let Some(base_name) = base_opt.clone() {
                                                path.push_str(".base");
                                                current = Some(base_name);
                                                continue;
                                            }
                                        }
                                        break;
                                    }
                                    if !found {
                                        // default: assume field on current class
                                        write!(self.output, ".{}", prop).unwrap();
                                    }
                                } else {
                                    // default when type unknown
                                    write!(self.output, ".{}", prop).unwrap();
                                }

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
                            } else {
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
                                        self.output.push_str(".as_str().unwrap_or(\"\")");
                                    }
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
                                    self.output.push_str(".as_str().unwrap_or(\"\")");
                                }
                            }
                        }
                        _ => {
                            // For other expressions, use dot notation
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
                        self.output.push_str(".as_str().unwrap_or(\"\")");
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
                // Generate constructor call with provided field values as arguments
                // Assume the fields correspond to constructor parameters in the same order
                write!(self.output, "{}::new(", type_name).unwrap();

                for (i, (_key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    // Add .to_string() for string literals
                    if let Expr::Literal(Literal::String(_)) = value {
                        self.generate_expr(value)?;
                        self.output.push_str(".to_string()");
                    } else {
                        self.generate_expr(value)?;
                    }
                }

                self.output.push(')');
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
                            // Literals always use Display
                            Expr::Literal(_) => {
                                self.output.push_str("{}");
                            }
                            // Simple identifiers: check if they're arrays
                            Expr::Identifier(name) => {
                                if self.array_vars.contains(name) {
                                    self.output.push_str("{:?}");
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
                            // Arrays and objects use Debug
                            Expr::ArrayLiteral(_) | Expr::ObjectLiteral(_) => {
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
                    self.output.push_str(&self.sanitize_name(&param.name));
                    if let Some(type_ref) = &param.type_ref {
                        self.output.push_str(": ");
                        self.output.push_str(&type_ref.to_rust_type());
                    }
                }

                self.output.push_str("| ");

                match &lambda.body {
                    LambdaBody::Expr(expr) => {
                        self.generate_expr(expr)?;
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
                    for arg in call.args.iter() {
                        match arg {
                            // Use {:?} for arrays, objects, and complex types
                            Expr::ArrayLiteral(_) | Expr::ObjectLiteral(_) => {
                                self.output.push_str("{:?}");
                            }
                            // MethodCall on arrays (map, filter, etc.) should use {:?}
                            Expr::MethodCall(method_call) => {
                                match method_call.method.as_str() {
                                    "map" | "filter" => {
                                        self.output.push_str("{:?}");
                                    }
                                    _ => {
                                        self.output.push_str("{:?}");
                                    }
                                }
                            }
                            // For now, use {:?} for everything to be safe
                            // TODO: Phase 2 - implement proper type inference to use {} vs {:?}
                            _ => {
                                self.output.push_str("{:?}");
                            }
                        }
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
                    } else {
                        self.generate_expr(arg)?;
                    }
                }
                self.output.push(')');
                return Ok(());
            }
        }

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
            // Error binding: let (value, err) = match task.await.unwrap() { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };
            write!(self.output, "let (").unwrap();
            for (i, binding_name) in task_info.binding_names.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                write!(self.output, "{}", binding_name).unwrap();
            }
            self.output.push_str(") = match ");
            write!(self.output, "{}.await.unwrap()", task_var_name).unwrap();
            self.output
                .push_str(" { Ok(v) => (v, None), Err(e) => (Default::default(), Some(e)) };\n");
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
        }
        
        // Check if this is a string method (no adapter means it's not an array method)
        // Special case: indexOf can be both string and array method
        // We detect string indexOf if the argument is a string literal
        let is_string_indexof = method_call.method == "indexOf" 
            && !method_call.args.is_empty()
            && matches!(&method_call.args[0], Expr::Literal(Literal::String(_)));
        
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
        
        // Generate the object
        self.generate_expr(&method_call.object)?;
        
        // Handle array methods with adapters
        match method_call.adapter {
            ArrayAdapter::Seq => {
                // Sequential: use .iter() and handle references in lambdas
                match method_call.method.as_str() {
                    "map" => {
                        // For map, use iter() and the lambda will work with &T
                        self.output.push_str(".iter()");
                    }
                    "filter" => {
                        // For filter, use iter() and work with references
                        // We'll add .copied() after filter
                        self.output.push_str(".iter()");
                    }
                    "reduce" => {
                        // reduce doesn't use .iter() - it operates directly on the vector
                        // We use .fold() which requires initial value and accumulator
                    }
                    "forEach" => {
                        self.output.push_str(".iter()");
                    }
                    "find" | "some" | "every" | "indexOf" | "includes" => {
                        self.output.push_str(".iter()");
                    }
                    _ => {
                        // For other methods, call directly
                    }
                }
            }
            ArrayAdapter::Par => {
                // Parallel: use rayon's .par_iter()
                self.output.push_str(".par_iter()");
                
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
                // Parallel + Vectorized
                // TODO: Implement combined parallel + SIMD
                self.output.push_str(".par_iter()");
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
            "forEach" => "for_each",
            "indexOf" => "position",
            "includes" => "any",
            "reduce" => "fold",  // Rust uses fold instead of reduce
            "some" => "any",      // Liva: some, Rust: any
            "every" => "all",     // Liva: every, Rust: all
            method_name => method_name,
        };
        
        self.output.push_str(rust_method);
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
            
            // For map/filter/reduce/forEach/find/some/every with .iter(), we need to dereference in the lambda
            // map: |&x| - filter: |&&x| - reduce: |acc, &x| - forEach: |&x| - find: |&&x| - some: |&&x| - every: |&&x|
            if (method_call.method == "map" || method_call.method == "filter" || method_call.method == "reduce" || method_call.method == "forEach" || method_call.method == "find" || method_call.method == "some" || method_call.method == "every") 
                && matches!(method_call.adapter, ArrayAdapter::Seq) {
                if let Expr::Lambda(lambda) = arg {
                    // Generate lambda with pattern |&x| or |&&x| or |acc, &x|
                    if lambda.is_move {
                        self.output.push_str("move ");
                    }
                    self.output.push('|');
                    for (idx, param) in lambda.params.iter().enumerate() {
                        if idx > 0 {
                            self.output.push_str(", ");
                        }
                        // reduce: first param (acc) no pattern, second param (&x) gets &
                        if method_call.method == "reduce" {
                            if idx == 0 {
                                // Accumulator: no dereferencing
                                self.output.push_str(&self.sanitize_name(&param.name));
                            } else {
                                // Element: dereference once
                                self.output.push('&');
                                self.output.push_str(&self.sanitize_name(&param.name));
                            }
                        } else {
                            // filter/find need && (closure takes &&T for filter/find)
                            // map/forEach/some/every need & (closure takes &T)
                            if method_call.method == "filter" || method_call.method == "find" {
                                self.output.push_str("&&");
                            } else {
                                self.output.push('&');
                            }
                            self.output.push_str(&self.sanitize_name(&param.name));
                        }
                    }
                    self.output.push_str("| ");
                    
                    match &lambda.body {
                        LambdaBody::Expr(expr) => {
                            self.generate_expr(expr)?;
                        }
                        LambdaBody::Block(block) => {
                            self.output.push('{');
                            self.indent();
                            self.output.push('\n');
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
            
            self.generate_expr(arg)?;
        }
        
        self.output.push(')');
        
        // Add transformations after the method call
        match (method_call.adapter, method_call.method.as_str()) {
            // Sequential map: just collect (lambda already returns owned values)
            (ArrayAdapter::Seq, "map") => {
                self.output.push_str(".collect::<Vec<_>>()");
            }
            // Sequential filter: copy values after filtering, then collect
            (ArrayAdapter::Seq, "filter") => {
                self.output.push_str(".copied().collect::<Vec<_>>()");
            }
            // Parallel map/filter with rayon
            (ArrayAdapter::Par, "map") | (ArrayAdapter::Par, "filter") => {
                self.output.push_str(".cloned().collect::<Vec<_>>()");
            }
            // Find returns Option<&T>, copy it
            (_, "find") => {
                self.output.push_str(".copied()");
            }
            // indexOf/position returns Option<usize>
            (_, "indexOf") => {
                self.output.push_str(".map(|i| i as i32).unwrap_or(-1)");
            }
            // some, every, includes return bool - no transformation needed
            (_, "some") | (_, "every") | (_, "includes") => {}
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
                    self.output.push_str("println!(\"");
                    for _ in method_call.args.iter() {
                        self.output.push_str("{:?}");
                    }
                    self.output.push_str("\", ");
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
                // console.error(...) -> eprintln!(...)
                if method_call.args.is_empty() {
                    self.output.push_str("eprintln!()");
                } else {
                    self.output.push_str("eprintln!(\"");
                    for _ in method_call.args.iter() {
                        self.output.push_str("{:?}");
                    }
                    self.output.push_str("\", ");
                    for (i, arg) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg)?;
                    }
                    self.output.push(')');
                }
            }
            "warn" => {
                // console.warn(...) -> eprintln!("Warning: ...", ...)
                if method_call.args.is_empty() {
                    self.output.push_str("eprintln!(\"Warning:\")");
                } else {
                    self.output.push_str("eprintln!(\"Warning: ");
                    for _ in method_call.args.iter() {
                        self.output.push_str("{:?}");
                    }
                    self.output.push_str("\", ");
                    for (i, arg) in method_call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg)?;
                    }
                    self.output.push(')');
                }
            }
            _ => {
                return Err(CompilerError::CodegenError(
                    SemanticErrorInfo::new(
                        "E3000",
                        &format!("Unknown console function: {}", method_call.method),
                        "Available console functions: log, error, warn"
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
        self.output.push_str(") })");
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
                if let Expr::Identifier(name) = call.callee.as_ref() {
                    name == "parseInt" || name == "parseFloat"
                } else {
                    false
                }
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
            _ => false,
        }
    }

    fn generate_literal(&mut self, lit: &Literal) -> Result<()> {
        match lit {
            Literal::Int(n) => write!(self.output, "{}", n).unwrap(),
            Literal::Float(f) => {
                // Always add _f64 suffix to avoid ambiguous numeric type errors
                write!(self.output, "{}_f64", f).unwrap();
            }
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
    #[allow(dead_code)]
    ctx: &'a DesugarContext,
    #[allow(dead_code)]
    scope_formats: Vec<HashMap<String, FormatKind>>,
    #[allow(dead_code)]
    in_method: bool,
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
        println!("DEBUG: IR generator including liva_rt module");
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
                self.generate_expr(object)?;

                if property == "length" {
                    self.output.push_str(".len()");
                    return Ok(());
                }

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
        ir::Expr::StructLiteral { fields, .. } => {
            fields.iter().any(|(_, value)| expr_has_unsupported(value))
        }
        ir::Expr::ArrayLiteral(elements) => elements.iter().any(expr_has_unsupported),
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
    println!("DEBUG: Checking if module has unsupported items...");
    if module_has_unsupported(module) {
        println!("DEBUG: Module has unsupported items, using AST generator");
        return generate_with_ast(program, ctx);
    }

    // For now, always use AST generator since it handles classes and inheritance correctly
    println!("DEBUG: Using AST generator for full compatibility");
    return generate_with_ast(program, ctx);
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

fn generate_cargo_toml(ctx: &DesugarContext) -> Result<String> {
    let mut cargo_toml = String::from(
        "[package]\n\
         name = \"liva_project\"\n\
         version = \"0.1.0\"\n\
         edition = \"2021\"\n\n\
         [dependencies]\n",
    );

    // Always add tokio since liva_rt uses it
    cargo_toml.push_str("tokio = { version = \"1\", features = [\"full\"] }\n");

    // Add serde_json for object literals
    cargo_toml.push_str("serde_json = \"1.0\"\n");

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
        });

        assert_eq!(gen.to_snake_case("CamelCase"), "camel_case");
        assert_eq!(gen.to_snake_case("myFunction"), "my_function");
        assert_eq!(gen.to_snake_case("snake_case"), "snake_case");
    }
}
