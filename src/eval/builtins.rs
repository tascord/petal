use std::{collections::BTreeMap, ops::RangeInclusive};

use crate::{
    errors::{Error, Hydrator}, eval::repl::ReplDisplay, helpers::extend, object::{ContextualObject, Object}, scope::MutScope, types::Num
};

pub static BUILTINS: &[&str] = &[];

macro_rules! map {
    ([$($name:expr),+ $(,)?]) => {
        Object::Map({
            let mut map: BTreeMap<ContextualObject, ContextualObject> = BTreeMap::new();

            $(
                map.insert(
                    Object::String(stringify!($name).to_string()).anonymous(),
                    Object::Builtin(stringify!($name).to_string(), false, $name).anonymous(),
                );
            )+

            map
        })
    };
}

pub fn get_builtin<'a>(ident: &str) -> Option<ContextualObject<'a>> {
    Some(
        match ident {
            "term" => map!([print, clear]),
            "process" => map!([exit]),
            _ => return None,
        }
        .anonymous(),
    )
}

//

fn exit<'a>(a: Vec<ContextualObject<'a>>, h: Hydrator, _: MutScope<'a>) -> Result<ContextualObject<'a>, Error> {
    assert_args_range(&a, 0..=1, h.clone())?;
    std::process::exit(
        match &a
            .first()
            .unwrap_or(&Object::Integer(crate::types::Int::_8(0)).anonymous())
            .0
        {
            Object::Integer(v) => v.to_max_value() as i32,
            _ => {
                return Err(partial!(
                    "Invalid argument",
                    "Expected an integer",
                    extend(a.iter().map(|a| a.1.clone()).collect::<Vec<_>>().as_slice()),
                    h.clone()
                ))
            }
        },
    );
}

//

fn print<'a>(a: Vec<ContextualObject<'a>>, _: Hydrator, _: MutScope<'a>) -> Result<ContextualObject<'a>, Error> {
    println!(
        "{}",
        a.iter()
            .map(|i| i.0.clone().pretty_print())
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok(Object::Null.anonymous())
}

fn clear<'a>(_: Vec<ContextualObject<'a>>, _: Hydrator, _: MutScope<'a>) -> Result<ContextualObject<'a>, Error> {
    print!("\x1B[2J\x1B[1;1H");
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
