#![allow(dead_code)]

use eval::repl;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./pet.pest"]
struct PetParser;

#[macro_use]
mod errors;
mod ast;
mod eval;
mod helpers;
mod object;
mod scope;
mod types;

fn main() {
    repl::repl()
}
