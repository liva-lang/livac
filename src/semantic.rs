use crate::ast::*;
use crate::error::{CompilerError, Result};
use std::collections::{HashMap, HashSet};

pub struct SemanticAnalyzer {
    // Track which functions are async
    async_functions: HashSet<String>,
    // Track defined types
    types: HashMap<String, TypeInfo>,
    // Current scope for variables
    current_scope: Vec<HashMap<String, TypeRef>>,
}

#[derive(Debug, Clone)]
struct TypeInfo {
    name: String,
    fields: HashMap<String, (Visibility, TypeRef)>,
    methods: HashMap<String, (Visibility, bool)>, // (visibility, is_async)
}

impl SemanticAnalyzer {
    fn new() -> Self {
        Self {
            async_functions: HashSet::new(),
            types: HashMap::new(),
            current_scope: vec![HashMap::new()],
        }
    }

    fn analyze_program(&mut self, mut program: Program) -> Result<Program> {
        // First pass: collect type definitions and function signatures
        self.collect_definitions(&program)?;

        // Second pass: infer async functions
        for item in &mut program.items {
            self.infer_async(item)?;
        }

        // Third pass: type checking and validation
        for item in &program.items {
            self.validate_item(item)?;
        }

        Ok(program)
    }

    fn collect_definitions(&mut self, program: &Program) -> Result<()> {
        for item in &program.items {
            match item {
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

    fn infer_async(&mut self, item: &mut TopLevel) -> Result<()> {
        match item {
            TopLevel::Function(func) => {
                if let Some(body) = &func.body {
                    if self.contains_async_call(body) {
                        func.is_async_inferred = true;
                        self.async_functions.insert(func.name.clone());
                    }
                } else if let Some(expr) = &func.expr_body {
                    if self.expr_contains_async(expr) {
                        func.is_async_inferred = true;
                        self.async_functions.insert(func.name.clone());
                    }
                }
            }
            TopLevel::Class(class) => {
                for member in &mut class.members {
                    if let Member::Method(method) = member {
                        let is_async = if let Some(body) = &method.body {
                            self.contains_async_call(body)
                        } else if let Some(expr) = &method.expr_body {
                            self.expr_contains_async(expr)
                        } else {
                            false
                        };

                        if is_async {
                            method.is_async_inferred = true;
                            // Update type info
                            if let Some(type_info) = self.types.get_mut(&class.name) {
                                if let Some(method_info) = type_info.methods.get_mut(&method.name) {
                                    method_info.1 = true;
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
                            self.contains_async_call(body)
                        } else if let Some(expr) = &method.expr_body {
                            self.expr_contains_async(expr)
                        } else {
                            false
                        };

                        if is_async {
                            method.is_async_inferred = true;
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn contains_async_call(&self, block: &BlockStmt) -> bool {
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
                if let Some(init) = &var.init {
                    self.expr_contains_async(init)
                } else {
                    false
                }
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
                    || self.contains_async_call(&while_stmt.body)
            }
            Stmt::For(for_stmt) => {
                self.expr_contains_async(&for_stmt.iterable)
                    || self.contains_async_call(&for_stmt.body)
            }
            Stmt::Switch(switch_stmt) => {
                self.expr_contains_async(&switch_stmt.discriminant)
                    || switch_stmt
                        .cases
                        .iter()
                        .any(|case| case.body.iter().any(|s| self.stmt_contains_async(s)))
            }
            Stmt::TryCatch(try_catch) => {
                self.contains_async_call(&try_catch.try_block)
                    || self.contains_async_call(&try_catch.catch_block)
            }
            Stmt::Throw(throw_stmt) => self.expr_contains_async(&throw_stmt.expr),
            Stmt::Return(ret) => ret
                .expr
                .as_ref()
                .map(|e| self.expr_contains_async(e))
                .unwrap_or(false),
            Stmt::Expr(expr_stmt) => self.expr_contains_async(&expr_stmt.expr),
            Stmt::Block(block) => self.contains_async_call(block),
        }
    }

    fn expr_contains_async(&self, expr: &Expr) -> bool {
        match expr {
            Expr::AsyncCall { .. } | Expr::TaskCall { .. } | Expr::FireCall { .. } => true,
            Expr::Call { callee, .. } => {
                // Check if calling a known async function
                if let Expr::Identifier(name) = &**callee {
                    self.async_functions.contains(name)
                } else {
                    false
                }
            }
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

    fn validate_item(&self, item: &TopLevel) -> Result<()> {
        match item {
            TopLevel::Function(func) => self.validate_function(func),
            TopLevel::Class(class) => self.validate_class(class),
            _ => Ok(()),
        }
    }

    fn validate_function(&self, func: &FunctionDecl) -> Result<()> {
        let type_params: std::collections::HashSet<String> = func.type_params.iter().cloned().collect();

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

        Ok(())
    }

    fn validate_class(&self, class: &ClassDecl) -> Result<()> {
        for member in &class.members {
            match member {
                Member::Field(field) => {
                    if let Some(type_ref) = &field.type_ref {
                        self.validate_type_ref(type_ref, &std::collections::HashSet::new())?;
                    }
                }
                Member::Method(method) => {
                    let type_params: std::collections::HashSet<String> = method.type_params.iter().cloned().collect();
                    for param in &method.params {
                        if let Some(type_ref) = &param.type_ref {
                            self.validate_type_ref(type_ref, &type_params)?;
                        }
                    }
                    if let Some(return_type) = &method.return_type {
                        self.validate_type_ref(return_type, &type_params)?;
                    }
                }
            }
        }

        // Check base class exists if specified
        if let Some(base) = &class.base {
            if !self.types.contains_key(base) {
                return Err(CompilerError::SemanticError(format!(
                    "Base class '{}' not found",
                    base
                )));
            }
        }

        Ok(())
    }

    fn validate_type_ref(&self, type_ref: &TypeRef, available_type_params: &std::collections::HashSet<String>) -> Result<()> {
        match type_ref {
            TypeRef::Simple(name) => {
                // Check if it's a built-in type, a defined type, or a type parameter
                if !is_builtin_type(name) && !self.types.contains_key(name) && !available_type_params.contains(name) {
                    return Err(CompilerError::TypeError(format!(
                        "Type '{}' not found",
                        name
                    )));
                }
            }
            TypeRef::Generic { base, args } => {
                self.validate_type_ref(&TypeRef::Simple(base.clone()), available_type_params)?;
                for arg in args {
                    self.validate_type_ref(arg, available_type_params)?;
                }
            }
            TypeRef::Array(inner) => self.validate_type_ref(inner, available_type_params)?,
            TypeRef::Optional(inner) => self.validate_type_ref(inner, available_type_params)?,
        }
        Ok(())
    }
}

fn is_builtin_type(name: &str) -> bool {
    matches!(
        name,
        "number"
            | "float"
            | "bool"
            | "char"
            | "string"
            | "bytes"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "Option"
            | "Result"
            | "Vec"
    )
}

pub fn analyze(program: Program) -> Result<Program> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    #[test]
    fn test_async_inference() {
        let source = r#"
            fetchUser() {
                let res = async fetchData("url")
                return res
            }
        "#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens, source).unwrap();
        let analyzed = analyze(program).unwrap();

        match &analyzed.items[0] {
            TopLevel::Function(f) => {
                assert!(f.is_async_inferred);
            }
            _ => panic!("Expected function"),
        }
    }
}
