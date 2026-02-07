use crate::Language;
use aho_corasick::AhoCorasick;
use std::{borrow::Cow, sync::LazyLock};

pub(crate) fn is_component(name: &str) -> bool {
    name.contains('-') || name.contains(|c: char| c.is_ascii_uppercase())
}

static NON_WS_SENSITIVE_TAGS: [&str; 76] = [
    "address",
    "blockquote",
    "button",
    "caption",
    "center",
    "colgroup",
    "dialog",
    "div",
    "figure",
    "figcaption",
    "footer",
    "form",
    "select",
    "option",
    "optgroup",
    "header",
    "hr",
    "legend",
    "listing",
    "main",
    "p",
    "plaintext",
    "pre",
    "progress",
    "search",
    "object",
    "details",
    "summary",
    "xmp",
    "area",
    "base",
    "basefont",
    "datalist",
    "head",
    "link",
    "meta",
    "meter",
    "noembed",
    "noframes",
    "param",
    "rp",
    "title",
    "html",
    "body",
    "article",
    "aside",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hgroup",
    "nav",
    "section",
    "table",
    "tr",
    "thead",
    "th",
    "tbody",
    "td",
    "tfoot",
    "dir",
    "dd",
    "dl",
    "dt",
    "menu",
    "ol",
    "ul",
    "li",
    "fieldset",
    "video",
    "audio",
    "picture",
    "source",
    "track",
];

pub(crate) fn is_whitespace_sensitive_tag(name: &str, language: Language) -> bool {
    match language {
        Language::Html | Language::Jinja | Language::Vento | Language::Mustache => {
            // There's also a tag called "a" in SVG, so we need to check it specially.
            name.eq_ignore_ascii_case("a")
                || !NON_WS_SENSITIVE_TAGS
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(name))
                    && !css_dataset::tags::SVG_TAGS
                        .iter()
                        .any(|tag| tag.eq_ignore_ascii_case(name))
        }
        Language::Xml => false,
        _ => {
            name == "a"
                || !NON_WS_SENSITIVE_TAGS.contains(&name)
                    && !css_dataset::tags::SVG_TAGS.contains(&name)
        }
    }
}

static VOID_ELEMENTS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr", "param",
];

pub(crate) fn is_void_element(name: &str, language: Language) -> bool {
    match language {
        Language::Html | Language::Jinja | Language::Vento | Language::Mustache => VOID_ELEMENTS
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(name)),
        Language::Xml => false,
        _ => VOID_ELEMENTS.contains(&name),
    }
}

pub(crate) fn is_html_tag(name: &str, language: Language) -> bool {
    match language {
        Language::Html | Language::Jinja | Language::Vento | Language::Mustache => {
            css_dataset::tags::STANDARD_HTML_TAGS
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(name))
                || css_dataset::tags::NON_STANDARD_HTML_TAGS
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(name))
        }
        Language::Xml => false,
        _ => {
            css_dataset::tags::STANDARD_HTML_TAGS.contains(&name)
                || css_dataset::tags::NON_STANDARD_HTML_TAGS.contains(&name)
        }
    }
}

pub(crate) fn is_svg_tag(name: &str, language: Language) -> bool {
    if matches!(
        language,
        Language::Html | Language::Jinja | Language::Vento | Language::Mustache
    ) {
        css_dataset::tags::SVG_TAGS
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(name))
    } else {
        css_dataset::tags::SVG_TAGS.contains(&name)
    }
}

pub(crate) fn is_mathml_tag(name: &str, language: Language) -> bool {
    match language {
        Language::Html | Language::Jinja | Language::Vento | Language::Mustache => {
            css_dataset::tags::MATH_ML_TAGS
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(name))
        }
        Language::Xml => false,
        _ => css_dataset::tags::MATH_ML_TAGS.contains(&name),
    }
}

pub(crate) fn parse_vento_tag(tag: &str) -> (&str, &str) {
    let trimmed = tag.trim();
    trimmed
        .split_once(|c: char| c.is_ascii_whitespace())
        .unwrap_or((trimmed, ""))
}

pub(crate) static UNESCAPING_AC: LazyLock<AhoCorasick> =
    LazyLock::new(|| AhoCorasick::new(["&quot;", "&#x22;", "&#x27;"]).unwrap());

pub(crate) fn detect_indent(s: &str) -> usize {
    s.lines()
        .skip(if s.starts_with([' ', '\t']) { 0 } else { 1 })
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.as_bytes()
                .iter()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count()
        })
        .min()
        .unwrap_or_default()
}

pub(crate) fn pascal2kebab(s: &'_ str) -> Cow<'_, str> {
    let uppers = s.chars().filter(char::is_ascii_uppercase).count();
    if uppers > 1
        || s.find(|c: char| c.is_ascii_uppercase())
            .is_some_and(|index| index > 0)
    {
        let mut result = String::with_capacity(s.len() + uppers);
        s.chars().fold('<', |prev, c| {
            if c.is_ascii_uppercase() && prev.is_ascii_lowercase() {
                result.push('-');
            }
            result.push(c.to_ascii_lowercase());
            c
        });
        Cow::from(result)
    } else {
        Cow::from(s)
    }
}

pub(crate) fn kebab2pascal(s: &'_ str) -> Cow<'_, str> {
    if s.contains('-')
        || s.find(|c: char| c.is_ascii_uppercase())
            .is_some_and(|index| index > 0)
    {
        let mut result = String::with_capacity(s.len());
        s.chars().fold('<', |prev, c| {
            if c == '-' {
            } else if matches!(prev, '-' | '<') {
                result.push(c.to_ascii_uppercase());
            } else {
                result.push(c);
            }
            c
        });
        Cow::from(result)
    } else {
        Cow::from(s)
    }
}

pub(crate) fn has_template_interpolation(s: &str, language: Language) -> bool {
    match language {
        Language::Html | Language::Xml => false,
        Language::Svelte | Language::Astro => s.contains('{'),
        Language::Vue | Language::Angular => s.contains("{{"),
        Language::Jinja | Language::Vento | Language::Mustache => {
            s.contains("{{") || s.contains("{%")
        }
    }
}
