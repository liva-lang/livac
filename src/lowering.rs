//! Lowering utilities: AST → IR.

use crate::ast;
use crate::ir;
use std::collections::HashMap;

pub fn lower_program(program: &ast::Program) -> ir::Module {
    let mut module = ir::Module::new();

    for item in &program.items {
        match item {
            ast::TopLevel::UseRust(use_rust) => {
                module.extern_crates.push(ir::ExternCrate {
                    crate_name: use_rust.crate_name.clone(),
                    alias: use_rust.alias.clone(),
                });
            }
            ast::TopLevel::Function(func) => {
                module.items.push(ir::Item::Function(lower_function(func)));
            }
            ast::TopLevel::Test(test) => {
                module.items.push(ir::Item::Test(lower_test(test)));
            }
            _ => module.items.push(ir::Item::Unsupported(item.clone())),
        }
    }

    module
}

fn lower_function(func: &ast::FunctionDecl) -> ir::Function {
    let params = func
        .params
        .iter()
        .map(|param| {
            let ty = if let Some(explicit) = &param.type_ref {
                ir::Type::from_ast(&Some(explicit.clone()))
            } else if let Some(inferred) = infer_param_type(func, &param.name) {
                inferred
            } else {
                ir::Type::Inferred
            };
            ir::Param {
                name: param.name.clone(),
                ty,
                default: param.default.as_ref().map(lower_expr),
            }
        })
        .collect();

    let body = if let Some(block) = &func.body {
        lower_block(block)
    } else if let Some(expr) = &func.expr_body {
        ir::Block {
            statements: vec![ir::Stmt::Return(Some(lower_expr(expr)))],
        }
    } else {
        ir::Block { statements: vec![] }
    };

    let ret_type = if let Some(ret) = &func.return_type {
        ir::Type::from_ast(&Some(ret.clone()))
    } else if let Some(expr) = &func.expr_body {
        infer_expr_return_type(expr)
    } else if let Some(body) = &func.body {
        infer_block_return_type(body)
    } else {
        ir::Type::Inferred
    };

    ir::Function {
        name: func.name.clone(),
        params,
        ret_type,
        body,
        async_kind: if func.is_async_inferred {
            ir::AsyncKind::Async
        } else {
            ir::AsyncKind::NotAsync
        },
        visibility: ir::Visibility::Public,
        source: func.clone(),
    }
}

fn lower_test(test: &ast::TestDecl) -> ir::Test {
    ir::Test {
        name: test.name.clone(),
        body: lower_block(&test.body),
        source: test.clone(),
    }
}

fn lower_block(block: &ast::BlockStmt) -> ir::Block {
    ir::Block {
        statements: block.stmts.iter().map(lower_stmt).collect(),
    }
}

fn lower_stmt(stmt: &ast::Stmt) -> ir::Stmt {
    match stmt {
        ast::Stmt::VarDecl(var) => {
            let value = var
                .init
                .as_ref()
                .map(lower_expr)
                .unwrap_or(ir::Expr::Literal(ir::Literal::Null));
            ir::Stmt::Let {
                name: var.name.clone(),
                ty: var
                    .type_ref
                    .as_ref()
                    .map(|ty| ir::Type::from_ast(&Some(ty.clone()))),
                value,
            }
        }
        ast::Stmt::ConstDecl(const_decl) => ir::Stmt::Const {
            name: const_decl.name.clone(),
            ty: const_decl
                .type_ref
                .as_ref()
                .map(|ty| ir::Type::from_ast(&Some(ty.clone()))),
            value: lower_expr(&const_decl.init),
        },
        ast::Stmt::Assign(assign) => ir::Stmt::Assign {
            target: lower_expr(&assign.target),
            value: lower_expr(&assign.value),
        },
        ast::Stmt::Return(ret) => ir::Stmt::Return(ret.expr.as_ref().map(lower_expr)),
        ast::Stmt::Throw(throw_stmt) => ir::Stmt::Throw(lower_expr(&throw_stmt.expr)),
        ast::Stmt::Expr(expr_stmt) => ir::Stmt::Expr(lower_expr(&expr_stmt.expr)),
        ast::Stmt::If(if_stmt) => ir::Stmt::If {
            condition: lower_expr(&if_stmt.condition),
            then_block: lower_block(&if_stmt.then_branch),
            else_block: if_stmt.else_branch.as_ref().map(|block| lower_block(block)),
        },
        ast::Stmt::While(while_stmt) => ir::Stmt::While {
            condition: lower_expr(&while_stmt.condition),
            body: lower_block(&while_stmt.body),
        },
        ast::Stmt::For(for_stmt) => ir::Stmt::For {
            var: for_stmt.var.clone(),
            iterable: lower_expr(&for_stmt.iterable),
            body: lower_block(&for_stmt.body),
        },
        ast::Stmt::Block(block) => ir::Stmt::Block(lower_block(block)),
        ast::Stmt::TryCatch(try_catch) => ir::Stmt::TryCatch {
            try_block: lower_block(&try_catch.try_block),
            error_var: try_catch.catch_var.clone(),
            catch_block: lower_block(&try_catch.catch_block),
        },
        ast::Stmt::Switch(switch_stmt) => ir::Stmt::Switch {
            discriminant: lower_expr(&switch_stmt.discriminant),
            cases: switch_stmt
                .cases
                .iter()
                .map(|case| ir::SwitchCase {
                    value: lower_expr(&case.value),
                    body: case.body.iter().map(lower_stmt).collect(),
                })
                .collect(),
            default: switch_stmt
                .default
                .as_ref()
                .map(|body| body.iter().map(lower_stmt).collect()),
        },
    }
}

fn lower_expr(expr: &ast::Expr) -> ir::Expr {
    match expr {
        ast::Expr::Literal(lit) => ir::Expr::Literal(lower_literal(lit)),
        ast::Expr::Identifier(name) => ir::Expr::Identifier(name.clone()),
        ast::Expr::Call { callee, args } => ir::Expr::Call {
            callee: Box::new(lower_expr(callee)),
            args: args.iter().map(lower_expr).collect(),
        },
        ast::Expr::AsyncCall { callee, args } => ir::Expr::AsyncCall {
            callee: Box::new(lower_expr(callee)),
            args: args.iter().map(lower_expr).collect(),
        },
        ast::Expr::ParallelCall { callee, args } => ir::Expr::ParallelCall {
            callee: Box::new(lower_expr(callee)),
            args: args.iter().map(lower_expr).collect(),
        },
        ast::Expr::TaskCall { mode, callee, args } => ir::Expr::TaskCall {
            mode: match mode {
                ast::ConcurrencyMode::Async => ir::ConcurrencyMode::Async,
                ast::ConcurrencyMode::Parallel => ir::ConcurrencyMode::Parallel,
            },
            callee: callee.clone(),
            args: args.iter().map(lower_expr).collect(),
        },
        ast::Expr::FireCall { mode, callee, args } => ir::Expr::FireCall {
            mode: match mode {
                ast::ConcurrencyMode::Async => ir::ConcurrencyMode::Async,
                ast::ConcurrencyMode::Parallel => ir::ConcurrencyMode::Parallel,
            },
            callee: callee.clone(),
            args: args.iter().map(lower_expr).collect(),
        },
        ast::Expr::Unary { op, operand } => match op {
            ast::UnOp::Await => ir::Expr::Await(Box::new(lower_expr(operand))),
            ast::UnOp::Neg => ir::Expr::Unary {
                op: ir::UnaryOp::Neg,
                operand: Box::new(lower_expr(operand)),
            },
            ast::UnOp::Not => ir::Expr::Unary {
                op: ir::UnaryOp::Not,
                operand: Box::new(lower_expr(operand)),
            },
        },
        ast::Expr::Binary { op, left, right } => {
            let op = match op {
                ast::BinOp::Add => ir::BinaryOp::Add,
                ast::BinOp::Sub => ir::BinaryOp::Sub,
                ast::BinOp::Mul => ir::BinaryOp::Mul,
                ast::BinOp::Div => ir::BinaryOp::Div,
                ast::BinOp::Mod => ir::BinaryOp::Mod,
                ast::BinOp::Eq => ir::BinaryOp::Eq,
                ast::BinOp::Ne => ir::BinaryOp::Ne,
                ast::BinOp::Lt => ir::BinaryOp::Lt,
                ast::BinOp::Le => ir::BinaryOp::Le,
                ast::BinOp::Gt => ir::BinaryOp::Gt,
                ast::BinOp::Ge => ir::BinaryOp::Ge,
                ast::BinOp::And => ir::BinaryOp::And,
                ast::BinOp::Or => ir::BinaryOp::Or,
                ast::BinOp::Range => {
                    return ir::Expr::Range {
                        start: Box::new(lower_expr(left)),
                        end: Box::new(lower_expr(right)),
                    }
                }
            };
            ir::Expr::Binary {
                op,
                left: Box::new(lower_expr(left)),
                right: Box::new(lower_expr(right)),
            }
        }
        ast::Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => ir::Expr::Ternary {
            condition: Box::new(lower_expr(condition)),
            then_expr: Box::new(lower_expr(then_expr)),
            else_expr: Box::new(lower_expr(else_expr)),
        },
        ast::Expr::StringTemplate { parts } => ir::Expr::StringTemplate(
            parts
                .iter()
                .map(|part| match part {
                    ast::StringTemplatePart::Text(text) => ir::TemplatePart::Text(text.clone()),
                    ast::StringTemplatePart::Expr(expr) => ir::TemplatePart::Expr(lower_expr(expr)),
                })
                .collect(),
        ),
        ast::Expr::Member { object, property } => ir::Expr::Member {
            object: Box::new(lower_expr(object)),
            property: property.clone(),
        },
        ast::Expr::Index { object, index } => ir::Expr::Index {
            object: Box::new(lower_expr(object)),
            index: Box::new(lower_expr(index)),
        },
        ast::Expr::ArrayLiteral(items) => {
            ir::Expr::ArrayLiteral(items.iter().map(lower_expr).collect())
        }
        ast::Expr::ObjectLiteral(fields) => ir::Expr::ObjectLiteral(
            fields
                .iter()
                .map(|(name, value)| (name.clone(), lower_expr(value)))
                .collect(),
        ),
    }
}

fn infer_param_type(func: &ast::FunctionDecl, name: &str) -> Option<ir::Type> {
    if let Some(body) = &func.body {
        if block_uses_param_as_array(body, name) {
            return Some(ir::Type::Array(Box::new(ir::Type::Custom(
                "serde_json::Value".into(),
            ))));
        }
    }
    None
}

fn block_uses_param_as_array(block: &ast::BlockStmt, name: &str) -> bool {
    block
        .stmts
        .iter()
        .any(|stmt| stmt_uses_param_as_array(stmt, name))
}

fn stmt_uses_param_as_array(stmt: &ast::Stmt, name: &str) -> bool {
    match stmt {
        ast::Stmt::For(for_stmt) => {
            expr_references_identifier(&for_stmt.iterable, name)
                || block_uses_param_as_array(&for_stmt.body, name)
        }
        ast::Stmt::While(while_stmt) => {
            expr_uses_param_as_array(&while_stmt.condition, name)
                || block_uses_param_as_array(&while_stmt.body, name)
        }
        ast::Stmt::Expr(expr_stmt) => expr_uses_param_as_array(&expr_stmt.expr, name),
        ast::Stmt::Assign(assign) => {
            expr_uses_param_as_array(&assign.target, name)
                || expr_uses_param_as_array(&assign.value, name)
        }
        ast::Stmt::Return(ret) => ret
            .expr
            .as_ref()
            .map(|expr| expr_uses_param_as_array(expr, name))
            .unwrap_or(false),
        ast::Stmt::If(if_stmt) => {
            expr_uses_param_as_array(&if_stmt.condition, name)
                || block_uses_param_as_array(&if_stmt.then_branch, name)
                || if_stmt
                    .else_branch
                    .as_ref()
                    .map(|block| block_uses_param_as_array(block, name))
                    .unwrap_or(false)
        }
        ast::Stmt::Block(block) => block_uses_param_as_array(block, name),
        ast::Stmt::TryCatch(try_catch) => {
            block_uses_param_as_array(&try_catch.try_block, name)
                || block_uses_param_as_array(&try_catch.catch_block, name)
        }
        ast::Stmt::Switch(switch_stmt) => {
            expr_uses_param_as_array(&switch_stmt.discriminant, name)
                || switch_stmt
                    .cases
                    .iter()
                    .any(|case| case.body.iter().any(|stmt| stmt_uses_param_as_array(stmt, name)))
                || switch_stmt
                    .default
                    .as_ref()
                    .map(|body| body.iter().any(|stmt| stmt_uses_param_as_array(stmt, name)))
                    .unwrap_or(false)
        }
        _ => false,
    }
}

fn expr_uses_param_as_array(expr: &ast::Expr, name: &str) -> bool {
    match expr {
        ast::Expr::Identifier(_) => false,
        ast::Expr::Index { object, .. } => expr_references_identifier(object, name),
        ast::Expr::Member { object, .. } => expr_uses_param_as_array(object, name),
        ast::Expr::Call { callee, args } => {
            if matches!(callee.as_ref(), ast::Expr::Identifier(id) if id == "len")
                && args.iter().any(|arg| expr_references_identifier(arg, name))
            {
                return true;
            }
            args.iter()
                .any(|arg| expr_uses_param_as_array(arg, name))
        }
        ast::Expr::Binary { left, right, .. } => {
            expr_uses_param_as_array(left, name) || expr_uses_param_as_array(right, name)
        }
        ast::Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_uses_param_as_array(condition, name)
                || expr_uses_param_as_array(then_expr, name)
                || expr_uses_param_as_array(else_expr, name)
        }
        ast::Expr::StringTemplate { parts } => parts.iter().any(|part| match part {
            ast::StringTemplatePart::Expr(expr) => expr_uses_param_as_array(expr, name),
            _ => false,
        }),
        ast::Expr::ArrayLiteral(items) => {
            items.iter().any(|item| expr_uses_param_as_array(item, name))
        }
        ast::Expr::ObjectLiteral(fields) => fields
            .iter()
            .any(|(_, value)| expr_uses_param_as_array(value, name)),
        ast::Expr::Unary { operand, .. } => expr_uses_param_as_array(operand, name),
        ast::Expr::AsyncCall { callee, args } | ast::Expr::ParallelCall { callee, args } => {
            expr_uses_param_as_array(callee, name)
                || args.iter().any(|arg| expr_uses_param_as_array(arg, name))
        }
        ast::Expr::TaskCall { args, .. } | ast::Expr::FireCall { args, .. } => args
            .iter()
            .any(|arg| expr_uses_param_as_array(arg, name)),
        _ => false,
    }
}

fn expr_references_identifier(expr: &ast::Expr, name: &str) -> bool {
    match expr {
        ast::Expr::Identifier(ident) => ident == name,
        ast::Expr::Binary { left, right, .. } => {
            expr_references_identifier(left, name) || expr_references_identifier(right, name)
        }
        ast::Expr::Unary { operand, .. } => expr_references_identifier(operand, name),
        ast::Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_references_identifier(condition, name)
                || expr_references_identifier(then_expr, name)
                || expr_references_identifier(else_expr, name)
        }
        ast::Expr::Call { callee, args } => {
            expr_references_identifier(callee, name)
                || args
                    .iter()
                    .any(|arg| expr_references_identifier(arg, name))
        }
        ast::Expr::Member { object, .. } => expr_references_identifier(object, name),
        ast::Expr::Index { object, index } => {
            expr_references_identifier(object, name)
                || expr_references_identifier(index, name)
        }
        ast::Expr::ArrayLiteral(items) => {
            items.iter().any(|item| expr_references_identifier(item, name))
        }
        ast::Expr::ObjectLiteral(fields) => fields
            .iter()
            .any(|(_, value)| expr_references_identifier(value, name)),
        ast::Expr::StringTemplate { parts } => parts.iter().any(|part| match part {
            ast::StringTemplatePart::Expr(expr) => expr_references_identifier(expr, name),
            _ => false,
        }),
        ast::Expr::AsyncCall { callee, args } | ast::Expr::ParallelCall { callee, args } => {
            expr_references_identifier(callee, name)
                || args
                    .iter()
                    .any(|arg| expr_references_identifier(arg, name))
        }
        ast::Expr::TaskCall { args, .. } | ast::Expr::FireCall { args, .. } => args
            .iter()
            .any(|arg| expr_references_identifier(arg, name)),
        _ => false,
    }
}

fn infer_expr_return_type(expr: &ast::Expr) -> ir::Type {
    let vars = HashMap::new();
    infer_expr_return_type_with_env(expr, &vars)
}

fn infer_expr_return_type_with_env(
    expr: &ast::Expr,
    vars: &HashMap<String, ir::Type>,
) -> ir::Type {
    match expr {
        ast::Expr::Literal(lit) => match lit {
            ast::Literal::Int(_) => ir::Type::Number,
            ast::Literal::Float(_) => ir::Type::Float,
            ast::Literal::Bool(_) => ir::Type::Bool,
            ast::Literal::String(_) => ir::Type::String,
            ast::Literal::Char(_) => ir::Type::Char,
        },
        ast::Expr::Identifier(name) => vars.get(name).cloned().unwrap_or(ir::Type::Inferred),
        ast::Expr::StringTemplate { .. } => ir::Type::String,
        ast::Expr::Binary { op, left, right } => match op {
            ast::BinOp::Eq
            | ast::BinOp::Ne
            | ast::BinOp::Lt
            | ast::BinOp::Le
            | ast::BinOp::Gt
            | ast::BinOp::Ge
            | ast::BinOp::And
            | ast::BinOp::Or => ir::Type::Bool,
            ast::BinOp::Add => {
                let left_ty = infer_expr_return_type_with_env(left, vars);
                let right_ty = infer_expr_return_type_with_env(right, vars);
                if left_ty == ir::Type::String || right_ty == ir::Type::String {
                    ir::Type::String
                } else {
                    infer_numeric_result_type_with_env(left, right, vars)
                }
            }
            ast::BinOp::Sub | ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Mod => {
                infer_numeric_result_type_with_env(left, right, vars)
            }
            ast::BinOp::Range => ir::Type::Array(Box::new(ir::Type::Number)),
        },
        ast::Expr::Ternary {
            then_expr, else_expr, ..
        } => {
            let then_ty = infer_expr_return_type_with_env(then_expr, vars);
            let else_ty = infer_expr_return_type_with_env(else_expr, vars);
            if then_ty == else_ty {
                then_ty
            } else {
                ir::Type::Inferred
            }
        }
        _ => ir::Type::Inferred,
    }
}

fn infer_numeric_result_type_with_env(
    left: &ast::Expr,
    right: &ast::Expr,
    vars: &HashMap<String, ir::Type>,
) -> ir::Type {
    let left_ty = infer_expr_return_type_with_env(left, vars);
    let right_ty = infer_expr_return_type_with_env(right, vars);
    if left_ty == ir::Type::Float || right_ty == ir::Type::Float {
        ir::Type::Float
    } else {
        ir::Type::Number
    }
}

fn infer_block_return_type(block: &ast::BlockStmt) -> ir::Type {
    let mut vars = HashMap::new();
    infer_block_return_type_with_env(block, &mut vars)
}

fn infer_block_return_type_with_env(
    block: &ast::BlockStmt,
    vars: &mut HashMap<String, ir::Type>,
) -> ir::Type {
    for stmt in &block.stmts {
        if let Some(ty) = infer_stmt_return_type_with_env(stmt, vars) {
            return ty;
        }
    }
    ir::Type::Inferred
}

fn infer_stmt_return_type_with_env(
    stmt: &ast::Stmt,
    vars: &mut HashMap<String, ir::Type>,
) -> Option<ir::Type> {
    match stmt {
        ast::Stmt::VarDecl(var) => {
            if let Some(init) = &var.init {
                let ty = infer_expr_return_type_with_env(init, vars);
                vars.insert(var.name.clone(), ty);
            }
            None
        }
        ast::Stmt::ConstDecl(const_decl) => {
            let ty = infer_expr_return_type_with_env(&const_decl.init, vars);
            vars.insert(const_decl.name.clone(), ty);
            None
        }
        ast::Stmt::Return(ret) => ret
            .expr
            .as_ref()
            .map(|expr| infer_expr_return_type_with_env(expr, vars)),
        ast::Stmt::Block(block) => {
            let mut inner_vars = vars.clone();
            let ty = infer_block_return_type_with_env(block, &mut inner_vars);
            if matches!(ty, ir::Type::Inferred) {
                None
            } else {
                Some(ty)
            }
        }
        ast::Stmt::If(if_stmt) => {
            let then_ty = {
                let mut inner = vars.clone();
                infer_block_return_type_with_env(&if_stmt.then_branch, &mut inner)
            };
            let else_ty = if let Some(else_block) = &if_stmt.else_branch {
                let mut inner = vars.clone();
                infer_block_return_type_with_env(else_block, &mut inner)
            } else {
                ir::Type::Inferred
            };
            if then_ty == else_ty && !matches!(then_ty, ir::Type::Inferred) {
                Some(then_ty)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn lower_literal(lit: &ast::Literal) -> ir::Literal {
    match lit {
        ast::Literal::Int(v) => ir::Literal::Int(*v),
        ast::Literal::Float(v) => ir::Literal::Float(*v),
        ast::Literal::Bool(v) => ir::Literal::Bool(*v),
        ast::Literal::String(s) => ir::Literal::String(s.clone()),
        ast::Literal::Char(c) => ir::Literal::Char(*c),
    }
}
