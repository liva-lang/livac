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
            ast::TopLevel::Class(_)
            | ast::TopLevel::Type(_)
            | ast::TopLevel::Import(_)
            | ast::TopLevel::Test(_) => {
                module.items.push(ir::Item::Unsupported(item.clone()));
            }
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
            default: param.default.as_ref().map(|expr| lower_expr(expr)),
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

    ir::Function {
        name: func.name.clone(),
        params,
        ret_type: ir::Type::from_ast(&func.return_type),
        body,
        async_kind: if func.is_async_inferred {
            ir::AsyncKind::Async
        } else {
            ir::AsyncKind::NotAsync
        },
        visibility: ir::Visibility::Public,
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
            if let Some(init) = &var.init {
                ir::Stmt::Let {
                    name: var.name.clone(),
                    ty: var
                        .type_ref
                        .as_ref()
                        .map(|ty| ir::Type::from_ast(&Some(ty.clone()))),
                    value: lower_expr(init),
                }
            } else {
                ir::Stmt::Unsupported(stmt.clone())
            }
        }
        ast::Stmt::Assign(assign) => ir::Stmt::Assign {
            target: lower_expr(&assign.target),
            value: lower_expr(&assign.value),
        },
        ast::Stmt::Return(ret) => ir::Stmt::Return(ret.expr.as_ref().map(|expr| lower_expr(expr))),
        ast::Stmt::Expr(expr) => ir::Stmt::Expr(lower_expr(&expr.expr)),
        _ => ir::Stmt::Unsupported(stmt.clone()),
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
                _ => return ir::Expr::Unsupported(expr.clone()),
            };
            ir::Expr::Binary {
                op,
                left: Box::new(lower_expr(left)),
                right: Box::new(lower_expr(right)),
            }
        }
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
        ast::Expr::ArrayLiteral(items) => {
            ir::Expr::ArrayLiteral(items.iter().map(lower_expr).collect())
        }
        ast::Expr::ObjectLiteral(fields) => ir::Expr::ObjectLiteral(
            fields
                .iter()
                .map(|(name, value)| (name.clone(), lower_expr(value)))
                .collect(),
        ),
        _ => ir::Expr::Unsupported(expr.clone()),
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
