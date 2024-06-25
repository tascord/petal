#![allow(dead_code)]

use ast::Program;
use clap::{arg, command, Parser as CommandParser};
use eval::repl;
use itertools::Itertools;
use miette::bail;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./pet.pest"]
struct PetParser;

#[derive(CommandParser, Debug)]
#[command(version = "1.0.0", about = "Interpreter for petal")]
struct Args {
    #[arg(name = "FILES", help = "The files to run")]
    files: Vec<String>,
    #[arg(short = 'q', default_value = "false", help = "Quiet mode")]
    quiet: bool,
}

#[macro_use]
mod errors;
mod ast;
mod eval;
mod helpers;
mod object;
mod scope;
mod types;

fn main() -> miette::Result<()> {
    let args = Args::parse();

    if args.files.is_empty() {
        repl::repl();
        return Ok(());
    };

    let files = args.files.iter().flat_map(|loc| {
        if !loc.contains('*') {
            vec![loc.to_string()]
        } else {
            glob::glob(&loc)
                .unwrap()
                .filter_map(Result::ok)
                .map(|p| p.into_os_string().to_str().unwrap().to_string())
                .collect::<Vec<_>>()
        }
    });

    let scope = scope::Scope::new();

    files
        .map(|path| {
            let content = std::fs::read_to_string(&path).unwrap();
            match Program::make(content, Some(path)).map(|p| p.eval(Some(scope.clone()))) {
                Err(e) => bail!(e),
                Ok(v) => match v {
                    Ok(_) => Ok(()),
                    Err(e) => bail!(e),
                },
            }
        })
        .try_collect()?;

    Ok(())
}
