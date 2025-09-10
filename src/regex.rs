use std::usize;

use crate::captures::Captures;
use crate::pattern::Pattern;
use crate::quantifier::Quantifier;
use crate::parse::assign_capture_indices;

#[derive(Debug)]
pub struct Regex {
    pub patterns: Vec<Pattern>,
    pub capture_group_count: usize,
}

impl Regex {
    pub fn parse(s: &str) -> Self {
        let mut next_index = 1;
        let mut patterns = Pattern::parse(s);
        for p in patterns.iter_mut() {
            assign_capture_indices(p, &mut next_index);
        }
        Self {
            patterns,
            capture_group_count: next_index - 1,
        }
    }

    pub fn matches(&self, input: &str) -> bool {
        let captures = Captures::new(self.capture_group_count);
        if let Some(Pattern::StartOfString) = self.patterns.first() {
            return try_match(input, &self.patterns[1..], captures).is_some();
        }
        for (cur, _) in input.char_indices() {
            if try_match(&input[cur..], &self.patterns, captures.clone()).is_some() {
                return true;
            }
        }

        return false;
    }
}

fn try_match(
    mut input: &str,
    patterns: &[Pattern],
    captures: Captures,
) -> Option<(usize, Captures)> {
    let mut total_matched_len = 0;
    for (idx, pattern) in patterns.iter().enumerate() {
        if input.is_empty() {
            let mut rest = &patterns[idx..];
            while let Some(pat) = rest.first() {
                match pat {
                    Pattern::EndOfString => return Some((total_matched_len, captures)),
                    _ if pat.is_optional() => {
                        rest = &rest[1..];
                        continue;
                    }
                    _ => return None,
                }
            }

            return Some((total_matched_len, captures));
        }

        if let Pattern::CaptureGroup(c_idx, p) = pattern {
            let cur_input = input;
            for (rev_idx, _) in input.char_indices().rev() {
                let prefix = &cur_input[..=rev_idx];
                if let Some((len, mut temp_captures)) = try_match(prefix, p, captures.clone()) {
                    temp_captures.capture(&prefix[..len], *c_idx);
                    if let Some((rest_len, temp_captures)) =
                        try_match(&cur_input[len..], &patterns[idx + 1..], temp_captures)
                    {
                        return Some((total_matched_len + len + rest_len, temp_captures));
                    }
                }
            }

            return None;
        }

        if let Pattern::BackReference(index) = pattern {
            let ref_str = captures.get_capture(*index)?;
            if input.starts_with(&ref_str) {
                total_matched_len += ref_str.len();
                input = &input[ref_str.len()..];
                continue;
            } else {
                return None;
            }
        }

        if let Pattern::Alternate(c_idx, p) = pattern {
            let cur_input = input;
            for sub_pattern in p {
                if let Some((match_len, mut temp_captures)) =
                    try_match(input, sub_pattern, captures.clone())
                {
                    temp_captures.capture(&input[..match_len], *c_idx);
                    let rest = &patterns[idx + 1..];
                    if let Some((rest_len, temp_captures)) =
                        try_match(&input[match_len..], rest, temp_captures)
                    {
                        return Some((total_matched_len + match_len + rest_len, temp_captures));
                    }
                }
                input = cur_input;
            }

            return None;
        }

        if let Pattern::PatternWithQuantifier(inner, quant) = pattern {
            let mut count = 0;
            let mut match_lengths = vec![];
            let cur_input = input;
            let min_required_match_count = match quant {
                Quantifier::Literal(n) => *n,
                Quantifier::OneOrMore => 1,
                _ => 0,
            };
            let mut temp_captures = captures.clone();
            loop {
                let prev_input = input;
                if let Some((len, sub_captures)) =
                    try_match(input, std::slice::from_ref(inner), captures.clone())
                {
                    if len == 0 {
                        input = prev_input;
                        break;
                    }
                    input = &input[len..];
                    match_lengths.push(len);
                    count += 1;
                    temp_captures = sub_captures;
                    match quant {
                        Quantifier::ZeroOrOne => break,
                        Quantifier::Literal(n) if count >= *n => break,
                        _ => {}
                    }
                } else {
                    input = prev_input;
                    break;
                }
            }
            match quant {
                Quantifier::ZeroOrOne => {
                    let (rest_len, temp_captures) =
                        try_match(input, &patterns[idx + 1..], temp_captures)?;
                    return Some((
                        total_matched_len + match_lengths.iter().sum::<usize>() + rest_len,
                        temp_captures,
                    ));
                }
                Quantifier::Literal(n) => {
                    if count != *n {
                        return None;
                    }
                    let (rest_len, temp_captures) =
                        try_match(input, &patterns[idx + 1..], temp_captures)?;
                    return Some((
                        total_matched_len + match_lengths.iter().sum::<usize>() + rest_len,
                        temp_captures,
                    ));
                }
                _ => {}
            }
            while count >= min_required_match_count {
                if let Some((rest_len, sub_captures)) =
                    try_match(input, &patterns[idx + 1..], temp_captures.clone())
                {
                    return Some((
                        total_matched_len + match_lengths.iter().sum::<usize>() + rest_len,
                        sub_captures,
                    ));
                }
                count -= 1;
                match_lengths.pop().unwrap();
                input = &cur_input[match_lengths.iter().sum::<usize>()..];
            }
            return None;
        }
        let match_len = pattern.matches(input)?;
        total_matched_len += match_len;
        input = &input[match_len..];
    }
    
    return Some((total_matched_len, captures));
}