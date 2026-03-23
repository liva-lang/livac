use crate::ast::*;
use crate::error::Result;
use serde::Serialize;
use std::collections::BTreeSet;

/// Dependency info for a user-declared `use rust` crate
#[derive(Debug, Clone, Serialize)]
pub struct RustCrateDep {
    pub name: String,
    pub alias: Option<String>,
    pub version: Option<String>,
    pub features: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct DesugarContext {
    pub rust_crates: Vec<RustCrateDep>,
    pub has_async: bool,
    pub has_parallel: bool,
    pub has_random: bool,                  // true if Math.random() is used
    pub has_rust_blocks: bool,             // true if any `rust { }` block is used
    pub has_logging: bool,                 // true if Log.* is used
    pub has_config: bool,                  // true if Config.* is used
    pub has_regex: bool,                   // true if Regex.* is used
    pub has_date: bool,                    // true if Date.* is used
    pub has_crypto: bool,                  // true if Crypto.* is used (sha2, md5, base64 crates)
    pub has_server: bool,                  // true if Server.create() is used (axum crate)
    pub has_db: bool,                      // true if DB.* is used (rusqlite crate)
    pub async_functions: BTreeSet<String>, // Functions that are async (BTreeSet for deterministic order)
    #[serde(skip)]
    pub source_filename: String,           // Source filename for error traces
}

impl DesugarContext {
    fn new() -> Self {
        Self {
            rust_crates: Vec::new(),
            has_async: false,
            has_parallel: false,
            has_random: false,
            has_rust_blocks: false,
            has_logging: false,
            has_config: false,
            has_regex: false,
            has_date: false,
            has_crypto: false,
            has_server: false,
            has_db: false,
            async_functions: BTreeSet::new(),
            source_filename: String::new(),
        }
    }
}

pub fn desugar(program: Program) -> Result<DesugarContext> {
    let mut ctx = DesugarContext::new();

    // Collect use rust declarations
    for item in &program.items {
        if let TopLevel::UseRust(use_rust) = item {
            ctx.rust_crates.push(RustCrateDep {
                name: use_rust.crate_name.clone(),
                alias: use_rust.alias.clone(),
                version: use_rust.version.clone(),
                features: use_rust.features.clone(),
            });
        }

        // Check for async/parallel usage and rust blocks
        check_concurrency(&item, &mut ctx);
    }

    // Add tokio if async is used
    if ctx.has_async {
        ctx.rust_crates.push(RustCrateDep {
            name: "tokio".to_string(),
            alias: None,
            version: None,
            features: Vec::new(),
        });
    }

    Ok(ctx)
}

fn check_concurrency(item: &TopLevel, ctx: &mut DesugarContext) {
    match item {
        TopLevel::Function(func) => {
            if func.is_async_inferred {
                ctx.has_async = true;
                ctx.async_functions.insert(func.name.clone());
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
                        // Track as ClassName.methodName for method calls
                        ctx.async_functions
                            .insert(format!("{}.{}", class.name, method.name));
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
        TopLevel::Test(test) => {
            check_block_concurrency_block(&test.body, ctx);
        }
        TopLevel::ExprStmt(expr) => {
            check_expr_concurrency(expr, ctx);
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
                DataParallelPolicy::Par | DataParallelPolicy::Vec | DataParallelPolicy::ParVec
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
        Stmt::Defer(defer_stmt) => {
            check_stmt_concurrency(&defer_stmt.body, ctx);
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
                ExecPolicy::Async | ExecPolicy::TaskAsync => {
                    ctx.has_async = true;
                }
                ExecPolicy::Par | ExecPolicy::TaskPar => {
                    ctx.has_parallel = true;
                }
                ExecPolicy::Normal => {}
            }

            check_expr_concurrency(&call.callee, ctx);
            for arg in &call.args {
                check_expr_concurrency(arg, ctx);
            }
        }
        Expr::MethodCall(method_call) => {
            // Check if it's Math.random()
            if let Expr::Identifier(name) = method_call.object.as_ref() {
                if name == "Math" && method_call.method == "random" {
                    ctx.has_random = true;
                }
                if name == "Log" {
                    ctx.has_logging = true;
                }
                if name == "Config" {
                    ctx.has_config = true;
                }
                if name == "Regex" {
                    ctx.has_regex = true;
                }
                if name == "Date" {
                    ctx.has_date = true;
                }
                if name == "Random" {
                    ctx.has_random = true;
                }
                if name == "Crypto" {
                    ctx.has_crypto = true;
                }
                if name == "Server" {
                    ctx.has_server = true;
                    ctx.has_async = true;
                }
                if name == "DB" {
                    ctx.has_db = true;
                }
            }

            // Check if it uses parallel array adapters
            match method_call.adapter {
                crate::ast::ArrayAdapter::Par | crate::ast::ArrayAdapter::ParVec => {
                    ctx.has_parallel = true;
                }
                _ => {}
            }

            // Continue checking nested expressions
            check_expr_concurrency(&method_call.object, ctx);
            for arg in &method_call.args {
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
        Expr::RustBlock { .. } => {
            ctx.has_rust_blocks = true;
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
    fn test_detect_task_and_par_calls() {
        let source = r#"
            use rust "serde" as sd

            compute() => 1

            main() {
                let handle = task async compute()
                par compute()
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
            .any(|dep| dep.name == "serde" && dep.alias.as_deref() == Some("sd")));
        assert!(ctx
            .rust_crates
            .iter()
            .any(|dep| dep.name == "tokio" && dep.alias.is_none()));
    }
}
