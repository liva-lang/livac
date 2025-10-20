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
            ast::TopLevel::Class(_class) => {
                // For now, skip classes in IR - they will be handled by AST generator
                // module.items.push(ir::Item::Class(lower_class(class)));
            }
            ast::TopLevel::Type(_type_decl) => {
                // For now, skip type declarations in IR - they will be handled by AST generator
                // module.items.push(ir::Item::Type(lower_type(type_decl)));
            }
            ast::TopLevel::Import(_) => {
                // Skip imports in IR
            }
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

    let contains_fail = contains_fail_in_function(func);

    let mut ret_type = if let Some(ret) = &func.return_type {
        ir::Type::from_ast(&Some(ret.clone()))
    } else if let Some(expr) = &func.expr_body {
        infer_expr_return_type(expr)
    } else if let Some(body) = &func.body {
        infer_block_return_type(body)
    } else {
        ir::Type::Inferred
    };

    // If function contains fail, wrap return type in Result<T, Error>
    if contains_fail {
        ret_type = match ret_type {
            ir::Type::Unit => ir::Type::Custom("Result<(), liva_rt::Error>".to_string()),
            ir::Type::Number => ir::Type::Custom("Result<i32, liva_rt::Error>".to_string()),
            ir::Type::Float => ir::Type::Custom("Result<f64, liva_rt::Error>".to_string()),
            ir::Type::Bool => ir::Type::Custom("Result<bool, liva_rt::Error>".to_string()),
            ir::Type::String => ir::Type::Custom("Result<String, liva_rt::Error>".to_string()),
            ir::Type::Custom(type_str) => {
                ir::Type::Custom(format!("Result<{}, liva_rt::Error>", type_str))
            }
            ir::Type::Inferred => ir::Type::Custom("Result<(), liva_rt::Error>".to_string()),
            _ => ir::Type::Custom("Result<(), liva_rt::Error>".to_string()), // Handle other types
        };
    }

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
        contains_fail,
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

fn lower_if_body(body: &ast::IfBody) -> ir::Block {
    match body {
        ast::IfBody::Block(block) => lower_block(block),
        ast::IfBody::Stmt(stmt) => ir::Block {
            statements: vec![lower_stmt(stmt)],
        },
    }
}

fn lower_stmt(stmt: &ast::Stmt) -> ir::Stmt {
    match stmt {
        ast::Stmt::VarDecl(var) => {
            let value = lower_expr(&var.init);
            // For now, handle only single binding
            if var.bindings.len() == 1 {
                let binding = &var.bindings[0];
                ir::Stmt::Let {
                    name: binding.name.clone(),
                    ty: binding
                        .type_ref
                        .as_ref()
                        .map(|ty| ir::Type::from_ast(&Some(ty.clone()))),
                    value,
                }
            } else {
                // Multiple bindings - for now just use first name
                ir::Stmt::Let {
                    name: var.bindings[0].name.clone(),
                    ty: None,
                    value,
                }
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
            then_block: lower_if_body(&if_stmt.then_branch),
            else_block: if_stmt.else_branch.as_ref().map(|body| lower_if_body(body)),
        },
        ast::Stmt::While(while_stmt) => ir::Stmt::While {
            condition: lower_expr(&while_stmt.condition),
            body: lower_block(&while_stmt.body),
        },
        ast::Stmt::For(for_stmt) => ir::Stmt::For {
            var: for_stmt.var.clone(),
            iterable: lower_expr(&for_stmt.iterable),
            policy: lower_data_parallel_policy(for_stmt.policy.clone()),
            options: lower_for_policy_options(&for_stmt.options),
            body: lower_block(&for_stmt.body),
        },
        ast::Stmt::Block(block) => ir::Stmt::Block(lower_block(block)),
        ast::Stmt::TryCatch(try_catch) => ir::Stmt::TryCatch {
            try_block: lower_block(&try_catch.try_block),
            error_var: try_catch.catch_var.clone(),
            catch_block: lower_block(&try_catch.catch_block),
        },
        ast::Stmt::Fail(fail_stmt) => ir::Stmt::Return(Some(ir::Expr::Call {
            callee: Box::new(ir::Expr::Identifier("Err".to_string())),
            args: vec![ir::Expr::Call {
                callee: Box::new(ir::Expr::Member {
                    object: Box::new(ir::Expr::Member {
                        object: Box::new(ir::Expr::Identifier("liva_rt".to_string())),
                        property: "Error".to_string(),
                    }),
                    property: "from".to_string(),
                }),
                args: vec![lower_expr(&fail_stmt.expr)],
            }],
        })),
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
        ast::Expr::Call(call) => lower_call_expr(call),
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
        ast::Expr::StructLiteral { type_name, fields } => ir::Expr::StructLiteral {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| (name.clone(), lower_expr(value)))
                .collect(),
        },
        ast::Expr::Fail(expr) => ir::Expr::Call {
            callee: Box::new(ir::Expr::Identifier("Err".to_string())),
            args: vec![ir::Expr::Call {
                callee: Box::new(ir::Expr::Member {
                    object: Box::new(ir::Expr::Member {
                        object: Box::new(ir::Expr::Identifier("liva_rt".to_string())),
                        property: "Error".to_string(),
                    }),
                    property: "from".to_string(),
                }),
                args: vec![lower_expr(expr)],
            }],
        },
        ast::Expr::Lambda(lambda) => {
            let params = lambda
                .params
                .iter()
                .map(|param| ir::LambdaParam {
                    name: param.name.clone(),
                    type_ref: param.type_ref.as_ref().map(|tr| tr.to_rust_type()),
                })
                .collect();

            let body = match &lambda.body {
                ast::LambdaBody::Expr(expr) => ir::LambdaBody::Expr(Box::new(lower_expr(expr))),
                ast::LambdaBody::Block(block) => {
                    ir::LambdaBody::Block(lower_block(block).statements)
                }
            };

            ir::Expr::Lambda(ir::LambdaExpr {
                is_move: lambda.is_move,
                params,
                return_type: lambda.return_type.as_ref().map(|tr| tr.to_rust_type()),
                body,
            })
        }
        ast::Expr::MethodCall(method_call) => {
            // TODO: Phase 2 - implement proper IR lowering for method calls
            // For now, lower as a regular function call with the object as first argument
            ir::Expr::Call {
                callee: Box::new(ir::Expr::Member {
                    object: Box::new(lower_expr(&method_call.object)),
                    property: method_call.method.clone(),
                }),
                args: method_call.args.iter().map(lower_expr).collect(),
            }
        }
    }
}

fn lower_data_parallel_policy(policy: ast::DataParallelPolicy) -> ir::DataParallelPolicy {
    match policy {
        ast::DataParallelPolicy::Seq => ir::DataParallelPolicy::Seq,
        ast::DataParallelPolicy::Par => ir::DataParallelPolicy::Par,
        ast::DataParallelPolicy::Vec => ir::DataParallelPolicy::Vec,
        ast::DataParallelPolicy::ParVec => ir::DataParallelPolicy::ParVec,
    }
}

fn lower_for_policy_options(options: &ast::ForPolicyOptions) -> ir::ForPolicyOptions {
    ir::ForPolicyOptions {
        ordered: options.ordered,
        chunk: options.chunk,
        threads: options
            .threads
            .as_ref()
            .map(|thread_option| match thread_option {
                ast::ThreadOption::Auto => ir::ThreadOption::Auto,
                ast::ThreadOption::Count(count) => ir::ThreadOption::Count(*count),
            }),
        simd_width: options
            .simd_width
            .as_ref()
            .map(|simd_option| match simd_option {
                ast::SimdWidthOption::Auto => ir::SimdWidthOption::Auto,
                ast::SimdWidthOption::Width(width) => ir::SimdWidthOption::Width(*width),
            }),
        prefetch: options.prefetch,
        reduction: options.reduction.as_ref().map(|reduction| match reduction {
            ast::ReductionOption::Safe => ir::ReductionOption::Safe,
            ast::ReductionOption::Fast => ir::ReductionOption::Fast,
        }),
        schedule: options.schedule.as_ref().map(|schedule| match schedule {
            ast::ScheduleOption::Static => ir::ScheduleOption::Static,
            ast::ScheduleOption::Dynamic => ir::ScheduleOption::Dynamic,
        }),
        detect: options.detect.as_ref().map(|detect| match detect {
            ast::DetectOption::Auto => ir::DetectOption::Auto,
        }),
    }
}

fn lower_call_expr(call: &ast::CallExpr) -> ir::Expr {
    let lowered_args: Vec<ir::Expr> = call.args.iter().map(lower_expr).collect();

    match call.exec_policy {
        ast::ExecPolicy::Normal => ir::Expr::Call {
            callee: Box::new(lower_expr(&call.callee)),
            args: lowered_args,
        },
        ast::ExecPolicy::Async => ir::Expr::AsyncCall {
            callee: Box::new(lower_expr(&call.callee)),
            args: lowered_args,
        },
        ast::ExecPolicy::Par => ir::Expr::ParallelCall {
            callee: Box::new(lower_expr(&call.callee)),
            args: lowered_args,
        },
        ast::ExecPolicy::TaskAsync => {
            lower_task_like_call(call, ir::ConcurrencyMode::Async, lowered_args)
        }
        ast::ExecPolicy::TaskPar => {
            lower_task_like_call(call, ir::ConcurrencyMode::Parallel, lowered_args)
        }
        ast::ExecPolicy::FireAsync => {
            lower_fire_like_call(call, ir::ConcurrencyMode::Async, lowered_args)
        }
        ast::ExecPolicy::FirePar => {
            lower_fire_like_call(call, ir::ConcurrencyMode::Parallel, lowered_args)
        }
    }
}

fn lower_task_like_call(
    call: &ast::CallExpr,
    mode: ir::ConcurrencyMode,
    args: Vec<ir::Expr>,
) -> ir::Expr {
    if let Some(name) = extract_callee_name(&call.callee) {
        ir::Expr::TaskCall {
            mode,
            callee: name,
            args,
        }
    } else {
        ir::Expr::Unsupported(ast::Expr::Call(call.clone()))
    }
}

fn lower_fire_like_call(
    call: &ast::CallExpr,
    mode: ir::ConcurrencyMode,
    args: Vec<ir::Expr>,
) -> ir::Expr {
    if let Some(name) = extract_callee_name(&call.callee) {
        ir::Expr::FireCall {
            mode,
            callee: name,
            args,
        }
    } else {
        ir::Expr::Unsupported(ast::Expr::Call(call.clone()))
    }
}

fn extract_callee_name(expr: &ast::Expr) -> Option<String> {
    match expr {
        ast::Expr::Identifier(name) => Some(name.clone()),
        _ => None,
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

fn if_body_uses_param_as_array(body: &ast::IfBody, name: &str) -> bool {
    match body {
        ast::IfBody::Block(block) => block_uses_param_as_array(block, name),
        ast::IfBody::Stmt(stmt) => stmt_uses_param_as_array(stmt, name),
    }
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
                || if_body_uses_param_as_array(&if_stmt.then_branch, name)
                || if_stmt
                    .else_branch
                    .as_ref()
                    .map(|body| if_body_uses_param_as_array(body, name))
                    .unwrap_or(false)
        }
        ast::Stmt::Block(block) => block_uses_param_as_array(block, name),
        ast::Stmt::TryCatch(try_catch) => {
            block_uses_param_as_array(&try_catch.try_block, name)
                || block_uses_param_as_array(&try_catch.catch_block, name)
        }
        ast::Stmt::Switch(switch_stmt) => {
            expr_uses_param_as_array(&switch_stmt.discriminant, name)
                || switch_stmt.cases.iter().any(|case| {
                    case.body
                        .iter()
                        .any(|stmt| stmt_uses_param_as_array(stmt, name))
                })
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
        ast::Expr::Call(call) => {
            if matches!(call.callee.as_ref(), ast::Expr::Identifier(id) if id == "len")
                && call
                    .args
                    .iter()
                    .any(|arg| expr_references_identifier(arg, name))
            {
                return true;
            }
            expr_uses_param_as_array(&call.callee, name)
                || call
                    .args
                    .iter()
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
        ast::Expr::ArrayLiteral(items) => items
            .iter()
            .any(|item| expr_uses_param_as_array(item, name)),
        ast::Expr::ObjectLiteral(fields) => fields
            .iter()
            .any(|(_, value)| expr_uses_param_as_array(value, name)),
        ast::Expr::Unary { operand, .. } => expr_uses_param_as_array(operand, name),
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
        ast::Expr::Member { object, .. } => expr_references_identifier(object, name),
        ast::Expr::Index { object, index } => {
            expr_references_identifier(object, name) || expr_references_identifier(index, name)
        }
        ast::Expr::ArrayLiteral(items) => items
            .iter()
            .any(|item| expr_references_identifier(item, name)),
        ast::Expr::ObjectLiteral(fields) => fields
            .iter()
            .any(|(_, value)| expr_references_identifier(value, name)),
        ast::Expr::StringTemplate { parts } => parts.iter().any(|part| match part {
            ast::StringTemplatePart::Expr(expr) => expr_references_identifier(expr, name),
            _ => false,
        }),
        ast::Expr::Call(call) => {
            expr_references_identifier(&call.callee, name)
                || call
                    .args
                    .iter()
                    .any(|arg| expr_references_identifier(arg, name))
        }
        _ => false,
    }
}

fn infer_expr_return_type(expr: &ast::Expr) -> ir::Type {
    let vars = HashMap::new();
    infer_expr_return_type_with_env(expr, &vars)
}

fn infer_expr_return_type_with_env(expr: &ast::Expr, vars: &HashMap<String, ir::Type>) -> ir::Type {
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
            then_expr,
            else_expr,
            ..
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

fn infer_if_body_return_type_with_env(
    body: &ast::IfBody,
    vars: &mut HashMap<String, ir::Type>,
) -> ir::Type {
    match body {
        ast::IfBody::Block(block) => infer_block_return_type_with_env(block, vars),
        ast::IfBody::Stmt(stmt) => {
            infer_stmt_return_type_with_env(stmt, vars).unwrap_or(ir::Type::Inferred)
        }
    }
}

fn infer_stmt_return_type_with_env(
    stmt: &ast::Stmt,
    vars: &mut HashMap<String, ir::Type>,
) -> Option<ir::Type> {
    match stmt {
        ast::Stmt::VarDecl(var) => {
            let ty = infer_expr_return_type_with_env(&var.init, vars);
            for binding in &var.bindings {
                vars.insert(binding.name.clone(), ty.clone());
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
                infer_if_body_return_type_with_env(&if_stmt.then_branch, &mut inner)
            };
            let else_ty = if let Some(else_body) = &if_stmt.else_branch {
                let mut inner = vars.clone();
                infer_if_body_return_type_with_env(else_body, &mut inner)
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

fn contains_fail_in_function(func: &ast::FunctionDecl) -> bool {
    // Check body for fail statements
    if let Some(body) = &func.body {
        if contains_fail_in_block(body) {
            return true;
        }
    }

    // Check expr_body for fail expressions
    if let Some(expr) = &func.expr_body {
        if contains_fail_in_expr(expr) {
            return true;
        }
    }
    false
}

fn contains_fail_in_block(block: &ast::BlockStmt) -> bool {
    for stmt in &block.stmts {
        if contains_fail_in_stmt(stmt) {
            return true;
        }
    }
    false
}

fn contains_fail_in_stmt(stmt: &ast::Stmt) -> bool {
    match stmt {
        ast::Stmt::Fail(_) => true,
        ast::Stmt::VarDecl(var) => contains_fail_in_expr(&var.init),
        ast::Stmt::Assign(assign) => contains_fail_in_expr(&assign.value),
        ast::Stmt::Return(ret) => ret
            .expr
            .as_ref()
            .map_or(false, |e| contains_fail_in_expr(e)),
        ast::Stmt::If(if_stmt) => {
            contains_fail_in_expr(&if_stmt.condition)
                || contains_fail_in_if_body(&if_stmt.then_branch)
                || if_stmt
                    .else_branch
                    .as_ref()
                    .map_or(false, |b| contains_fail_in_if_body(b))
        }
        ast::Stmt::While(while_stmt) => {
            contains_fail_in_expr(&while_stmt.condition) || contains_fail_in_block(&while_stmt.body)
        }
        ast::Stmt::For(for_stmt) => contains_fail_in_block(&for_stmt.body),
        ast::Stmt::Switch(switch) => {
            contains_fail_in_expr(&switch.discriminant)
                || switch
                    .cases
                    .iter()
                    .any(|case| case.body.iter().any(|s| contains_fail_in_stmt(s)))
                || switch
                    .default
                    .as_ref()
                    .map_or(false, |b| b.iter().any(|s| contains_fail_in_stmt(s)))
        }
        ast::Stmt::TryCatch(try_catch) => {
            contains_fail_in_block(&try_catch.try_block)
                || contains_fail_in_block(&try_catch.catch_block)
        }
        ast::Stmt::Throw(throw) => contains_fail_in_expr(&throw.expr),
        ast::Stmt::Expr(expr_stmt) => contains_fail_in_expr(&expr_stmt.expr),
        _ => false,
    }
}

fn contains_fail_in_if_body(body: &ast::IfBody) -> bool {
    match body {
        ast::IfBody::Block(block) => contains_fail_in_block(block),
        ast::IfBody::Stmt(stmt) => contains_fail_in_stmt(stmt),
    }
}

fn contains_fail_in_expr(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::Fail(_) => true,
        ast::Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            contains_fail_in_expr(condition)
                || contains_fail_in_expr(then_expr)
                || contains_fail_in_expr(else_expr)
        }
        ast::Expr::Binary { left, right, .. } => {
            contains_fail_in_expr(left) || contains_fail_in_expr(right)
        }
        ast::Expr::Unary { operand, .. } => contains_fail_in_expr(operand),
        ast::Expr::Call(call) => {
            contains_fail_in_expr(&call.callee)
                || call.args.iter().any(|arg| contains_fail_in_expr(arg))
        }
        ast::Expr::Member { object, .. } => contains_fail_in_expr(object),
        ast::Expr::Index { object, index } => {
            contains_fail_in_expr(object) || contains_fail_in_expr(index)
        }
        ast::Expr::ObjectLiteral(fields) => {
            fields.iter().any(|(_, value)| contains_fail_in_expr(value))
        }
        ast::Expr::StructLiteral { fields, .. } => {
            fields.iter().any(|(_, value)| contains_fail_in_expr(value))
        }
        ast::Expr::ArrayLiteral(elements) => {
            elements.iter().any(|elem| contains_fail_in_expr(elem))
        }
        ast::Expr::Lambda(lambda) => match &lambda.body {
            ast::LambdaBody::Expr(body) => contains_fail_in_expr(body),
            ast::LambdaBody::Block(block) => contains_fail_in_block(block),
        },
        ast::Expr::StringTemplate { parts } => parts.iter().any(|part| match part {
            ast::StringTemplatePart::Expr(expr) => contains_fail_in_expr(expr),
            _ => false,
        }),
        _ => false,
    }
}
