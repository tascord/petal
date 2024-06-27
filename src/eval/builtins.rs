use std::ops::RangeInclusive;

use crate::{
    errors::{Error, Hydrator}, eval::repl::ReplDisplay, helpers::extend, object::{ContextualObject, Object}
};

pub static BUILTINS: &[&str] = &["print"];

pub fn get_builtin<'a>(ident: &str) -> Option<ContextualObject<'a>> {
    Some(
        match ident {
            "print" => Object::Builtin(ident.to_string(), false, print),
            _ => return None,
        }
        .anonymous(),
    )
}

//

fn print<'a>(a: Vec<ContextualObject<'a>>, _: Hydrator) -> Result<ContextualObject<'a>, Error> {
    println!(
        "{}",
        a.iter()
            .map(|i| i.0.clone().pretty_print())
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok(Object::Null.anonymous())
}

//

pub fn assert_args_len<'a>(
    args: &Vec<ContextualObject<'a>>,
    len: usize,
    h: Hydrator,
) -> Result<(), Error> {
    if args.len() != len {
        return Err(partial!(
            "Invalid number of arguments",
            format!("Expected {} arguments, got {}", len, args.len()),
            extend(
                args.iter()
                    .map(|a| a.1.clone())
                    .collect::<Vec<_>>()
                    .as_slice()
            ),
            h.clone()
        ));
    }
    Ok(())
}

pub fn assert_args_range<'a>(
    args: &Vec<ContextualObject<'a>>,
    range: RangeInclusive<usize>,
    h: Hydrator,
) -> Result<(), Error> {
    if !range.contains(&args.len()) {
        return Err(partial!(
            "Invalid number of arguments",
            format!(
                "Expected between {} and {} arguments, got {}",
                range.start(),
                range.end(),
                args.len()
            ),
            extend(
                args.iter()
                    .map(|a| a.1.clone())
                    .collect::<Vec<_>>()
                    .as_slice()
            ),
            h.clone()
        ));
    }
    Ok(())
}
