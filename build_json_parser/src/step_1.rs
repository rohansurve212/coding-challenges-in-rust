use std::env;
use std::fs;
use std::process;

#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,
    RightBrace,
}

fn lex(input: &str) -> Result<Vec<Token>, &'static str> {
    let mut tokens: Vec<Token> = Vec::new();

    for c in input.chars() {
        match c {
            '{' => tokens.push(Token::LeftBrace),
            '}' => tokens.push(Token::RightBrace),
            // Ignore whitespace
            c if c.is_whitespace() => continue,
            // Any other character is invalid for our simple parser
            _ => return Err("Invalid character found")
        }
    }

    Ok(tokens)
}

fn parse(tokens: &Vec<Token>) -> Result<(), &'static str> {
    // For this simple case, we justneed to verify we have exactly 
    // one left brace followed by one right brace.
    if tokens.len() != 2 {
        return Err("Invalid JSON structure.");
    }

    if tokens[0] != Token::LeftBrace || tokens[1] != Token::RightBrace {
        return Err("Invalid JSON structure.");
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            process::exit(1);
        }
    };

    // Perform Lexical Analysis
    let tokens = match lex(&content) {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Invalid JSON: {}", e);
            process::exit(1);
        }
    };

    // Parse the tokens
    match parse(&tokens) {
        Ok(_) => {
            println!("Valid JSON");
            process::exit(0);
        },
        Err(e) => {
            println!("Invalid JSON: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_empty_object() {
        let input = "{}";
        let tokens = lex(input).unwrap();
        assert!(parse(&tokens).is_ok());
    }

    #[test]
    fn test_invalid_single_brace() {
        let input = "{";
        let tokens = lex(input).unwrap();
        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn test_invalid_empty() {
        let input = "";
        let tokens = lex(input).unwrap();
        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn test_invalid_with_content() {
        let input = "{a}";
        assert!(lex(input).is_err());
    }
}
