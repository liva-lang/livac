use crate::error::{CompilerError, Result, SemanticErrorInfo};
use crate::span::{SourceMap, Span};
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    // Keywords
    #[token("let")]
    Let,
    #[token("const")]
    Const,
    #[token("import")]
    Import,
    #[token("from")]
    From,
    #[token("use")]
    Use,
    #[token("rust")]
    Rust,
    #[token("type")]
    Type,
    #[token("test")]
    Test,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("throw")]
    Throw,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("return")]
    Return,
    #[token("async")]
    Async,
    #[token("parallel")]
    Parallel,
    #[token("par")]
    Par,
    #[token("task")]
    Task,
    #[token("fire")]
    Fire,
    #[token("await")]
    Await,
    #[token("fail")]
    Fail,
    #[token("move")]
    Move,
    #[token("seq")]
    Seq,
    #[token("vec")]
    Vec,
    #[token("parvec")]
    ParVec,
    #[token("with")]
    With,
    #[token("ordered")]
    Ordered,
    #[token("chunk")]
    Chunk,
    #[token("threads")]
    Threads,
    #[token("simdWidth")]
    SimdWidth,
    #[token("prefetch")]
    Prefetch,
    #[token("reduction")]
    Reduction,
    #[token("schedule")]
    Schedule,
    #[token("detect")]
    Detect,
    #[token("auto")]
    Auto,
    #[token("safe")]
    Safe,
    #[token("fast")]
    Fast,
    #[token("static")]
    Static,
    #[token("dynamic")]
    Dynamic,
    #[token("as")]
    As,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,

    // Logical operators (words)
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,

    // Type keywords
    #[token("number")]
    Number,
    #[token("float")]
    Float,
    #[token("bool")]
    Bool,
    #[token("char")]
    CharType,
    #[token("string")]
    String,
    #[token("bytes")]
    Bytes,

    // Operators
    #[token("=")]
    Assign,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,
    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token("&&")]
    AndAnd,
    #[token("|")]
    Pipe,
    #[token("||")]
    OrOr,
    #[token("!")]
    Bang,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token(";")]
    Semicolon,
    #[token("_")]
    Underscore,
    #[token(".")]
    Dot,
    #[token("...")]
    DotDotDot,
    #[token("..=")]
    DotDotEq,
    #[token("..")]
    DotDot,
    #[token("?")]
    Question,
    #[token("=>")]
    Arrow,

    // Literals
    #[regex(r"[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<i64>().ok())]
    IntLiteral(i64),

    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<f64>().ok())]
    FloatLiteral(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        let content = &s[1..s.len()-1];
        Some(content.to_string())
    })]
    StringLiteral(String),

    #[regex(r"'([^'\\]|\\.)+'", |lex| {
        let s = lex.slice();
        let content = &s[1..s.len()-1];
        content.chars().next()
    })]
    CharLiteral(char),

    // String template
    #[regex(r#"\$"([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        let content = &s[2..s.len()-1];
        Some(content.to_string())
    })]
    StringTemplate(String),

    // Identifiers
    #[regex(r"_[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    PrivateIdent(String),

    #[regex(r"[a-zA-Z][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
}

#[derive(Debug, Clone)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: Span,
}

#[allow(dead_code)]
impl TokenWithSpan {
    pub fn new(token: Token, span: Span) -> Self {
        Self { token, span }
    }

    pub fn start(&self) -> usize {
        self.span.start
    }

    pub fn end(&self) -> usize {
        self.span.end
    }

    pub fn line_col(&self, map: &SourceMap) -> (usize, usize) {
        self.span.start_position(map)
    }

    pub fn snippet<'a>(&self, source: &'a str) -> &'a str {
        self.span.snippet(source)
    }
}

pub fn tokenize(source: &str) -> Result<Vec<TokenWithSpan>> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();
    let source_map = SourceMap::new(source);

    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => {
                let span = Span::from(lexer.span());
                tokens.push(TokenWithSpan::new(token, span));
            }
            Err(_) => {
                let span = Span::from(lexer.span());
                let (line, col) = span.start_position(&source_map);
                let snippet = span.snippet(source);
                let display = if snippet.is_empty() { "<EOF>" } else { snippet };

                let source_line = source
                    .lines()
                    .nth(line.saturating_sub(1))
                    .unwrap_or("")
                    .to_string();

                let error = SemanticErrorInfo::new(
                    "E1000",
                    "Invalid token",
                    &format!("Encountered an invalid token: '{}'", display),
                )
                .with_location("<input>", line)
                .with_column(col)
                .with_source_line(source_line)
                .with_help("Check for unexpected characters or typos in your code");

                return Err(CompilerError::LexerError(error));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let source = "let x = 10";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[1].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[2].token, Token::Assign);
        assert_eq!(tokens[3].token, Token::IntLiteral(10));
    }

    #[test]
    fn test_visibility() {
        let source = "public _private";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].token, Token::Ident("public".to_string()));
        assert_eq!(tokens[1].token, Token::PrivateIdent("_private".to_string()));
    }

    #[test]
    fn test_logical_operators() {
        let source = "and or not && || !";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].token, Token::And);
        assert_eq!(tokens[1].token, Token::Or);
        assert_eq!(tokens[2].token, Token::Not);
        assert_eq!(tokens[3].token, Token::AndAnd);
        assert_eq!(tokens[4].token, Token::OrOr);
        assert_eq!(tokens[5].token, Token::Bang);
    }

    #[test]
    fn test_string_template() {
        let source = r#"$"Hello {name}""#;
        let tokens = tokenize(source).unwrap();

        assert!(matches!(tokens[0].token, Token::StringTemplate(_)));
    }
}
