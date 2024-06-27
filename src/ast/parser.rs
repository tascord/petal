use itertools::Itertools;
use pest::iterators::Pair;

use crate::{
    errors::{Error, Hydrator},
    Rule,
};

use super::{
    op::{get_dyadic, get_dyads, get_monads, get_mondaic},
    ContextualNode, Node,
};
type NodeRes<'a> = Result<ContextualNode<'a>, Error>;

pub fn build_ast_from_expr<'a>(e: Pair<'a, Rule>, h: Hydrator) -> NodeRes<'a> {
    match e.as_rule() {
        Rule::expr | Rule::ltl => {
            build_ast_from_expr(e.clone().into_inner().next().unwrap(), h.clone())
        }
        Rule::terms => {
            let terms = e
                .clone()
                .into_inner()
                .map(|t| build_ast_from_term(t.clone(), h.clone()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(match terms.len() {
                1 => terms.first().unwrap().clone(),
                _ => Node::Terms(terms).provide_context(e.as_span()),
            })
        }

        Rule::string
        | Rule::boolean
        | Rule::float
        | Rule::int
        | Rule::null
        | Rule::identifier
        | Rule::array => build_ast_from_term(e.clone(), h),

        Rule::monadic => {
            let (verb, expr) = takes!(e, 2);
            build_mondaic(verb, build!(expr, h), h)
        }
        Rule::dyadic => {
            let mut inner = e.clone().into_inner().rev();
            let mut right = build!(inner.next().unwrap(), h);

            for chunk in &inner.chunks(2) {
                let (verb, left) = chunk.collect_tuple().unwrap();
                let left = build!(left, h.clone());
                right = build_dyadic(verb, left, right, h.clone())?;
            }

            Ok(right)
        }

        Rule::var_decl => {
            if has!(e.clone(), "colon") {
                let (_, ident, _, typed, expr) = takes!(e.clone(), 5);
                Ok(Node::Delclaration {
                    ident: ident!(ident, h.clone())?,
                    typed: Some(typed.as_str().to_string()),
                    expr: Box::new(build_ast_from_expr(expr, h.clone())?),
                }
                .provide_context(e.as_span()))
            } else {
                let (_, ident, expr) = takes!(e.clone(), 3);
                Ok(Node::Delclaration {
                    ident: ident!(ident, h.clone())?,
                    typed: None,
                    expr: Box::new(build_ast_from_expr(expr, h.clone())?),
                }
                .provide_context(e.as_span()))
            }
        }

        Rule::fn_decl => {
            let (outline, block) = takes!(e.clone(), 2);
            let body = block
                .clone()
                .into_inner()
                .map(|t| build_ast_from_expr(t, h.clone()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| {
                    partial!(
                        "building function body",
                        e.to_string(),
                        block.as_span(),
                        h.clone()
                    )
                });

            let identifier = outline
                .clone()
                .into_inner()
                .find(|p| p.as_rule() == Rule::identifier)
                .unwrap();
            let return_type = outline
                .clone()
                .into_inner()
                .last()
                .filter(|p| p.as_rule() == Rule::typed)
                .map(|n| n.as_str().to_string());
            let args = outline
                .clone()
                .into_inner()
                .find(|p| p.as_rule() == Rule::typed_args)
                .map(|n| {
                    n.into_inner()
                        .map(|n| {
                            let (a, _, b) = takes!(n, 3);
                            (a.as_str().to_string(), b.as_str().to_string())
                        })
                        .collect::<Vec<_>>()
                });

            Ok(Node::FunctionDeclaration {
                ident: identifier.as_str().to_string(),
                args: args.unwrap_or_default(),
                return_type,
                body: body?,
            }
            .provide_context(e.as_span()))
        }

        Rule::fn_call => {
            let args = e.clone().into_inner().collect::<Vec<_>>();
            let ident = ident!(args.first().unwrap(), h.clone())?;
            Ok(Node::FunctionCall {
                ident,
                args: args
                    .into_iter()
                    .skip(1)
                    .map(|t| build_ast_from_term(t.clone(), h.clone()))
                    .collect::<Result<Vec<_>, _>>()?,
            }
            .provide_context(e.as_span()))
        }

        Rule::index => {
            let (left, right) = takes!(e.clone(), 2);
            Ok(Node::Index(
                Box::new(build!(left, h.clone())),
                Box::new(build!(right, h.clone())),
            )
            .provide_context(e.as_span()))
        }

        Rule::ret_stmt => {
            let expr = e.clone().into_inner().next().unwrap();
            Ok(
                Node::Return(Box::new(build_ast_from_expr(expr, h.clone())?))
                    .provide_context(e.as_span()),
            )
        }

        _ => {
            eprintln!("{:?} not yet implemented", e.as_rule());
            todo!()
        }
    }
}

fn build_ast_from_term<'a>(t: Pair<'a, Rule>, h: Hydrator) -> NodeRes<'a> {
    match t.as_rule() {
        Rule::expr => build_ast_from_expr(t.clone(), h).map(|e| e.0.clone()),
        Rule::identifier => Ok(Node::Ident(String::from(t.as_str()))),

        Rule::string => Ok(Node::String(
            t.as_str()[1..t.as_str().len() - 1].to_string(),
        )),
        Rule::boolean => Ok(Node::Bool(t.as_str().trim().parse::<bool>().map_err(
            |er| partial!("parsing boolean", er.to_string(), t.as_span(), h.clone()),
        )?)),
        Rule::float => Ok(Node::Float(t.as_str().trim().parse::<f64>().map_err(
            |er| partial!("parsing float", er.to_string(), t.as_span(), h.clone()),
        )?)),
        Rule::int => Ok(Node::Int(t.as_str().trim().parse::<i128>().map_err(
            |er| partial!("parsing integer", er.to_string(), t.as_span(), h.clone()),
        )?)),
        Rule::array => {
            let elements = t
                .clone()
                .into_inner()
                .map(|t| build_ast_from_term(t, h.clone()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Node::Array(elements))
        }
        Rule::null => Ok(Node::Null),

        _ => todo!(),
    }
    .map(|n| n.provide_context(t.as_span()))
}

fn build_mondaic<'a>(pair: Pair<'a, Rule>, expr: ContextualNode<'a>, h: Hydrator) -> NodeRes<'a> {
    Ok(Node::MondaicOp {
        verb: get_mondaic(pair.as_str().to_string()).ok_or(partial!(
            "parsing mondaic",
            format!("Unexpected verb: {}", pair.as_str()),
            format!("Try one of: {}", get_monads().join(", ")),
            pair.as_span(),
            h.clone()
        ))?,
        expr: Box::new(expr),
    }
    .provide_context(pair.as_span()))
}

fn build_dyadic<'a>(
    pair: Pair<'a, Rule>,
    lhs: ContextualNode<'a>,
    rhs: ContextualNode<'a>,
    h: Hydrator,
) -> NodeRes<'a> {
    Ok(Node::DyadicOp {
        verb: get_dyadic(pair.as_str().to_string()).ok_or(partial!(
            "parsing dyadic",
            format!("Unexpected verb: '{}'", pair.as_str()),
            format!("Try one of: {}", get_dyads().join(", ")),
            pair.as_span(),
            h.clone()
        ))?,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
    .provide_context(pair.as_span()))
}
