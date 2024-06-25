use crate::{
    errors::{Error, Hydrator},
    helpers::extend,
    object::{ContextualObject, Object},
    types::{Int, VariablySized},
};

pub static BUILTINS: &[&str] = &["len", "print"];

pub fn get_builtin<'a>(ident: &str) -> Option<ContextualObject<'a>> {
    Some(
        match ident {
            "len" => Object::Builtin(ident.to_string(), len),
            "print" => Object::Builtin(ident.to_string(), print),
            _ => return None,
        }
        .anonymous(),
    )
}

//

fn len<'a>(a: Vec<ContextualObject<'a>>, h: Hydrator) -> Result<ContextualObject<'a>, Error> {
    assert_args_len(&a, 1, h.clone())?;
    let v = a.first().unwrap();
    Ok(Object::Integer(Int::fit(match &v.0.clone() {
        Object::Array(arr) => arr.len() as i128,
        Object::String(v) => v.len() as i128,
        _ => {
            return Err(partial!(
                "Invalid type provided",
                format!("Can't get length of type {}", v.0.typed()),
                v.1,
                h.clone()
            ))
        }
    }))
    .anonymous())
}

//

fn print<'a>(a: Vec<ContextualObject<'a>>, _: Hydrator) -> Result<ContextualObject<'a>, Error> {
    println!(
        "{}",
        a.iter()
            .map(|i| i.0.clone().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok(Object::Null.anonymous())
}

//

fn assert_args_len<'a>(
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
