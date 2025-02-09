use std::env;
use std::fs;
use std::process;

// Token Definition
#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,       // Represents {
    RightBrace,      // Represents }
    String(String),  // Represents any string value (both keys and values)
    Number(f64),     // Represents any number value
    Boolean(bool),   // Represents any boolean value
    Null,
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
                '\\' => return Err("Error sequences not yet supported"),
                '\n' => return Err("Unterminated string literal"),
                c => {
                    result.push(c);
                    self.advance();
                },
            }
        }
        Err("Unterminated string literal")
    }

    fn read_while<F>(&mut self, predicate: F) -> String 
    where F: Fn(char) -> bool {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if predicate(c) {
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn lex_number(&mut self) -> Result<f64, &'static str> {
        let number_str = self.read_while(|c| {
            c.is_ascii_digit() || c == '-' || c == '.' || c == 'e' || c == 'E' || c == '+'
        });

        number_str.parse::<f64>()
        .map_err(|_| "Invalid number format")
    }

    fn lex_identifier(&mut self) -> Result<Token, &'static str> {
        let identifier = self.read_while(|c| c.is_ascii_alphabetic());

        match identifier.as_str() {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            "null" => Ok(Token::Null),
            _ => Err("Invalid identifier")
        }
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
                c if c.is_ascii_digit() || c == '-' => {
                    let number = self.lex_number()?;
                    tokens.push(Token::Number(number));
                },
                c if c.is_ascii_alphabetic() => {
                    let token = self.lex_identifier()?;
                    tokens.push(token);
                }
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
    tokens: Vec<Token>,
    position: usize,
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
        self.position += 1
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

                // Case 4: We're not at first item and don't see comma or rightbrace
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
                Some(Token::String(_)) |
                Some(Token::Number(_)) |
                Some(Token::Boolean(_)) |
                Some(Token::Null) => self.advance(),
                _ => return Err("Expected value"),
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

    let mut lexer = Lexer::new(&content);
    let tokens = match lexer.lex_tokens() {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Invalid JSON: {}", e);
            process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    match parser.parse_object() {
        Ok(_) => {
            println!("Valid JSON");
            process::exit(0);
        }
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
    fn test_valid_boolean() {
        assert!(parse_json(r#"{"key": true}"#).is_ok());
        assert!(parse_json(r#"{"key": false}"#).is_ok());
    }

    #[test]
    fn test_valid_null() {
        assert!(parse_json(r#"{"key": null}"#).is_ok());
    }

    #[test]
    fn test_valid_number() {
        assert!(parse_json(r#"{"key": 123}"#).is_ok());
        assert!(parse_json(r#"{"key": -123}"#).is_ok());
        assert!(parse_json(r#"{"key": 123.456}"#).is_ok());
    }

    #[test]
    fn test_invalid_boolean_case() {
        assert!(parse_json(r#"{"key": True}"#).is_err());
        assert!(parse_json(r#"{"key": FALSE}"#).is_err());
    }

    #[test]
    fn test_valid_mixed_values() {
        assert!(parse_json(r#"{
            "key1": true,
            "key2": false,
            "key3": null,
            "key4": "value",
            "key5": 101
        }"#).is_ok());
    }
}
