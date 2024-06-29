use crate::{
    errors::{Error, Hydrator},
    object::{ContextualObject, Object},
    types::{Int, VariablySized},
};

use super::builtins::{assert_args_len, assert_args_range};

pub fn list_instrinsics(typed: &str) -> &[&str] {
    match typed {
        "string" => &["to_string", "len", "split"],
        "array" => &["to_string", "len", "join"],
        "map" => &["to_string", "keys", "values", "entries"],
        _ => &["to_string"],
    }
}

pub fn get_intrinsic<'a>(ident: &str) -> Option<ContextualObject<'a>> {
    Some(
        match ident {
            "to_string" => Object::Builtin(ident.to_string(), true, to_string),
            "len" => Object::Builtin(ident.to_string(), true, len),
            "split" => Object::Builtin(ident.to_string(), true, split),
            "join" => Object::Builtin(ident.to_string(), true, join),
            "keys" => Object::Builtin(ident.to_string(), true, keys),
            "values" => Object::Builtin(ident.to_string(), true, values),
            "entries" => Object::Builtin(ident.to_string(), true, entries),
            _ => return None,
        }
        .anonymous(),
    )
}

//

fn to_string<'a>(a: Vec<ContextualObject<'a>>, h: Hydrator) -> Result<ContextualObject<'a>, Error> {
    assert_args_len(&a, 1, h.clone())?;
    Ok(Object::String(a.first().unwrap().0.to_string()).anonymous())
}

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

fn split<'a>(a: Vec<ContextualObject<'a>>, h: Hydrator) -> Result<ContextualObject<'a>, Error> {
    assert_args_range(&a, 1..=2, h.clone())?;

    let (v, sep) = (
        a.get(0).unwrap(),
        a.get(1)
            .cloned()
            .unwrap_or(Object::String("".to_string()).anonymous()),
    );

    let sep = match &sep.0 {
        Object::String(v) => v,
        _ => {
            return Err(partial!(
                "checking types",
                format!("Can't split value of type {}", sep.0.typed()),
                sep.1,
                h.clone()
            ))
        }
    };

    let v = match &v.0 {
        Object::String(v) => v,
        _ => {
            return Err(partial!(
                "checking types",
                format!("Can't split valye of type {}", v.0.typed()),
                v.1,
                h.clone()
            ))
        }
    };

    let mut split = v
        .split(sep)
        .map(|s| Object::String(s.to_string()).anonymous())
        .collect::<Vec<_>>();

    if sep.is_empty() {
        split = split[1..split.len() - 1].to_vec();
    }

    Ok(Object::Array(split).anonymous())
}

fn join<'a>(a: Vec<ContextualObject<'a>>, h: Hydrator) -> Result<ContextualObject<'a>, Error> {
    assert_args_len(&a, 2, h.clone())?;
    let (v, sep) = (a.first().unwrap(), a.last().unwrap());
    let sep = match &sep.0 {
        Object::String(v) => v,
        _ => {
            return Err(partial!(
                "checking types",
                format!("Can't join with type {}", sep.0.typed()),
                sep.1,
                h.clone()
            ))
        }
    };

    let v = match &v.0 {
        Object::Array(v) => v,
        _ => {
            return Err(partial!(
                "checking types",
                format!("Can't join type {}", v.0.typed()),
                v.1,
                h.clone()
            ))
        }
    };

    Ok(Object::String(
        v.iter()
            .map(|o| o.0.to_string())
            .collect::<Vec<_>>()
            .join(sep),
    )
    .anonymous())
}

fn keys<'a>(a: Vec<ContextualObject<'a>>, h: Hydrator) -> Result<ContextualObject<'a>, Error> {
    assert_args_len(&a, 1, h.clone())?;
    let v = a.first().unwrap();
    let v = match &v.0 {
        Object::Map(v) => v,
        _ => {
            return Err(partial!(
                "checking types",
                format!("Can't get keys of type {}", v.0.typed()),
                v.1,
                h.clone()
            ))
        }
    };

    Ok(Object::Array(v.keys().map(|k| k.clone()).collect::<Vec<_>>()).anonymous())
}

fn values<'a>(a: Vec<ContextualObject<'a>>, h: Hydrator) -> Result<ContextualObject<'a>, Error> {
    assert_args_len(&a, 1, h.clone())?;
    let v = a.first().unwrap();
    let v = match &v.0 {
        Object::Map(v) => v,
        _ => {
            return Err(partial!(
                "checking types",
                format!("Can't get values of type {}", v.0.typed()),
                v.1,
                h.clone()
            ))
        }
    };

    Ok(Object::Array(v.values().map(|v| v.clone()).collect::<Vec<_>>()).anonymous())
}

fn entries<'a>(a: Vec<ContextualObject<'a>>, h: Hydrator) -> Result<ContextualObject<'a>, Error> {
    assert_args_len(&a, 1, h.clone())?;
    let v = a.first().unwrap();
    let v = match &v.0 {
        Object::Map(v) => v,
        _ => {
            return Err(partial!(
                "checking types",
                format!("Can't get entries of type {}", v.0.typed()),
                v.1,
                h.clone()
            ))
        }
    };

    Ok(Object::Array(
        v.iter()
            .map(|(k, v)| Object::Array(vec![k.clone(), v.clone()]).anonymous())
            .collect::<Vec<_>>(),
    )
    .anonymous())
}
