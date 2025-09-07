use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::process;

#[derive(Debug)]
struct InvalidRegexError;

#[derive(Debug, Clone)]
enum PatternToken {
    CharacterClass(CharacterClass),
    Anchor(Anchor),
    MatchGroup(MatchGroup),
    Quantifier(Quantifier),
}

#[derive(Debug, Clone)]
enum MatchGroup {
    Start(usize),
    End(usize),
    Delimiter(usize),
}

#[derive(Debug, Clone)]
enum CharacterClass {
    Literal(char),
    Digit,
    Word,
    CharacterGroup(Vec<char>, bool),
    Alternation(Vec<Vec<(CharacterClass, Quantifier)>>),
    Backreference(usize),
    Wildcard,
}
impl CharacterClass {
    fn min(&self) -> usize {
        match self {
            CharacterClass::Alternation(groups) => {
                groups.iter().map(|g| g.len()).min().unwrap_or(0)
            }
            CharacterClass::Backreference(_) => 0,
            _ => 1,
        }
    }
}

impl PartialEq for CharacterClass {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(l0), Self::Literal(r0)) => l0 == r0,
            (Self::CharacterGroup(l0, l1), Self::CharacterGroup(r0, r1)) => l0 == r0 && l1 == r1,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(Debug, Clone)]
enum Anchor {
    StartAnchor,
    EndAnchor,
}

#[derive(Debug, Clone, Copy)]
enum Quantifier {
    LowerBound(usize),
    Bounded(usize, usize),
}

impl Quantifier {
    fn min(&self) -> usize {
        match self {
            Quantifier::LowerBound(b) => *b,
            Quantifier::Bounded(low, _) => *low,
        }
    }
    fn get_satisfied_amount(&self, count: usize) -> Option<usize> {
        match self {
            Quantifier::LowerBound(bound) if count >= *bound => Some(count),
            Quantifier::Bounded(low, high) if count >= *low => Some(count.min(*high)),
            _ => None,
        }
    }
    fn max(&self) -> Option<usize> {
        match self {
            Quantifier::LowerBound(_) => None,
            Quantifier::Bounded(_, high) => Some(*high),
        }
    }
}

fn parse_pattern(pattern: &str) -> Result<Vec<PatternToken>, InvalidRegexError> {
    let mut iter = pattern.chars();
    let mut vec = Vec::new();

    let mut group_depth = 0usize;

    while let Some(char) = iter.next() {
        let symbol = match char {
            '\\' => Some(PatternToken::CharacterClass(
                match iter.next().ok_or(InvalidRegexError)? {
                    'd' => CharacterClass::Digit,
                    'w' => CharacterClass::Word,
                    '\\' => CharacterClass::Literal('\\'),
                    e => match e.to_digit(10) {
                        Some(digit) => CharacterClass::Backreference(digit as usize),
                        None => Err(InvalidRegexError)?,
                    },
                },
            )),
            '[' => Some(PatternToken::CharacterClass({
                let match_chars = iter
                    .by_ref()
                    .take_while(|c| !c.eq(&']'))
                    .collect::<Vec<_>>();

                if let &'^' = match_chars.first().ok_or(InvalidRegexError)? {
                    CharacterClass::CharacterGroup(match_chars[1..].to_vec(), false)
                } else {
                    CharacterClass::CharacterGroup(match_chars, true)
                }
            })),
            '^' => Some(PatternToken::Anchor(Anchor::StartAnchor)),
            '$' => Some(PatternToken::Anchor(Anchor::EndAnchor)),
            '+' => Some(PatternToken::Quantifier(Quantifier::LowerBound(1))),
            '?' => Some(PatternToken::Quantifier(Quantifier::Bounded(0, 1))),
            '.' => Some(PatternToken::CharacterClass(CharacterClass::Wildcard)),
            '|' => Some(PatternToken::MatchGroup(MatchGroup::Delimiter(group_depth))),
            '(' => {
                group_depth += 1;
                Some(PatternToken::MatchGroup(MatchGroup::Start(group_depth)))
            }
            ')' => {
                let token = Some(PatternToken::MatchGroup(MatchGroup::End(group_depth)));
                group_depth -= 1;
                token
            }
            l => Some(PatternToken::CharacterClass(CharacterClass::Literal(l))),
        };
        if let Some(symbol) = symbol {
            vec.push(symbol);
        }
    }

    Ok(vec)
}
fn match_pattern(input_line: &str, pattern: &str) -> Option<String> {
    let pattern = parse_pattern(pattern).expect("valid regex string");

    let input_chars = input_line.chars().collect::<Vec<_>>();

    match pattern.as_slice() {
        [
            PatternToken::Anchor(Anchor::StartAnchor),
            other @ ..,
            PatternToken::Anchor(Anchor::EndAnchor),
        ] => {
            let vec = other.iter().collect::<Vec<_>>();
            let class_pattern = extract_character_classes(&vec);
            let mut backref_queue = HashMap::new();
            match match_chars_to_pattern(&input_chars, &class_pattern, &mut backref_queue, 0) {
                Some(matched) if input_chars.len() == matched.len() => {
                    Some(matched.iter().collect())
                }
                _ => None,
            }
        }
        [PatternToken::Anchor(Anchor::StartAnchor), other @ ..] => {
            let vec = other.iter().collect::<Vec<_>>();
            let class_pattern = extract_character_classes(&vec);
            let mut backref_queue = HashMap::new();
            match match_chars_to_pattern(&input_chars, &class_pattern, &mut backref_queue, 0) {
                Some(matched) if other.len() == matched.len() => Some(matched.iter().collect()),
                _ => None,
            }
        }
        [other @ .., PatternToken::Anchor(Anchor::EndAnchor)] => {
            let vec = other.iter().collect::<Vec<_>>();
            let mut class_pattern = extract_character_classes(&vec);
            class_pattern.reverse();

            let mut backref_queue = HashMap::new();
            let reversed_str: Vec<char> = input_chars.iter().rev().cloned().collect();
            match_chars_to_pattern(
                reversed_str.as_slice(),
                &class_pattern,
                &mut backref_queue,
                0,
            )
            .map(|matched| matched.iter().rev().collect())
        }
        _ => (0..input_chars.len()).find_map(|start| {
            let vec = pattern.iter().collect::<Vec<_>>();
            let mut backref_queue = HashMap::new();

            match_chars_to_pattern(
                &input_chars[start..],
                &extract_character_classes(&vec),
                &mut backref_queue,
                0,
            )
            .map(|matched| matched.iter().collect())
        }),
    }
}

fn extract_character_classes(other: &[&PatternToken]) -> Vec<(CharacterClass, Quantifier)> {
    let mut vec = Vec::new();
    let mut iter = other.iter();

    while let Some(token) = iter.next() {
        match token {
            PatternToken::CharacterClass(character_class) => {
                vec.push((character_class.clone(), Quantifier::Bounded(1, 1)));
            }
            PatternToken::Quantifier(quantifier) => {
                if let Some((last_class, _)) = vec.pop() {
                    vec.push((last_class, *quantifier));
                } else {
                    return vec;
                }
            }
            PatternToken::MatchGroup(match_group) => match match_group {
                MatchGroup::Start(start_depth) => {
                    // Recursively extract the group until we find the matching end
                    let group: Vec<_> = iter
                                .by_ref()
                                .take_while(|token| {
                                    !matches!(token, PatternToken::MatchGroup(MatchGroup::End(end_depth)) if start_depth == end_depth)
                                })
                                .collect();

                    let groups = group
                                .split(|token| {
                                    matches!(token, PatternToken::MatchGroup(MatchGroup::Delimiter(del_depth)) if del_depth == start_depth)
                                })
                                .map(|chunk| {
                                    extract_character_classes(
                                        &chunk.iter().map(|&&t| t).collect::<Vec<_>>(),
                                    )
                                })
                                .collect::<Vec<_>>();

                    vec.push((
                        CharacterClass::Alternation(groups),
                        Quantifier::Bounded(1, 1),
                    ));
                }
                MatchGroup::End(_) => (),
                MatchGroup::Delimiter(_) => (),
            },
            PatternToken::Anchor(_) => (),
        }
    }
    vec.chunk_by(|a, b| a.0 == b.0)
        .map(|chunk| {
            let (class, _) = chunk.first().unwrap();
            (
                class.clone(),
                chunk
                    .iter()
                    .map(|(_, quant)| quant)
                    .fold(Quantifier::Bounded(0, 0), |left, right| {
                        combine_quantifiers(&left, right)
                    }),
            )
        })
        .collect::<Vec<_>>()
}

fn combine_quantifiers(left: &Quantifier, right: &Quantifier) -> Quantifier {
    match left {
        Quantifier::LowerBound(bound) => match right {
            Quantifier::LowerBound(bound2) => Quantifier::LowerBound(bound + bound2),
            Quantifier::Bounded(low, _) => Quantifier::LowerBound(bound + low),
        },
        Quantifier::Bounded(low, high) => match right {
            Quantifier::LowerBound(bound) => Quantifier::LowerBound(bound + low),
            Quantifier::Bounded(low2, high2) => Quantifier::Bounded(low + low2, high + high2),
        },
    }
}

fn match_chars_to_pattern<'a>(
    window: &'a [char],
    pattern: &[(CharacterClass, Quantifier)],
    backref_queue: &mut HashMap<usize, &'a [char]>,
    depth: usize,
) -> Option<&'a [char]> {
    let mut count = 0usize;
    let mut pattern_iter = pattern.iter();

    loop {
        let Some((class, quantifier)) = pattern_iter.next() else {
            //println!("Pattern exhausted with count = {:#?}\n", count);
            return Some(&window[..count]);
        };

        // Calculates the minimum number of chars needed to satisfy the rest of the pattern
        // This is so LowerBound Quantifiers doesn't consume the whole input.
        // Substitute backreferences with their captured lengths
        let match_slice = {
            let minimums = pattern_iter
                .clone()
                .map(|(class, q)| match class {
                    CharacterClass::Backreference(index) => {
                        backref_queue.get(&(*index - 1)).map_or(0, |s| s.len()) * q.min()
                    }
                    _ => q.min() * class.min(),
                })
                .collect::<Vec<_>>();

            let minimum_chars_for_complete_match = minimums.iter().sum::<usize>();
            let range_end = window
                .len()
                .saturating_sub(minimum_chars_for_complete_match);

            if range_end < count {
                // There are not enough chars in the window to satisfy the pattern
                return None;
            };

            &window[(count)..(range_end)]
        };

        let parent_backref_id = backref_queue.len() + depth;

        let consumed_chars = match class {
            CharacterClass::Backreference(index) => {
                if let Some(captured) = backref_queue.get(&(*index - 1)) {
                    println!(
                        "Backreference to index {} captured {:?}",
                        index - 1,
                        captured
                    );

                    let literals = captured
                        .iter()
                        .map(|c| (CharacterClass::Literal(*c), Quantifier::Bounded(1, 1)))
                        .collect::<Vec<_>>();

                    match_chars_to_alternation(
                        match_slice,
                        &[literals],
                        quantifier,
                        backref_queue,
                        depth + 1,
                    )
                } else {
                    panic!("Invalid backreference");
                }
            }
            CharacterClass::Alternation(groups) => {
                //println!("Matching alternation groups");
                match_chars_to_alternation(
                    match_slice,
                    groups,
                    quantifier,
                    backref_queue,
                    depth + 1,
                )
            }
            _ => match_chars_to_class(match_slice, class, quantifier),
        };

        // If the class is an alternation, we need to store the matched chars for backreferencing
        if let CharacterClass::Alternation(_) = class {
            if let Some(matched) = consumed_chars {
                backref_queue.insert(parent_backref_id, matched);
            }
        }

        count += consumed_chars?.len();
    }
}

fn match_chars_to_alternation<'a>(
    chars: &'a [char],
    groups: &[Vec<(CharacterClass, Quantifier)>],
    quantifier: &Quantifier,
    backref_queue: &mut HashMap<usize, &'a [char]>,
    depth: usize,
) -> Option<&'a [char]> {
    // println!(
    //     "Matching alternation groups {:?} with quantifier {:?}",
    //     groups, quantifier
    // );
    let mut matches = Vec::<&[char]>::new();

    while let Some(size) = groups.iter().find_map(|group| {
        match_chars_to_pattern(
            &chars[matches.iter().map(|m| m.len()).sum()..],
            group,
            backref_queue,
            depth,
        )
    }) {
        matches.push(size);
        if let Some(max) = quantifier.max() {
            if matches.len() >= max {
                break;
            }
        }
    }

    quantifier
        .get_satisfied_amount(matches.len())
        .map(|amount| {
            let char_length = matches.iter().map(|m| m.len()).take(amount).sum();
            //println!("Alternation matched {:?} chars", char_length);
            &chars[..char_length]
        })
}

fn match_chars_to_class<'a>(
    chars: &'a [char],
    class: &CharacterClass,
    quantifier: &Quantifier,
) -> Option<&'a [char]> {
    let captured_chars: Vec<_> = chars
        .iter()
        .take_while(|char| match_char_to_class(char, class))
        .collect();
    quantifier
        .get_satisfied_amount(captured_chars.len())
        .map(|size| &chars[..size])
}

fn match_char_to_class(char: &char, class: &CharacterClass) -> bool {
    match class {
        CharacterClass::Literal(l) => l.eq(char),
        CharacterClass::Digit => char.is_ascii_digit(),
        CharacterClass::Word => char.is_ascii_alphanumeric() || char.eq(&'_'),
        CharacterClass::CharacterGroup(items, is_positive) => match is_positive {
            true => items.contains(char),
            false => !items.contains(char),
        },
        CharacterClass::Wildcard => true,
        CharacterClass::Alternation(_) => false,
        CharacterClass::Backreference(_) => false,
    }
}

fn match_std_input_to_pattern(pattern: &str) {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let input_line = input_line.trim_end();

    // Uncomment this block to pass the first stage
    match match_pattern(input_line, pattern) {
        Some(matched) => {
            println!("{matched}");
            process::exit(0);
        }
        None => {
            process::exit(1);
        }
    }
}

fn match_file_to_pattern(pattern: &str, file_path: &str) -> Vec<String> {
    let file = File::open(file_path).expect("Unable to open the file");
    let reader = io::BufReader::new(file);

    reader
        .lines()
        .filter_map(|line| {
            let str = line.as_ref().expect("able to read line").trim();
            match_pattern(str, pattern)
        })
        .collect::<Vec<_>>()
}


// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    let args = env::args().collect::<Vec<_>>();

    if args.get(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = args.get(2).unwrap();

    let file_path_slice = args.get(3..);

    match file_path_slice {
        None => {
            match_std_input_to_pattern(pattern);
        }
        Some([]) => {
            match_std_input_to_pattern(pattern);
        }
        Some(file_paths) if file_paths.len() == 1 => {
            let matches = match_file_to_pattern(pattern, file_paths.first().unwrap());
            matches.iter().for_each(|matched| {
                println!("{matched}");
            });
            process::exit(if matches.is_empty() { 1 } else { 0 });
        }
        Some(file_paths) => {
            let matches: Vec<_> = file_paths
                .iter()
                .flat_map(|file_path| {
                    match_file_to_pattern(pattern, file_path)
                        .iter()
                        .map(|s| format!("{file_path}:{s}"))
                        .collect::<Vec<_>>()
                })
                .collect();
            matches.iter().for_each(|matched| println!("{matched}"));
            process::exit(if matches.is_empty() { 1 } else { 0 });
        }
    }
}