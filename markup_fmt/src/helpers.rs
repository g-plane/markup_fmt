use crate::Language;
use aho_corasick::AhoCorasick;
use std::sync::LazyLock;

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
    if matches!(language, Language::Html | Language::Jinja | Language::Vento) {
        // There's also a tag called "a" in SVG, so we need to check it specially.
        name.eq_ignore_ascii_case("a")
            || !NON_WS_SENSITIVE_TAGS
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(name))
                && !css_dataset::tags::SVG_TAGS
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(name))
    } else {
        name == "a"
            || !NON_WS_SENSITIVE_TAGS.iter().any(|tag| *tag == name)
                && !css_dataset::tags::SVG_TAGS.iter().any(|tag| *tag == name)
    }
}

static VOID_ELEMENTS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr", "param",
];

pub(crate) fn is_void_element(name: &str, language: Language) -> bool {
    if matches!(language, Language::Html | Language::Jinja | Language::Vento) {
        VOID_ELEMENTS
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(name))
    } else {
        VOID_ELEMENTS.iter().any(|tag| *tag == name)
    }
}

pub(crate) fn is_html_tag(name: &str, language: Language) -> bool {
    if matches!(language, Language::Html | Language::Jinja | Language::Vento) {
        css_dataset::tags::STANDARD_HTML_TAGS
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(name))
            || css_dataset::tags::NON_STANDARD_HTML_TAGS
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(name))
    } else {
        css_dataset::tags::STANDARD_HTML_TAGS
            .iter()
            .any(|tag| *tag == name)
            || css_dataset::tags::NON_STANDARD_HTML_TAGS
                .iter()
                .any(|tag| *tag == name)
    }
}

pub(crate) fn is_svg_tag(name: &str, language: Language) -> bool {
    if matches!(language, Language::Html | Language::Jinja | Language::Vento) {
        css_dataset::tags::SVG_TAGS
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(name))
    } else {
        css_dataset::tags::SVG_TAGS.iter().any(|tag| *tag == name)
    }
}

pub(crate) fn is_mathml_tag(name: &str, language: Language) -> bool {
    if matches!(language, Language::Html | Language::Jinja | Language::Vento) {
        css_dataset::tags::MATH_ML_TAGS
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(name))
    } else {
        css_dataset::tags::MATH_ML_TAGS
            .iter()
            .any(|tag| *tag == name)
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
