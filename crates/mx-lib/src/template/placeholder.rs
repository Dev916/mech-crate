//! Placeholder expansion for recipe payloads.
//!
//! Recipes ship two very different kinds of file:
//!
//! * **scaffolding** — compose files, Dockerfiles, env files, READMEs. These
//!   carry `{{SERVICE_NAME}}`-style tokens that the installer must substitute.
//! * **app sources** — Blade views, Vue SFCs, Zola themes. These carry `{{ }}`
//!   and `{% %}` that belong to *another* renderer entirely and must reach the
//!   generated project byte-for-byte.
//!
//! Running the payload through a general template engine cannot tell the two
//! apart: Tera parsed `{{ page.title }}` as a variable lookup and
//! `{% extends "base.html" %}` as inheritance, so `mx add` blew up on every
//! recipe that shipped an app tree (bd:mech-crate-bj4, bd:mech-crate-0tj).
//!
//! [`expand_placeholders`] is the narrow substitution that replaces it: it only
//! recognises a bare identifier that is a *known* placeholder, and copies
//! everything else through untouched. Filters, function calls, statement blocks
//! and unknown names are not template syntax here — they are literal text.
//!
//! Tera's whitespace-control markers (`{{- NAME }}`, `{{ NAME -}}`) are honoured
//! so that recipes written against the old engine render byte-identically.

use std::collections::HashMap;

/// Expand `{{ NAME }}` tokens whose `NAME` is a key of `vars`.
///
/// Anything else — `{{ page.title }}`, `{{ x | filter }}`, `{% block %}`,
/// `{{SERVICE_NAME.charAt(0)}}` — is copied through verbatim.
///
/// ```
/// # use std::collections::HashMap;
/// # use mx_lib::template::expand_placeholders;
/// let vars = HashMap::from([("SERVICE_NAME".to_string(), "api".to_string())]);
/// assert_eq!(expand_placeholders("apps/{{SERVICE_NAME}}", &vars), "apps/api");
/// assert_eq!(expand_placeholders("{{ page.title }}", &vars), "{{ page.title }}");
/// ```
pub fn expand_placeholders(input: &str, vars: &HashMap<String, String>) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'{' && bytes.get(i + 1) == Some(&b'{') {
            if let Some(tag) = parse_tag(input, i) {
                if let Some(value) = vars.get(tag.name) {
                    if tag.trim_left {
                        while out.ends_with(char::is_whitespace) {
                            out.pop();
                        }
                    }
                    out.push_str(value);
                    i = tag.end;
                    if tag.trim_right {
                        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                            i += 1;
                        }
                    }
                    continue;
                }
            }
        }

        // Not a placeholder we own — copy one character through unchanged.
        let ch = input[i..].chars().next().expect("index is a char boundary");
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

/// A parsed `{{ NAME }}` tag.
struct Tag<'a> {
    /// The identifier between the braces.
    name: &'a str,
    /// Byte index just past the closing `}}`.
    end: usize,
    /// `{{-` — swallow whitespace already emitted before the tag.
    trim_left: bool,
    /// `-}}` — swallow whitespace following the tag in the input.
    trim_right: bool,
}

/// Parse `{{ [-] IDENT [-] }}` starting at `start` (which must point at `{{`).
///
/// Returns `None` for anything more elaborate than a bare identifier, which is
/// what keeps foreign template syntax literal.
fn parse_tag(input: &str, start: usize) -> Option<Tag<'_>> {
    let bytes = input.as_bytes();
    let mut i = start + 2;

    let trim_left = bytes.get(i) == Some(&b'-');
    if trim_left {
        i += 1;
    }

    while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
        i += 1;
    }

    let name_start = i;
    if !bytes
        .get(i)
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
    {
        return None;
    }
    while bytes
        .get(i)
        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
    {
        i += 1;
    }
    let name = &input[name_start..i];

    while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
        i += 1;
    }

    let trim_right = bytes.get(i) == Some(&b'-');
    if trim_right {
        i += 1;
    }

    if bytes.get(i) != Some(&b'}') || bytes.get(i + 1) != Some(&b'}') {
        return None;
    }

    Some(Tag {
        name,
        end: i + 2,
        trim_left,
        trim_right,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> HashMap<String, String> {
        HashMap::from([
            ("SERVICE_NAME".to_string(), "my-app".to_string()),
            ("SERVICE_UPPER".to_string(), "MY_APP".to_string()),
        ])
    }

    #[test]
    fn substitutes_known_placeholders() {
        assert_eq!(
            expand_placeholders("apps/{{SERVICE_NAME}}/src", &vars()),
            "apps/my-app/src"
        );
    }

    #[test]
    fn tolerates_inner_whitespace() {
        assert_eq!(
            expand_placeholders("a {{ SERVICE_NAME }} b", &vars()),
            "a my-app b"
        );
    }

    #[test]
    fn honours_left_whitespace_trim() {
        // The shipped env.shared writes `${ {{- SERVICE_UPPER }}_DB_PASSWORD}`.
        assert_eq!(
            expand_placeholders("DB_PASSWORD=${ {{- SERVICE_UPPER }}_DB_PASSWORD}", &vars()),
            "DB_PASSWORD=${MY_APP_DB_PASSWORD}"
        );
    }

    #[test]
    fn honours_right_whitespace_trim() {
        assert_eq!(
            expand_placeholders("[{{ SERVICE_NAME -}}   x]", &vars()),
            "[my-appx]"
        );
    }

    #[test]
    fn leaves_foreign_expressions_alone() {
        for src in [
            "{{ page.title }}",
            "{{ config.base_url | safe }}",
            "{{SERVICE_NAME.charAt(0).toUpperCase()}}",
            "{{ term.pages | length }}",
            "{{ sessionSecret || 'undefined (correct!)' }}",
            "{{ page.date | date(format=\"%B %d, %Y\") }}",
        ] {
            assert_eq!(expand_placeholders(src, &vars()), src, "mangled {src}");
        }
    }

    #[test]
    fn leaves_statement_blocks_alone() {
        let src = "{% extends \"base.html\" %}\n{% block content %}{{ page.title }}{% endblock %}";
        assert_eq!(expand_placeholders(src, &vars()), src);
    }

    #[test]
    fn leaves_unknown_identifiers_alone() {
        assert_eq!(
            expand_placeholders("{{ NOT_A_PLACEHOLDER }}", &vars()),
            "{{ NOT_A_PLACEHOLDER }}"
        );
    }

    #[test]
    fn handles_unterminated_and_empty_tags() {
        for src in ["{{", "{{ }}", "{{SERVICE_NAME", "{{{{", "}}"] {
            assert_eq!(expand_placeholders(src, &vars()), src, "mangled {src}");
        }
    }

    #[test]
    fn is_utf8_safe() {
        assert_eq!(
            expand_placeholders("日本語 {{SERVICE_NAME}} — ✅", &vars()),
            "日本語 my-app — ✅"
        );
    }

    #[test]
    fn substitutes_every_occurrence() {
        assert_eq!(
            expand_placeholders("{{SERVICE_NAME}}/{{SERVICE_NAME}}", &vars()),
            "my-app/my-app"
        );
    }

    #[test]
    fn empty_var_map_is_a_no_op() {
        let src = "apps/{{SERVICE_NAME}}";
        assert_eq!(expand_placeholders(src, &HashMap::new()), src);
    }
}
