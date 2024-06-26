use crate::{
    errors::{Error, Hydrator},
    object::{ContextualObject, Object},
    types::{Int, VariablySized},
};

use super::builtins::assert_args_len;

pub fn list_instrinsics(typed: &str) -> &[&str] {
    match typed {
        "string" => &["len"],
        _ => &[],
    }
}

pub fn get_intrinsic<'a>(ident: &str) -> Option<ContextualObject<'a>> {
    Some(
        match ident {
            "len" => Object::Builtin(ident.to_string(), true, len),
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
                "checking types",
                format!("Can't get length of type {}", v.0.typed()),
                v.1,
                h.clone()
            ))
        }
    }))
    .anonymous())
}
