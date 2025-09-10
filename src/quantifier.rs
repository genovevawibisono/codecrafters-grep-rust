#[derive(Debug, Clone)]

pub enum Quantifier {
    ZeroOrOne,
    ZeroOrMore,
    OneOrMore,
    Literal(usize),
}
