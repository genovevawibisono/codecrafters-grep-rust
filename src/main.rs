use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern == "\\d" {
        return input_line.chars().any(|c| c.is_ascii_digit());
    } else if pattern == "\\w" {
        return input_line.chars().any(|c| c.is_alphanumeric() || c == '_');
    } else if pattern.chars().count() > 2 && pattern.starts_with("[^") && pattern.ends_with("]") {
        let exclude = &pattern[2..pattern.len() - 1];
        return input_line.chars().any(|c| !exclude.contains(c));
    } else if pattern.chars().count() > 2 && pattern.starts_with('[') && pattern.ends_with(']') {
        let charmatch = &pattern[1..pattern.len() - 1];
        return input_line.chars().any(|c| charmatch.contains(c));
    } else if pattern.chars().count() == 1 {
        return input_line.contains(pattern);
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");
    
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }
    
    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    
    // Remove the trailing newline character
    let input_line = input_line.trim_end();
    
    if match_pattern(&input_line, &pattern) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}