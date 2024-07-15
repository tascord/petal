use std::collections::HashMap;

thread_local! {
    pub static MONDAIC_SYMBOL_MAP: HashMap<&'static str, Mondaic> = {
        let mut m = HashMap::new();
        m.insert("!", Mondaic::Negate);
        m
    };

    pub static DYADIC_SYMBOL_MAP: HashMap<&'static str, Dyadic> = {
        let mut m = HashMap::new();
        m.insert("**", Dyadic::Pow);
        m.insert("==", Dyadic::Equality);
        m.insert("+", Dyadic::Add);
        m.insert("-", Dyadic::Subtract);
        m.insert("*", Dyadic::Multiply);
        m.insert("/", Dyadic::Divide);
        m.insert("||", Dyadic::Or);
        m.insert("&&", Dyadic::And);
        m.insert(">", Dyadic::GreaterThan);
        m.insert("<", Dyadic::LessThan);
        m.insert(">=", Dyadic::GreaterThanOrEqual);
        m.insert("<=", Dyadic::LessThanOrEqual);
        m
    };
}

pub fn get_dyadic(verb: String) -> Option<Dyadic> {
    DYADIC_SYMBOL_MAP.with(|m| m.get(verb.as_str()).copied())
}

pub fn get_dyads() -> Vec<String> {
    DYADIC_SYMBOL_MAP.with(|m| m.keys().copied().map(|s| s.to_string()).collect())
}

pub fn get_mondaic(verb: String) -> Option<Mondaic> {
    MONDAIC_SYMBOL_MAP.with(|m| m.get(verb.as_str()).copied())
}

pub fn get_monads() -> Vec<String> {
    MONDAIC_SYMBOL_MAP.with(|m| m.keys().copied().map(|s| s.to_string()).collect())
}

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mondaic {
    Negate,
}

impl Mondaic {
    pub fn to_symbol(&self) -> String {
        MONDAIC_SYMBOL_MAP.with(|m| m.iter().find(|(_, v)| **v == *self).unwrap().0.to_string())
    }
}

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dyadic {
    Pow,
    Equality,
    Add,
    Subtract,
    Multiply,
    Divide,
    And,
    Or,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl Dyadic {
    pub fn to_symbol(&self) -> String {
        DYADIC_SYMBOL_MAP.with(|m| m.iter().find(|(_, v)| **v == *self).unwrap().0.to_string())
    }
}
