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
    if input_line.is_empty() {
        return false;
    }
    return input_line.chars().nth(0).unwrap().is_alphanumeric() || 
            input_line.chars().nth(0).unwrap() == '_';
}

fn last_char_is_alphanumeric_or_underscore(input_line: &str) -> bool {
    if input_line.is_empty() {
        return false;
    }
    let last = input_line.chars().last().unwrap();
    return last.is_alphanumeric() || last == '_';
}

fn line_consists_of_alphanumeric_and_underscore(input_line: &str) -> bool {
    return input_line.chars().any(|c| c.is_alphanumeric() || c == '_')
}

fn ascii_digit_pattern(input_line: &str) -> bool {
    return input_line.chars().any(|c| c.is_ascii_digit())
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let pattern = expand_plus(&pattern);
    println!("DEBUG: pattern = {:?}, text = {:?}", pattern, input_line);

    if pattern.starts_with("^") && pattern.ends_with('$') {
        let pattern_core = &pattern[1..pattern.len() - 1];
        return input_line == pattern_core;
    }

    if pattern.starts_with("^") {
        let pattern_without_anchor = &pattern[1..];
        return match_pattern_at_start(input_line, pattern_without_anchor);
    }

    if pattern.ends_with('$') {
        let pattern_without_anchor = &pattern[..pattern.len() - 1];
        return match_pattern_at_end(input_line, pattern_without_anchor);
    }

    if pattern == "\\d" {
        return ascii_digit_pattern(input_line);
    } else if pattern == "\\w" {
        return line_consists_of_alphanumeric_and_underscore(input_line);
    } else if pattern.chars().count() > 2 && pattern.starts_with("[^") && pattern.ends_with("]") {
        let exclude = &pattern[2..pattern.len() - 1];
        return not_match_characters(exclude, input_line);
    } else if pattern.chars().count() > 2 && pattern.starts_with('[') && pattern.ends_with(']') {
        let charmatch = &pattern[1..pattern.len() - 1];
        return match_characters(charmatch, input_line);
    } else if pattern.chars().count() == 1 {
        return input_line.contains(&pattern);
    }

    // Try pattern at every position (substring search)
    for start in 0..=input_line.len() {
        if check_pattern(&input_line[start..], &pattern) {
            return true;
        }
    }

    return false;
}

fn check_pattern(input: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }

    if pattern.starts_with("\\d") {
        if !input.is_empty() && input.chars().next().unwrap().is_ascii_digit() {
            return check_pattern(&input[1..], &pattern[2..]);
        } else {
            return false;
        }
    }

    if pattern.starts_with("\\w") {
        if !input.is_empty() && (input.chars().next().unwrap().is_alphanumeric() || input.chars().next().unwrap() == '_') {
            return check_pattern(&input[1..], &pattern[2..]);
        } else {
            return false;
        }
    }

    let mut chars = pattern.chars();
    let first = chars.next().unwrap();
    let rest = chars.as_str();

    if rest.starts_with('*') {
        let rest_after_star = &rest[1..];
        let mut i = 0;
        let input_chars: Vec<char> = input.chars().collect();

        while i <= input_chars.len() && (i == 0 || first == '.' || input_chars[i - 1] == first) {
            let remaining: String = input_chars[i..].iter().collect();
            if check_pattern(&remaining, rest_after_star) {
                return true;
            }
            i += 1;
        }
        return false;
    }

    if rest.starts_with('?') {
        let rest_after_q = &rest[1..];
        // 0 occurrences
        if check_pattern(input, rest_after_q) {
            return true;
        }
        // 1 occurrence if first matches (or if first is '.')
        if !input.is_empty() && (first == '.' || input.chars().next().unwrap() == first) {
            return check_pattern(&input[1..], rest_after_q);
        }
        return false;
    }


    if !input.is_empty() && (first == '.' || input.chars().next().unwrap() == first) {
        return check_pattern(&input[1..], rest);
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

fn match_pattern_at_end(input_line: &str, pattern: &str) -> bool {
    if pattern == "\\d" {
        return !input_line.is_empty() && input_line.chars().last().unwrap().is_ascii_digit();
    } else if pattern == "\\w" {
        return last_char_is_alphanumeric_or_underscore(input_line);
    } else if pattern.starts_with("[^") && pattern.ends_with("]") {
        if input_line.is_empty() {
            return false;
        }
        let exclude = &pattern[2..pattern.len() - 1];
        let last_character = input_line.chars().last().unwrap();
        return !exclude.contains(last_character);
    } else if pattern.starts_with("[") && pattern.ends_with("]") {
        if input_line.is_empty() {
            return false;
        }
        let character_match = &pattern[1..pattern.len() - 1];
        let last_character = input_line.chars().last().unwrap();
        return character_match.contains(last_character);
    }
    // default: just check if input ends with pattern
    return input_line.ends_with(pattern);
}

fn expand_plus(pattern: &str) -> String {
    let mut expanded = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i + 1] == '+' {
            // treat the previous char/group as one + star
            if chars[i] == ']' {
                // find matching [
                let mut j = i;
                while j > 0 && chars[j] != '[' {
                    j -= 1;
                }
                let elem: String = chars[j..=i].iter().collect();
                expanded.push_str(&elem);
                expanded.push_str(&elem);
                expanded.push('*');
            } else {
                let elem = chars[i];
                expanded.push(elem);
                expanded.push(elem);   // start of repetition
                expanded.push('*');
            }
            i += 2;
        } else {
            expanded.push(chars[i]);
            i += 1;
        }
    }

    return expanded;
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");
    
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }
    
    let pattern = env::args().nth(2).unwrap();
    let pattern = expand_plus(&pattern);
    eprintln!("Expanded pattern: {}", pattern);

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