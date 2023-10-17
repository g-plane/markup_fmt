pub(crate) fn is_component(name: &str) -> bool {
    name.contains('-') || name.contains(|c: char| c.is_ascii_uppercase())
}

pub(crate) fn is_whitespace_sensitive_tag(name: &str) -> bool {
    [
        "address",
        "blockquote",
        "center",
        "dialog",
        "div",
        "figure",
        "figcaption",
        "footer",
        "form",
        "header",
        "hr",
        "legend",
        "listing",
        "main",
        "p",
        "plaintext",
        "pre",
        "search",
        "xmp",
        "area",
        "base",
        "basefont",
        "datalist",
        "head",
        "link",
        "meta",
        "noembed",
        "noframes",
        "param",
        "rp",
        "title",
        "template",
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
    ]
    .iter()
    .all(|tag| !tag.eq_ignore_ascii_case(name))
}

pub(crate) fn is_void_element(name: &str) -> bool {
    [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
        "track", "wbr",
    ]
    .iter()
    .any(|tag| tag.eq_ignore_ascii_case(name))
}
