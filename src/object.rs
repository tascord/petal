use std::{collections::BTreeMap, fmt::Display};

use owo_colors::OwoColorize;
use pest::Span;

use crate::{
    errors::{Error, Hydrator},
    eval::repl::ReplDisplay,
    helpers::extend,
    types::{Float, Int, Num, VariablySized},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Object<'a> {
    Integer(Int),
    Float(Float),
    Bool(bool),
    String(String),
    Array(Vec<ContextualObject<'a>>),
    Map(BTreeMap<ContextualObject<'a>, ContextualObject<'a>>),
    Return(Box<ContextualObject<'a>>),
    // Function(Vec<Expr>, Block, Scope),
    Builtin(
        String,
        fn(Vec<ContextualObject<'a>>, Hydrator) -> Result<ContextualObject<'a>, Error>,
    ),
    Null,
}

impl<'a> Object<'a> {
    pub fn provide_context(&self, span: Span<'a>) -> ContextualObject<'a> {
        ContextualObject(self.clone(), span)
    }

    pub fn anonymous(self) -> ContextualObject<'a> {
        self.provide_context(Span::new("", 0, 0).unwrap())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ContextualObject<'a>(pub Object<'a>, pub Span<'a>);

impl<'a> Object<'a> {
    pub fn coerce(
        a: ContextualObject<'a>,
        b: ContextualObject<'a>,
        h: Hydrator,
    ) -> Result<(ContextualObject<'a>, ContextualObject<'a>), Error> {
        // let span = extend(&[a.clone])
        Ok(match (a.clone().0, b.clone().0) {
            // To Int
            (Object::Integer(c), Object::Integer(d)) => (
                Object::Integer(c).provide_context(a.1.clone()),
                Object::Integer(d).provide_context(b.1.clone()),
            ),
            (Object::Float(c), Object::Float(d)) => (
                Object::Float(c).provide_context(a.1.clone()),
                Object::Float(d).provide_context(b.1.clone()),
            ),

            // To Float
            (Object::Integer(c), Object::Float(d)) => (
                Object::Float(Float::fit(c.to_max_value() as f64)).provide_context(a.1.clone()),
                Object::Float(d).provide_context(b.1.clone()),
            ),
            (Object::Float(c), Object::Integer(d)) => (
                Object::Float(c).provide_context(a.1.clone()),
                Object::Float(Float::fit(d.to_max_value() as f64)).provide_context(a.1.clone()),
            ),

            // To String
            (_, Object::String(_)) => (
                Object::String(a.0.to_string()).provide_context(a.1.clone()),
                b,
            ),
            (Object::String(_), _) => (
                a.clone(),
                Object::String(a.0.to_string()).provide_context(b.1.clone()),
            ),

            _ => {
                return Err(partial!(
                    format!("coercing {} -> {}", a.0.typed(), b.0.typed()),
                    "Cannot coerce types".to_string(),
                    "You might be missing a cast",
                    extend(&[a.1.clone(), b.1.clone()]),
                    h.clone()
                ))
            }
        })
    }

    pub fn typed(&self) -> String {
        match self {
            Object::Integer(_) => "int",
            Object::Float(_) => "float",
            Object::Bool(_) => "bool",
            Object::String(_) => "string",
            Object::Array(_) => "array",
            Object::Map(_) => "map",
            Object::Return(_) => "return",
            Object::Builtin(_, _) => "builtin",
            Object::Null => "null",
        }
        .to_string()
    }
}

impl Display for Object<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(v) => f.write_str(v.to_max_value().to_string().as_str()),
            Object::Float(v) => f.write_str(v.to_max_value().to_string().as_str()),
            Object::Bool(v) => f.write_str(v.to_string().as_str()),
            Object::String(v) => f.write_str(v.to_string().as_str()),

            Object::Return(v) => write!(f, "return {}", (*v.clone()).0.to_string()),
            Object::Builtin(_, _) => f.write_str("#pet.builtin"),
            Object::Null => write!(f, "null"),

            Object::Array(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|v| v.0.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),

            Object::Map(v) => write!(
                f,
                "{{{}}}",
                v.iter()
                    .map(|(k, v)| format!("{}: {}", k.0.to_string(), v.0.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl ReplDisplay for Object<'_> {
    fn pretty_print(&self) -> String {
        match self {
            Object::Integer(v) => v.to_max_value().to_string().yellow().to_string(),
            Object::Float(v) => v.to_max_value().to_string().yellow().to_string(),
            Object::Bool(v) => v.to_string().green().to_string(),
            Object::String(v) => format!("\"{v}\"").cyan().to_string(),
            Object::Array(v) => format!(
                "{}{ar}{}",
                "[".blue(),
                "]".blue(),
                ar = v
                    .iter()
                    .map(|i| i.0.pretty_print())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Object::Map(v) => format!(
                "{}{ma}{}",
                "[".blue(),
                "]".blue(),
                ma = v
                    .iter()
                    .map(|i| format!(
                        "{}{ke}: {va}{}",
                        "{".blue(),
                        "}".blue(),
                        ke = i.0 .0.pretty_print(),
                        va = i.1 .0.pretty_print()
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Object::Return(v) => format!("{} {}", "return".red(), v.0.pretty_print()),
            Object::Builtin(..) => "#pet.builtin".purple().to_string(),
            Object::Null => "null".magenta().to_string(),
        }
    }
}
