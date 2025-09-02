use std::env;
use std::io;
use std::process;

fn match_characters(characters: &str, input_line: &str) -> bool {
    return input_line.chars().any(|c| characters.contains(c))
}

fn not_match_characters(characters: &str, input_line: &str) -> bool {
    return input_line.chars().any(|c| !characters.contains(c))
}

fn first_char_is_alphanumeric_or_underscore(input_line: &str) -> bool {
    return input_line.chars().nth(0).unwrap().is_alphanumeric() || 
            input_line.chars().nth(0).unwrap() == '_';
}

fn line_consists_of_alphanumeric_and_underscore(input_line: &str) -> bool {
    return input_line.chars().any(|c| c.is_alphanumeric() || c == '_')
}

fn ascii_digit_pattern(input_line: &str) -> bool {
    return input_line.chars().any(|c| c.is_ascii_digit())
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern.starts_with("^") {
        let pattern_without_anchor = &pattern[1..];
        return match_pattern_at_start(input_line, pattern_without_anchor);
    }

    if pattern == "\\d" {
        return ascii_digit_pattern(input_line);
    } else if pattern == "\\w" {
        return line_consists_of_alphanumeric_and_underscore(input_line)
    } else if pattern.chars().count() > 2 && pattern.starts_with("[^") && pattern.ends_with("]") {
        let exclude = &pattern[2..pattern.len() - 1];
        return not_match_characters(exclude, input_line)
    } else if pattern.chars().count() > 2 && pattern.starts_with('[') && pattern.ends_with(']') {
        let charmatch = &pattern[1..pattern.len() - 1];
        return match_characters(charmatch, input_line)
    } else if pattern.chars().count() == 1 {
        return input_line.contains(pattern);
    } 
    return check_pattern(input_line, pattern)
}

fn check_pattern(input_line: &str, pattern: &str) -> bool {
    let mut expression = pattern;
    let mut input_iterator = input_line;

    while expression.len() > 0 && input_iterator.len() > 0 {
        if expression.starts_with(r"\d") && input_iterator.chars().nth(0).unwrap().is_digit(10) {
            expression = &expression[2..];
            input_iterator = &input_iterator[1..];
        } else if expression.starts_with(r"\w") && first_char_is_alphanumeric_or_underscore(input_iterator) {
            expression = &expression[2..];
            input_iterator = &input_iterator[1..];
        } else if expression.starts_with(' ') && input_iterator.chars().nth(0).unwrap() == ' ' {
            expression = &expression[1..];
            input_iterator = &input_iterator[1..];
        } else if input_iterator.contains(expression) {
            println!("Pattern: {} matches input: {}", pattern, input_line);
            return true;
        } else {
            input_iterator = &input_iterator[1..];
        }
    }
    return false;
}

fn check_pattern_anchored(input_line: &str, pattern: &str) -> bool {
    let mut expression = pattern;
    let mut input_iterator = input_line;

    while expression.len() > 0 && input_iterator.len() > 0 {
        if expression.starts_with(r"\d") && input_iterator.chars().nth(0).unwrap().is_digit(10) {
            expression = &expression[2..];
            input_iterator = &input_iterator[1..];
        } else if expression.starts_with(r"\w") && first_char_is_alphanumeric_or_underscore(input_iterator) {
            expression = &expression[2..];
            input_iterator = &input_iterator[1..];
        } else if expression.starts_with(' ') && input_iterator.chars().nth(0).unwrap() == ' ' {
            expression = &expression[1..];
            input_iterator = &input_iterator[1..];
        } else if input_iterator.starts_with(expression) {
            return true;
        } else {
            return false;
        }
    }
    return expression.len() == 0;
}

fn match_pattern_at_start(input_line: &str, pattern: &str) -> bool {
    if pattern == "\\d" {
        return !input_line.is_empty() && input_line.chars().nth(0).unwrap().is_ascii_digit();
    } else if pattern == "\\w" {
        return first_char_is_alphanumeric_or_underscore(input_line);
    } else if pattern.chars().count() > 3 && pattern.starts_with("[^") && pattern.ends_with("]") {
        if input_line.is_empty() {
            return false;
        }
        let exclude = &pattern[2..pattern.len() - 1];
        let first_character = input_line.chars().nth(0).unwrap();
        return !exclude.contains(first_character);
    } else if pattern.chars().count() > 2 && pattern.starts_with("[") && pattern.ends_with("]") {
        if input_line.is_empty() {
            return false;
        }
        let character_match = &pattern[1..pattern.len() - 1];
        let first_character = input_line.chars().nth(0).unwrap();
        return character_match.contains(first_character);
    } 
    return check_pattern_anchored(input_line, pattern);
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