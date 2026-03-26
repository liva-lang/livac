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
    #[token("enum")]
    Enum,
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
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("async")]
    Async,
    #[token("parallel")]
    Parallel,
    #[token("par")]
    Par,
    #[token("task")]
    Task,
    #[token("await")]
    Await,
    #[token("fail")]
    Fail,
    #[token("defer")]
    Defer,
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
    // Note: 'data' is a contextual keyword, handled in parser via identifier check
    // Not a hard keyword - allows 'data' as variable name

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
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,
    #[token("/=")]
    SlashAssign,
    #[token("%=")]
    PercentAssign,
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
        // B26 fix: interpret escape sequences instead of taking first char blindly
        if content.starts_with('\\') && content.len() >= 2 {
            match content.chars().nth(1) {
                Some('n') => Some('\n'),
                Some('t') => Some('\t'),
                Some('r') => Some('\r'),
                Some('\\') => Some('\\'),
                Some('\'') => Some('\''),
                Some('0') => Some('\0'),
                _ => content.chars().next(),
            }
        } else {
            content.chars().next()
        }
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

    /// Inline Rust code block — NOT produced by logos, synthesized in `tokenize()`
    /// when `rust { ... }` is detected. The String contains the raw Rust source
    /// between the braces (exclusive).
    RustBlock(String),
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

/// Information about a `$"..."` template string found during pre-processing.
/// B02: Template strings with nested quotes inside `{...}` interpolation
/// (e.g., `$"{fn("arg")}"`) require pre-scanning because the logos regex
/// cannot track brace depth.
#[derive(Debug)]
struct TemplateStringInfo {
    /// Byte offset where the `$"` starts
    start: usize,
    /// Byte offset right after the closing `"` (exclusive)
    end: usize,
    /// The content between `$"` and the final `"` (i.e., what goes into StringTemplate)
    content: String,
}

/// Scan source for `$"..."` template strings, tracking brace depth so that
/// `"` chars inside `{...}` interpolation do not prematurely close the template.
/// Only templates that contain quotes inside interpolation braces actually NEED
/// this, but we process all of them for uniformity.
fn find_template_strings(source: &str) -> Vec<TemplateStringInfo> {
    let bytes = source.as_bytes();
    let mut templates = Vec::new();
    let mut i = 0;

    while i + 1 < bytes.len() {
        // Skip // line comments
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // Skip /* block comments */
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // Skip regular string literals (non-template)
        if bytes[i] == b'"' && (i == 0 || bytes[i - 1] != b'$') {
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 1; // skip escaped char
                } else if bytes[i] == b'"' {
                    break;
                }
                i += 1;
            }
            i += 1;
            continue;
        }

        // Skip single-quote strings
        if bytes[i] == b'\'' && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric()) {
            // Could be char literal or single-quote string — skip to matching '
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 1;
                } else if bytes[i] == b'\'' {
                    break;
                }
                i += 1;
            }
            i += 1;
            continue;
        }

        // Detect $" — start of template string
        if bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'"' {
            let start = i;
            i += 2; // skip $"

            let mut brace_depth: usize = 0;
            let mut found_end = false;

            while i < bytes.len() {
                if brace_depth > 0 {
                    // Inside {…} interpolation — quotes don't close the template
                    match bytes[i] {
                        b'{' => brace_depth += 1,
                        b'}' => brace_depth -= 1,
                        b'"' => {
                            // String literal inside interpolation — skip it
                            i += 1;
                            while i < bytes.len() {
                                if bytes[i] == b'\\' {
                                    i += 1;
                                } else if bytes[i] == b'"' {
                                    break;
                                }
                                i += 1;
                            }
                            // i is now at the closing " of the inner string
                        }
                        b'\'' => {
                            // Single-quote string inside interpolation — skip it
                            i += 1;
                            while i < bytes.len() {
                                if bytes[i] == b'\\' {
                                    i += 1;
                                } else if bytes[i] == b'\'' {
                                    break;
                                }
                                i += 1;
                            }
                        }
                        _ => {}
                    }
                } else {
                    // At template level (not inside braces)
                    match bytes[i] {
                        b'{' => brace_depth += 1,
                        b'"' => {
                            // This is the real closing quote
                            let content = source[start + 2..i].to_string();
                            templates.push(TemplateStringInfo {
                                start,
                                end: i + 1,
                                content,
                            });
                            i += 1;
                            found_end = true;
                            break;
                        }
                        b'\\' => {
                            i += 1; // skip escaped char
                        }
                        _ => {}
                    }
                }
                i += 1;
            }

            if !found_end {
                // Unterminated template — let logos handle the error
                i = start + 1;
            }
            continue;
        }

        i += 1;
    }

    templates
}

/// Information about a `rust { ... }` block found during pre-processing.
#[derive(Debug)]
struct RustBlockInfo {
    /// Byte offset where the `rust` keyword starts
    rust_keyword_start: usize,
    /// Byte offset right after the opening `{`
    content_start: usize,
    /// Byte offset of the closing `}` (exclusive of content)
    content_end: usize,
    /// Byte offset right after the closing `}`
    closing_brace_end: usize,
    /// The raw Rust source code between the braces
    content: String,
}

/// Scan source for `rust { ... }` blocks (not `use rust "..."`) and extract them.
/// Returns a list of blocks with byte ranges and raw content.
fn find_rust_blocks(source: &str) -> Vec<RustBlockInfo> {
    let bytes = source.as_bytes();
    let mut blocks = Vec::new();
    let mut i = 0;

    while i + 4 <= bytes.len() {
        // B42 fix: Skip // line comments entirely
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // B42 fix: Skip /* block comments */ entirely
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // B42 fix: Skip string literals (avoid matching "rust" inside strings)
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 1; // skip escaped char
                } else if bytes[i] == b'"' {
                    break;
                }
                i += 1;
            }
            i += 1;
            continue;
        }

        // Look for "rust" as a whole word (compare bytes to avoid UTF-8 boundary issues)
        if bytes[i] == b'r' && bytes[i + 1] == b'u' && bytes[i + 2] == b's' && bytes[i + 3] == b't'
        {
            let prev_is_word =
                i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            let next_is_word = i + 4 < bytes.len()
                && (bytes[i + 4].is_ascii_alphanumeric() || bytes[i + 4] == b'_');

            if !prev_is_word && !next_is_word {
                let rust_start = i;
                let mut j = i + 4;

                // Skip whitespace
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }

                // Check for opening brace (not a string → that's `use rust "crate"`)
                if j < bytes.len() && bytes[j] == b'{' {
                    let content_start = j + 1;
                    if let Some((content_end, closing_brace_end)) =
                        find_balanced_brace(source, j)
                    {
                        let content = source[content_start..content_end].to_string();
                        blocks.push(RustBlockInfo {
                            rust_keyword_start: rust_start,
                            content_start,
                            content_end,
                            closing_brace_end,
                            content,
                        });
                        i = closing_brace_end;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }

    blocks
}

/// Find the matching closing `}` for the opening `{` at byte position `open_pos`.
/// Handles Rust string literals, char literals, line comments, and block comments.
/// Returns `Some((content_end, brace_end))` where content_end is at the `}` and
/// brace_end is one byte past it.
fn find_balanced_brace(source: &str, open_pos: usize) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut depth: usize = 0;
    let mut i = open_pos;

    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((i, i + 1));
                }
            }
            b'"' => {
                // Skip string literal (handle escaped chars)
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i += 1; // skip next byte (escaped)
                    } else if bytes[i] == b'"' {
                        break;
                    }
                    i += 1;
                }
            }
            b'\'' => {
                // B43 fix: Properly distinguish char literals from lifetimes
                // Char literals: 'x', '\n', '\\', '\u{XXXX}'
                // Lifetimes: 'a, 'static — identifier after ', NO closing quote
                if i + 1 < bytes.len() {
                    let next = bytes[i + 1];
                    if next == b'\\' {
                        // Escaped char literal: '\n', '\\', '\u{...}'
                        // Skip: ' + \ + char + '
                        i += 2; // skip ' and \
                        if i < bytes.len() && bytes[i] == b'u' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                            // Unicode escape '\u{XXXX}' — skip to closing }
                            i += 2;
                            while i < bytes.len() && bytes[i] != b'}' {
                                i += 1;
                            }
                            // Skip } and closing '
                            if i < bytes.len() { i += 1; } // }
                            if i < bytes.len() && bytes[i] == b'\'' { /* closing ' handled by i += 1 below */ }
                        } else {
                            // Simple escape: '\n', '\t', etc. — skip escaped char + closing '
                            i += 1; // skip the escaped char
                            if i < bytes.len() && bytes[i] == b'\'' { /* closing ' handled by i += 1 below */ }
                        }
                    } else if (next.is_ascii_alphabetic() || next == b'_')
                        && i + 2 < bytes.len()
                        && bytes[i + 2] != b'\''
                    {
                        // Lifetime: 'a, 'static, '_  — next is alpha/underscore but
                        // char after that is NOT a closing quote → it's a lifetime, not a char
                        // Just skip the ' and let normal processing continue
                        // (don't consume anything extra)
                    } else if i + 2 < bytes.len() && bytes[i + 2] == b'\'' {
                        // Char literal: 'x' — next char followed by closing quote
                        i += 2; // skip to closing '
                    }
                    // else: lone ' at end of input, just skip it
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                // Line comment — skip to end of line
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue; // don't double-increment
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                // Block comment — skip to */
                i += 2;
                while i + 1 < bytes.len() {
                    if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        i += 1; // will be incremented again below
                        break;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    None // unmatched
}

pub fn tokenize(source: &str) -> Result<Vec<TokenWithSpan>> {
    // Phase 1a: Find rust { } blocks and extract their content
    let rust_blocks = find_rust_blocks(source);

    // Phase 1b: B02 — Find $"..." template strings with brace-depth tracking
    let template_strings = find_template_strings(source);

    // Phase 2: Replace rust block and template string interiors with spaces
    // so logos doesn't choke on Rust-specific syntax or nested-quote templates
    let tokenize_source = if rust_blocks.is_empty() && template_strings.is_empty() {
        source.to_string()
    } else {
        let mut bytes = source.as_bytes().to_vec();
        for block in &rust_blocks {
            // Replace content between { and } (exclusive) with spaces
            for b in bytes[block.content_start..block.content_end].iter_mut() {
                *b = b' ';
            }
        }
        for tmpl in &template_strings {
            // Replace the entire $"..." with spaces — we'll synthesize the token later
            for b in bytes[tmpl.start..tmpl.end].iter_mut() {
                *b = b' ';
            }
        }
        // Safety: replacing valid UTF-8 bytes with ASCII spaces always yields valid UTF-8
        String::from_utf8(bytes).unwrap()
    };

    // Phase 3: Tokenize the (possibly modified) source with logos
    let mut lexer = Token::lexer(&tokenize_source);
    let mut tokens = Vec::new();
    let source_map = SourceMap::new(source); // Use ORIGINAL source for line/col

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

    // Phase 4: Post-process — replace patterns that correspond to extracted
    // rust blocks and template strings with their proper tokens.

    // B02: Inject synthesized StringTemplate tokens for pre-scanned template strings
    if !template_strings.is_empty() {
        // Insert template tokens at the right position based on byte offset.
        // Template string regions were blanked out, so no logos tokens were emitted for them.
        // We need to insert StringTemplate tokens at the correct position in the stream.
        let mut synth_tokens = Vec::new();
        for tmpl in &template_strings {
            synth_tokens.push(TokenWithSpan::new(
                Token::StringTemplate(tmpl.content.clone()),
                Span {
                    start: tmpl.start,
                    end: tmpl.end,
                },
            ));
        }
        // Merge: tokens are sorted by start position; insert synth tokens in order
        let mut merged = Vec::with_capacity(tokens.len() + synth_tokens.len());
        let mut si = 0;
        for tok in tokens.iter() {
            while si < synth_tokens.len() && synth_tokens[si].span.start < tok.span.start {
                merged.push(synth_tokens[si].clone());
                si += 1;
            }
            merged.push(tok.clone());
        }
        while si < synth_tokens.len() {
            merged.push(synth_tokens[si].clone());
            si += 1;
        }
        tokens = merged;
    }

    // Phase 5: Replace Token::Rust + Token::LBrace + Token::RBrace patterns
    // that correspond to extracted rust blocks with Token::RustBlock(content)
    if rust_blocks.is_empty() {
        return Ok(tokens);
    }

    let mut result_tokens = Vec::with_capacity(tokens.len());
    let mut idx = 0;
    while idx < tokens.len() {
        if tokens[idx].token == Token::Rust {
            // Check if this Rust keyword position matches a known block
            if let Some(block) = rust_blocks
                .iter()
                .find(|b| b.rust_keyword_start == tokens[idx].span.start)
            {
                // Emit a single RustBlock token spanning the whole `rust { ... }`
                result_tokens.push(TokenWithSpan::new(
                    Token::RustBlock(block.content.clone()),
                    Span {
                        start: block.rust_keyword_start,
                        end: block.closing_brace_end,
                    },
                ));
                // Skip the Rust token + LBrace + RBrace (3 tokens)
                idx += 3;
                continue;
            }
        }
        result_tokens.push(tokens[idx].clone());
        idx += 1;
    }

    Ok(result_tokens)
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

    // B42: find_rust_blocks must skip `rust` inside // comments
    #[test]
    fn test_find_rust_blocks_skips_line_comments() {
        let source = "// This uses rust {} interop\nlet x = 1";
        let blocks = find_rust_blocks(source);
        assert!(blocks.is_empty(), "Should not find rust block inside // comment");
    }

    #[test]
    fn test_find_rust_blocks_skips_block_comments() {
        let source = "/* rust { some code } */\nlet x = 1";
        let blocks = find_rust_blocks(source);
        assert!(blocks.is_empty(), "Should not find rust block inside /* */ comment");
    }

    #[test]
    fn test_find_rust_blocks_skips_string_literals() {
        let source = r#"let x = "rust { }"
let y = 1"#;
        let blocks = find_rust_blocks(source);
        assert!(blocks.is_empty(), "Should not find rust block inside string literal");
    }

    #[test]
    fn test_find_rust_blocks_finds_real_block_after_comment() {
        let source = "// This uses rust interop\nrust { let x = 1; }";
        let blocks = find_rust_blocks(source);
        assert_eq!(blocks.len(), 1, "Should find exactly one rust block");
        assert!(blocks[0].content.contains("let x = 1;"));
    }

    // B43: find_balanced_brace must handle lifetimes properly
    #[test]
    fn test_balanced_brace_with_lifetime() {
        // Lifetime 'a should NOT be treated as char literal start
        let source = "{ fn peek<'a>(tokens: &'a [Token]) -> &'a Token { tokens[0] } }";
        let result = find_balanced_brace(source, 0);
        assert!(result.is_some(), "Should find balanced brace with lifetimes");
        let (content_end, _) = result.unwrap();
        // The content_end should be at the final }, not consumed by lifetime confusion
        assert_eq!(source.as_bytes()[content_end], b'}');
    }

    #[test]
    fn test_balanced_brace_with_char_literal() {
        // Real char literals should still work
        let source = "{ let ch = 'x'; let b = '{'; }";
        let result = find_balanced_brace(source, 0);
        assert!(result.is_some(), "Should find balanced brace with char literals");
    }

    #[test]
    fn test_balanced_brace_with_escaped_char() {
        let source = r"{ let ch = '\n'; let b = '\\'; }";
        let result = find_balanced_brace(source, 0);
        assert!(result.is_some(), "Should find balanced brace with escaped chars");
    }

    #[test]
    fn test_balanced_brace_lifetime_does_not_consume_braces() {
        // Critical: lifetime 'a followed by braces should not consume them
        let source = "{ fn f<'a>(x: &'a str) -> &'a str { x } let y = 1; }";
        let result = find_balanced_brace(source, 0);
        assert!(result.is_some());
        let (content_end, brace_end) = result.unwrap();
        // Should match the outermost closing }
        assert_eq!(brace_end, source.len(), "Should close at outermost brace, got content_end={content_end}");
    }

    // B02: Template strings with nested quotes inside interpolation
    #[test]
    fn test_find_template_strings_simple() {
        let source = r#"$"Hello {name}""#;
        let templates = find_template_strings(source);
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].content, "Hello {name}");
    }

    #[test]
    fn test_find_template_strings_nested_quotes() {
        // B02: The core failing case — fn("arg") inside template interpolation
        let source = r#"$"result: {fn("arg")}""#;
        let templates = find_template_strings(source);
        assert_eq!(templates.len(), 1, "Should find exactly one template string");
        assert_eq!(templates[0].content, r#"result: {fn("arg")}"#);
    }

    #[test]
    fn test_find_template_strings_multiple_nested_quotes() {
        let source = r#"$"{replace("hello", "world")} done""#;
        let templates = find_template_strings(source);
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].content, r#"{replace("hello", "world")} done"#);
    }

    #[test]
    fn test_tokenize_template_nested_quotes() {
        // B02: Full tokenization of template with nested quotes
        let source = r#"let x = $"result: {fn("arg")}""#;
        let tokens = tokenize(source).unwrap();
        // Should have: Let, Ident("x"), Assign, StringTemplate("result: {fn(\"arg\")}")
        assert_eq!(tokens.len(), 4, "Tokens: {:?}", tokens.iter().map(|t| &t.token).collect::<Vec<_>>());
        assert!(matches!(&tokens[3].token, Token::StringTemplate(s) if s == r#"result: {fn("arg")}"#),
            "Expected StringTemplate with nested quotes, got {:?}", tokens[3].token);
    }

    #[test]
    fn test_find_template_strings_no_false_positive_regular_string() {
        // Regular strings should NOT be captured
        let source = r#"let x = "hello world""#;
        let templates = find_template_strings(source);
        assert!(templates.is_empty(), "Regular strings should not be captured");
    }
}
