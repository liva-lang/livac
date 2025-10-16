use crate::ast::*;
use crate::error::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct DesugarContext {
    pub rust_crates: Vec<(String, Option<String>)>,
    pub has_async: bool,
    pub has_parallel: bool,
}

impl DesugarContext {
    fn new() -> Self {
        Self {
            rust_crates: Vec::new(),
            has_async: false,
            has_parallel: false,
        }
    }
}

pub fn desugar(program: Program) -> Result<DesugarContext> {
    let mut ctx = DesugarContext::new();

    // Collect use rust declarations
    for item in &program.items {
        if let TopLevel::UseRust(use_rust) = item {
            ctx.rust_crates
                .push((use_rust.crate_name.clone(), use_rust.alias.clone()));
        }

        // Check for async/parallel usage
        check_concurrency(&item, &mut ctx);
    }

    // Add tokio if async is used
    if ctx.has_async {
        ctx.rust_crates.push(("tokio".to_string(), None));
    }

    Ok(ctx)
}

fn check_concurrency(item: &TopLevel, ctx: &mut DesugarContext) {
    match item {
        TopLevel::Function(func) => {
            if func.is_async_inferred {
                ctx.has_async = true;
            }
            if let Some(body) = &func.body {
                check_block_concurrency_block(body, ctx);
            }
            if let Some(expr) = &func.expr_body {
                check_expr_concurrency(expr, ctx);
            }
        }
        TopLevel::Class(class) => {
            for member in &class.members {
                if let Member::Method(method) = member {
                    if method.is_async_inferred {
                        ctx.has_async = true;
                    }
                    if let Some(body) = &method.body {
                        check_block_concurrency_block(body, ctx);
                    }
                    if let Some(expr) = &method.expr_body {
                        check_expr_concurrency(expr, ctx);
                    }
                }
            }
        }
        _ => {}
    }
}

fn check_block_concurrency(body: &IfBody, ctx: &mut DesugarContext) {
    match body {
        IfBody::Block(block) => {
            for stmt in &block.stmts {
                check_stmt_concurrency(stmt, ctx);
            }
        }
        IfBody::Stmt(stmt) => {
            check_stmt_concurrency(stmt, ctx);
        }
    }
}

fn check_block_concurrency_block(block: &BlockStmt, ctx: &mut DesugarContext) {
    for stmt in &block.stmts {
        check_stmt_concurrency(stmt, ctx);
    }
}

fn check_stmt_concurrency(stmt: &Stmt, ctx: &mut DesugarContext) {
    match stmt {
        Stmt::VarDecl(var) => {
            check_expr_concurrency(&var.init, ctx);
        }
        Stmt::ConstDecl(const_decl) => check_expr_concurrency(&const_decl.init, ctx),
        Stmt::Assign(assign) => {
            check_expr_concurrency(&assign.target, ctx);
            check_expr_concurrency(&assign.value, ctx);
        }
        Stmt::If(if_stmt) => {
            check_expr_concurrency(&if_stmt.condition, ctx);
            check_block_concurrency(&if_stmt.then_branch, ctx);
            if let Some(else_branch) = &if_stmt.else_branch {
                check_block_concurrency(else_branch, ctx);
            }
        }
        Stmt::While(while_stmt) => {
            check_expr_concurrency(&while_stmt.condition, ctx);
            check_block_concurrency_block(&while_stmt.body, ctx);
        }
        Stmt::For(for_stmt) => {
            if matches!(
                for_stmt.policy,
                DataParallelPolicy::Par | DataParallelPolicy::Vec | DataParallelPolicy::Boost
            ) {
                ctx.has_parallel = true;
            }
            check_expr_concurrency(&for_stmt.iterable, ctx);
            check_block_concurrency_block(&for_stmt.body, ctx);
        }
        Stmt::Return(ret) => {
            if let Some(expr) = &ret.expr {
                check_expr_concurrency(expr, ctx);
            }
        }
        Stmt::Expr(expr_stmt) => check_expr_concurrency(&expr_stmt.expr, ctx),
        Stmt::Block(block) => check_block_concurrency_block(block, ctx),
        _ => {}
    }
}

fn check_expr_concurrency(expr: &Expr, ctx: &mut DesugarContext) {
    match expr {
        Expr::Call(call) => {
            match call.exec_policy {
                ExecPolicy::Async | ExecPolicy::TaskAsync | ExecPolicy::FireAsync => {
                    ctx.has_async = true;
                }
                ExecPolicy::Par | ExecPolicy::TaskPar | ExecPolicy::FirePar => {
                    ctx.has_parallel = true;
                }
                ExecPolicy::Normal => {}
            }

            check_expr_concurrency(&call.callee, ctx);
            for arg in &call.args {
                check_expr_concurrency(arg, ctx);
            }
        }
        Expr::Lambda(lambda) => match &lambda.body {
            LambdaBody::Expr(expr) => check_expr_concurrency(expr, ctx),
            LambdaBody::Block(block) => check_block_concurrency_block(block, ctx),
        },
        Expr::Binary { left, right, .. } => {
            check_expr_concurrency(left, ctx);
            check_expr_concurrency(right, ctx);
        }
        Expr::Unary { operand, .. } => check_expr_concurrency(operand, ctx),
        Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            check_expr_concurrency(condition, ctx);
            check_expr_concurrency(then_expr, ctx);
            check_expr_concurrency(else_expr, ctx);
        }
        Expr::Member { object, .. } => check_expr_concurrency(object, ctx),
        Expr::Index { object, index } => {
            check_expr_concurrency(object, ctx);
            check_expr_concurrency(index, ctx);
        }
        Expr::ArrayLiteral(elements) => {
            for elem in elements {
                check_expr_concurrency(elem, ctx);
            }
        }
        Expr::ObjectLiteral(fields) => {
            for (_, value) in fields {
                check_expr_concurrency(value, ctx);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;
    use crate::semantic::analyze;

    #[test]
    fn test_detect_async() {
        let source = r#"
            main() {
                let x = async fetchUser()
            }
        "#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens, source).unwrap();
        let analyzed = analyze(program).unwrap();
        let ctx = desugar(analyzed).unwrap();

        assert!(ctx.has_async);
    }

    #[test]
    fn test_detect_parallel() {
        let source = r#"
            main() {
                let x = par compute()
            }
        "#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens, source).unwrap();
        let analyzed = analyze(program).unwrap();
        let ctx = desugar(analyzed).unwrap();

        assert!(ctx.has_parallel);
    }

    #[test]
    fn test_detect_task_and_fire_calls() {
        let source = r#"
            use rust "serde" as sd

            compute() => 1

            main() {
                let handle = task async compute()
                fire par compute()
                let values = [par compute(), task par compute()]
                return handle
            }
        "#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens, source).unwrap();
        let analyzed = analyze(program).unwrap();
        let ctx = desugar(analyzed).unwrap();

        assert!(ctx.has_async);
        assert!(ctx.has_parallel);
        assert!(ctx
            .rust_crates
            .iter()
            .any(|(name, alias)| name == "serde" && alias.as_deref() == Some("sd")));
        assert!(ctx
            .rust_crates
            .iter()
            .any(|(name, alias)| name == "tokio" && alias.is_none()));
    }
}
