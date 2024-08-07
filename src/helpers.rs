use pest::Span;

pub fn extend<'a>(spans: &[Span<'a>]) -> Span<'a> {
    if spans.is_empty() {
        return Span::new("", 0, 0).unwrap();
    }

    let positions = spans
        .iter()
        .map(|s| (s.start(), s.end()))
        .collect::<Vec<_>>();
    let start = positions.iter().map(|(s, _)| *s).min().unwrap();
    let end = positions.iter().map(|(_, e)| *e).max().unwrap();

    Span::new(
        spans
            .first()
            .unwrap_or(&Span::new("", 0, 0).unwrap())
            .get_input(),
        start,
        end,
    )
    .unwrap_or(Span::new("", 0, 0).unwrap())
}
