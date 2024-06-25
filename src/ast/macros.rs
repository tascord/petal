macro_rules! takes {
    ($pairs:expr, $amount:literal) => {{
        use itertools::Itertools;
        $pairs
            .clone()
            .into_inner()
            .take($amount)
            .collect_tuple()
            .unwrap()
    }};
}

macro_rules! has {
    ($pairs:expr, $typed:literal) => {
        $pairs
            .into_inner()
            .any(|a| format!("{:?}", a.as_rule()) == $typed.to_string())
    };
}

macro_rules! ident {
    ($pair:expr, $h:expr) => {
        match format!("{:?}", $pair.as_rule()) == "identifier".to_string() {
            true => Ok($pair.as_str().to_string()),
            false => Err(partial!(
                "reading identifier",
                format!("Expected identifier, found {:?}", $pair.as_rule()),
                $pair.as_span(),
                $h.clone()
            )),
        }
    };
}

macro_rules! build {
    ($e:expr, $h:expr) => {
        build_ast_from_expr($e, $h.clone())?
    };
}
