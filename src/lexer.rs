use crate::error::{CompilerError, Result};
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
    #[token("move")]
    Move,
    #[token("seq")]
    Seq,
    #[token("vec")]
    Vec,
    #[token("boost")]
    Boost,
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
    #[token(";")]
    Semicolon,
    #[token(".")]
    Dot,
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
    #[regex(r"__[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    PrivateIdent(String),

    #[regex(r"_[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    ProtectedIdent(String),

    #[regex(r"[a-zA-Z][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
}

#[derive(Debug, Clone)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: std::ops::Range<usize>,
}

pub fn tokenize(source: &str) -> Result<Vec<TokenWithSpan>> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();

    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => {
                tokens.push(TokenWithSpan {
                    token,
                    span: lexer.span(),
                });
            }
            Err(_) => {
                let span = lexer.span();
                let line = source[..span.start].lines().count();
                let col = source[..span.start]
                    .lines()
                    .last()
                    .map(|l| l.len())
                    .unwrap_or(0);

                return Err(CompilerError::LexerError(format!(
                    "Invalid token at line {}, column {}: '{}'",
                    line, col, &source[span]
                )));
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
        let source = "public _protected __private";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[0].token, Token::Ident("public".to_string()));
        assert_eq!(
            tokens[1].token,
            Token::ProtectedIdent("_protected".to_string())
        );
        assert_eq!(
            tokens[2].token,
            Token::PrivateIdent("__private".to_string())
        );
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
