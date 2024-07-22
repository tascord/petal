use itertools::Itertools;

use crate::{
    errors::{Error, Hydrator},
    object::{ContextualObject, Object},
    scope::MutScope,
    types::{Int, VariablySized},
};

use super::{
    builtins::{assert_args_len, assert_args_range},
    tasks::Microtasker,
};

pub fn list_instrinsics(typed: &str) -> &[&str] {
    match typed {
        "string" => &["to_string", "len", "split"],
        "array" => &["to_string", "len", "join", "map"],
        "map" => &["to_string", "keys", "values", "entries"],
        "promise" => &["to_string", "await"],
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
            "map" => Object::Builtin(ident.to_string(), true, map),
            "await" => Object::Builtin(ident.to_string(), true, wait),
            _ => return None,
        }
        .anonymous(),
    )
}

//

fn to_string<'a>(
    a: Vec<ContextualObject<'a>>,
    h: Hydrator,
    _: MutScope<'a>,
) -> Result<ContextualObject<'a>, Error> {
    assert_args_len(&a, 1, h.clone())?;
    Ok(Object::String(a.first().unwrap().0.to_string()).anonymous())
}

fn len<'a>(
    a: Vec<ContextualObject<'a>>,
    h: Hydrator,
    _: MutScope<'a>,
) -> Result<ContextualObject<'a>, Error> {
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

fn split<'a>(
    a: Vec<ContextualObject<'a>>,
    h: Hydrator,
    _: MutScope<'a>,
) -> Result<ContextualObject<'a>, Error> {
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

fn join<'a>(
    a: Vec<ContextualObject<'a>>,
    h: Hydrator,
    _: MutScope<'a>,
) -> Result<ContextualObject<'a>, Error> {
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

fn keys<'a>(
    a: Vec<ContextualObject<'a>>,
    h: Hydrator,
    _: MutScope<'a>,
) -> Result<ContextualObject<'a>, Error> {
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

fn values<'a>(
    a: Vec<ContextualObject<'a>>,
    h: Hydrator,
    _: MutScope<'a>,
) -> Result<ContextualObject<'a>, Error> {
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

fn entries<'a>(
    a: Vec<ContextualObject<'a>>,
    h: Hydrator,
    _: MutScope<'a>,
) -> Result<ContextualObject<'a>, Error> {
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

fn map<'a>(
    a: Vec<ContextualObject<'a>>,
    h: Hydrator,
    s: MutScope<'a>,
) -> Result<ContextualObject<'a>, Error> {
    assert_args_len(&a, 2, h.clone())?;
    let (v, f) = (a.first().unwrap(), a.last().unwrap());
    let v = match &v.0 {
        Object::Array(v) => v,
        _ => {
            return Err(partial!(
                "checking types",
                format!("Can't map type {}", v.0.typed()),
                v.1,
                h.clone()
            ))
        }
    };

    let f = match &f.0 {
        Object::Lambda(..) => f,
        Object::Builtin(..) => f,
        _ => {
            return Err(partial!(
                "checking types",
                format!("Can't map with type {}", f.0.typed()),
                f.1,
                h.clone()
            ))
        }
    };

    v.into_iter()
        .map(|v| f.call(vec![v.clone()], s.clone(), h.clone()))
        .try_collect()
        .map(|v| Object::Array(v).anonymous())
}

//

fn wait<'a>(
    a: Vec<ContextualObject<'a>>,
    h: Hydrator,
    _s: MutScope<'a>,
) -> Result<ContextualObject<'a>, Error> {
    assert_args_len(&a, 1, h.clone())?;
    let id = match &a.first().unwrap().0 {
        Object::Promise(_, v) => v,
        _ => {
            return Err(partial!(
                "checking types",
                format!("Can't wait for type {}", a.first().unwrap().0.typed()),
                a.first().unwrap().1,
                h.clone()
            ))
        }
    };

    let ret: ContextualObject<'static> = Microtasker.write().unwrap().wait(id.clone());

    unsafe {
        Ok(std::mem::transmute::<
            ContextualObject<'static>,
            ContextualObject<'a>,
        >(ret))
    }
}
