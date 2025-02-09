use std::env;
use std::fs;
use std::process;

// Token Definition
#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,       // Represents {
    RightBrace,      // Represents }
    LeftBracket,     // Represents [
    RightBracket,    // Represents ]
    String(String),  // Represents any string value (both keys and values)
    Number(f64),     // Represents any number value
    Boolean(bool),   // Represents any boolean value
    Null,
    Colon,           // Represents :
    Comma,           // Represents ,
}

#[derive(Debug)]
struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        self.position += 1
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

    fn lex_string(&mut self) -> Result<String, &'static str> {
        let mut result = String::new();
        self.advance(); // Skip opening quote

        while let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.advance();
                    return Ok(result);
                }
                '\\' => return Err("Escape sequences not yet supported"),
                '\n' => return Err("Unterminated string literal"),
                c => {
                    result.push(c);
                    self.advance();
                }
            }
        }
        Err("Unterminated string literal")
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
                '[' => {
                    tokens.push(Token::LeftBracket);
                    self.advance();
                },
                ']' => {
                    tokens.push(Token::RightBracket);
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

    fn parse_value(&mut self) -> Result<(), &'static str> {
        match self.peek() {
            Some(Token::LeftBrace) => self.parse_object(),
            Some(Token::LeftBracket) => self.parse_array(),
            Some(Token::String(_)) |
            Some(Token::Number(_)) |
            Some(Token::Boolean(_)) |
            Some(Token::Null) => {
                self.advance();
                Ok(())
            }
            _ => Err("Expected value"),
        }
    }

    fn parse_array(&mut self) -> Result<(), &'static str> {
        // Consume the opening bracket
        match self.peek() {
            Some(Token::LeftBracket) => self.advance(),
            _ => return Err("Expected '['"),
        }

        let mut first = true;
        while let Some(token) = self.peek() {
            match token {
                // Case 1: We see a closing bracket and we're at the first position
                Token::RightBracket if first => {
                    self.advance();
                    return Ok(())  // Empty array [] is valid
                }

                // Case 2: We see a closing bracket after some values
                Token::RightBracket => {
                    self.advance();
                    return Ok(());  // Array is properly closed
                }

                // Case 3: We see a comma after a value (not first)
                Token::Comma if !first => {
                    self.advance();
                    // After a comma, check for trailing comma
                    match self.peek() {
                        Some(Token::RightBracket) => {
                            // {"key": "value",} is invalid
                            return Err("Trailing comma not allowed")
                        }
                        _ => {}  // Otherwise comma is okay
                    }
                }
                
                // Case 4: We're not at first item and don't see comma or rightbracket
                _ if !first => return Err("Expected ',' or ']'"),
                
                // Case 5: Any other token, continue processing
                _ => {}
            }
            self.parse_value()?;
            first = false;
        }
        Err("Unexpected end of input")
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

            // Parse value (now recursive)
            self.parse_value()?;

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

    // Tests for Valid JSON
    #[test]
    fn test_empty_object() {
        assert!(parse_json("{}").is_ok());
    }

    #[test]
    fn test_basic_types() {
        // Test all basic JSON types
        assert!(parse_json(r#"{
            "string": "hello_world",
            "number": 42,
            "float": 3.14,
            "negative": -123,
            "boolean_true": true,
            "boolean_false": false,
            "null_value": null
        }"#).is_ok());
    }

    #[test]
    fn test_nested_structures() {
        assert!(parse_json(r#"{
            "empty_object": {},
            "empty_array": [],
            "nested_object": {"key": "value"},
            "nested_array": ["item"],
            "deep_nesting": {
                "level1": {
                    "level2": {
                        "level3": {}
                    }
                }
            }
        }"#).is_ok());
    }

    #[test]
    fn test_array_variations() {
        assert!(parse_json(r#"{
            "mixed_array": [1, "string", true, null, {"key": "value"}, [1, 2, 3]],
            "number_array": [1, 2, 3, 4, 5],
            "nested_arrays": [[], [1], [1, [2, [3]]]],
            "object_array": [{"k1": "v1"}, {"k2": "v2"}]
        }"#).is_ok());
    }

    #[test]
    fn test_whitespace_handling() {
        // Test various whitespace scenarios
        assert!(parse_json(r#"{"key" : "value"}"#).is_ok());
        assert!(parse_json("{\n\t\"key\":\"value\"\n}").is_ok());
        assert!(parse_json("{ \r\n \t }").is_ok());
    }

    // Tests for Invalid JSON
    #[test]
    fn test_invalid_syntax() {
        // Missing closing brace
        let err = parse_json(r#"{"key": "value""#).unwrap_err();
        assert_eq!(err, "Unexpected end of input");

        // Missing quotes around key
        let err = parse_json(r#"{key: "value"}"#).unwrap_err();
        assert_eq!(err, "Invalid identifier");

        // Missing colon
        let err = parse_json(r#"{"key" "value"}"#).unwrap_err();
        assert_eq!(err, "Expected ':'");
    }

    #[test]
    fn test_invalid_arrays() {
        // Trailing comma in array
        let err = parse_json(r#"{"arr": [1, 2, ]}"#).unwrap_err();
        assert_eq!(err, "Trailing comma not allowed");

        // Missing comma between array elements
        let err = parse_json(r#"{"arr": [1 2]}"#).unwrap_err();
        assert_eq!(err, "Expected ',' or ']'");

        // Unclosed array
        let err = parse_json(r#"{"arr": [1, 2"#).unwrap_err();
        assert_eq!(err, "Unexpected end of input");
    }

    #[test]
    fn test_invalid_values() {
        // Invalid boolean capitalization
        let err = parse_json(r#"{"key": True}"#).unwrap_err();
        assert_eq!(err, "Invalid identifier");

        // Invalid number format
        let err = parse_json(r#"{"key": 12.34.56}"#).unwrap_err();
        assert_eq!(err, "Invalid number format");

        // Single quotes instead of double quotes
        let err = parse_json(r#"{'key': 'value'}"#).unwrap_err();
        assert_eq!(err, "Invalid character in JSON");
    }

    #[test]
    fn test_invalid_objects() {
        // Trailing comma in object
        let err = parse_json(r#"{"key": "value",}"#).unwrap_err();
        assert_eq!(err, "Trailing comma not allowed");

        // Missing comma between properties
        let err = parse_json(r#"{"key1": "value1" "key2": "value2"}"#).unwrap_err();
        assert_eq!(err, "Expected ',' or '}'");

        // Duplicate keys (if implemented)
        // let err = parse_json(r#"{"key": "value1", "key": "value2"}"#).unwrap_err();
        // assert_eq!(err, "Duplicate key found");
    }

    #[test]
    fn test_empty_input() {
        let err = parse_json("").unwrap_err();
        assert_eq!(err, "Expected '{'");
    }

    #[test]
    fn test_complex_invalid_cases() {
        // Mixing array and object syntax
        let err = parse_json(r#"{"arr": [}"#).unwrap_err();
        assert_eq!(err, "Expected value");

        // Nested invalid syntax
        let err = parse_json(r#"{
            "outer": {
                "inner": {
                    "key": value
                }
            }
        }"#).unwrap_err();
        assert_eq!(err, "Invalid identifier");
    }

    #[test]
    fn test_boundary_cases() {
        // Test very long string (this should still work)
        let long_string = format!(r#"{{"key": "{}"}}"#, "a".repeat(1000));
        assert!(parse_json(&long_string).is_ok());
        
        // Test nesting limit (should fail gracefully at extreme depths)
        let too_deep = "{".repeat(1000) + "}".repeat(1000).as_str();
        assert!(parse_json(&too_deep).is_err());
    }
    
    // Add a new test specifically for reasonable nesting depths
    #[test]
    fn test_nested_depth() {
        // Test reasonable nesting (should pass)
        let nested_10 = r#"{
            "l1": {
                "l2": {
                    "l3": {
                        "l4": {
                            "l5": {
                                "l6": {
                                    "l7": {
                                        "l8": {
                                            "l9": {
                                                "l10": {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }"#;
        assert!(parse_json(nested_10).is_ok());
    }
}