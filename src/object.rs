use std::{collections::BTreeMap, fmt::Display};

use owo_colors::OwoColorize;
use pest::Span;

use crate::{
    ast::ContextualNode,
    errors::{Error, Hydrator},
    eval::{repl::ReplDisplay, step},
    helpers::extend,
    scope::{MutScope, Scope},
    types::{Float, Int, Num, VariablySized},
};

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
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
        bool,
        fn(
            Vec<ContextualObject<'a>>,
            Hydrator,
            MutScope<'a>,
        ) -> Result<ContextualObject<'a>, Error>,
    ),
    Lambda(Vec<String>, Option<String>, Vec<ContextualNode<'a>>),
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

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct ContextualObject<'a>(pub Object<'a>, pub Span<'a>);

impl PartialOrd for ContextualObject<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(core::cmp::Ordering::Equal) => {
                self.1.start_pos().partial_cmp(&other.1.start_pos())
            }
            ord => ord,
        }
    }
}

impl Ord for ContextualObject<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<'a> Object<'a> {
    pub fn coerce(
        a: ContextualObject<'a>,
        b: ContextualObject<'a>,
        h: Hydrator,
    ) -> Result<(ContextualObject<'a>, ContextualObject<'a>), Error> {
        if std::mem::discriminant(&a.0) == std::mem::discriminant(&b.0) {
            return Ok((a, b));
        }

        // let span = extend(&[a.clone])
        Ok(match (a.clone().0, b.clone().0) {
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
                a,
                Object::String(b.0.to_string()).provide_context(b.1.clone()),
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
            Object::Builtin(..) => "builtin",
            Object::Lambda(..) => "lambda",
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
            Object::Builtin(name, ..) => write!(f, "#pet.builtin({name})"),
            Object::Lambda(args, typed, ..) => write!(
                f,
                "#pet.lambda({args}): {typed}",
                args = args.join(", "),
                typed = match typed {
                    Some(t) => format!(" -> {}", t),
                    None => "".to_string(),
                }
            ),
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
            Object::Builtin(name, ..) => format!(
                "{}({})",
                "#pet.builtin".purple().to_string(),
                name.magenta().to_string()
            ),
            Object::Lambda(args, typed, ..) => format!(
                "{}({}): {}",
                "#pet.lambda".purple().to_string(),
                format!("{}", args.join(", ")).magenta().to_string(),
                match typed {
                    Some(t) => format!("{}", t.magenta().to_string()),
                    None => "".to_string(),
                }
            ),
            Object::Null => "null".magenta().to_string(),
        }
    }
}

impl<'a> ContextualObject<'a> {
    pub fn call(
        &self,
        mut args: Vec<ContextualObject<'a>>,
        scope: MutScope<'a>,
        h: Hydrator,
    ) -> Result<ContextualObject<'a>, Error> {
        let call_scope = Scope::new_child(scope.clone(), "#pet.call");

        match &self.0 {
            Object::Lambda(fn_args, _, body) => {
                if fn_args.len() != args.len() {
                    return Err(partial!(
                        "evaluating function call",
                        format!("Expected {} arguments, got {}", fn_args.len(), args.len()),
                        self.1.clone(),
                        h.clone()
                    ));
                }

                for (value, name) in args.into_iter().zip(fn_args.into_iter()) {
                    call_scope
                        .borrow_mut()
                        .set(&name, value, self.1, h.clone())?;
                }

                let mut result: ContextualObject = Object::Null.anonymous();
                for node in body {
                    result = step(&node, call_scope.clone(), h.clone())?;
                    if let Object::Return(expr) = &result.0 {
                        result = *expr.clone();
                        break;
                    }
                }

                Ok(result)
            }
            Object::Builtin(_, needs_self, f) => {
                if *needs_self {
                    let slf = scope.borrow().get_self().ok_or(partial!(
                        "evaluating function call",
                        "No self provided for method call".to_string(),
                        self.1.clone(),
                        h.clone()
                    ))?;

                    args.insert(0, slf.clone());
                }

                let v = f(args, h.clone(), scope);
                v
            }
            _ => Err(partial!(
                "evaluating function call",
                "Can't call a non-function".to_string(),
                self.1.clone(),
                h.clone()
            )),
        }
    }
}
