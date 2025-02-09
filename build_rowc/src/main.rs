use std::env; //Access command line arguments
use std::fs;  //File System operations
use std::io::{self, Read};
use std::process; //Program exit functionality

struct Counts {
    lines: usize,
    words: usize,
    bytes: usize,
}

#[derive(Clone)]
enum Input {
    File(String),
    Stdin,
}

fn main() {
    let args: Vec::<String> = env::args().collect();

    let (flag, input) = match args.len() {
        1 => (None, Input::Stdin),
        2 => {
            if args[1].starts_with('-') {
                (Some(args[1].as_str()), Input::Stdin)
            } else {
                (None, Input::File(args[1].clone()))
            }
        },
        3 => (Some(args[1].as_str()), Input::File(args[2].clone())),
        _ => {
            eprintln!("Usage: {} [-c|-l|-w|-m] [file]", args[0]);
            process::exit(1);
        }
    };

    if let Some(f) = flag {
        if !vec!["-c", "-l", "-w", "-m"].contains(&f) {
            eprintln!("Usage: {} [-c|-l|-w|-m] [file]", args[0]);
            process::exit(1);
        }
    }

    let result = match (flag, &input) {
        (Some("-c"), input) => count_bytes(input.clone()).map(|c| format!("{:>8}", c)),
        (Some("-l"), input) => count_lines(input.clone()).map(|c| format!("{:>8}", c)),
        (Some("-w"), input) => count_words(input.clone()).map(|c| format!("{:>8}", c)),
        (Some("-m"), input) => count_chars(input.clone()).map(|c| format!("{:>8}", c)),
        (None, input) => count_all(input.clone()).map(|c| format!("{:>8} {:>8} {:>8}", c.lines, c.words, c.bytes)),
        _ => unreachable!(),
    };

    match result {
        Ok(output) => {
            match input {
                Input::File(path) => println!("{} {}", output, path),
                Input::Stdin => println!("{}", output),
            }
        },
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}

fn read_to_string(input: Input) -> io::Result<String> {
    match input {
        Input::File(path) => fs::read_to_string(path),
        Input::Stdin => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn read_to_bytes(input: Input) -> io::Result<Vec<u8>> {
    match input {
        Input::File(path) => fs::read(path),
        Input::Stdin => {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn count_bytes(input: Input) -> io::Result<usize> {
    let contents = read_to_bytes(input)?;
    Ok(contents.len())
}

fn count_lines(input: Input) -> io::Result<usize> {
    let contents = read_to_string(input)?;
    Ok(contents.lines().count())
}

fn count_words(input: Input) -> io::Result<usize> {
    let contents = read_to_string(input)?;
    Ok(contents.split_whitespace().count())
}

fn count_chars(input: Input) -> io::Result<usize> {
    let contents = read_to_string(input)?;
    Ok(contents.chars().count())
}

fn count_all(input: Input) -> io::Result<Counts> {
    let contents = read_to_string(input)?;
    let bytes = contents.as_bytes().len();
    let words = contents.split_whitespace().count();
    let lines = contents.lines().count();

    Ok(Counts {
        lines,
        words,
        bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_count_chars() {
        let test_content = "Hello, 世界!\n";
        let test_filename = "test_chars.txt";
        
        let mut file = File::create(test_filename).unwrap();
        file.write_all(test_content.as_bytes()).unwrap();
        
        let result = count_chars(Input::File(test_filename.to_string()));
        fs::remove_file(test_filename).unwrap();
        
        assert_eq!(result.unwrap(), 9);
    }

    #[test]
    fn test_count_all() {
        let test_content = "Line 1 words\nLine 2 more\nLine 3 text\n";
        let test_filename = "test_all.txt";
        
        let mut file = File::create(test_filename).unwrap();
        file.write_all(test_content.as_bytes()).unwrap();
        
        let result = count_all(Input::File(test_filename.to_string())).unwrap();
        fs::remove_file(test_filename).unwrap();
        
        assert_eq!(result.lines, 3);
        assert_eq!(result.words, 9);
        assert_eq!(result.bytes, test_content.len());
    }
}