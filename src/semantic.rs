use crate::ast::*;
use crate::error::{CompilerError, Result};
use std::collections::{HashMap, HashSet};

pub struct SemanticAnalyzer {
    // Track which functions are async
    async_functions: HashSet<String>,
    // Track defined types
    types: HashMap<String, TypeInfo>,
    // Track function signatures (arity, optional type information)
    functions: HashMap<String, FunctionSignature>,
    // Track external modules brought via `use rust`
    external_modules: HashSet<String>,
    // Current scope for variables
    current_scope: Vec<HashMap<String, Option<TypeRef>>>,
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

impl SemanticAnalyzer {
    fn new() -> Self {
        Self {
            async_functions: HashSet::new(),
            types: HashMap::new(),
            functions: HashMap::new(),
            external_modules: HashSet::new(),
            current_scope: vec![HashMap::new()],
        }
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
                    self.contains_async_call(body)
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
                            self.contains_async_call(body)
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
                            self.contains_async_call(body)
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
            Expr::Call(call) => {
                match call.exec_policy {
                    ExecPolicy::Async
                    | ExecPolicy::TaskAsync
                    | ExecPolicy::TaskPar
                    | ExecPolicy::FireAsync
                    | ExecPolicy::FirePar => return true,
                    ExecPolicy::Par | ExecPolicy::Normal => {}
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
                LambdaBody::Block(block) => self.contains_async_call(block),
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

        self.enter_scope();

        for param in &func.params {
            if self.declare_symbol(&param.name, param.type_ref.clone()) {
                self.exit_scope();
                return Err(CompilerError::SemanticError(format!(
                    "Parameter '{}' defined multiple times",
                    param.name
                )));
            }
        }

        if let Some(body) = &func.body {
            self.validate_block(body)?;
        }

        if let Some(expr) = &func.expr_body {
            self.validate_expr(expr)?;
        }

        self.exit_scope();
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
                    base
                )));
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
                self.exit_scope();
                return Err(CompilerError::SemanticError(format!(
                    "Parameter '{}' defined multiple times",
                    param.name
                )));
            }
        }

        if let Some(body) = &method.body {
            self.validate_block(body)?;
        }

        if let Some(expr) = &method.expr_body {
            self.validate_expr(expr)?;
        }

        self.exit_scope();
        Ok(())
    }

    fn validate_block(&mut self, block: &BlockStmt) -> Result<()> {
        self.enter_scope();
        for stmt in &block.stmts {
            self.validate_stmt(stmt)?;
        }
        self.exit_scope();
        Ok(())
    }

    fn validate_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        let empty: HashSet<String> = HashSet::new();

        match stmt {
            Stmt::VarDecl(var) => {
                if let Some(type_ref) = &var.type_ref {
                    self.validate_type_ref(type_ref, &empty)?;
                }
                if let Some(init) = &var.init {
                    self.validate_expr(init)?;
                }
                if self.declare_symbol(&var.name, var.type_ref.clone()) {
                    return Err(CompilerError::SemanticError(format!(
                        "Variable '{}' already defined in this scope",
                        var.name
                    )));
                }
            }
            Stmt::ConstDecl(const_decl) => {
                if let Some(type_ref) = &const_decl.type_ref {
                    self.validate_type_ref(type_ref, &empty)?;
                }
                self.validate_expr(&const_decl.init)?;
                if self.declare_symbol(&const_decl.name, const_decl.type_ref.clone()) {
                    return Err(CompilerError::SemanticError(format!(
                        "Constant '{}' already defined in this scope",
                        const_decl.name
                    )));
                }
            }
            Stmt::Assign(assign) => {
                self.validate_assignment_target(&assign.target)?;
                self.validate_expr(&assign.value)?;
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
                self.validate_block(&while_stmt.body)?;
            }
            Stmt::For(for_stmt) => {
                self.validate_expr(&for_stmt.iterable)?;
                self.enter_scope();
                if self.declare_symbol(&for_stmt.var, None) {
                    self.exit_scope();
                    return Err(CompilerError::SemanticError(format!(
                        "Loop variable '{}' already defined",
                        for_stmt.var
                    )));
                }
                self.validate_block(&for_stmt.body)?;
                self.exit_scope();
            }
            Stmt::Switch(switch_stmt) => {
                self.validate_expr(&switch_stmt.discriminant)?;
                for case in &switch_stmt.cases {
                    self.enter_scope();
                    for stmt in &case.body {
                        self.validate_stmt(stmt)?;
                    }
                    self.exit_scope();
                }
                if let Some(default) = &switch_stmt.default {
                    self.enter_scope();
                    for stmt in default {
                        self.validate_stmt(stmt)?;
                    }
                    self.exit_scope();
                }
            }
            Stmt::TryCatch(try_catch) => {
                self.validate_block(&try_catch.try_block)?;
                self.enter_scope();
                self.declare_symbol(&try_catch.catch_var, None);
                self.validate_block(&try_catch.catch_block)?;
                self.exit_scope();
            }
            Stmt::Throw(throw_stmt) => {
                self.validate_expr(&throw_stmt.expr)?;
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.validate_expr(expr)?;
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.validate_expr(&expr_stmt.expr)?;
            }
            Stmt::Block(block) => {
                self.validate_block(block)?;
            }
        }

        Ok(())
    }

    fn validate_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(_) => Ok(()),
            Expr::Identifier(_name) => Ok(()),
            Expr::Binary { left, right, .. } => {
                self.validate_expr(left)?;
                self.validate_expr(right)
            }
            Expr::Unary { operand, .. } => self.validate_expr(operand),
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
            Expr::Member { object, .. } => self.validate_expr(object),
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
            Expr::Lambda(lambda) => self.validate_lambda(lambda),
        }
    }

    fn validate_call_expr(&mut self, call: &CallExpr) -> Result<()> {
        self.validate_call(&call.callee, &call.args)
    }

    fn validate_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<()> {
        match callee {
            Expr::Identifier(name) => {
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

        self.enter_scope();

        for param in &lambda.params {
            if let Some(type_ref) = &param.type_ref {
                let empty: HashSet<String> = HashSet::new();
                self.validate_type_ref(type_ref, &empty)?;
            }

            if self.declare_symbol(&param.name, param.type_ref.clone()) {
                self.exit_scope();
                return Err(CompilerError::SemanticError(format!(
                    "Parameter '{}' defined multiple times",
                    param.name
                )));
            }
        }

        let result = match &lambda.body {
            LambdaBody::Expr(expr) => self.validate_expr(expr),
            LambdaBody::Block(block) => {
                self.validate_block(block)?;
                Ok(())
            }
        };

        self.exit_scope();
        result
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
                    name, required, total, arity
                )));
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
                        name
                    )));
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
    }

    fn exit_scope(&mut self) {
        self.current_scope.pop();
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
        }
    }
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
    use std::collections::{HashMap, HashSet};

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

    fn async_expr() -> Expr {
        let mut call = CallExpr::new(Expr::Identifier("fetch".into()), vec![]);
        call.exec_policy = ExecPolicy::Async;
        Expr::Call(call)
    }

    #[test]
    fn test_expr_contains_async_variants() {
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.async_functions.insert("do_async".into());

        let mut task_async = CallExpr::new(Expr::Identifier("worker".into()), vec![]);
        task_async.exec_policy = ExecPolicy::TaskAsync;
        assert!(analyzer.expr_contains_async(&Expr::Call(task_async)));

        let mut fire_par = CallExpr::new(Expr::Identifier("fire".into()), vec![]);
        fire_par.exec_policy = ExecPolicy::FirePar;
        assert!(analyzer.expr_contains_async(&Expr::Call(fire_par)));

        assert!(analyzer.expr_contains_async(&Expr::Binary {
            op: BinOp::Add,
            left: Box::new(async_expr()),
            right: Box::new(Expr::Identifier("x".into())),
        }));
        assert!(analyzer.expr_contains_async(&Expr::Unary {
            op: UnOp::Await,
            operand: Box::new(async_expr()),
        }));
        assert!(analyzer.expr_contains_async(&Expr::Ternary {
            condition: Box::new(async_expr()),
            then_expr: Box::new(Expr::Identifier("a".into())),
            else_expr: Box::new(Expr::Identifier("b".into())),
        }));
        assert!(analyzer.expr_contains_async(&Expr::Member {
            object: Box::new(async_expr()),
            property: "field".into(),
        }));
        assert!(analyzer.expr_contains_async(&Expr::Index {
            object: Box::new(async_expr()),
            index: Box::new(Expr::Literal(Literal::Int(0))),
        }));
        assert!(analyzer
            .expr_contains_async(&Expr::ObjectLiteral(vec![("value".into(), async_expr()),])));
        assert!(analyzer.expr_contains_async(&Expr::ArrayLiteral(vec![async_expr()])));
        assert!(analyzer.expr_contains_async(&Expr::StringTemplate {
            parts: vec![StringTemplatePart::Expr(Box::new(async_expr()))],
        }));

        let call_async = CallExpr::new(Expr::Identifier("do_async".into()), vec![]);
        assert!(analyzer.expr_contains_async(&Expr::Call(call_async)));

        let call_sync = CallExpr::new(Expr::Identifier("sync".into()), vec![]);
        assert!(!analyzer.expr_contains_async(&Expr::Call(call_sync)));
    }

    #[test]
    fn test_contains_async_across_statements() {
        let analyzer = SemanticAnalyzer::new();
        let block = BlockStmt {
            stmts: vec![
                Stmt::VarDecl(VarDecl {
                    name: "v".into(),
                    type_ref: None,
                    init: Some(async_expr()),
                }),
                Stmt::ConstDecl(ConstDecl {
                    name: "c".into(),
                    type_ref: None,
                    init: async_expr(),
                }),
                Stmt::Assign(AssignStmt {
                    target: async_expr(),
                    value: async_expr(),
                }),
                Stmt::If(IfStmt {
                    condition: async_expr(),
                    then_branch: BlockStmt {
                        stmts: vec![Stmt::Expr(ExprStmt { expr: async_expr() })],
                    },
                    else_branch: Some(BlockStmt {
                        stmts: vec![Stmt::Expr(ExprStmt { expr: async_expr() })],
                    }),
                }),
                Stmt::While(WhileStmt {
                    condition: async_expr(),
                    body: BlockStmt {
                        stmts: vec![Stmt::Expr(ExprStmt { expr: async_expr() })],
                    },
                }),
                Stmt::For(ForStmt::new(
                    "item".into(),
                    async_expr(),
                    BlockStmt {
                        stmts: vec![Stmt::Expr(ExprStmt { expr: async_expr() })],
                    },
                )),
                Stmt::Switch(SwitchStmt {
                    discriminant: async_expr(),
                    cases: vec![CaseClause {
                        value: Expr::Literal(Literal::Int(1)),
                        body: vec![Stmt::Expr(ExprStmt { expr: async_expr() })],
                    }],
                    default: Some(vec![Stmt::Expr(ExprStmt { expr: async_expr() })]),
                }),
                Stmt::TryCatch(TryCatchStmt {
                    try_block: BlockStmt {
                        stmts: vec![Stmt::Expr(ExprStmt { expr: async_expr() })],
                    },
                    catch_var: "err".into(),
                    catch_block: BlockStmt {
                        stmts: vec![Stmt::Expr(ExprStmt { expr: async_expr() })],
                    },
                }),
                Stmt::Throw(ThrowStmt { expr: async_expr() }),
                Stmt::Return(ReturnStmt {
                    expr: Some(async_expr()),
                }),
                Stmt::Expr(ExprStmt { expr: async_expr() }),
                Stmt::Block(BlockStmt {
                    stmts: vec![Stmt::Expr(ExprStmt { expr: async_expr() })],
                }),
            ],
        };

        assert!(analyzer.contains_async_call(&block));
    }

    #[test]
    fn test_validate_type_refs_and_class_bases() {
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.types.insert(
            "Base".into(),
            TypeInfo {
                name: "Base".into(),
                fields: HashMap::new(),
                methods: HashMap::new(),
            },
        );

        let class = ClassDecl {
            name: "Derived".into(),
            base: Some("Base".into()),
            members: vec![Member::Field(FieldDecl {
                name: "values".into(),
                visibility: Visibility::Public,
                type_ref: Some(TypeRef::Array(Box::new(TypeRef::Simple("number".into())))),
                init: None,
            })],
        };
        analyzer
            .validate_class(&class)
            .expect("class should be valid");

        let mut type_params = HashSet::new();
        type_params.insert("T".into());
        analyzer
            .validate_type_ref(
                &TypeRef::Generic {
                    base: "Option".into(),
                    args: vec![TypeRef::Simple("T".into())],
                },
                &type_params,
            )
            .expect("generic type should be valid");

        assert!(analyzer
            .validate_type_ref(&TypeRef::Simple("Unknown".into()), &HashSet::new())
            .is_ok());

        let err = analyzer
            .validate_class(&ClassDecl {
                name: "Broken".into(),
                base: Some("Missing".into()),
                members: vec![],
            })
            .expect_err("missing base class should error");
        matches!(err, CompilerError::SemanticError(_));
    }

    #[test]
    fn test_is_builtin_type_matches() {
        let analyzer = SemanticAnalyzer::new();
        let empty = HashSet::new();
        assert!(analyzer
            .validate_type_ref(&TypeRef::Simple("number".into()), &empty)
            .is_ok());
        assert!(analyzer
            .validate_type_ref(&TypeRef::Simple("Custom".into()), &empty)
            .is_ok());
    }
}
