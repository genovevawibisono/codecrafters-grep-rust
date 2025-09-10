use crate::quantifier::Quantifier;

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(char),
    Digit,
    Alphanumeric,
    Any,
    PositiveGroup(Vec<Pattern>),
    NegativeGroup(Vec<Pattern>),
    Alternate(usize, Vec<Vec<Pattern>>),
    StartOfString,
    EndOfString,
    OneOrMore,
    ZeroOrMore,
    OneOrZero,
    LiteralQuantifier(usize),
    PatternWithQuantifier(Box<Pattern>, Quantifier),
    CaptureGroup(usize, Vec<Pattern>),
    BackReference(usize),
}

impl Pattern {
    pub fn matches(&self, s: &str) -> Option<usize> {
        match self {
            Self::EndOfString => {
                s.is_empty().then_some(0)
            },
            _ => {
                let length = self.char_matches(s);
                return (length > 0).then_some(length);
            }
        }
    }

    fn char_matches(&self, s: &str) -> usize {
        let Some(c) = s.chars().next() else {
            return 0;
        };
        
        if match self {
            Self::Literal(p) => {
                c == *p
            },
            Self::Digit => c.is_ascii_digit(),
            Self::Alphanumeric => {
                c.is_ascii_alphanumeric() || c == '_'
            },
            Self::Any => true,
            Self::PositiveGroup(p) => {
                p.iter().any(|x| x.char_matches(s) > 0)
            },
            Self::NegativeGroup(p) => {
                !p.iter().any(|x| x.char_matches(s) > 0)
            },
            _ => false,
        } {
            return c.len_utf8();
        }

        return 0;
    }

    pub fn is_optional(&self) -> bool {
        match self {
            Pattern::PatternWithQuantifier(_, Quantifier::ZeroOrOne) |
            Pattern::PatternWithQuantifier(_, Quantifier::ZeroOrMore) => true,
            Pattern::Alternate(_, alts) => {
                alts.iter().any(|a| a.iter().any(|x| x.is_optional()))
            }
            _ => false,
        }
    }
}
