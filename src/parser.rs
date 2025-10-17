use crate::ast::*;
use crate::error::{CompilerError, Result};
use crate::lexer::{tokenize, Token, TokenWithSpan};
use crate::span::SourceMap;

pub struct Parser {
    tokens: Vec<TokenWithSpan>,
    current: usize,
    _source: String,
    source_map: SourceMap,
}

impl Parser {
    fn new(tokens: Vec<TokenWithSpan>, source: String) -> Self {
        let source_map = SourceMap::new(&source);
        Self {
            tokens,
            current: 0,
            _source: source,
            source_map,
        }
    }

    fn peek(&self) -> Option<&Token> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.current].token)
        }
    }

    fn peek_token(&self, offset: usize) -> Option<&Token> {
        self.tokens
            .get(self.current + offset)
            .map(|token| &token.token)
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

    /// Get the span of the current token
    fn current_span(&self) -> Option<crate::span::Span> {
        if self.current < self.tokens.len() {
            Some(self.tokens[self.current].span)
        } else {
            None
        }
    }

    /// Get the span of the previous token
    fn previous_span(&self) -> Option<crate::span::Span> {
        if self.current > 0 && self.current - 1 < self.tokens.len() {
            Some(self.tokens[self.current - 1].span)
        } else {
            None
        }
    }

    fn is_lambda_start_from(&self, offset: usize) -> bool {
        match self.peek_token(offset) {
            Some(Token::LParen) => {
                let mut depth = 0usize;
                let mut idx = offset;
                while let Some(tok) = self.peek_token(idx) {
                    match tok {
                        Token::LParen => depth += 1,
                        Token::RParen => {
                            if depth == 0 {
                                return false;
                            }
                            depth -= 1;
                            if depth == 0 {
                                idx += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    idx += 1;
                }

                if depth != 0 {
                    return false;
                }

                let mut idx_after = idx;
                if matches!(self.peek_token(idx_after), Some(Token::Colon)) {
                    idx_after += 1;
                    loop {
                        match self.peek_token(idx_after) {
                            Some(Token::Arrow) => return true,
                            Some(Token::Comma)
                            | Some(Token::RParen)
                            | Some(Token::Assign)
                            | Some(Token::Semicolon)
                            | Some(Token::LBrace)
                            | Some(Token::RBrace)
                            | None => break,
                            _ => idx_after += 1,
                        }
                    }
                    matches!(self.peek_token(idx_after), Some(Token::Arrow))
                } else {
                    matches!(self.peek_token(idx_after), Some(Token::Arrow))
                }
            }
            Some(Token::Ident(_))
            | Some(Token::ProtectedIdent(_))
            | Some(Token::PrivateIdent(_)) => {
                matches!(self.peek_token(offset + 1), Some(Token::Arrow))
            }
            _ => false,
        }
    }

    fn is_lambda_start(&self) -> bool {
        self.is_lambda_start_from(0)
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

        self.tokens[token_index].line_col(&self.source_map)
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
        if let Some(token) = self.peek() {
            if Self::is_exec_modifier(token) {
                return Err(self.error(format!(
                    "Modifier '{}' cannot be applied to declarations",
                    Self::modifier_name(token)
                )));
            }
        }
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
            let body_opt = Some(body);
            let contains_fail = self.function_body_contains_fail(&None, &body_opt);
            return Ok(TopLevel::Function(FunctionDecl {
                name,
                type_params,
                params,
                return_type,
                body: None,
                expr_body: body_opt,
                is_async_inferred: false,
                contains_fail,
            }));
        }

        // Block function
        self.expect(Token::LBrace)?;
        let body = self.parse_block_stmt()?;
        self.expect(Token::RBrace)?;

        Ok(TopLevel::Function(FunctionDecl {
            name: name.clone(),
            type_params,
            params,
            return_type,
            body: Some(body.clone()),
            expr_body: None,
            is_async_inferred: false,
            contains_fail: self.function_body_contains_fail(&Some(body), &None),
        }))
    }

    fn function_body_contains_fail(&self, body: &Option<BlockStmt>, expr_body: &Option<Expr>) -> bool {
        if let Some(block) = body {
            self.block_contains_fail(block)
        } else if let Some(expr) = expr_body {
            self.expr_contains_fail(expr)
        } else {
            false
        }
    }

    fn block_contains_fail(&self, block: &BlockStmt) -> bool {
        for stmt in &block.stmts {
            if self.stmt_contains_fail(stmt) {
                return true;
            }
        }
        false
    }

    fn stmt_contains_fail(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Fail(_) => true,
            Stmt::VarDecl(var) => self.expr_contains_fail(&var.init),
            Stmt::Assign(assign) => self.expr_contains_fail(&assign.value),
            Stmt::Return(ret) => ret.expr.as_ref().map_or(false, |e| self.expr_contains_fail(e)),
            Stmt::If(if_stmt) => {
                self.expr_contains_fail(&if_stmt.condition) ||
                self.if_body_contains_fail(&if_stmt.then_branch) ||
                if_stmt.else_branch.as_ref().map_or(false, |b| self.if_body_contains_fail(b))
            }
            Stmt::While(while_stmt) => {
                self.expr_contains_fail(&while_stmt.condition) ||
                self.block_contains_fail(&while_stmt.body)
            }
            Stmt::For(for_stmt) => self.block_contains_fail(&for_stmt.body),
            Stmt::Switch(switch) => {
                self.expr_contains_fail(&switch.discriminant) ||
                switch.cases.iter().any(|case| case.body.iter().any(|s| self.stmt_contains_fail(s))) ||
                switch.default.as_ref().map_or(false, |b| b.iter().any(|s| self.stmt_contains_fail(s)))
            }
            Stmt::TryCatch(try_catch) => {
                self.block_contains_fail(&try_catch.try_block) ||
                self.block_contains_fail(&try_catch.catch_block)
            }
            Stmt::Throw(throw) => self.expr_contains_fail(&throw.expr),
            Stmt::Expr(expr_stmt) => self.expr_contains_fail(&expr_stmt.expr),
            _ => false,
        }
    }

    fn if_body_contains_fail(&self, body: &IfBody) -> bool {
        match body {
            IfBody::Block(block) => self.block_contains_fail(block),
            IfBody::Stmt(stmt) => self.stmt_contains_fail(stmt),
        }
    }

    fn expr_contains_fail(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Fail(_) => true,
            Expr::Ternary { condition, then_expr, else_expr } => {
                self.expr_contains_fail(condition) ||
                self.expr_contains_fail(then_expr) ||
                self.expr_contains_fail(else_expr)
            }
            Expr::Binary { left, right, .. } => {
                self.expr_contains_fail(left) || self.expr_contains_fail(right)
            }
            Expr::Unary { operand, .. } => self.expr_contains_fail(operand),
            Expr::Call(call) => {
                self.expr_contains_fail(&call.callee) ||
                call.args.iter().any(|arg| self.expr_contains_fail(arg))
            }
            Expr::Member { object, .. } => self.expr_contains_fail(object),
            Expr::Index { object, index } => {
                self.expr_contains_fail(object) || self.expr_contains_fail(index)
            }
            Expr::ObjectLiteral(fields) => {
                fields.iter().any(|(_, value)| self.expr_contains_fail(value))
            }
            Expr::StructLiteral { fields, .. } => {
                fields.iter().any(|(_, value)| self.expr_contains_fail(value))
            }
            Expr::ArrayLiteral(elements) => {
                elements.iter().any(|elem| self.expr_contains_fail(elem))
            }
            Expr::Lambda(lambda) => match &lambda.body {
                LambdaBody::Expr(body) => self.expr_contains_fail(body),
                LambdaBody::Block(block) => self.block_contains_fail(block),
            },
            Expr::StringTemplate { parts } => {
                parts.iter().any(|part| match part {
                    StringTemplatePart::Expr(expr) => self.expr_contains_fail(expr),
                    _ => false,
                })
            }
            _ => false,
        }
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
            if let Some(token) = self.peek() {
                if Self::is_exec_modifier(token) {
                    return Err(self.error(format!(
                        "Modifier '{}' cannot be applied to class members",
                        Self::modifier_name(token)
                    )));
                }
            }
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
                        expr_body: Some(body.clone()),
                        is_async_inferred: false,
                        contains_fail: self.function_body_contains_fail(&None, &Some(body)),
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
                        body: Some(body.clone()),
                        expr_body: None,
                        is_async_inferred: false,
                        contains_fail: self.function_body_contains_fail(&Some(body), &None),
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

    fn parse_let_bindings(&mut self) -> Result<Vec<VarBinding>> {
        let mut bindings = Vec::new();

        // Parse first binding
        let name = self.parse_identifier()?;
        let span = self.previous_span();
        let type_ref = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        bindings.push(VarBinding { name, type_ref, span });

        // Parse additional bindings if present (for fallible binding)
        while self.match_token(&Token::Comma) {
            let name = self.parse_identifier()?;
            let span = self.previous_span();
            let type_ref = if self.match_token(&Token::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            bindings.push(VarBinding { name, type_ref, span });
        }

        Ok(bindings)
    }

    fn parse_simple_statement(&mut self) -> Result<Stmt> {
        if self.match_token(&Token::Return) {
            let value = if !self.check(&Token::Semicolon) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            Ok(Stmt::Return(ReturnStmt { expr: value }))
        } else if self.match_token(&Token::Fail) {
            let value = self.parse_expression()?;
            Ok(Stmt::Fail(FailStmt { expr: value }))
        } else if self.match_token(&Token::Throw) {
            let value = self.parse_expression()?;
            Ok(Stmt::Throw(ThrowStmt { expr: value }))
        } else {
            // Parse assignment statement: target = value
            let target = self.parse_expression()?;
            if self.match_token(&Token::Assign) {
                let value = self.parse_expression()?;
                Ok(Stmt::Assign(AssignStmt { target, value }))
            } else {
                // Expression statement
                Ok(Stmt::Expr(ExprStmt { expr: target }))
            }
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        if self.match_token(&Token::Let) {
            let bindings = self.parse_let_bindings()?;
            self.expect(Token::Assign)?;
            let init = self.parse_expression()?;
            self.match_token(&Token::Semicolon); // Optional semicolon

            let is_fallible = bindings.len() > 1;

            return Ok(Stmt::VarDecl(VarDecl {
                bindings,
                init,
                is_fallible,
            }));
        }

        if self.match_token(&Token::Const) {
            let name = self.parse_identifier()?;
            let span = self.previous_span();
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
                span,
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

        if self.match_token(&Token::Fail) {
            let value = self.parse_expression()?;
            return Ok(Stmt::Fail(FailStmt { expr: value }));
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

            // Check if it's a simple statement (like if cond fail "msg")
            let then_branch = if self.check(&Token::LBrace) {
                self.expect(Token::LBrace)?;
                let block = self.parse_block_stmt()?;
                self.expect(Token::RBrace)?;
                IfBody::Block(block)
            } else {
                // Parse a simple statement
                let stmt = self.parse_simple_statement()?;
                IfBody::Stmt(Box::new(stmt))
            };

            let else_branch = if self.match_token(&Token::Else) {
                if self.check(&Token::If) {
                    // This is an else-if, parse it recursively as a nested if statement
                    let else_if_stmt = self.parse_statement()?;
                    Some(IfBody::Stmt(Box::new(else_if_stmt)))
                } else if self.check(&Token::LBrace) {
                    // This is a regular else block
                    self.expect(Token::LBrace)?;
                    let block = self.parse_block_stmt()?;
                    self.expect(Token::RBrace)?;
                    Some(IfBody::Block(block))
                } else {
                    // This is a simple else statement
                    let stmt = self.parse_simple_statement()?;
                    Some(IfBody::Stmt(Box::new(stmt)))
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
            let mut policy = DataParallelPolicy::Seq;
            if self.match_token(&Token::Seq) {
                policy = DataParallelPolicy::Seq;
            } else if self.match_token(&Token::Par) {
                policy = DataParallelPolicy::Par;
            } else if self.match_token(&Token::Vec) {
                policy = DataParallelPolicy::Vec;
            } else if self.match_token(&Token::Boost) {
                policy = DataParallelPolicy::Boost;
            }

            let var = self.parse_identifier()?;
            self.expect(Token::In)?;
            let iterable = self.parse_expression()?;

            let options = if self.match_token(&Token::With) {
                self.parse_for_options()?
            } else {
                ForPolicyOptions::default()
            };

            self.expect(Token::LBrace)?;
            let body = self.parse_block_stmt()?;
            self.expect(Token::RBrace)?;
            self.match_token(&Token::Semicolon); // Optional semicolon

            let mut stmt = ForStmt::new(var, iterable, body);
            stmt.policy = policy;
            stmt.options = options;

            return Ok(Stmt::For(stmt));
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
        self.parse_lambda_expression()
    }

    fn parse_lambda_expression(&mut self) -> Result<Expr> {
        if self.match_token(&Token::Move) {
            if !self.is_lambda_start() {
                return Err(self.error("Expected lambda parameters after 'move'".into()));
            }
            return self.parse_lambda(true);
        }

        if self.is_lambda_start() {
            return self.parse_lambda(false);
        }

        self.parse_assignment()
    }

    fn parse_lambda(&mut self, is_move: bool) -> Result<Expr> {
        let params = if self.match_token(&Token::LParen) {
            let params = self.parse_lambda_param_list()?;
            self.expect(Token::RParen)?;
            params
        } else {
            let name = self.parse_identifier()?;
            vec![LambdaParam {
                name,
                type_ref: None,
            }]
        };

        let return_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(Token::Arrow)?;
        let body = self.parse_lambda_body()?;

        Ok(Expr::Lambda(LambdaExpr {
            is_move,
            params,
            return_type,
            body,
            captures: Vec::new(),
        }))
    }

    fn parse_lambda_param_list(&mut self) -> Result<Vec<LambdaParam>> {
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

            params.push(LambdaParam { name, type_ref });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_lambda_body(&mut self) -> Result<LambdaBody> {
        if self.match_token(&Token::LBrace) {
            let block = self.parse_block_stmt()?;
            self.expect(Token::RBrace)?;
            Ok(LambdaBody::Block(block))
        } else {
            let expr = self.parse_expression()?;
            Ok(LambdaBody::Expr(Box::new(expr)))
        }
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
            return self.parse_exec_call(ExecPolicy::Async, "async");
        }

        if self.match_token(&Token::Par) {
            return self.parse_exec_call(ExecPolicy::Par, "par");
        }

        if self.match_token(&Token::Task) {
            let policy = if self.match_token(&Token::Async) {
                ExecPolicy::TaskAsync
            } else if self.match_token(&Token::Par) {
                ExecPolicy::TaskPar
            } else {
                return Err(self.error("Expected 'async' or 'par' after 'task'".into()));
            };
            return self.parse_exec_call(policy, "task");
        }

        if self.match_token(&Token::Fire) {
            let policy = if self.match_token(&Token::Async) {
                ExecPolicy::FireAsync
            } else if self.match_token(&Token::Par) {
                ExecPolicy::FirePar
            } else {
                return Err(self.error("Expected 'async' or 'par' after 'fire'".into()));
            };
            return self.parse_exec_call(policy, "fire");
        }

        self.parse_call()
    }

    fn parse_exec_call(&mut self, policy: ExecPolicy, modifier: &str) -> Result<Expr> {
        let expr = self.parse_call()?;
        match expr {
            Expr::Call(mut call) => {
                if !matches!(call.exec_policy, ExecPolicy::Normal) {
                    return Err(self.error("Execution policy already applied to call".into()));
                }
                call.exec_policy = policy;
                Ok(Expr::Call(call))
            }
            _ => Err(self.error(format!("Expected function call after '{}'", modifier))),
        }
    }

    fn parse_for_options(&mut self) -> Result<ForPolicyOptions> {
        let mut options = ForPolicyOptions::default();

        loop {
            if self.check(&Token::LBrace) {
                break;
            }

            if self.match_token(&Token::Ordered) {
                options.ordered = true;
                continue;
            }

            if self.match_token(&Token::Chunk) {
                let value = self.parse_option_int("chunk")?;
                options.chunk = Some(value);
                continue;
            }

            if self.match_token(&Token::Threads) {
                if self.match_token(&Token::Auto) {
                    options.threads = Some(ThreadOption::Auto);
                } else {
                    let value = self.parse_option_int("threads")?;
                    options.threads = Some(ThreadOption::Count(value));
                }
                continue;
            }

            if self.match_token(&Token::SimdWidth) {
                if self.match_token(&Token::Auto) {
                    options.simd_width = Some(SimdWidthOption::Auto);
                } else {
                    let value = self.parse_option_int("simdWidth")?;
                    options.simd_width = Some(SimdWidthOption::Width(value));
                }
                continue;
            }

            if self.match_token(&Token::Prefetch) {
                let value = self.parse_option_int("prefetch")?;
                options.prefetch = Some(value);
                continue;
            }

            if self.match_token(&Token::Reduction) {
                if self.match_token(&Token::Safe) {
                    options.reduction = Some(ReductionOption::Safe);
                } else if self.match_token(&Token::Fast) {
                    options.reduction = Some(ReductionOption::Fast);
                } else {
                    return Err(self.error("Expected 'safe' or 'fast' after 'reduction'".into()));
                }
                continue;
            }

            if self.match_token(&Token::Schedule) {
                if self.match_token(&Token::Static) {
                    options.schedule = Some(ScheduleOption::Static);
                } else if self.match_token(&Token::Dynamic) {
                    options.schedule = Some(ScheduleOption::Dynamic);
                } else {
                    return Err(
                        self.error("Expected 'static' or 'dynamic' after 'schedule'".into())
                    );
                }
                continue;
            }

            if self.match_token(&Token::Detect) {
                if self.match_token(&Token::Auto) {
                    options.detect = Some(DetectOption::Auto);
                } else {
                    return Err(self.error("Expected 'auto' after 'detect'".into()));
                }
                continue;
            }

            return Err(self.error("Unknown for-option in 'with' clause".into()));
        }

        Ok(options)
    }

    fn parse_option_int(&mut self, option_name: &str) -> Result<i64> {
        match self.peek() {
            Some(Token::IntLiteral(value)) => {
                let result = *value;
                self.advance();
                Ok(result)
            }
            _ => Err(self.error(format!("Expected integer literal after '{}'", option_name))),
        }
    }

    fn parse_call(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&Token::LParen) {
                expr = self.finish_call(expr)?;
            } else if self.check(&Token::LBrace) {
                // Check if this is a struct literal like TypeName { field: value }
                if let Expr::Identifier(type_name) = &expr {
                    // Only allow struct literals for identifiers that start with uppercase (type names)
                    if type_name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        self.advance(); // consume the {
                        let fields = self.parse_object_fields()?;
                        self.expect(Token::RBrace)?;
                        expr = Expr::StructLiteral {
                            type_name: type_name.clone(),
                            fields,
                        };
                    } else {
                        // Not a struct literal, don't consume the { and continue
                        break;
                    }
                } else {
                    // Not an identifier, don't consume the { and continue
                    break;
                }
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
        Ok(Expr::Call(CallExpr::new(callee, args)))
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

        if self.match_token(&Token::Fail) {
            let expr = self.parse_expression()?;
            return Ok(Expr::Fail(Box::new(expr)));
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

    fn parse_object_fields(&mut self) -> Result<Vec<(String, Expr)>> {
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

        Ok(fields)
    }

    fn parse_object_literal(&mut self) -> Result<Expr> {
        let fields = self.parse_object_fields()?;
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

impl Parser {
    fn is_exec_modifier(token: &Token) -> bool {
        matches!(token, Token::Async | Token::Par | Token::Task | Token::Fire)
    }

    fn modifier_name(token: &Token) -> &'static str {
        match token {
            Token::Async => "async",
            Token::Par => "par",
            Token::Task => "task",
            Token::Fire => "fire",
            _ => "unknown",
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
        let parts = super::parse_string_template_parts("First user: {users[0].name}\\n").unwrap();
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
