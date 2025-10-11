use std::fs;
use livac::lexer::tokenize;

fn main() {
    let content = fs::read_to_string("examples.liva").unwrap();
    let tokens = tokenize(&content).unwrap();

    println!("Tokens found: {}", tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?} at span {:?}",
            i,
            token.token,
            token.span
        );
    }
}
