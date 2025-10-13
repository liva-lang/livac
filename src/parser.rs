use crate::ast::*;
use crate::error::{CompilerError, Result};
use crate::lexer::{tokenize, Token, TokenWithSpan};

pub struct Parser {
    tokens: Vec<TokenWithSpan>,
    current: usize,
    source: String,
}

impl Parser {
    fn new(tokens: Vec<TokenWithSpan>, source: String) -> Self {
        Self {
            tokens,
            current: 0,
            source,
        }
    }

    fn peek(&self) -> Option<&Token> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.current].token)
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Option<&Token> {
        if self.current == 0 {
            None
        } else {
            Some(&self.tokens[self.current - 1].token)
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(self.peek().unwrap()) == std::mem::discriminant(token)
        }
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token: Token) -> Result<()> {
        if self.check(&token) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("Expected {:?}", token)))
        }
    }

    fn error(&self, message: String) -> CompilerError {
        let (line, col) = if self.current < self.tokens.len() {
            self.calculate_line_col(self.current)
        } else if !self.tokens.is_empty() {
            self.calculate_line_col(self.tokens.len() - 1)
        } else {
            (1, 1)
        };
        CompilerError::ParseError {
            line,
            col,
            msg: message,
        }
    }

    fn calculate_line_col(&self, token_index: usize) -> (usize, usize) {
        if token_index >= self.tokens.len() {
            return (1, 1);
        }

        let span = &self.tokens[token_index].span;
        let mut line = 1;
        let mut col = 1;

        for (_i, ch) in self.source[..span.start].chars().enumerate() {
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            items.push(self.parse_top_level()?);
        }

        Ok(Program { items })
    }

    fn parse_top_level(&mut self) -> Result<TopLevel> {
        if self.match_token(&Token::Import) {
            let name = self.parse_identifier()?;
            return Ok(TopLevel::Import(ImportDecl { name, alias: None }));
        }

        if self.match_token(&Token::Use) {
            self.expect(Token::Rust)?;
            let crate_name = self.parse_string_literal()?;
            let alias = if self.match_token(&Token::As) {
                Some(self.parse_identifier()?)
            } else {
                None
            };
            return Ok(TopLevel::UseRust(UseRustDecl { crate_name, alias }));
        }

        if self.match_token(&Token::Type) {
            let name = self.parse_identifier()?;
            self.expect(Token::LBrace)?;
            let members = self.parse_members()?;
            self.expect(Token::RBrace)?;
            return Ok(TopLevel::Type(TypeDecl { name, members }));
        }

        if self.match_token(&Token::Test) {
            let is_string_name = if let Some(Token::StringLiteral(_)) = self.peek() {
                true
            } else {
                false
            };

            let name = if is_string_name {
                self.parse_string_literal()?
            } else {
                self.parse_identifier()?
            };

            // Tests with string names don't have parentheses: test "name" { ... }
            // Tests with identifier names have parentheses: test name() { ... }
            if !is_string_name {
                self.expect(Token::LParen)?;
                self.expect(Token::RParen)?;
            }

            self.expect(Token::LBrace)?;
            let body = self.parse_block_stmt()?;
            self.expect(Token::RBrace)?;
            return Ok(TopLevel::Test(TestDecl { name, body }));
        }

        // Check if we have any tokens left to parse
        if self.is_at_end() {
            return Err(self.error("Unexpected end of file".into()));
        }

        // Try to parse as class or function
        let name = self.parse_identifier()?;

        // Check for inheritance
        let base = if self.match_token(&Token::Colon) {
            Some(self.parse_identifier()?)
        } else {
            None
        };

        if self.match_token(&Token::LBrace) {
            // It's a class
            let members = self.parse_members()?;
            self.expect(Token::RBrace)?;
            return Ok(TopLevel::Class(ClassDecl {
                name,
                base,
                members,
            }));
        }

        // Otherwise it's a function
        let type_params = if self.check(&Token::Lt) {
            // Parse type parameters first
            self.advance(); // consume '<'
            self.parse_type_params()?
        } else {
            vec![]
        };

        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;

        let return_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        if self.check(&Token::Arrow) || self.check(&Token::Assign) {
            self.advance();
            // One-liner function (=> expr or = expr)
            let body = self.parse_expression()?;
            return Ok(TopLevel::Function(FunctionDecl {
                name,
                type_params,
                params,
                return_type,
                body: Some(BlockStmt {
                    stmts: vec![Stmt::Return(ReturnStmt {
                        expr: Some(body.clone()),
                    })],
                }),
                expr_body: Some(body),
                is_async_inferred: false,
            }));
        }

        // Block function
        self.expect(Token::LBrace)?;
        let body = self.parse_block_stmt()?;
        self.expect(Token::RBrace)?;

        Ok(TopLevel::Function(FunctionDecl {
            name,
            type_params,
            params,
            return_type,
            body: Some(body),
            expr_body: None,
            is_async_inferred: false,
        }))
    }

    fn parse_type_params(&mut self) -> Result<Vec<String>> {
        let mut type_params = Vec::new();

        while !self.is_at_end() && !self.check(&Token::Gt) {
            let param_name = self.parse_identifier()?;
            type_params.push(param_name);

            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.expect(Token::Gt)?;

        Ok(type_params)
    }

    fn parse_members(&mut self) -> Result<Vec<Member>> {
        let mut members = Vec::new();

        while !self.is_at_end() && !self.check(&Token::RBrace) {
            let name = self.parse_identifier()?;
            let visibility = Visibility::from_name(&name);

            // Check if it's a method (has parentheses or type parameters)
            if self.peek() == Some(&Token::Lt) || self.peek() == Some(&Token::LParen) {
                let type_params = if self.check(&Token::Lt) {
                    // Parse type parameters first
                    self.advance(); // consume '<'
                    self.parse_type_params()?
                } else {
                    vec![]
                };

                self.expect(Token::LParen)?;
                let params = self.parse_params()?;
                self.expect(Token::RParen)?;

                let return_type = if self.match_token(&Token::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                let expr_body = if self.check(&Token::Arrow) || self.check(&Token::Assign) {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                if let Some(body) = expr_body {
                    // One-liner method (=> expr or = expr)
                    members.push(Member::Method(MethodDecl {
                        name,
                        visibility,
                        type_params,
                        params,
                        return_type,
                        body: Some(BlockStmt {
                            stmts: vec![Stmt::Return(ReturnStmt {
                                expr: Some(body.clone()),
                            })],
                        }),
                        expr_body: Some(body),
                        is_async_inferred: false,
                    }));

                    // Consume optional semicolon for one-liner methods
                    self.match_token(&Token::Semicolon);
                } else {
                    // Block method
                    self.expect(Token::LBrace)?;
                    let body = self.parse_block_stmt()?;
                    self.expect(Token::RBrace)?;
                    members.push(Member::Method(MethodDecl {
                        name,
                        visibility,
                        type_params,
                        params,
                        return_type,
                        body: Some(body),
                        expr_body: None,
                        is_async_inferred: false,
                    }));

                    // Consume optional semicolon for block methods
                    self.match_token(&Token::Semicolon);
                }
            } else {
                // It's a field
                let type_ref = if self.match_token(&Token::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                let init = if self.match_token(&Token::Assign) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                members.push(Member::Field(FieldDecl {
                    name,
                    visibility,
                    type_ref,
                    init,
                }));

                // Consume optional semicolon
                self.match_token(&Token::Semicolon);
            }
        }

        Ok(members)
    }

    fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();

        if self.check(&Token::RParen) {
            return Ok(params);
        }

        loop {
            let name = self.parse_identifier()?;
            let type_ref = if self.match_token(&Token::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };

            let default = if self.match_token(&Token::Assign) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            params.push(Param {
                name,
                type_ref,
                default,
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_type(&mut self) -> Result<TypeRef> {
        let base = match self.advance() {
            Some(Token::Ident(s)) => s.clone(),
            Some(Token::Number) => "number".to_string(),
            Some(Token::Float) => "float".to_string(),
            Some(Token::Bool) => "bool".to_string(),
            Some(Token::CharType) => "char".to_string(),
            Some(Token::String) => "string".to_string(),
            Some(Token::Bytes) => "bytes".to_string(),
            _ => return Err(self.error("Expected type".into())),
        };

        // Check for generic type parameters
        if self.check(&Token::Lt) {
            self.advance(); // consume '<'
            let mut args = Vec::new();

            loop {
                args.push(self.parse_type()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }

            self.expect(Token::Gt)?;
            Ok(TypeRef::Generic { base, args })
        } else {
            Ok(TypeRef::Simple(base))
        }
    }

    fn parse_block_stmt(&mut self) -> Result<BlockStmt> {
        let mut stmts = Vec::new();

        while !self.is_at_end() && !self.check(&Token::RBrace) {
            stmts.push(self.parse_statement()?);
        }

        Ok(BlockStmt { stmts })
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        if self.match_token(&Token::Let) {
            let name = self.parse_identifier()?;
            let type_ref = if self.match_token(&Token::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect(Token::Assign)?;
            let value = self.parse_expression()?;
            self.match_token(&Token::Semicolon); // Optional semicolon
            return Ok(Stmt::VarDecl(VarDecl {
                name,
                type_ref,
                init: Some(value),
            }));
        }

        if self.match_token(&Token::Const) {
            let name = self.parse_identifier()?;
            let type_ref = if self.match_token(&Token::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect(Token::Assign)?;
            let value = self.parse_expression()?;
            self.match_token(&Token::Semicolon);
            return Ok(Stmt::ConstDecl(ConstDecl {
                name,
                type_ref,
                init: value,
            }));
        }

        if self.match_token(&Token::Return) {
            let value = if !self.check(&Token::Semicolon) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            return Ok(Stmt::Return(ReturnStmt { expr: value }));
        }

        if self.match_token(&Token::Throw) {
            let value = self.parse_expression()?;
            return Ok(Stmt::Throw(ThrowStmt { expr: value }));
        }

        if self.match_token(&Token::Try) {
            self.expect(Token::LBrace)?;
            let try_block = self.parse_block_stmt()?;
            self.expect(Token::RBrace)?;
            self.expect(Token::Catch)?;
            self.expect(Token::LParen)?;
            let catch_var = self.parse_identifier()?;
            self.expect(Token::RParen)?;
            self.expect(Token::LBrace)?;
            let catch_block = self.parse_block_stmt()?;
            self.expect(Token::RBrace)?;
            return Ok(Stmt::TryCatch(TryCatchStmt {
                try_block,
                catch_var,
                catch_block,
            }));
        }

        if self.match_token(&Token::If) {
            // Parse condition - paréntesis opcionales
            let condition = if self.match_token(&Token::LParen) {
                let cond = self.parse_expression()?;
                self.expect(Token::RParen)?;
                cond
            } else {
                self.parse_expression()?
            };
            self.expect(Token::LBrace)?;
            let then_branch = self.parse_block_stmt()?;
            self.expect(Token::RBrace)?;

            let else_branch = if self.match_token(&Token::Else) {
                if self.check(&Token::If) {
                    // This is an else-if, parse it recursively as a nested if statement
                    let else_if_stmt = self.parse_statement()?;
                    Some(BlockStmt {
                        stmts: vec![else_if_stmt],
                    })
                } else {
                    // This is a regular else block
                    self.expect(Token::LBrace)?;
                    let else_stmt = self.parse_block_stmt()?;
                    self.expect(Token::RBrace)?;
                    Some(else_stmt)
                }
            } else {
                None
            };

            return Ok(Stmt::If(IfStmt {
                condition,
                then_branch,
                else_branch,
            }));
        }

        if self.match_token(&Token::While) {
            let condition = self.parse_expression()?;
            self.expect(Token::LBrace)?;
            let body = self.parse_block_stmt()?;
            self.expect(Token::RBrace)?;
            self.match_token(&Token::Semicolon); // Optional semicolon
            return Ok(Stmt::While(WhileStmt { condition, body }));
        }

        if self.match_token(&Token::Switch) {
            let discriminant = self.parse_expression()?;
            self.expect(Token::LBrace)?;
            let mut cases = Vec::new();
            let mut default = None;

            while !self.is_at_end() && !self.check(&Token::RBrace) {
                if self.match_token(&Token::Case) {
                    let value = self.parse_expression()?;
                    self.expect(Token::Colon)?;
                    let mut body = Vec::new();

                    // Parse statements until next case, default, or end
                    while !self.is_at_end()
                        && !self.check(&Token::Case)
                        && !self.check(&Token::Default)
                        && !self.check(&Token::RBrace)
                    {
                        body.push(self.parse_statement()?);
                    }

                    cases.push(CaseClause { value, body });
                } else if self.match_token(&Token::Default) {
                    self.expect(Token::Colon)?;
                    let mut body = Vec::new();

                    while !self.is_at_end() && !self.check(&Token::RBrace) {
                        body.push(self.parse_statement()?);
                    }

                    default = Some(body);
                } else {
                    return Err(
                        self.error("Expected 'case' or 'default' in switch statement".into())
                    );
                }
            }

            self.expect(Token::RBrace)?;
            self.match_token(&Token::Semicolon); // Optional semicolon
            return Ok(Stmt::Switch(SwitchStmt {
                discriminant,
                cases,
                default,
            }));
        }

        if self.match_token(&Token::For) {
            let var = self.parse_identifier()?;
            self.expect(Token::In)?;
            let iterable = self.parse_expression()?;
            self.expect(Token::LBrace)?;
            let body = self.parse_block_stmt()?;
            self.expect(Token::RBrace)?;
            self.match_token(&Token::Semicolon); // Optional semicolon
            return Ok(Stmt::For(ForStmt::new(var, iterable, body)));
        }

        // Expression statement
        let expr = self.parse_expression()?;
        if self.match_token(&Token::Assign) {
            if !is_valid_assignment_target(&expr) {
                return Err(self.error("Invalid assignment target".into()));
            }
            let value = self.parse_expression()?;
            self.match_token(&Token::Semicolon);
            return Ok(Stmt::Assign(AssignStmt {
                target: expr,
                value,
            }));
        }

        self.match_token(&Token::Semicolon); // Optional semicolon
        Ok(Stmt::Expr(ExprStmt { expr }))
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr> {
        let expr = self.parse_or()?;

        if self.match_token(&Token::DotDot) {
            let right = self.parse_assignment()?;
            return Ok(Expr::Binary {
                op: BinOp::Range,
                left: Box::new(expr),
                right: Box::new(right),
            });
        }

        if self.match_token(&Token::Question) {
            // Ternary operator: condition ? true_expr : false_expr
            let then_expr = self.parse_expression()?;
            self.expect(Token::Colon)?;
            let else_expr = self.parse_assignment()?; // Right associative
            return Ok(Expr::Ternary {
                condition: Box::new(expr),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            });
        }

        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_and()?;

        while self.match_token(&Token::Or) {
            let right = self.parse_and()?;
            expr = Expr::Binary {
                op: BinOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&Token::And) {
            let right = self.parse_equality()?;
            expr = Expr::Binary {
                op: BinOp::And,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;

        while self.match_token(&Token::Eq) || self.match_token(&Token::Ne) {
            let op = if self.previous() == Some(&Token::Eq) {
                BinOp::Eq
            } else {
                BinOp::Ne
            };
            let right = self.parse_comparison()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_term()?;

        while self.match_token(&Token::Gt)
            || self.match_token(&Token::Ge)
            || self.match_token(&Token::Lt)
            || self.match_token(&Token::Le)
        {
            let op = match self.previous() {
                Some(&Token::Gt) => BinOp::Gt,
                Some(&Token::Ge) => BinOp::Ge,
                Some(&Token::Lt) => BinOp::Lt,
                Some(&Token::Le) => BinOp::Le,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut expr = self.parse_factor()?;

        while self.match_token(&Token::Plus) || self.match_token(&Token::Minus) {
            let op = if self.previous() == Some(&Token::Plus) {
                BinOp::Add
            } else {
                BinOp::Sub
            };
            let right = self.parse_factor()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;

        while self.match_token(&Token::Star)
            || self.match_token(&Token::Slash)
            || self.match_token(&Token::Percent)
        {
            let op = match self.previous() {
                Some(&Token::Star) => BinOp::Mul,
                Some(&Token::Slash) => BinOp::Div,
                Some(&Token::Percent) => BinOp::Mod,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.match_token(&Token::Bang) {
            let right = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnOp::Not,
                operand: Box::new(right),
            });
        }

        if self.match_token(&Token::Minus) {
            let right = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnOp::Neg,
                operand: Box::new(right),
            });
        }

        if self.match_token(&Token::Await) {
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnOp::Await,
                operand: Box::new(operand),
            });
        }

        if self.match_token(&Token::Async) {
            // Parse the expression that should be made async
            let expr = self.parse_call()?;
            match expr {
                Expr::Call { callee, args } => {
                    return Ok(Expr::AsyncCall { callee, args });
                }
                _ => {
                    return Err(self.error("Expected function call after 'async'".into()));
                }
            }
        }

        if self.match_token(&Token::Parallel) {
            // Parse the expression that should be made parallel
            let expr = self.parse_call()?;
            match expr {
                Expr::Call { callee, args } => {
                    return Ok(Expr::ParallelCall { callee, args });
                }
                _ => {
                    return Err(self.error("Expected function call after 'parallel'".into()));
                }
            }
        }

        if self.match_token(&Token::Task) {
            // Parse task mode (async or parallel)
            let mode = if self.match_token(&Token::Async) {
                crate::ast::ConcurrencyMode::Async
            } else if self.match_token(&Token::Parallel) {
                crate::ast::ConcurrencyMode::Parallel
            } else {
                return Err(self.error("Expected 'async' or 'parallel' after 'task'".into()));
            };

            // Parse the expression
            let expr = self.parse_call()?;
            match expr {
                Expr::Call { callee, args } => {
                    // For now, we need to extract the function name from the callee expression
                    // This is a temporary solution - TaskCall should be updated to use Box<Expr> like AsyncCall/ParallelCall
                    match *callee {
                        Expr::Identifier(name) => {
                            return Ok(Expr::TaskCall {
                                mode,
                                callee: name,
                                args,
                            });
                        }
                        _ => {
                            return Err(self.error(
                                "Task calls currently only support simple function names".into(),
                            ));
                        }
                    }
                }
                _ => {
                    return Err(
                        self.error("Expected function call after 'task async/parallel'".into())
                    );
                }
            }
        }

        if self.match_token(&Token::Fire) {
            // Parse fire mode (async or parallel)
            let mode = if self.match_token(&Token::Async) {
                crate::ast::ConcurrencyMode::Async
            } else if self.match_token(&Token::Parallel) {
                crate::ast::ConcurrencyMode::Parallel
            } else {
                return Err(self.error("Expected 'async' or 'parallel' after 'fire'".into()));
            };

            // Parse the expression
            let expr = self.parse_call()?;
            match expr {
                Expr::Call { callee, args } => {
                    // For now, we need to extract the function name from the callee expression
                    // This is a temporary solution - FireCall should be updated to use Box<Expr> like AsyncCall/ParallelCall
                    match *callee {
                        Expr::Identifier(name) => {
                            return Ok(Expr::FireCall {
                                mode,
                                callee: name,
                                args,
                            });
                        }
                        _ => {
                            return Err(self.error(
                                "Fire calls currently only support simple function names".into(),
                            ));
                        }
                    }
                }
                _ => {
                    return Err(
                        self.error("Expected function call after 'fire async/parallel'".into())
                    );
                }
            }
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&Token::LParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&Token::Dot) {
                let name = self.parse_identifier()?;
                expr = Expr::Member {
                    object: Box::new(expr),
                    property: name,
                };
            } else if self.match_token(&Token::LBracket) {
                let index = self.parse_expression()?;
                self.expect(Token::RBracket)?;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();

        if !self.check(&Token::RParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        Ok(args)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let args = self.parse_args()?;
        self.expect(Token::RParen)?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
        })
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        if self.match_token(&Token::False) {
            return Ok(Expr::Literal(Literal::Bool(false)));
        }
        if self.match_token(&Token::True) {
            return Ok(Expr::Literal(Literal::Bool(true)));
        }

        if let Some(token) = self.peek() {
            match token {
                Token::IntLiteral(n) => {
                    let value = *n;
                    self.advance();
                    return Ok(Expr::Literal(Literal::Int(value)));
                }
                Token::FloatLiteral(f) => {
                    let value = *f;
                    self.advance();
                    return Ok(Expr::Literal(Literal::Float(value)));
                }
                Token::StringLiteral(s) => {
                    let value = s.clone();
                    self.advance();
                    return Ok(Expr::Literal(Literal::String(value)));
                }
                Token::CharLiteral(c) => {
                    let value = *c;
                    self.advance();
                    return Ok(Expr::Literal(Literal::Char(value)));
                }
                Token::StringTemplate(s) => {
                    let template = s.clone();
                    self.advance();
                    let parts = parse_string_template_parts(&template)?;
                    return Ok(Expr::StringTemplate { parts });
                }
                Token::Ident(name) => {
                    let value = name.clone();
                    self.advance();
                    return Ok(Expr::Identifier(value));
                }
                _ => {}
            }
        }

        if self.match_token(&Token::LParen) {
            let expr = self.parse_expression()?;
            self.expect(Token::RParen)?;
            return Ok(expr); // Just return the expression without wrapping
        }

        if self.match_token(&Token::LBrace) {
            return self.parse_object_literal();
        }

        if self.match_token(&Token::LBracket) {
            return self.parse_array_literal();
        }

        Err(self.error("Expected expression".into()))
    }

    fn parse_expression_root(&mut self) -> Result<Expr> {
        let expr = self.parse_expression()?;
        if !self.is_at_end() {
            return Err(self.error("Unexpected tokens after expression".into()));
        }
        Ok(expr)
    }

    fn parse_array_literal(&mut self) -> Result<Expr> {
        let mut elements = Vec::new();

        if !self.check(&Token::RBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.expect(Token::RBracket)?;
        Ok(Expr::ArrayLiteral(elements))
    }

    fn parse_object_literal(&mut self) -> Result<Expr> {
        let mut fields = Vec::new();

        while !self.is_at_end() && self.peek() != Some(&Token::RBrace) {
            let key = self.parse_identifier()?;
            self.expect(Token::Colon)?;
            let value = self.parse_expression()?;
            fields.push((key, value));

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.expect(Token::RBrace)?;
        Ok(Expr::ObjectLiteral(fields))
    }

    fn parse_identifier(&mut self) -> Result<String> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(s.clone()),
            Some(Token::ProtectedIdent(s)) => Ok(s.clone()),
            Some(Token::PrivateIdent(s)) => Ok(s.clone()),
            _ => Err(self.error("Expected identifier".into())),
        }
    }

    fn parse_string_literal(&mut self) -> Result<String> {
        match self.advance() {
            Some(Token::StringLiteral(s)) => Ok(s.clone()),
            _ => Err(self.error("Expected string literal".into())),
        }
    }
}

pub fn parse(tokens: Vec<TokenWithSpan>, source: &str) -> Result<Program> {
    let mut parser = Parser::new(tokens, source.to_string());
    parser.parse_program()
}

fn is_valid_assignment_target(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Identifier(_) | Expr::Member { .. } | Expr::Index { .. }
    )
}

fn parse_string_template_parts(raw: &str) -> Result<Vec<StringTemplatePart>> {
    let mut parts = Vec::new();
    let mut buffer = String::new();
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                if let Some(escaped) = chars.next() {
                    match escaped {
                        'n' => buffer.push('\n'),
                        'r' => buffer.push('\r'),
                        't' => buffer.push('\t'),
                        '\\' => buffer.push('\\'),
                        '"' => buffer.push('"'),
                        '{' => buffer.push('{'),
                        '}' => buffer.push('}'),
                        other => buffer.push(other),
                    }
                } else {
                    buffer.push('\\');
                }
            }
            '{' => {
                if let Some('{') = chars.peek() {
                    chars.next();
                    buffer.push('{');
                    continue;
                }
                if !buffer.is_empty() {
                    parts.push(StringTemplatePart::Text(buffer.clone()));
                    buffer.clear();
                }
                let mut depth = 1usize;
                let mut expr_src = String::new();
                while let Some(next) = chars.next() {
                    match next {
                        '{' => {
                            depth += 1;
                            expr_src.push(next);
                        }
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            } else {
                                expr_src.push('}');
                            }
                        }
                        _ => expr_src.push(next),
                    }
                }
                if depth != 0 {
                    return Err(CompilerError::ParseError {
                        line: 1,
                        col: 1,
                        msg: "Unclosed interpolation in string template".into(),
                    });
                }
                let expr_src_trimmed = expr_src.trim();
                if expr_src_trimmed.is_empty() {
                    return Err(CompilerError::ParseError {
                        line: 1,
                        col: 1,
                        msg: "Empty interpolation in string template".into(),
                    });
                }
                match parse_template_expression(expr_src_trimmed) {
                    Ok(expr) => parts.push(StringTemplatePart::Expr(Box::new(expr))),
                    Err(_) => {
                        let normalized = expr_src_trimmed.replace('\'', "\"");
                        if normalized != expr_src_trimmed {
                            if let Ok(expr) = parse_template_expression(&normalized) {
                                parts.push(StringTemplatePart::Expr(Box::new(expr)));
                                continue;
                            }
                        }
                        parts.push(StringTemplatePart::Text(format!(
                            "{{{}}}",
                            expr_src_trimmed
                        )));
                    }
                }
            }
            '}' => {
                if let Some('}') = chars.peek() {
                    chars.next();
                    buffer.push('}');
                } else {
                    return Err(CompilerError::ParseError {
                        line: 1,
                        col: 1,
                        msg: "Unmatched '}' in string template".into(),
                    });
                }
            }
            _ => buffer.push(ch),
        }
    }

    if !buffer.is_empty() {
        parts.push(StringTemplatePart::Text(buffer));
    }

    Ok(parts)
}

fn parse_template_expression(fragment: &str) -> Result<Expr> {
    let tokens = tokenize(fragment)?;
    let mut parser = Parser::new(tokens, fragment.to_string());
    parser.parse_expression_root()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_function() {
        let source = "sum(a: number, b: number): number => a + b";
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens, source).unwrap();

        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_class() {
        let source = r#"
            Persona {
                nombre: string
                _edad: number
                __dni: string
            }
        "#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens, source).unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            TopLevel::Class(c) => {
                assert_eq!(c.name, "Persona");
                assert_eq!(c.members.len(), 3);
            }
            _ => panic!("Expected class"),
        }
    }

    #[test]
    fn test_parse_async_call() {
        let source = r#"
            main() {
                let x = async fetchUser()
            }
        "#;
        let tokens = tokenize(source).unwrap();
        let program = parse(tokens, source).unwrap();

        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_string_template_with_complex_expression() {
        let parts =
            super::parse_string_template_parts("First user: {users[0].name}\\n").unwrap();
        assert_eq!(parts.len(), 3);
        match &parts[1] {
            StringTemplatePart::Expr(_) => {}
            other => panic!("expected expression part, got {:?}", other),
        }
        match &parts[2] {
            StringTemplatePart::Text(text) => assert_eq!(text, "\n"),
            other => panic!("expected trailing newline text, got {:?}", other),
        }
    }
}
