use itertools::Itertools;
use owo_colors::OwoColorize;
use rustyline::{
    hint::{Hint, Hinter},
    history::DefaultHistory,
    Completer, Helper, Highlighter, Validator,
};

use crate::{
    ast::Program,
    scope::{MutScope, Scope},
};

use super::builtins;

#[derive(Completer, Helper, Validator, Highlighter)]
struct PetalHinter<'a>(MutScope<'a>);

#[derive(Debug, PartialEq, Eq)]
struct CommandHint(pub String);

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.0
    }

    fn completion(&self) -> Option<&str> {
        Some(&self.0)
    }
}

impl CommandHint {
    fn suffix(&self, strip_chars: usize) -> CommandHint {
        CommandHint(self.0[strip_chars..].to_owned())
    }
}

impl<'a> Hinter for PetalHinter<'a> {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        
        let (snip, cut) = &get_snippet_from_line(line);
        
        if pos == 0 || snip.trim().is_empty() {
            return None;
        }

        let mut hints = self.0.clone().borrow().list_vars();
        hints.extend(
            vec!["let", "fn", "if", "else", "exit"]
                .into_iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>(),
        );
        hints.extend(
            builtins::BUILTINS
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>(),
        );

        hints
            .into_iter()
            .filter(|hint| hint.starts_with(snip))
            .sorted_by({
                move |a, b| {
                    let a = a.chars().zip(line.chars()).filter(|(a, b)| a != b).count();
                    let b = b.chars().zip(line.chars()).filter(|(a, b)| a != b).count();
                    a.cmp(&b)
                }
            })
            .next()
            .map(|hint| CommandHint(hint[pos - cut..].to_string()))
    }
}

pub fn repl<'a>() {
    print!("\x1B[2J\x1B[1;1H");
    println!("# {} repl", "petal".bright_magenta());
    println!("type 'exit' to exit\n");

    let repl_scope = Scope::new();
    let mut rl = rustyline::Editor::<PetalHinter, DefaultHistory>::new().unwrap();
    rl.set_helper(Some(PetalHinter(repl_scope.clone())));

    loop {
        let scope = repl_scope.clone();
        match rl.readline(&format!("{}{}", ">".dimmed(), " ".white())) {
            Err(_) => {}
            Ok(program) => {
                if program == "exit" {
                    break;
                }

                let program = format!(
                    "{program}{}",
                    match should_append_semicolon(&program) {
                        true => ";",
                        false => "",
                    }
                );

                match Program::<'a>::make(program.to_string(), None) {
                    Ok(p) => match p.eval(Some(scope)) {
                        Ok(v) => println!("{}\n", v.0.pretty_print()),
                        Err(e) => println!("\n{:?}\n", e),
                    },
                    Err(e) => println!("\n{:?}\n", e),
                }
            }
        }
    }
}

fn should_append_semicolon(i: &str) -> bool {
    !vec!["struct", "trait", "fn", "pub", "local", "impl", "return"]
        .iter()
        .any(|t| i.starts_with(t))
}

fn get_snippet_from_line(line: &str) -> (String, usize) {
    let reset_chars = vec!['(', ',', ';', ' ', '.'];
    let mut in_string = false;
    let mut buf = String::new();

    let mut last_char = ' ';
    for c in line.chars() {
        if in_string && c == '"' && last_char != '\\' {
            in_string = false;
            buf.clear();
            continue;
        }

        if in_string {
            continue;
        }

        last_char = c;
        if reset_chars.iter().any(|ch| *ch == c) {
            buf.clear();
        } else {
            buf.push(c);
        }
    }

    (buf.clone(), line.len() - buf.len())
}

pub trait ReplDisplay {
    fn pretty_print(&self) -> String;
}
