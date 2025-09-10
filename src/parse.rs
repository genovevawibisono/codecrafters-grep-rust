use crate::pattern::{Pattern};
use crate::quantifier::{Quantifier};

impl From<&Pattern> for Quantifier {
    fn from(p: &Pattern) -> Self {
        match p {
            Pattern::OneOrZero => Self::ZeroOrOne,
            Pattern::OneOrMore => Self::OneOrMore,
            Pattern::ZeroOrMore => Self::ZeroOrMore,
            Pattern::LiteralQuantifier(n) => Self::Literal(*n),
            _ => panic!("Unknown quantifier"),
        }
    }
}

impl From<&str> for Pattern {
    fn from(s: &str) -> Self {
        match s {
            r"\d" => Self::Digit,
            r"\w" => Self::Alphanumeric,
            r"\\" => Self::Literal('\\'),
            "." => Self::Any,
            "$" => Self::EndOfString,
            "^" => Self::StartOfString,
            "+" => Self::OneOrMore,
            "?" => Self::OneOrZero,
            "*" => Self::ZeroOrMore,
            val if val.starts_with("[^") => {
                Self::NegativeGroup(Self::parse(&val[2..val.len() - 1]))
            }
            val if val.starts_with('[') => Self::PositiveGroup(Self::parse(&val[1..val.len() - 1])),
            val if val.starts_with('(') && split_either_variants(val).len() == 1 => {
                Self::CaptureGroup(0, Self::parse(&val[1..val.len() - 1]))
            }
            val if val.starts_with('(') && val.contains('|') => Self::Alternate(
                0,
                split_either_variants(val)
                    .into_iter()
                    .map(Pattern::parse)
                    .collect(),
            ),
            val if val.starts_with('{') => Self::LiteralQuantifier(
                val[1..val.len() - 1]
                    .parse()
                    .unwrap_or_else(|_| panic!("Invalid quantifier in pattern: {val}")),
            ),
            val if val.starts_with('\\') => Self::BackReference(val[1..].parse().unwrap()),
            _ => Self::Literal(s.chars().next().unwrap_or_default()),
        }
    }
}

impl Pattern {
    pub fn parse(mut s: &str) -> Vec<Pattern> {
        let mut patterns = vec![];
        while !s.is_empty() {
            match s.chars().next() {
                Some('[') => {
                    let end = find_closing_char(s, '[').unwrap_or(s.len());
                    patterns.push(Pattern::from(&s[..=end]));
                    s = &s[end + 1..];
                }
                Some('(') => {
                    let end = find_closing_char(s, '(').unwrap_or(s.len());
                    patterns.push(Pattern::from(&s[..=end]));
                    s = &s[end + 1..];
                }
                Some('{') => {
                    let end = find_closing_char(s, '{').unwrap_or(s.len());
                    patterns.push(Pattern::from(&s[..=end]));
                    s = &s[end + 1..];
                }
                Some('\\') => {
                    patterns.push(Pattern::from(&s[..2]));
                    s = &s[2..];
                }
                Some(c) if char_is_regex_special(&c) => {
                    patterns.push(Pattern::from(&s[..1]));
                    s = &s[1..];
                }
                _ => {
                    patterns.push(Pattern::from(&s[..1]));
                    s = &s[1..];
                }
            }
            handle_quantifier(&mut patterns);
        }

        return patterns;
    }

    fn is_quantifier(&self) -> bool {
        matches!(
            self,
            Pattern::OneOrZero
                | Pattern::OneOrMore
                | Pattern::LiteralQuantifier(_)
                | Pattern::ZeroOrMore
        )
    }
}

fn handle_quantifier(patterns: &mut Vec<Pattern>) {
    if let Some(last) = patterns.last() {
        if !last.is_quantifier() {
            return;
        }
        let (Some(quantifier), Some(previous)) = (patterns.pop(), patterns.pop()) else {
            panic!();
        };
        patterns.push(Pattern::PatternWithQuantifier(
            Box::new(previous),
            Quantifier::from(&quantifier),
        ));
    }
}

fn find_closing_char(s: &str, opening_char: char) -> Option<usize> {
    let mut depth = 0;
    let closing_char = match opening_char {
        '[' => ']',
        '(' => ')',
        '{' => '}',
        _ => return None,
    };
    for (i, c) in s.char_indices() {
        match c {
            _ if c == opening_char => depth += 1,
            _ if c == closing_char => {
                if depth == 1 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    return None;
}

fn split_either_variants(s: &str) -> Vec<&str> {
    let inner = &s[1..s.len() - 1];
    let mut variants = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    for (i, c) in inner.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            '|' if depth == 0 => {
                variants.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    variants.push(&inner[start..]);
    return variants;
}

fn char_is_regex_special(c: &char) -> bool {
    return matches!(c, '.' | '*' | '+' | '?' | '|' | '^' | '$');
}

pub fn assign_capture_indices(pattern: &mut Pattern, next_index: &mut usize) {
    match pattern {
        Pattern::CaptureGroup(ref mut idx, ref mut inner) => {
            *idx = *next_index;
            *next_index += 1;
            for p in inner {
                assign_capture_indices(p, next_index);
            }
        }
        Pattern::Alternate(ref mut idx, ref mut inner) => {
            *idx = *next_index;
            *next_index += 1;
            for p in inner {
                for s in p {
                    assign_capture_indices(s, next_index);
                }
            }
        }
        Pattern::PatternWithQuantifier(inner, _) => {
            assign_capture_indices(inner, next_index);
        }
        _ => {}
    }
}