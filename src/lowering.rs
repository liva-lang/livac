//! Lowering utilities: AST → IR.

use crate::ast;
use crate::ir;

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
        .map(|param| ir::Param {
            name: param.name.clone(),
            ty: ir::Type::from_ast(&param.type_ref),
            default: param.default.as_ref().map(lower_expr),
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
    } else if func.expr_body.is_some() {
        // For expression-bodied functions without explicit return type, infer i32
        ir::Type::Number
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
                ast::BinOp::Range => return ir::Expr::Unsupported(expr.clone()),
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

fn lower_literal(lit: &ast::Literal) -> ir::Literal {
    match lit {
        ast::Literal::Int(v) => ir::Literal::Int(*v),
        ast::Literal::Float(v) => ir::Literal::Float(*v),
        ast::Literal::Bool(v) => ir::Literal::Bool(*v),
        ast::Literal::String(s) => ir::Literal::String(s.clone()),
        ast::Literal::Char(c) => ir::Literal::Char(*c),
    }
}
