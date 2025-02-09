use std::env;
use std::fs;
use std::process;

// Token Definition
#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,       // Represents {
    RightBrace,      // Represents }
    String(String),  // Represents any string value (both keys and values)
    Colon,           // Represents :
    Comma,           // Represents ,
}

// Lexer Implementation
#[derive(Debug)]
struct Lexer {
    input: Vec<char>, // The input string converted into a vector of characters
    position: usize,  // Current position in the input
}

impl Lexer {
    // Constructor
    fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    // Look at current character without consuming
    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    // Move to next character
    fn advance(&mut self) {
        self.position += 1;
    }

    // Handles string parsing, including quotes
    fn lex_string(&mut self) -> Result<String, &'static str> {
        let mut result = String::new();
        self.advance(); // Skip opening quote

        while let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.advance(); // Skip closing quote
                    return Ok(result);
                },
                '\\' => {
                    return Err("Error sequences not yet supported");
                },
                '\n' => {
                    return Err("Unterminated string literal");
                },
                c => {
                    result.push(c);
                    self.advance();
                },
            }
        }
        Err("Unterminated string literal")
    }

    // Main lexing function that produces tokens
    fn lex_tokens(&mut self) -> Result<Vec<Token>, &'static str> {
        let mut tokens = Vec::new();

        while let Some(c) = self.peek() {
            match c {
                '{' => {
                    tokens.push(Token::LeftBrace);
                    self.advance();
                },
                '}' => {
                    tokens.push(Token::RightBrace);
                    self.advance();
                },
                ':' => {
                    tokens.push(Token::Colon);
                    self.advance();
                },
                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                },
                '"' => {
                    let string = self.lex_string()?;
                    tokens.push(Token::String(string));
                },
                c if c.is_whitespace() => {
                    self.advance();
                },
                _ => return Err("Invalid character in JSON"),
            }
        }
        Ok(tokens)
    } 
}

// Parser Implementation
struct Parser {
    tokens: Vec<Token>, // The tokens produced by the lexer
    position: usize,    // Current position in token stream
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn parse_object(&mut self) -> Result<(), &'static str> {
        //Expect opening brace
        match self.peek() {
            Some(Token::LeftBrace) => self.advance(),
            _ => return Err("Expected '{'"),
        }

        let mut first = true;
        while let Some(token) = self.peek() {
            match token {
                // Case 1: We see a closing brace and we're at the first position
                Token::RightBrace if first => {
                    self.advance();
                    return Ok(()); // Empty object {} is valid
                }

                // Case 2: We see a closing brace after some key-value pairs
                Token::RightBrace => {
                    self.advance();
                    return Ok(());  // Object is properly closed
                }

                // Case 3: We see a comma after a key-value pair (not first)
                Token::Comma if !first => {
                    self.advance();
                    // After a comma, check for trailing comma
                    match self.peek() {
                        Some(Token::RightBrace) => {
                            // {"key": "value",} is invalid
                            return Err("Trailing comma not allowed")
                        }
                        _ => {}  // Otherwise comma is okay
                    }
                }

                // Case 4: We're not at first item and don't see comma
                _ if !first => {
                    // If we've already processed a pair but don't see 
                    // a comma or closing brace, it's an error
                    return Err("Expected ',' or '}'")
                }

                // Case 5: Any other token, continue processing
                _ => {}
            }

            // Parse key
            match self.peek() {
                Some(Token::String(_)) => self.advance(),
                _ => return Err("Expected string key"),
            }

            // Parse colon
            match self.peek() {
                Some(Token::Colon) => self.advance(),
                _ => return Err("Expected ':'"),
            }

            // Parse value (only strings supported for now)
            match self.peek() {
                Some(Token::String(_)) => self.advance(),
                _ => return Err("Expected string value"),
            }

            first = false;
        }

        Err("Unexpected end of input")
    }
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

    // Perform lexical analysis
    let mut lexer = Lexer::new(&content);
    let tokens = match lexer.lex_tokens() {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Invalid JSON: {}", e);
            process::exit(1);
        }
    };

    // Parse the tokens
    let mut parser = Parser::new(tokens);
    match parser.parse_object() {
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

    fn parse_json(input: &str) -> Result<(), &'static str> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.lex_tokens()?;
        let mut parser = Parser::new(tokens);
        parser.parse_object()
    }

    #[test]
    fn test_valid_empty_object() {
        assert!(parse_json("{}").is_ok());
    }

    #[test]
    fn test_valid_single_pair() {
        assert!(parse_json(r#"{"key": "value"}"#).is_ok());
    }

    #[test]
    fn test_valid_multiple_pairs() {
        assert!(parse_json(r#"{"key": "value", "key2": "value"}"#).is_ok());
    }

    #[test]
    fn test_invalid_trailing_comma() {
        assert!(parse_json(r#"{"key": "value",}"#).is_err());
    }

    #[test]
    fn test_invalid_missing_quotes() {
        assert!(parse_json(r#"{key: "value"}"#).is_err());
    }

    #[test]
    fn test_invalid_missing_colon() {
        assert!(parse_json(r#"{"key" "value"}"#).is_err());
    }
}