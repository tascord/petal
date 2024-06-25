use itertools::Itertools;

use crate::{
    ast::{op::Dyadic, ContextualNode, Node, Program},
    errors::{Error, Hydrator},
    helpers::extend,
    object::{ContextualObject, Object},
    scope::MutScope,
    types::{Float, Int, Num, VariablySized},
};

pub mod builtins;
pub mod repl;

pub fn eval<'a>(
    prog: Program<'a>,
    scope: MutScope<'a>,
    h: Hydrator,
) -> miette::Result<ContextualObject<'a>> {
    let mut result: ContextualObject = Object::Null.anonymous();
    for node in prog.tree {
        result = step(&node, scope.clone(), h.clone())?;
        if let Object::Return(expr) = &result.0 {
            result = *expr.clone();
            break;
        }
    }

    Ok(result)
}

fn step<'a>(
    node: &ContextualNode<'a>,
    scope: MutScope<'a>,
    h: Hydrator,
) -> Result<ContextualObject<'a>, Error> {
    match node.0.clone() {
        // Literals
        Node::Float(v) => Ok(Object::Float(Float::fit(v)).provide_context(node.1.clone())),
        Node::Int(v) => Ok(Object::Integer(Int::fit(v)).provide_context(node.1.clone())),
        Node::Bool(v) => Ok(Object::Bool(v).provide_context(node.1.clone())),
        Node::String(v) => Ok(Object::String(v).provide_context(node.1.clone())),

        // Operations
        Node::DyadicOp { verb, lhs, rhs } => step_dyad(verb, *lhs, *rhs, scope, h.clone()),

        // Variables
        Node::Delclaration { ident, expr, .. } => {
            scope
                .borrow_mut()
                .set(&ident, step(&*expr, scope.clone(), h.clone())?);
            Ok(Object::Null.anonymous())
        }

        // Return
        Node::Return(expr) => {
            Ok(Object::Return(Box::new(step(&*expr, scope, h)?)).provide_context(node.1.clone()))
        }

        // Identifiers
        Node::Ident(ident) => Ok(scope.borrow().get(&ident).unwrap().clone()),

        // Builitins
        Node::FunctionCall { ident, args } => {
            let v = scope.borrow().get(&ident).ok_or(partial!(
                "evaluating function call",
                format!("Unknown function: {}", ident),
                node.1.clone(),
                h.clone()
            ))?;

            let v = v.clone();
            match v.0 {
                Object::Builtin(_, f) => f(
                    args.into_iter()
                        .map(|a| step(&a, scope.clone(), h.clone()))
                        .try_collect()?,
                    h.clone(),
                ),
                _ => Err(partial!(
                    "evaluating function call",
                    format!("{} is not a function", ident),
                    node.1.clone(),
                    h.clone()
                )),
            }
        }

        _ => todo!(),
    }
}

fn step_dyad<'a>(
    verb: Dyadic,
    left: ContextualNode<'a>,
    right: ContextualNode<'a>,
    scope: MutScope<'a>,
    h: Hydrator,
) -> Result<ContextualObject<'a>, Error> {
    let left = step(&left, scope.clone(), h.clone())?;
    let right = step(&right, scope.clone(), h.clone())?;
    let span = extend(&[left.1.clone(), right.1.clone()]);

    let (left, right) = Object::coerce(left, right, h.clone())?;

    Ok(match (left.0.clone(), right.0.clone()) {
        (Object::Float(a), Object::Float(b)) => match verb {
            Dyadic::Add => Object::Float(Float::fit(a.to_max_value() + b.to_max_value())),
            Dyadic::Subtract => Object::Float(Float::fit(a.to_max_value() - b.to_max_value())),
            Dyadic::Multiply => Object::Float(Float::fit(a.to_max_value() * b.to_max_value())),
            Dyadic::Divide => Object::Float(Float::fit(a.to_max_value() / b.to_max_value())),
            Dyadic::Pow => Object::Float(Float::fit(a.to_max_value().powf(b.to_max_value()))),
            Dyadic::Equality => Object::Bool(a.to_max_value() == b.to_max_value()),
        },
        (Object::Integer(a), Object::Integer(b)) => match verb {
            Dyadic::Add => Object::Integer(Int::fit(a.to_max_value() + b.to_max_value())),
            Dyadic::Subtract => Object::Integer(Int::fit(a.to_max_value() - b.to_max_value())),
            Dyadic::Multiply => Object::Integer(Int::fit(a.to_max_value() * b.to_max_value())),
            Dyadic::Divide => Object::Integer(Int::fit(a.to_max_value() / b.to_max_value())),
            Dyadic::Pow => Object::Integer(Int::fit(a.to_max_value().pow(b.to_max_value() as u32))),
            Dyadic::Equality => Object::Bool(a.to_max_value() == b.to_max_value()),
        },
        (Object::String(a), Object::String(b)) => match verb {
            Dyadic::Add => Object::String(format!("{}{}", a, b)),
            Dyadic::Equality => Object::Bool(a == b),
            _ => {
                return Err(partial!(
                    "evaluating dyadic",
                    format!("can't use verb {} on strings", verb.to_symbol()),
                    "You can still use '+' to concat, and '==' to compare strings.",
                    span,
                    h.clone()
                ))
            }
        },
        _ => {
            return Err(partial!(
                "evaluating dyadic",
                format!(
                    "can't use verb {} on type {}",
                    verb.to_symbol(),
                    left.0.typed()
                ),
                span,
                h.clone()
            ))
        }
    }
    .provide_context(span))
}
