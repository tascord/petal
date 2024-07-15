use itertools::Itertools;

use crate::{
    ast::{op::Dyadic, ContextualNode, Node, Program},
    errors::{Error, Hydrator},
    helpers::extend,
    object::{ContextualObject, Object},
    scope::{MutScope, Scope},
    types::{Float, Int, Num, VariablySized},
};

pub mod builtins;
pub mod intrinsics;
pub mod repl;
pub mod tasks;

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

pub fn step<'a>(
    node: &ContextualNode<'a>,
    scope: MutScope<'a>,
    h: Hydrator,
) -> Result<ContextualObject<'a>, Error> {
    // println!("Step :: {:?}", node.0);
    match node.0.clone() {
        // Literals
        Node::Float(v) => Ok(Object::Float(Float::fit(v)).provide_context(node.1.clone())),
        Node::Int(v) => Ok(Object::Integer(Int::fit(v)).provide_context(node.1.clone())),
        Node::Bool(v) => Ok(Object::Bool(v).provide_context(node.1.clone())),
        Node::String(v) => Ok(Object::String(v).provide_context(node.1.clone())),
        Node::Array(v) => {
            let v = v
                .into_iter()
                .map(|n| step(&n, scope.clone(), h.clone()))
                .try_collect()?;
            Ok(Object::Array(v).provide_context(node.1.clone()))
        }
        Node::Null => Ok(Object::Null.provide_context(node.1.clone())),

        // Operations
        Node::DyadicOp { verb, lhs, rhs } => step_dyad(verb, *lhs, *rhs, scope, h.clone()),

        // Variables
        Node::Delclaration { ident, expr, .. } => {
            let value = step(&*expr, scope.clone(), h.clone())?;
            scope
                .borrow_mut()
                .set(&ident, value.clone(), node.1, h.clone())?;
            Ok(value)
        }

        Node::Assignment { ident, expr } => {
            let value = step(&*expr, scope.clone(), h.clone())?;
            scope
                .borrow_mut()
                .assign(&ident, value.clone(), node.1, h.clone())?;
            Ok(value)
        }

        // Return
        Node::Return(expr) => {
            Ok(Object::Return(Box::new(step(&*expr, scope, h)?)).provide_context(node.1.clone()))
        }

        // Identifiers
        Node::Ident(ident) => Ok(scope
            .borrow()
            .get(&ident)
            .ok_or(partial!(
                "finding variable",
                format!("Unknown identifier: {}", ident),
                node.1.clone(),
                h.clone()
            ))?
            .clone()),

        // Functions
        Node::FunctionCall { ident, args } => {
            let v = scope.borrow().get(&ident).ok_or(partial!(
                "evaluating function call",
                format!("Unknown function: {}", ident),
                node.1.clone(),
                h.clone()
            ))?;

            let v = v.clone();
            match v.0 {
                Object::Builtin(..) | Object::Lambda(..) => {
                    let args = args
                        .into_iter()
                        .map(|a| step(&a, scope.clone(), h.clone()))
                        .try_collect()?;

                    v.call(args, scope.clone(), h)
                }

                _ => Err(partial!(
                    "evaluating function call",
                    format!("{} is not a function", ident),
                    node.1.clone(),
                    h.clone()
                )),
            }
        }

        // Indexing
        Node::Index(left, right) => {
            let left = step(&*left, scope.clone(), h.clone())?;
            let mut container: MutScope<'a> = Scope::new_from_object(left, scope.clone())?;

            for (index, item) in right.clone().into_iter().enumerate() {
                let obj = match item.0 {
                    Node::Int(v) => container
                        .borrow()
                        .get(&v.to_string())
                        .unwrap_or(Object::Null.anonymous()),
                    Node::String(v) | Node::Ident(v) => container
                        .borrow()
                        .get(&v)
                        .unwrap_or(Object::Null.anonymous()),
                    Node::FunctionCall { ident, args } => {
                        let object = container
                            .borrow()
                            .get(&ident)
                            .ok_or(partial!(
                                "evaluating index",
                                format!("Unknown element: {}", ident),
                                item.1.clone(),
                                h.clone()
                            ))?
                            .0;

                        match object {
                            Object::Builtin(_, slf, f) => {
                                let prev = container.borrow().name.clone();
                                container.clone().borrow_mut().name = "object_fncall".to_string();

                                let mut args: Vec<ContextualObject<'a>> = args
                                    .into_iter()
                                    .map(|a| step(&a, container.clone(), h.clone()))
                                    .try_collect()?;

                                container.clone().borrow_mut().name = prev;

                                if slf {
                                    let slf = container.borrow().get_self().ok_or(partial!(
                                        "evaluating function call",
                                        "No self provided for method call".to_string(),
                                        item.1.clone(),
                                        h.clone()
                                    ))?;

                                    args.insert(0, slf.clone());
                                }

                                f(args, h.clone(), scope.clone())?
                            }
                            _ => {
                                return Err(partial!(
                                    "evaluating index",
                                    format!("Can't index with this type"),
                                    item.1.clone(),
                                    h.clone()
                                ))
                            }
                        }
                    }
                    _ => {
                        return Err(partial!(
                            "evaluating index",
                            "Can't index with this type".to_string(),
                            item.1.clone(),
                            h.clone()
                        ))
                    }
                };

                if index == right.len() - 1 {
                    return Ok(obj);
                } else {
                    container = Scope::new_from_object(obj, scope.clone())?;
                }
            }

            Ok(Object::Null.anonymous())
        }

        Node::Lambda {
            args,
            return_type,
            body,
        } => Ok(Object::Lambda(args, return_type, body).provide_context(node.1.clone())),

        Node::Conditional { arms, else_arm } => {
            for (cond, body) in arms {
                let cond = step(&cond, scope.clone(), h.clone())?;
                if let Object::Bool(true) = cond.0 {
                    for node in body {
                        let result = step(&node, scope.clone(), h.clone())?;
                        if let Object::Return(expr) = &result.0 {
                            return Ok(*expr.clone());
                        }
                    }
                    return Ok(Object::Null.anonymous());
                }
            }

            if let Some(else_arm) = else_arm {
                for node in else_arm {
                    let result = step(&node, scope.clone(), h.clone())?;
                    if let Object::Return(expr) = &result.0 {
                        return Ok(*expr.clone());
                    }
                }
            }

            Ok(Object::Null.anonymous())
        }

        Node::LoopWhile { condition, body } => {
            let mut loops = 0;

            loop {
                loops += 1;
                let cond = step(&*condition, scope.clone(), h.clone())?;
                if let Object::Bool(true) = cond.0 {
                    for node in &body {
                        let result = step(&node, scope.clone(), h.clone())?;
                        if let Object::Return(expr) = &result.0 {
                            return Ok(*expr.clone());
                        }
                    }
                } else {
                    break;
                }
            }

            Ok(Object::Integer(Int::fit(loops)).anonymous())
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
            Dyadic::GreaterThan => Object::Bool(a.to_max_value() > b.to_max_value()),
            Dyadic::LessThan => Object::Bool(a.to_max_value() < b.to_max_value()),
            Dyadic::GreaterThanOrEqual => Object::Bool(a.to_max_value() >= b.to_max_value()),
            Dyadic::LessThanOrEqual => Object::Bool(a.to_max_value() <= b.to_max_value()),
            _ => {
                return Err(partial!(
                    "evaluating dyadic",
                    format!("can't use verb {} on floats", verb.to_symbol()),
                    span,
                    h.clone()
                ))
            }
        },
        (Object::Integer(a), Object::Integer(b)) => match verb {
            Dyadic::Add => Object::Integer(Int::fit(a.to_max_value() + b.to_max_value())),
            Dyadic::Subtract => Object::Integer(Int::fit(a.to_max_value() - b.to_max_value())),
            Dyadic::Multiply => Object::Integer(Int::fit(a.to_max_value() * b.to_max_value())),
            Dyadic::Divide => Object::Integer(Int::fit(a.to_max_value() / b.to_max_value())),
            Dyadic::Pow => Object::Integer(Int::fit(a.to_max_value().pow(b.to_max_value() as u32))),
            Dyadic::Equality => Object::Bool(a.to_max_value() == b.to_max_value()),
            Dyadic::GreaterThan => Object::Bool(a.to_max_value() > b.to_max_value()),
            Dyadic::LessThan => Object::Bool(a.to_max_value() < b.to_max_value()),
            Dyadic::GreaterThanOrEqual => Object::Bool(a.to_max_value() >= b.to_max_value()),
            Dyadic::LessThanOrEqual => Object::Bool(a.to_max_value() <= b.to_max_value()),
            _ => {
                return Err(partial!(
                    "evaluating dyadic",
                    format!("can't use verb {} on integers", verb.to_symbol()),
                    span,
                    h.clone()
                ))
            }
        },
        (Object::Bool(a), Object::Bool(b)) => match verb {
            Dyadic::Equality => Object::Bool(a == b),
            Dyadic::Or => Object::Bool(a || b),
            Dyadic::And => Object::Bool(a && b),
            _ => {
                return Err(partial!(
                    "evaluating dyadic",
                    format!("can't use verb {} on bools", verb.to_symbol()),
                    span,
                    h.clone()
                ))
            }
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
