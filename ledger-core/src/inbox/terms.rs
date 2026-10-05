//! Reading RDF terms back out of query rows, as the dataset prints them.

/// The text of an IRI term (`<…>`), or of a `mailto:` IRI without the scheme.
pub fn iri(term: &str) -> String {
    let t = term.trim().trim_start_matches('<').trim_end_matches('>');
    t.strip_prefix("mailto:").unwrap_or(t).to_string()
}

/// The lexical form of a literal term (`"…"`, `"…"^^<…>`, `"…"@lang`),
/// unescaped. A term that is not a literal comes back unchanged.
pub fn literal(term: &str) -> String {
    let Some(body) = term.strip_prefix('"') else { return term.to_string() };
    let mut out = String::new();
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => break,
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('u') => out.push(hex(&mut chars, 4)),
                Some('U') => out.push(hex(&mut chars, 8)),
                Some(other) => out.push(other),
                None => {}
            },
            c => out.push(c),
        }
    }
    out
}

fn hex(chars: &mut std::str::Chars<'_>, n: usize) -> char {
    let digits: String = chars.take(n).collect();
    u32::from_str_radix(&digits, 16).ok().and_then(char::from_u32).unwrap_or('\u{fffd}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals_and_iris_read_back() {
        assert_eq!(literal(r#""a \"quoted\" line\nnext""#), "a \"quoted\" line\nnext");
        assert_eq!(literal(r#""2026-10-05T00:00:00Z"^^<http://www.w3.org/2001/XMLSchema#dateTime>"#), "2026-10-05T00:00:00Z");
        assert_eq!(literal(r#""café""#), "café");
        assert_eq!(iri("<mailto:owner@customer.example>"), "owner@customer.example");
        assert_eq!(iri("<urn:ledger-set:ledger-design>"), "urn:ledger-set:ledger-design");
    }
}
