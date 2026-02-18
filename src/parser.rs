use crate::ast::*;
use crate::error::{CompilerError, Result, SemanticErrorInfo};
use crate::lexer::{tokenize, Token, TokenWithSpan};
use crate::span::SourceMap;

pub struct Parser {
    tokens: Vec<TokenWithSpan>,
    current: usize,
    source: String,
    source_map: SourceMap,
}

impl Parser {
    fn new(tokens: Vec<TokenWithSpan>, source: String) -> Self {
        let source_map = SourceMap::new(&source);
        Self {
            tokens,
            current: 0,
            source,
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

    /// Check if the token AFTER the current one matches (lookahead 1)
    fn peek_next_is(&self, token: &Token) -> bool {
        match self.peek_token(1) {
            Some(t) => std::mem::discriminant(t) == std::mem::discriminant(token),
            None => false,
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
    #[allow(dead_code)]
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
            Some(Token::Ident(_)) | Some(Token::PrivateIdent(_)) => {
                matches!(self.peek_token(offset + 1), Some(Token::Arrow))
            }
            Some(Token::LBrace) => {
                // Object destructuring: {x, y} =>
                // Scan forward to find closing brace and check for arrow
                let mut depth = 0usize;
                let mut idx = offset;
                while let Some(tok) = self.peek_token(idx) {
                    match tok {
                        Token::LBrace => depth += 1,
                        Token::RBrace => {
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
                matches!(self.peek_token(idx), Some(Token::Arrow))
            }
            Some(Token::LBracket) => {
                // Array destructuring: [x, y] =>
                // Scan forward to find closing bracket and check for arrow
                let mut depth = 0usize;
                let mut idx = offset;
                while let Some(tok) = self.peek_token(idx) {
                    match tok {
                        Token::LBracket => depth += 1,
                        Token::RBracket => {
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
                matches!(self.peek_token(idx), Some(Token::Arrow))
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
        self.error_with_help(message, None)
    }

    fn error_with_help(&self, message: String, help: Option<String>) -> CompilerError {
        let token_index = if self.current < self.tokens.len() {
            self.current
        } else if !self.tokens.is_empty() {
            self.tokens.len() - 1
        } else {
            return CompilerError::ParseError(
                SemanticErrorInfo::new("E2000", "Parse Error", &message)
                    .with_location("<input>", 1),
            );
        };

        let (line, col) = self.calculate_line_col(token_index);
        let source_line = self
            .source
            .lines()
            .nth(line.saturating_sub(1))
            .unwrap_or("")
            .to_string();

        // Get context lines (2 before and 2 after)
        let lines: Vec<&str> = self.source.lines().collect();
        let total_lines = lines.len();

        let start_before = line.saturating_sub(3);
        let end_before = line.saturating_sub(1);
        let context_before: Vec<String> = if start_before < end_before && end_before <= total_lines
        {
            lines[start_before..end_before]
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        let start_after = line;
        let end_after = (line + 2).min(total_lines);
        let context_after: Vec<String> = if start_after < end_after {
            lines[start_after..end_after]
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        // Get token length from span
        let token_length = if token_index < self.tokens.len() {
            self.tokens[token_index].span.len()
        } else {
            3 // default length
        };

        let mut error = SemanticErrorInfo::new("E2000", "Parse Error", &message)
            .with_location("<input>", line)
            .with_column(col)
            .with_source_line(source_line)
            .with_length(token_length)
            .with_context(context_before, context_after);

        if let Some(help_text) = help {
            error = error.with_help(&help_text);
        }

        CompilerError::ParseError(error)
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
            return self.parse_import_decl();
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

            // Check for type parameters: type Name<T> or type Name<T, U>
            let type_params = if self.check(&Token::Lt) {
                self.advance(); // consume '<'
                self.parse_type_parameters()?
            } else {
                vec![]
            };

            // Check if it's a type alias (=) or interface ({)
            if self.match_token(&Token::Assign) {
                // Type alias: type Point = (int, int)
                let target_type = self.parse_type()?;
                let span = None; // TODO: capture span
                return Ok(TopLevel::TypeAlias(TypeAliasDecl {
                    name,
                    type_params,
                    target_type,
                    span,
                }));
            } else {
                // Interface: type Name { ... }
                self.expect(Token::LBrace)?;
                let members = self.parse_members()?;
                self.expect(Token::RBrace)?;
                return Ok(TopLevel::Type(TypeDecl {
                    name,
                    type_params,
                    members,
                }));
            }
        }

        // Enum declaration: enum Color { Red, Green, Blue }
        if self.match_token(&Token::Enum) {
            return self.parse_enum_decl();
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

        // Top-level const declaration
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
            return Ok(TopLevel::ConstDecl(ConstDecl {
                name,
                type_ref,
                init: value,
                span,
            }));
        }

        // Check if we have any tokens left to parse
        if self.is_at_end() {
            return Err(self.error("Unexpected end of file".into()));
        }

        // Detect top-level expression statements (e.g., describe(...) from liva/test)
        // Heuristic: identifier followed by ( and then a string literal → function call, not declaration
        // Function declarations have: name(param: Type, ...) → params are identifiers with colons
        // Function calls have: name("string", () => { ... }) → args are expressions
        if let Some(token) = self.peek() {
            let is_potential_call = match token {
                Token::Ident(_) | Token::Test => {
                    // Check: ident ( stringLiteral
                    if let Some(Token::LParen) = self.peek_token(1) {
                        matches!(self.peek_token(2), Some(Token::StringLiteral(_)))
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if is_potential_call {
                let expr = self.parse_expression()?;
                return Ok(TopLevel::ExprStmt(expr));
            }
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
        // Check for 'data' modifier (contextual keyword - data class)
        // 'data' is not a reserved keyword, so it can be used as a variable name.
        // We detect it here by checking: current token is Ident("data") AND next token is also Ident.
        let is_data = if let Some(Token::Ident(name)) = self.peek() {
            if name == "data" && self.peek_next_is(&Token::Ident(String::new())) {
                self.advance(); // consume 'data' identifier
                true
            } else {
                false
            }
        } else {
            false
        };

        let name = self.parse_identifier()?;

        // Check for type parameters
        let type_params = if self.check(&Token::Lt) {
            self.advance(); // consume '<'
            self.parse_type_parameters()?
        } else {
            vec![]
        };

        // Check for interface implementation (: Interface1, Interface2, ...)
        let implements = if self.match_token(&Token::Colon) {
            let mut interfaces = vec![self.parse_identifier()?];
            while self.match_token(&Token::Comma) {
                interfaces.push(self.parse_identifier()?);
            }
            interfaces
        } else {
            vec![]
        };

        if self.match_token(&Token::LBrace) {
            // It's a class
            let members = self.parse_members()?;
            self.expect(Token::RBrace)?;
            return Ok(TopLevel::Class(ClassDecl {
                name,
                type_params,
                implements,
                members,
                needs_serde: false, // Will be set by semantic analyzer if used with JSON.parse
                is_data,
            }));
        }

        // Otherwise it's a function
        // type_params already parsed above

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

    /// Parse import declaration
    /// Supports:
    /// - Named imports: `import { add, multiply } from "./math.liva"`
    /// - Wildcard imports: `import * as math from "./math.liva"`
    fn parse_import_decl(&mut self) -> Result<TopLevel> {
        // Check for wildcard import: import * as alias from "path"
        if self.match_token(&Token::Star) {
            self.expect(Token::As)?;
            let alias = self.parse_identifier()?;
            self.expect(Token::From)?;
            let source = self.parse_string_literal()?;

            return Ok(TopLevel::Import(ImportDecl {
                imports: vec![],
                source,
                is_wildcard: true,
                alias: Some(alias),
            }));
        }

        // Named imports: import { name1, name2, ... } from "path"
        self.expect(Token::LBrace)?;

        let mut imports = Vec::new();

        // Parse first import
        if !self.check(&Token::RBrace) {
            imports.push(self.parse_identifier()?);

            // Parse remaining imports
            while self.match_token(&Token::Comma) {
                // Allow trailing comma
                if self.check(&Token::RBrace) {
                    break;
                }
                imports.push(self.parse_identifier()?);
            }
        }

        self.expect(Token::RBrace)?;
        self.expect(Token::From)?;
        let source = self.parse_string_literal()?;

        Ok(TopLevel::Import(ImportDecl {
            imports,
            source,
            is_wildcard: false,
            alias: None,
        }))
    }

    fn function_body_contains_fail(
        &self,
        body: &Option<BlockStmt>,
        expr_body: &Option<Expr>,
    ) -> bool {
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
            Stmt::VarDecl(var) => var.or_fail_msg.is_some() || self.expr_contains_fail(&var.init),
            Stmt::Assign(assign) => self.expr_contains_fail(&assign.value),
            Stmt::Return(ret) => ret
                .expr
                .as_ref()
                .map_or(false, |e| self.expr_contains_fail(e)),
            Stmt::If(if_stmt) => {
                self.expr_contains_fail(&if_stmt.condition)
                    || self.if_body_contains_fail(&if_stmt.then_branch)
                    || if_stmt
                        .else_branch
                        .as_ref()
                        .map_or(false, |b| self.if_body_contains_fail(b))
            }
            Stmt::While(while_stmt) => {
                self.expr_contains_fail(&while_stmt.condition)
                    || self.block_contains_fail(&while_stmt.body)
            }
            Stmt::For(for_stmt) => self.block_contains_fail(&for_stmt.body),
            Stmt::Switch(switch) => {
                self.expr_contains_fail(&switch.discriminant)
                    || switch
                        .cases
                        .iter()
                        .any(|case| case.body.iter().any(|s| self.stmt_contains_fail(s)))
                    || switch
                        .default
                        .as_ref()
                        .map_or(false, |b| b.iter().any(|s| self.stmt_contains_fail(s)))
            }
            Stmt::TryCatch(try_catch) => {
                self.block_contains_fail(&try_catch.try_block)
                    || self.block_contains_fail(&try_catch.catch_block)
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
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.expr_contains_fail(condition)
                    || self.expr_contains_fail(then_expr)
                    || self.expr_contains_fail(else_expr)
            }
            Expr::Binary { left, right, .. } => {
                self.expr_contains_fail(left) || self.expr_contains_fail(right)
            }
            Expr::Unary { operand, .. } => self.expr_contains_fail(operand),
            Expr::Call(call) => {
                self.expr_contains_fail(&call.callee)
                    || call.args.iter().any(|arg| self.expr_contains_fail(arg))
            }
            Expr::Member { object, .. } => self.expr_contains_fail(object),
            Expr::Index { object, index } => {
                self.expr_contains_fail(object) || self.expr_contains_fail(index)
            }
            Expr::ObjectLiteral(fields) => fields
                .iter()
                .any(|(_, value)| self.expr_contains_fail(value)),
            Expr::StructLiteral { fields, .. } => fields
                .iter()
                .any(|(_, value)| self.expr_contains_fail(value)),
            Expr::ArrayLiteral(elements) => {
                elements.iter().any(|elem| self.expr_contains_fail(elem))
            }
            Expr::Lambda(lambda) => match &lambda.body {
                LambdaBody::Expr(body) => self.expr_contains_fail(body),
                LambdaBody::Block(block) => self.block_contains_fail(block),
            },
            Expr::StringTemplate { parts } => parts.iter().any(|part| match part {
                StringTemplatePart::Expr(expr) => self.expr_contains_fail(expr),
                _ => false,
            }),
            _ => false,
        }
    }

    fn parse_type_parameters(&mut self) -> Result<Vec<TypeParameter>> {
        let mut type_params = Vec::new();

        while !self.is_at_end() && !self.check(&Token::Gt) {
            let param_name = self.parse_identifier()?;

            // Check for constraints: T: Add or T: Add + Sub + Mul
            let constraints = if self.match_token(&Token::Colon) {
                let mut constraint_list = Vec::new();

                // Parse first constraint
                constraint_list.push(self.parse_identifier()?);

                // Parse additional constraints with + operator
                while self.match_token(&Token::Plus) {
                    constraint_list.push(self.parse_identifier()?);
                }

                constraint_list
            } else {
                Vec::new()
            };

            type_params.push(if constraints.is_empty() {
                TypeParameter::new(param_name)
            } else {
                TypeParameter::with_constraints(param_name, constraints)
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }
        self.expect(Token::Gt)?;

        Ok(type_params)
    }

    /// Parse enum declaration: enum Color { Red, Green, Blue }
    /// or with associated data: enum Shape { Circle(radius: number), Point }
    fn parse_enum_decl(&mut self) -> Result<TopLevel> {
        let name = self.parse_identifier()?;

        // Optional type parameters: enum Option<T> { Some(value: T), None }
        let type_params = if self.check(&Token::Lt) {
            self.advance();
            self.parse_type_parameters()?
        } else {
            vec![]
        };

        self.expect(Token::LBrace)?;

        let mut variants = Vec::new();
        while !self.is_at_end() && !self.check(&Token::RBrace) {
            let variant_name = self.parse_identifier()?;

            // Check for associated data: Circle(radius: number, color: string)
            let fields = if self.match_token(&Token::LParen) {
                let mut fields = Vec::new();
                while !self.check(&Token::RParen) && !self.is_at_end() {
                    let field_name = self.parse_identifier()?;
                    self.expect(Token::Colon)?;
                    let type_ref = self.parse_type()?;
                    fields.push(EnumField {
                        name: field_name,
                        type_ref,
                    });
                    if !self.check(&Token::RParen) {
                        self.expect(Token::Comma)?;
                    }
                }
                self.expect(Token::RParen)?;
                fields
            } else {
                vec![]
            };

            variants.push(EnumVariant {
                name: variant_name,
                fields,
            });

            // Allow optional comma or newline between variants
            self.match_token(&Token::Comma);
        }

        self.expect(Token::RBrace)?;

        Ok(TopLevel::Enum(EnumDecl {
            name,
            type_params,
            variants,
        }))
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
                    self.parse_type_parameters()?
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
                } else if self.check(&Token::LBrace) {
                    // Block method with body
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
                } else {
                    // Interface method signature (no body)
                    members.push(Member::Method(MethodDecl {
                        name,
                        visibility,
                        type_params,
                        params,
                        return_type,
                        body: None,
                        expr_body: None,
                        is_async_inferred: false,
                        contains_fail: false,
                    }));

                    // Consume optional semicolon for interface method signatures
                    self.match_token(&Token::Semicolon);
                }
            } else {
                // It's a field
                // Check for optional field syntax: name?:
                let is_optional = self.match_token(&Token::Question);

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
                    is_optional,
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
            // Parse pattern WITHOUT type annotation (handled separately below)
            let pattern = self.parse_param_pattern()?;

            // Parse type annotation for the parameter
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
                pattern,
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
        // Parse the base type (which could be tuple, array, or simple type)
        let base_type = self.parse_base_type()?;

        // Check for union type: T | U | V
        if self.check(&Token::Pipe) {
            let mut types = vec![base_type];

            while self.match_token(&Token::Pipe) {
                types.push(self.parse_base_type()?);
            }

            // Flatten nested unions and remove duplicates
            let flattened = self.flatten_union_types(types);

            return Ok(TypeRef::Union(flattened));
        }

        Ok(base_type)
    }

    fn flatten_union_types(&self, types: Vec<TypeRef>) -> Vec<TypeRef> {
        let mut result = Vec::new();

        for ty in types {
            match ty {
                TypeRef::Union(inner_types) => {
                    // Recursively flatten nested unions
                    result.extend(self.flatten_union_types(inner_types));
                }
                _ => {
                    // Only add if not already in the result (remove duplicates)
                    if !result.iter().any(|t| t == &ty) {
                        result.push(ty);
                    }
                }
            }
        }

        result
    }

    fn parse_base_type(&mut self) -> Result<TypeRef> {
        // Check for tuple type syntax: (T1, T2, T3) or ()
        if self.check(&Token::LParen) {
            self.advance(); // consume '('

            // Empty tuple type: ()
            if self.match_token(&Token::RParen) {
                return Ok(TypeRef::Tuple(vec![]));
            }

            // Parse first type
            let first = self.parse_type()?;

            // Check for comma (tuple) or RParen (error - grouped types don't make sense)
            if self.match_token(&Token::Comma) {
                let mut types = vec![first];

                // Parse remaining types (allow trailing comma)
                if !self.check(&Token::RParen) {
                    loop {
                        types.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                        // Allow trailing comma before )
                        if self.check(&Token::RParen) {
                            break;
                        }
                    }
                }

                self.expect(Token::RParen)?;
                return Ok(TypeRef::Tuple(types));
            } else {
                // Error: grouped type doesn't make sense in Liva
                return Err(self.error(
                    "Unexpected type in parentheses - did you mean a tuple type like (T,)?".into(),
                ));
            }
        }

        // Check for array type syntax: [T]
        if self.check(&Token::LBracket) {
            self.advance(); // consume '['
            let inner = Box::new(self.parse_type()?);
            self.expect(Token::RBracket)?;
            return Ok(TypeRef::Array(inner));
        }

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

            // After generic, check for optional/fallible
            let mut result = TypeRef::Generic { base, args };

            // Check for ? or ! suffix
            if self.match_token(&Token::Question) {
                result = TypeRef::Optional(Box::new(result));
            } else if self.match_token(&Token::Bang) {
                result = TypeRef::Fallible(Box::new(result));
            }

            Ok(result)
        } else {
            let mut result = TypeRef::Simple(base);

            // Check for ? or ! suffix
            if self.match_token(&Token::Question) {
                result = TypeRef::Optional(Box::new(result));
            } else if self.match_token(&Token::Bang) {
                result = TypeRef::Fallible(Box::new(result));
            }

            Ok(result)
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

        // Parse first binding (can be identifier, object pattern, or array pattern)
        let binding = self.parse_binding_pattern()?;
        bindings.push(binding);

        // Parse additional bindings if present (for fallible binding: let x, err = ...)
        while self.match_token(&Token::Comma) {
            let binding = self.parse_binding_pattern()?;
            bindings.push(binding);
        }

        Ok(bindings)
    }
    /// Parse a parameter pattern WITHOUT type annotation (just the pattern itself)
    /// Used in parse_params() where type is handled separately
    fn parse_param_pattern(&mut self) -> Result<BindingPattern> {
        if self.check(&Token::LBrace) {
            // Object destructuring: {name, age}
            self.parse_object_pattern()
        } else if self.check(&Token::LBracket) {
            // Array destructuring: [first, second]
            self.parse_array_pattern()
        } else {
            // Simple identifier
            let name = self.parse_identifier()?;
            Ok(BindingPattern::Identifier(name))
        }
    }

    /// Parse a single binding pattern: identifier, {obj}, [array], or (tuple)
    fn parse_binding_pattern(&mut self) -> Result<VarBinding> {
        let start_pos = self.current;

        let pattern = if self.check(&Token::LBrace) {
            // Object destructuring: {name, age} or {name: userName}
            self.parse_object_pattern()?
        } else if self.check(&Token::LBracket) {
            // Array destructuring: [first, second] or [head, ...tail]
            self.parse_array_pattern()?
        } else if self.check(&Token::LParen) {
            // Tuple destructuring: (x, y, z)
            self.parse_tuple_pattern()?
        } else {
            // Simple identifier
            let name = self.parse_identifier()?;
            BindingPattern::Identifier(name)
        };

        // Get span from start position to current
        let span = if start_pos < self.tokens.len() {
            let start = self.tokens[start_pos].span.start;
            let end = if self.current > 0 && self.current - 1 < self.tokens.len() {
                self.tokens[self.current - 1].span.end
            } else {
                start
            };
            Some(crate::span::Span { start, end })
        } else {
            None
        };

        // Optional type annotation
        let type_ref = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        Ok(VarBinding {
            pattern,
            type_ref,
            span,
        })
    }

    /// Parse object destructuring pattern: {name, age} or {name: userName, age: userAge}
    fn parse_object_pattern(&mut self) -> Result<BindingPattern> {
        self.expect(Token::LBrace)?;

        let mut fields = Vec::new();

        // Parse first field
        if !self.check(&Token::RBrace) {
            fields.push(self.parse_object_pattern_field()?);

            // Parse remaining fields
            while self.match_token(&Token::Comma) {
                if self.check(&Token::RBrace) {
                    break; // Trailing comma
                }
                fields.push(self.parse_object_pattern_field()?);
            }
        }

        self.expect(Token::RBrace)?;

        Ok(BindingPattern::Object(ObjectPattern { fields }))
    }

    /// Parse a single field in object pattern: name or name: binding
    fn parse_object_pattern_field(&mut self) -> Result<ObjectPatternField> {
        let key = self.parse_identifier()?;

        let binding = if self.match_token(&Token::Colon) {
            // Renamed: {name: userName}
            self.parse_identifier()?
        } else {
            // Shorthand: {name} means {name: name}
            key.clone()
        };

        Ok(ObjectPatternField { key, binding })
    }

    /// Parse array destructuring pattern: [first, second] or [first, , third] or [head, ...tail]
    fn parse_array_pattern(&mut self) -> Result<BindingPattern> {
        self.expect(Token::LBracket)?;

        let mut elements = Vec::new();
        let mut rest = None;

        // Parse elements
        if !self.check(&Token::RBracket) {
            loop {
                // Check for rest pattern: ...rest
                if self.match_token(&Token::DotDotDot) {
                    let rest_name = self.parse_identifier()?;
                    rest = Some(rest_name);
                    break; // Rest pattern must be last
                }

                // Check for skip: [a, , c]
                if self.check(&Token::Comma) {
                    elements.push(None);
                } else {
                    let name = self.parse_identifier()?;
                    elements.push(Some(name));
                }

                if !self.match_token(&Token::Comma) {
                    break;
                }

                if self.check(&Token::RBracket) {
                    break; // Trailing comma
                }
            }
        }

        self.expect(Token::RBracket)?;

        Ok(BindingPattern::Array(ArrayPattern { elements, rest }))
    }

    /// Parse tuple destructuring pattern: (x, y, z)
    fn parse_tuple_pattern(&mut self) -> Result<BindingPattern> {
        self.expect(Token::LParen)?;

        let mut elements = Vec::new();

        // Parse elements
        if !self.check(&Token::RParen) {
            // Parse first element
            elements.push(self.parse_identifier()?);

            // Parse remaining elements
            while self.match_token(&Token::Comma) {
                if self.check(&Token::RParen) {
                    break; // Trailing comma
                }
                elements.push(self.parse_identifier()?);
            }
        }

        self.expect(Token::RParen)?;

        Ok(BindingPattern::Tuple(TuplePattern { elements }))
    }

    fn parse_simple_statement(&mut self) -> Result<Stmt> {
        if self.match_token(&Token::Return) {
            let value = if self.is_at_end()
                || self.check(&Token::Semicolon)
                || self.check(&Token::RBrace)
            {
                None
            } else {
                Some(self.parse_expression()?)
            };
            Ok(Stmt::Return(ReturnStmt { expr: value }))
        } else if self.match_token(&Token::Break) {
            Ok(Stmt::Break)
        } else if self.match_token(&Token::Continue) {
            Ok(Stmt::Continue)
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

            // Check for `or fail "message"` — error propagation shorthand (v1.1.0)
            let or_fail_msg = if self.check(&Token::Or) && self.peek_next_is(&Token::Fail) {
                self.advance(); // consume `or`
                self.advance(); // consume `fail`
                let msg = self.parse_expression()?;
                Some(Box::new(msg))
            } else {
                None
            };

            self.match_token(&Token::Semicolon); // Optional semicolon

            let is_fallible = bindings.len() > 1 || or_fail_msg.is_some();

            return Ok(Stmt::VarDecl(VarDecl {
                bindings,
                init,
                is_fallible,
                or_fail_msg,
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
            let value = if self.is_at_end()
                || self.check(&Token::Semicolon)
                || self.check(&Token::RBrace)
            {
                None
            } else {
                Some(self.parse_expression()?)
            };
            return Ok(Stmt::Return(ReturnStmt { expr: value }));
        }

        if self.match_token(&Token::Break) {
            return Ok(Stmt::Break);
        }

        if self.match_token(&Token::Continue) {
            return Ok(Stmt::Continue);
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

            // Check if it's a simple statement (like if cond fail "msg") or => one-liner
            let then_branch = if self.check(&Token::LBrace) {
                self.expect(Token::LBrace)?;
                let block = self.parse_block_stmt()?;
                self.expect(Token::RBrace)?;
                IfBody::Block(block)
            } else {
                // Consume optional => for one-liner syntax: if cond => expr
                self.match_token(&Token::Arrow);
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
                    // Consume optional => for one-liner else: else => expr
                    self.match_token(&Token::Arrow);
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
            let condition = self.parse_expression_no_lambda()?;
            let body = if self.match_token(&Token::Arrow) {
                // One-liner: while cond => stmt
                let stmt = self.parse_simple_statement()?;
                BlockStmt { stmts: vec![stmt] }
            } else {
                self.expect(Token::LBrace)?;
                let body = self.parse_block_stmt()?;
                self.expect(Token::RBrace)?;
                body
            };
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
            } else if self.match_token(&Token::ParVec) {
                policy = DataParallelPolicy::ParVec;
            }

            let var = self.parse_identifier()?;
            self.expect(Token::In)?;
            let iterable = self.parse_expression_no_lambda()?;

            let options = if self.match_token(&Token::With) {
                self.parse_for_options()?
            } else {
                ForPolicyOptions::default()
            };

            let body = if self.match_token(&Token::Arrow) {
                // One-liner: for x in items => stmt
                let stmt = self.parse_simple_statement()?;
                BlockStmt { stmts: vec![stmt] }
            } else {
                self.expect(Token::LBrace)?;
                let body = self.parse_block_stmt()?;
                self.expect(Token::RBrace)?;
                body
            };
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

    /// Parse an expression without lambda detection — used in for/while contexts
    /// where `=>` means the body arrow, not a lambda.
    fn parse_expression_no_lambda(&mut self) -> Result<Expr> {
        self.parse_assignment()
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
            // Single parameter without parentheses: x => ... or {x, y} => ...
            let pattern = self.parse_param_pattern()?;
            vec![LambdaParam {
                pattern,
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
            // Parse pattern (can be identifier, object destructuring, or array destructuring)
            let pattern = self.parse_param_pattern()?;
            let type_ref = if self.match_token(&Token::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };

            params.push(LambdaParam { pattern, type_ref });

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

        if self.match_token(&Token::DotDotEq) {
            let right = self.parse_assignment()?;
            return Ok(Expr::Binary {
                op: BinOp::RangeInclusive,
                left: Box::new(expr),
                right: Box::new(right),
            });
        }

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

        while (self.check(&Token::Or) && !self.peek_next_is(&Token::Fail))
            || self.check(&Token::OrOr)
        {
            self.advance(); // consume `or` / `||`
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

        while self.match_token(&Token::And) || self.match_token(&Token::AndAnd) {
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
        if self.match_token(&Token::Bang) || self.match_token(&Token::Not) {
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
            Expr::MethodCall(method_call) => {
                // For method calls (like HTTP.get()), wrap in a Call expression with the policy
                Ok(Expr::Call(CallExpr {
                    callee: Box::new(Expr::MethodCall(method_call)),
                    args: Vec::new(),
                    exec_policy: policy,
                    type_args: Vec::new(),
                }))
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
            // Check for type arguments like sum<float>
            if self.check(&Token::Lt) && self.is_type_argument_list() {
                let type_args = self.parse_type_arguments()?;

                // After type arguments, we must have a function call
                if self.match_token(&Token::LParen) {
                    let args = self.parse_args()?;
                    self.expect(Token::RParen)?;
                    expr = Expr::Call(CallExpr::with_type_args(expr, type_args, args));
                } else {
                    return Err(self.error("Expected '(' after type arguments".to_string()));
                }
            } else if self.match_token(&Token::LParen) {
                expr = self.finish_call(expr)?;
            } else if self.check(&Token::LBrace) {
                // Check if this is a struct literal like TypeName { field: value }
                if let Expr::Identifier(type_name) = &expr {
                    // Only allow struct literals for identifiers that start with uppercase (type names)
                    if type_name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        // Bug #64 fix: Verify this is actually a struct literal by looking ahead.
                        // A struct literal has: { } or { ident: expr, ... }
                        // This prevents misinterpreting `LIMIT { continue }` as a struct literal
                        // when LIMIT is a const used in an if-condition before a block.
                        let is_struct_literal = matches!(self.peek_token(1), Some(Token::RBrace))
                            || (matches!(
                                self.peek_token(1),
                                Some(Token::Ident(_)) | Some(Token::PrivateIdent(_))
                            ) && matches!(self.peek_token(2), Some(Token::Colon)));
                        if is_struct_literal {
                            self.advance(); // consume the {
                            let fields = self.parse_object_fields()?;
                            self.expect(Token::RBrace)?;
                            expr = Expr::StructLiteral {
                                type_name: type_name.clone(),
                                fields,
                            };
                        } else {
                            // Not a struct literal (e.g., LIMIT { continue }), don't consume the {
                            break;
                        }
                    } else {
                        // Not a struct literal, don't consume the { and continue
                        break;
                    }
                } else {
                    // Not an identifier, don't consume the { and continue
                    break;
                }
            } else if self.match_token(&Token::Dot) {
                let name = self.parse_method_name()?;

                // Check if this is a method call (followed by parentheses)
                if self.check(&Token::LParen) {
                    self.advance(); // consume the (

                    // Check if this is an adapter method (par, vec, parvec)
                    let (adapter, options) = if name == "par" || name == "vec" || name == "parvec" {
                        let adapter = match name.as_str() {
                            "par" => ArrayAdapter::Par,
                            "vec" => ArrayAdapter::Vec,
                            "parvec" => ArrayAdapter::ParVec,
                            _ => ArrayAdapter::Seq,
                        };

                        // Check for adapter options like {threads: 4, chunk: 2}
                        let options = if self.match_token(&Token::LBrace) {
                            self.parse_adapter_options()?
                        } else {
                            AdapterOptions::default()
                        };

                        self.expect(Token::RParen)?;

                        // The adapter call returns the same array with the adapter applied
                        // We need to continue parsing to get the actual method call
                        if !self.match_token(&Token::Dot) {
                            return Err(self.error(format!(
                                "Expected method call after adapter '.{}()'",
                                name
                            )));
                        }

                        // Now parse the actual array method (map, filter, etc.)
                        (adapter, options)
                    } else {
                        // Not an adapter, this is a regular method call
                        // Parse arguments and create MethodCall expression
                        let args = self.parse_args()?;
                        self.expect(Token::RParen)?;

                        expr = Expr::MethodCall(MethodCallExpr {
                            object: Box::new(expr),
                            method: name,
                            args,
                            adapter: ArrayAdapter::Seq,
                            adapter_options: AdapterOptions::default(),
                        });
                        continue;
                    };

                    // Parse the actual method name after adapter
                    let method_name = self.parse_identifier()?;
                    self.expect(Token::LParen)?;
                    let args = self.parse_args()?;
                    self.expect(Token::RParen)?;

                    expr = Expr::MethodCall(MethodCallExpr {
                        object: Box::new(expr),
                        method: method_name,
                        args,
                        adapter,
                        adapter_options: options,
                    });
                } else {
                    // Regular member access (not a method call)
                    expr = Expr::Member {
                        object: Box::new(expr),
                        property: name,
                    };
                }
            } else if self.match_token(&Token::LBracket) {
                let index = self.parse_expression()?;
                self.expect(Token::RBracket)?;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(&Token::DoubleColon) {
                // Phase 11.4: Method references — Utils::validate, logger::log, User::new
                if let Expr::Identifier(object_name) = &expr {
                    let method = self.parse_identifier()?;
                    expr = Expr::MethodRef {
                        object: object_name.clone(),
                        method,
                    };
                } else {
                    return Err(self.error("Expected identifier before '::'".to_string()));
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_adapter_options(&mut self) -> Result<AdapterOptions> {
        let mut options = AdapterOptions::default();

        loop {
            if self.match_token(&Token::RBrace) {
                break;
            }

            // Parse option name
            let option_name = self.parse_identifier()?;
            self.expect(Token::Colon)?;

            match option_name.as_str() {
                "threads" => {
                    if let Some(Token::IntLiteral(value)) = self.peek() {
                        options.threads = Some(*value as i32);
                        self.advance();
                    } else {
                        return Err(self.error("Expected integer value for 'threads'".into()));
                    }
                }
                "chunk" => {
                    if let Some(Token::IntLiteral(value)) = self.peek() {
                        options.chunk = Some(*value as i32);
                        self.advance();
                    } else {
                        return Err(self.error("Expected integer value for 'chunk'".into()));
                    }
                }
                "simdWidth" => {
                    if let Some(Token::IntLiteral(value)) = self.peek() {
                        options.simd_width = Some(*value as i32);
                        self.advance();
                    } else {
                        return Err(self.error("Expected integer value for 'simdWidth'".into()));
                    }
                }
                "ordered" => {
                    if self.match_token(&Token::True) {
                        options.ordered = Some(true);
                    } else if self.match_token(&Token::False) {
                        options.ordered = Some(false);
                    } else {
                        return Err(self.error("Expected 'true' or 'false' for 'ordered'".into()));
                    }
                }
                _ => {
                    return Err(self.error(format!("Unknown adapter option: {}", option_name)));
                }
            }

            // Check for comma or end of options
            if !self.match_token(&Token::Comma) {
                self.expect(Token::RBrace)?;
                break;
            }
        }

        Ok(options)
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

    /// Check if we're looking at a type argument list like <float> or <int, string>
    /// This distinguishes from < as a comparison operator
    fn is_type_argument_list(&self) -> bool {
        // We need a mutable reference to look ahead, so we'll use a simple heuristic:
        // After <, we expect:
        // 1. An identifier (type name) or type keyword (float, bool, string, etc.)
        // 2. Optional comma and more types
        // 3. A > followed by (

        // Look for the pattern: < identifier (possibly with more types) > (
        let mut offset = 1; // Start after <

        // Must have an identifier or type keyword after <
        let is_type_token = matches!(
            self.peek_token(offset),
            Some(Token::Ident(_))
                | Some(Token::Number)
                | Some(Token::Float)
                | Some(Token::Bool)
                | Some(Token::String)
                | Some(Token::CharType)
                | Some(Token::Bytes)
        );

        if !is_type_token {
            return false;
        }
        offset += 1;

        // Skip through type parameters (identifiers and commas)
        loop {
            match self.peek_token(offset) {
                Some(Token::Comma) => {
                    offset += 1;
                    // After comma, expect another identifier or type keyword
                    let is_type_token = matches!(
                        self.peek_token(offset),
                        Some(Token::Ident(_))
                            | Some(Token::Number)
                            | Some(Token::Float)
                            | Some(Token::Bool)
                            | Some(Token::String)
                            | Some(Token::CharType)
                            | Some(Token::Bytes)
                    );

                    if !is_type_token {
                        return false;
                    }
                    offset += 1;
                }
                Some(Token::Gt) => {
                    // Found closing >, check if followed by (
                    offset += 1;
                    return matches!(self.peek_token(offset), Some(Token::LParen));
                }
                Some(Token::LBracket) => {
                    // Array type like [int]
                    offset += 1;
                    if !matches!(self.peek_token(offset), Some(Token::Ident(_))) {
                        return false;
                    }
                    offset += 1;
                    if !matches!(self.peek_token(offset), Some(Token::RBracket)) {
                        return false;
                    }
                    offset += 1;
                }
                _ => return false,
            }
        }
    }

    /// Parse type arguments like <float> or <int, string>
    fn parse_type_arguments(&mut self) -> Result<Vec<TypeRef>> {
        self.expect(Token::Lt)?;

        let mut type_args = Vec::new();

        loop {
            let ty = self.parse_type()?;
            type_args.push(ty);

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.expect(Token::Gt)?;
        Ok(type_args)
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
        if self.match_token(&Token::Null) {
            return Ok(Expr::Literal(Literal::Null));
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
                // Allow 'test' keyword as identifier in expression context (liva/test library)
                Token::Test => {
                    self.advance();
                    return Ok(Expr::Identifier("test".to_string()));
                }
                _ => {}
            }
        }

        if self.match_token(&Token::Fail) {
            let expr = self.parse_expression()?;
            return Ok(Expr::Fail(Box::new(expr)));
        }

        if self.match_token(&Token::Switch) {
            return self.parse_switch_expr();
        }

        if self.match_token(&Token::LParen) {
            // Handle empty tuple: ()
            if self.match_token(&Token::RParen) {
                return Ok(Expr::Tuple(vec![]));
            }

            // Parse first element
            let first = self.parse_expression()?;

            // Check for comma (tuple) or RParen (grouped expr)
            if self.match_token(&Token::Comma) {
                // It's a tuple!
                let mut elements = vec![first];

                // Parse remaining elements (allow trailing comma)
                if !self.check(&Token::RParen) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                        // Allow trailing comma before )
                        if self.check(&Token::RParen) {
                            break;
                        }
                    }
                }

                self.expect(Token::RParen)?;
                return Ok(Expr::Tuple(elements));
            } else {
                // Just a grouped expression
                self.expect(Token::RParen)?;
                return Ok(first); // Return the expression, not a tuple
            }
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

    /// Parse switch expression: switch x { 1 => "one", 2 => "two", _ => "other" }
    fn parse_switch_expr(&mut self) -> Result<Expr> {
        let discriminant = Box::new(self.parse_expression()?);
        self.expect(Token::LBrace)?;

        let mut arms = Vec::new();

        while !self.is_at_end() && !self.check(&Token::RBrace) {
            // Parse pattern
            let pattern = self.parse_pattern()?;

            // Parse optional guard (if condition)
            let guard = if self.match_token(&Token::If) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            // Expect =>
            self.expect(Token::Arrow)?;

            // Parse body (expression or block)
            let body = if self.check(&Token::LBrace) {
                self.advance(); // consume {
                let mut stmts = Vec::new();
                while !self.is_at_end() && !self.check(&Token::RBrace) {
                    stmts.push(self.parse_statement()?);
                }
                self.expect(Token::RBrace)?;
                SwitchBody::Block(stmts)
            } else {
                let expr = self.parse_expression()?;
                SwitchBody::Expr(Box::new(expr))
            };

            arms.push(SwitchArm {
                pattern,
                guard,
                body,
            });

            // Optional comma between arms
            self.match_token(&Token::Comma);
        }

        self.expect(Token::RBrace)?;

        if arms.is_empty() {
            return Err(self.error("Switch expression must have at least one arm".into()));
        }

        Ok(Expr::Switch(SwitchExpr { discriminant, arms }))
    }

    /// Parse a pattern for pattern matching
    fn parse_pattern(&mut self) -> Result<Pattern> {
        self.parse_or_pattern()
    }

    /// Parse or-pattern: pattern | pattern | ...
    fn parse_or_pattern(&mut self) -> Result<Pattern> {
        let mut patterns = vec![self.parse_single_pattern()?];

        while self.match_token(&Token::Pipe) {
            patterns.push(self.parse_single_pattern()?);
        }

        if patterns.len() == 1 {
            Ok(patterns.into_iter().next().unwrap())
        } else {
            Ok(Pattern::Or(patterns))
        }
    }

    /// Parse a single pattern (no or-patterns)
    fn parse_single_pattern(&mut self) -> Result<Pattern> {
        // Wildcard pattern: _
        if self.match_token(&Token::Underscore) {
            return Ok(Pattern::Wildcard);
        }

        // Tuple pattern: (p1, p2, ...)
        if self.match_token(&Token::LParen) {
            let mut patterns = Vec::new();

            if !self.check(&Token::RParen) {
                loop {
                    patterns.push(self.parse_pattern()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                    // Allow trailing comma
                    if self.check(&Token::RParen) {
                        break;
                    }
                }
            }

            self.expect(Token::RParen)?;

            // Single element is not a tuple, just a grouped pattern
            if patterns.len() == 1 {
                return Ok(patterns.into_iter().next().unwrap());
            }

            return Ok(Pattern::Tuple(patterns));
        }

        // Array pattern: [p1, p2, ...]
        if self.match_token(&Token::LBracket) {
            let mut patterns = Vec::new();

            if !self.check(&Token::RBracket) {
                loop {
                    patterns.push(self.parse_pattern()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                    // Allow trailing comma
                    if self.check(&Token::RBracket) {
                        break;
                    }
                }
            }

            self.expect(Token::RBracket)?;
            return Ok(Pattern::Array(patterns));
        }

        // Check for range patterns
        if self.check(&Token::DotDot) || self.check(&Token::DotDotEq) {
            // Open start range: ..10 or ..=10
            let inclusive = self.match_token(&Token::DotDotEq);
            if !inclusive {
                self.expect(Token::DotDot)?;
            }
            let end = Some(Box::new(self.parse_primary()?));
            return Ok(Pattern::Range(RangePattern {
                start: None,
                end,
                inclusive,
            }));
        }

        // Parse first expression (could be literal, binding, or start of range)
        let expr = self.parse_primary()?;

        // Check if this is a range pattern
        if self.check(&Token::DotDot) || self.check(&Token::DotDotEq) {
            let inclusive = self.match_token(&Token::DotDotEq);
            if !inclusive {
                self.expect(Token::DotDot)?;
            }

            let end = if self.check(&Token::Arrow) || self.check(&Token::If) {
                // Open end range: 10..
                None
            } else {
                Some(Box::new(self.parse_primary()?))
            };

            return Ok(Pattern::Range(RangePattern {
                start: Some(Box::new(expr)),
                end,
                inclusive,
            }));
        }

        // Convert expression to pattern
        match expr {
            Expr::Literal(lit) => Ok(Pattern::Literal(lit)),
            Expr::Identifier(name) => {
                // Check for enum variant pattern: EnumName.Variant or EnumName.Variant(bindings)
                if self.match_token(&Token::Dot) {
                    let variant_name = self.parse_identifier()?;
                    let bindings = if self.match_token(&Token::LParen) {
                        let mut bindings = Vec::new();
                        while !self.check(&Token::RParen) && !self.is_at_end() {
                            bindings.push(self.parse_identifier()?);
                            if !self.check(&Token::RParen) {
                                self.expect(Token::Comma)?;
                            }
                        }
                        self.expect(Token::RParen)?;
                        bindings
                    } else {
                        vec![]
                    };
                    Ok(Pattern::EnumVariant {
                        enum_name: name,
                        variant_name,
                        bindings,
                    })
                } else if self.match_token(&Token::Colon) {
                    // Type pattern: name: type
                    let type_ref = self.parse_type()?;
                    Ok(Pattern::Typed { name, type_ref })
                } else {
                    // Identifiers can be bindings (lowercase) or enum variants (capitalized)
                    Ok(Pattern::Binding(name))
                }
            }
            _ => Err(self.error("Invalid pattern".into())),
        }
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
            Some(Token::PrivateIdent(s)) => Ok(s.clone()),
            // Type keywords are valid identifiers in parameter/variable name contexts
            Some(Token::Number) => Ok("number".to_string()),
            Some(Token::Float) => Ok("float".to_string()),
            Some(Token::Bool) => Ok("bool".to_string()),
            Some(Token::String) => Ok("string".to_string()),
            Some(Token::CharType) => Ok("char".to_string()),
            Some(Token::Bytes) => Ok("bytes".to_string()),
            Some(Token::Type) => Ok("type".to_string()),
            // Allow 'test' as identifier in import contexts (liva/test library)
            Some(Token::Test) => Ok("test".to_string()),
            _ => Err(self.error("Expected identifier".into())),
        }
    }

    /// Parse identifier or keyword token as method/field name
    /// This allows reserved keywords like "par", "vec", "parvec" and type keywords
    /// like "number", "float", "string", "bool" to be used as field/method names
    fn parse_method_name(&mut self) -> Result<String> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(s.clone()),
            Some(Token::PrivateIdent(s)) => Ok(s.clone()),
            Some(Token::Par) => Ok("par".to_string()),
            Some(Token::Vec) => Ok("vec".to_string()),
            Some(Token::ParVec) => Ok("parvec".to_string()),
            Some(Token::IntLiteral(n)) => Ok(n.to_string()), // Tuple member access: .0, .1, .2
            // Type keywords allowed as field/method names
            Some(Token::Number) => Ok("number".to_string()),
            Some(Token::Float) => Ok("float".to_string()),
            Some(Token::Bool) => Ok("bool".to_string()),
            Some(Token::String) => Ok("string".to_string()),
            Some(Token::CharType) => Ok("char".to_string()),
            Some(Token::Bytes) => Ok("bytes".to_string()),
            // Other keywords that may appear as field names
            Some(Token::Type) => Ok("type".to_string()),
            Some(Token::Null) => Ok("null".to_string()),
            Some(Token::Not) => Ok("not".to_string()),
            _ => Err(self.error("Expected method name".into())),
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
                    return Err(CompilerError::ParseError(
                        SemanticErrorInfo::new(
                            "E2001",
                            "Unclosed interpolation",
                            "String template has an unclosed interpolation expression",
                        )
                        .with_location("<input>", 1)
                        .with_help(
                            "Make sure all '{' characters in interpolations have matching '}'",
                        ),
                    ));
                }
                let expr_src_trimmed = expr_src.trim();
                if expr_src_trimmed.is_empty() {
                    return Err(CompilerError::ParseError(
                        SemanticErrorInfo::new(
                            "E2002",
                            "Empty interpolation",
                            "String template has an empty interpolation: {}",
                        )
                        .with_location("<input>", 1)
                        .with_help(
                            "Add an expression inside the interpolation or remove the empty braces",
                        ),
                    ));
                }
                // First, try to normalize single quotes to double quotes for string literals
                // This allows $"Status: {age >= 18 ? 'adult' : 'minor'}" to work
                let normalized = normalize_template_strings(expr_src_trimmed);
                match parse_template_expression(&normalized) {
                    Ok(expr) => parts.push(StringTemplatePart::Expr(Box::new(expr))),
                    Err(_) => {
                        // If normalized version failed, try original
                        if let Ok(expr) = parse_template_expression(expr_src_trimmed) {
                            parts.push(StringTemplatePart::Expr(Box::new(expr)));
                            continue;
                        }
                        // Both failed - treat as literal text
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
                    return Err(CompilerError::ParseError(
                        SemanticErrorInfo::new(
                            "E2003",
                            "Unmatched closing brace",
                            "String template has an unmatched '}' character",
                        )
                        .with_location("<input>", 1)
                        .with_help(
                            "Use '}}' to escape a literal '}' character in a string template",
                        ),
                    ));
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

/// Normalize single-quoted strings to double-quoted strings in template expressions.
/// This allows using 'string' syntax inside templates which is more natural
/// since the template itself uses double quotes.
/// Example: $"Status: {age >= 18 ? 'adult' : 'minor'}" -> age >= 18 ? "adult" : "minor"
fn normalize_template_strings(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_double_quote = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_double_quote = !in_double_quote;
                result.push(ch);
            }
            '\'' if !in_double_quote => {
                // Convert single quote to double quote (string literal, not char)
                result.push('"');
            }
            '\\' => {
                result.push(ch);
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            }
            _ => result.push(ch),
        }
    }
    result
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
