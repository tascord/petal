use std::rc::Rc;

use op::{Dyadic, Mondaic};

use parser::build_ast_from_expr;
use pest::{Parser, Span};

use crate::{
    errors::{Error, Hydrator},
    eval::eval,
    object::ContextualObject,
    scope::{MutScope, Scope},
    PetParser, Rule,
};

#[macro_use]
mod macros;
pub mod op;
mod parser;

#[derive(Clone, Debug)]
pub struct ContextualNode<'a>(pub Node<'a>, pub Span<'a>);

#[derive(Clone, Debug)]
pub enum Node<'a> {
    // Literals
    Float(f64),
    Int(i128),
    Bool(bool),
    String(String),
    Null,

    // Operators
    MondaicOp {
        verb: Mondaic,
        expr: Box<ContextualNode<'a>>,
    },

    DyadicOp {
        verb: Dyadic,
        lhs: Box<ContextualNode<'a>>,
        rhs: Box<ContextualNode<'a>>,
    },

    Terms(Vec<ContextualNode<'a>>),
    Ident(String),
    Index(Box<ContextualNode<'a>>, Vec<ContextualNode<'a>>),
    Return(Box<ContextualNode<'a>>),

    Delclaration {
        ident: String,
        typed: Option<String>,
        expr: Box<ContextualNode<'a>>,
    },

    FunctionDeclaration {
        ident: String,
        args: Vec<(String, String)>,
        return_type: Option<String>,
        body: Vec<ContextualNode<'a>>,
    },

    FunctionCall {
        ident: String,
        args: Vec<ContextualNode<'a>>,
    },

    Struct {
        ident: String,
        typed: String,
        fields: Vec<(String, ContextualNode<'a>)>,
    },

    Array(Vec<ContextualNode<'a>>),
}

impl<'a> Node<'a> {
    pub fn provide_context(self, span: Span<'a>) -> ContextualNode<'a> {
        ContextualNode(self, span)
    }
}

impl ContextualNode<'_> {
    pub fn inner(&self) -> &Node {
        &self.0
    }

    pub fn span(&self) -> Span<'_> {
        self.1.clone()
    }
}

pub struct Program<'a> {
    pub tree: Vec<ContextualNode<'a>>,
    hydrator: Hydrator,
}

impl<'a> Program<'a> {
    pub fn make(input: String, path: Option<String>) -> miette::Result<Program<'a>> {
        let refed = Box::leak(input.clone().into_boxed_str());

        let h: Hydrator = (
            path.unwrap_or_else(|| "#pet.eval".to_string()),
            Rc::new(input.to_string()),
        );

        let pairs = PetParser::parse(Rule::program, refed).map_err(|e| {
            let span = match e.line_col {
                pest::error::LineColLocation::Pos((l, c)) => (l, c),
                pest::error::LineColLocation::Span((l, c), _) => (l, c),
            };

            partial!(
                "parsing input",
                e.to_string(),
                Span::new(&h.1.to_string(), span.0, span.1).unwrap(),
                h
            )
        })?;

        let mut ast: Vec<ContextualNode<'a>> = vec![];
        for pair in pairs {
            match pair.as_rule() {
                Rule::expr | Rule::ltl => {
                    ast.push(build_ast_from_expr(pair, h.clone())?);
                }
                _ => {}
            }
        }

        Ok(Program {
            tree: ast,
            hydrator: h.clone(),
        })
    }

    pub fn eval(self, scope: Option<MutScope<'a>>) -> miette::Result<ContextualObject<'a>> {
        let h = self.hydrator.clone();
        eval(self, scope.unwrap_or(Scope::new("#pet.repl")), h)
    }
}
