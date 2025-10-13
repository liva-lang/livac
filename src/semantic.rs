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
    awaitable_scopes: Vec<HashMap<String, AwaitableInfo>>,
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
    fn new() -> Self {
        Self {
            async_functions: HashSet::new(),
            types: HashMap::new(),
            functions: HashMap::new(),
            external_modules: HashSet::new(),
            current_scope: vec![HashMap::new()],
            awaitable_scopes: vec![HashMap::new()],
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
                self.exit_scope()?;
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
                self.exit_scope()?;
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

        self.exit_scope()?;
        Ok(())
    }

    fn validate_block(&mut self, block: &BlockStmt) -> Result<()> {
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
                if let Some(type_ref) = &var.type_ref {
                    self.validate_type_ref(type_ref, &empty)?;
                }
                let mut declared_type = var.type_ref.clone();
                if let Some(init) = &var.init {
                    self.validate_expr(init)?;
                    if declared_type.is_none() {
                        declared_type = self.infer_expr_type(init);
                    }
                }
                if self.declare_symbol(&var.name, declared_type.clone()) {
                    return Err(CompilerError::SemanticError(format!(
                        "Variable '{}' already defined in this scope",
                        var.name
                    )));
                }
                if let Some(init) = &var.init {
                    self.update_awaitable_from_expr(&var.name, init)?;
                } else {
                    self.clear_awaitable(&var.name);
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
                    return Err(CompilerError::SemanticError(format!(
                        "Constant '{}' already defined in this scope",
                        const_decl.name
                    )));
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
                self.validate_block(&while_stmt.body)?;
            }
            Stmt::For(for_stmt) => {
                self.validate_expr(&for_stmt.iterable)?;
                self.enter_scope();
                if self.declare_symbol(&for_stmt.var, None) {
                    self.exit_scope()?;
                    return Err(CompilerError::SemanticError(format!(
                        "Loop variable '{}' already defined",
                        for_stmt.var
                    )));
                }
                self.validate_block(&for_stmt.body)?;
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
                self.validate_block(&try_catch.try_block)?;
                self.enter_scope();
                self.declare_symbol(&try_catch.catch_var, None);
                self.validate_block(&try_catch.catch_block)?;
                self.exit_scope()?;
            }
            Stmt::Throw(throw_stmt) => {
                self.validate_expr(&throw_stmt.expr)?;
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.expr {
                    self.validate_expr(expr)?;
                    self.handle_return(expr);
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.validate_expr(&expr_stmt.expr)?;
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
            Expr::Lambda(lambda) => self.validate_lambda(lambda),
        }
    }

    fn validate_call_expr(&mut self, call: &CallExpr) -> Result<()> {
        if let Some((first, second)) = Self::extract_modifier_chain(&call.callee) {
            return Err(CompilerError::SemanticError(format!(
                "E0602: duplicate execution modifiers '{}' and '{}' on the same call",
                first, second
            )));
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
                            policy
                        )))
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

        self.exit_scope()?;
        result
    }

    fn validate_for_loop(&self, for_stmt: &ForStmt) -> Result<()> {
        self.validate_for_loop_options(for_stmt.policy.clone(), &for_stmt.options)?;

        if matches!(
            for_stmt.policy,
            DataParallelPolicy::Par | DataParallelPolicy::Boost
        ) && Self::block_contains_await(&for_stmt.body)
        {
            return Err(CompilerError::SemanticError(
                "E0605: `await` is not allowed inside `for par` or `for boost` loops.".into(),
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
            if !matches!(policy, DataParallelPolicy::Vec | DataParallelPolicy::Boost) {
                return Err(CompilerError::SemanticError(
                    "E0705: `simdWidth` option requires `for vec` or `for boost` policy.".into(),
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

    fn block_contains_await(block: &BlockStmt) -> bool {
        block
            .stmts
            .iter()
            .any(|stmt| Self::stmt_contains_await(stmt))
    }

    fn stmt_contains_await(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::VarDecl(var) => var
                .init
                .as_ref()
                .map_or(false, |expr| Self::expr_contains_await(expr)),
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
                    || Self::block_contains_await(&while_stmt.body)
            }
            Stmt::For(for_stmt) => {
                Self::expr_contains_await(&for_stmt.iterable)
                    || Self::block_contains_await(&for_stmt.body)
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
                Self::block_contains_await(&try_catch.try_block)
                    || Self::block_contains_await(&try_catch.catch_block)
            }
            Stmt::Throw(throw_stmt) => Self::expr_contains_await(&throw_stmt.expr),
            Stmt::Return(ret) => ret
                .expr
                .as_ref()
                .map_or(false, |expr| Self::expr_contains_await(expr)),
            Stmt::Expr(expr_stmt) => Self::expr_contains_await(&expr_stmt.expr),
            Stmt::Block(block) => Self::block_contains_await(block),
        }
    }

    fn expr_contains_await(expr: &Expr) -> bool {
        match expr {
            Expr::Unary { op: UnOp::Await, .. } => true,
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
            Expr::ArrayLiteral(elements) => {
                elements.iter().any(|value| Self::expr_contains_await(value))
            }
            Expr::Lambda(lambda) => match &lambda.body {
                LambdaBody::Expr(body) => Self::expr_contains_await(body),
                LambdaBody::Block(block) => Self::block_contains_await(block),
            },
            Expr::StringTemplate { parts } => parts.iter().any(|part| match part {
                StringTemplatePart::Expr(expr) => Self::expr_contains_await(expr),
                _ => false,
            }),
            Expr::Literal(_) | Expr::Identifier(_) => false,
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
                name
            )));
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
                    name
                )));
            }
            return Err(CompilerError::SemanticError(format!(
                "E0603: expression '{}' is not awaitable.",
                name
            )));
        }

        Err(CompilerError::SemanticError(format!(
            "E0603: expression '{}' is not awaitable.",
            name
        )))
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
