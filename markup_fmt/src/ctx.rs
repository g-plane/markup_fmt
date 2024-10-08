use crate::{
    config::{LanguageOptions, Quotes, WhitespaceSensitivity},
    helpers, Language,
};
use memchr::memchr;
use std::borrow::Cow;
use tiny_pretty::Doc;

const TYPE_PARAMS_INDENT: usize = "<script setup lang=\"ts\" generic=\"\">".len();

const QUOTES: [&str; 3] = ["\"", "\"", "'"];

pub(crate) struct Ctx<'b, E, F>
where
    F: for<'a> FnMut(&'a str, Hints<'b>) -> Result<Cow<'a, str>, E>,
{
    pub(crate) source: &'b str,
    pub(crate) language: Language,
    pub(crate) indent_width: usize,
    pub(crate) print_width: usize,
    pub(crate) options: &'b LanguageOptions,
    pub(crate) indent_level: usize,
    pub(crate) external_formatter: F,
    pub(crate) external_formatter_errors: Vec<E>,
}

impl<'b, E, F> Ctx<'b, E, F>
where
    F: for<'a> FnMut(&'a str, Hints<'b>) -> Result<Cow<'a, str>, E>,
{
    pub(crate) fn script_indent(&self) -> bool {
        match self.language {
            Language::Html | Language::Jinja | Language::Vento | Language::Angular => self
                .options
                .html_script_indent
                .unwrap_or(self.options.script_indent),
            Language::Vue => self
                .options
                .vue_script_indent
                .unwrap_or(self.options.script_indent),
            Language::Svelte => self
                .options
                .svelte_script_indent
                .unwrap_or(self.options.script_indent),
            Language::Astro => self
                .options
                .astro_script_indent
                .unwrap_or(self.options.script_indent),
        }
    }

    pub(crate) fn style_indent(&self) -> bool {
        match self.language {
            Language::Html | Language::Jinja | Language::Vento | Language::Angular => self
                .options
                .html_style_indent
                .unwrap_or(self.options.style_indent),
            Language::Vue => self
                .options
                .vue_style_indent
                .unwrap_or(self.options.style_indent),
            Language::Svelte => self
                .options
                .svelte_style_indent
                .unwrap_or(self.options.style_indent),
            Language::Astro => self
                .options
                .astro_style_indent
                .unwrap_or(self.options.style_indent),
        }
    }

    pub(crate) fn is_whitespace_sensitive(&self, tag_name: &str) -> bool {
        match self.language {
            Language::Vue | Language::Svelte | Language::Astro | Language::Angular
                if helpers::is_component(tag_name) =>
            {
                matches!(
                    self.options
                        .component_whitespace_sensitivity
                        .clone()
                        .unwrap_or(self.options.whitespace_sensitivity.clone()),
                    WhitespaceSensitivity::Css | WhitespaceSensitivity::Strict
                )
            }
            _ => match self.options.whitespace_sensitivity {
                WhitespaceSensitivity::Css => {
                    helpers::is_whitespace_sensitive_tag(tag_name, self.language.clone())
                }
                WhitespaceSensitivity::Strict => true,
                WhitespaceSensitivity::Ignore => false,
            },
        }
    }

    pub(crate) fn with_escaping_quotes(
        &mut self,
        s: &str,
        mut processer: impl FnMut(String, &mut Self) -> String,
    ) -> String {
        let escaped = helpers::UNESCAPING_AC.replace_all(s, &QUOTES);
        let proceeded = processer(escaped, self);
        if memchr(b'\'', proceeded.as_bytes()).is_some()
            && memchr(b'"', proceeded.as_bytes()).is_some()
        {
            match self.options.quotes {
                Quotes::Double => proceeded.replace('"', "&quot;"),
                Quotes::Single => proceeded.replace('\'', "&#x27;"),
            }
        } else {
            proceeded
        }
    }

    pub(crate) fn format_expr(&mut self, code: &str, attr: bool, start: usize) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            // Trim original code before sending it to the external formatter.
            // This makes sure the code will be trimmed
            // though external formatter isn't available.
            let wrapped = self
                .source
                .get(0..start.saturating_sub(3))
                .unwrap_or_default()
                .replace(|c: char| !c.is_ascii_whitespace(), " ")
                + "<>{"
                + code.trim()
                + "}</>";
            let formatted = self.format_with_external_formatter(
                wrapped,
                Hints {
                    print_width: self
                        .print_width
                        .saturating_sub(self.indent_level)
                        .saturating_sub(2), // this is technically wrong, just workaround
                    attr,
                    ext: "tsx",
                },
            );
            let formatted = formatted.trim_matches(|c: char| c.is_ascii_whitespace() || c == ';');
            let formatted = formatted
                .strip_prefix("<>")
                .and_then(|s| s.strip_suffix("</>"))
                .unwrap_or(formatted)
                .trim();
            formatted
                .strip_prefix('{')
                .and_then(|s| s.strip_suffix('}'))
                .unwrap_or(formatted)
                .trim_start()
                .trim_end_matches(|c: char| c.is_ascii_whitespace() || c == ';')
                .to_owned()
        }
    }

    pub(crate) fn format_binding(&mut self, code: &str, start: usize) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            let wrapped = self
                .source
                .get(0..start.saturating_sub(4))
                .unwrap_or_default()
                .replace(|c: char| !c.is_ascii_whitespace(), " ")
                + "let "
                + code.trim()
                + " = 0";
            let formatted = self.format_with_external_formatter(
                wrapped,
                Hints {
                    print_width: self
                        .print_width
                        .saturating_sub(self.indent_level)
                        .saturating_sub(2), // this is technically wrong, just workaround
                    attr: false,
                    ext: "ts",
                },
            );
            let formatted = formatted.trim_matches(|c: char| c.is_ascii_whitespace() || c == ';');
            formatted
                .strip_prefix("let ")
                .and_then(|s| s.strip_suffix(" = 0"))
                .unwrap_or(formatted)
                .to_owned()
        }
    }

    pub(crate) fn format_type_params(&mut self, code: &str, start: usize) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            let wrapped = self
                .source
                .get(0..start.saturating_sub(7))
                .unwrap_or_default()
                .replace(|c: char| !c.is_ascii_whitespace(), " ")
                + "type T<"
                + code.trim()
                + "> = 0";
            let formatted = self.format_with_external_formatter(
                wrapped,
                Hints {
                    print_width: self
                        .print_width
                        .saturating_sub(self.indent_level)
                        .saturating_sub(TYPE_PARAMS_INDENT), // this is technically wrong, just workaround
                    attr: true,
                    ext: "ts",
                },
            );
            let formatted = formatted.trim_matches(|c: char| c.is_ascii_whitespace() || c == ';');
            formatted
                .strip_prefix("type T<")
                .and_then(|s| s.strip_suffix("> = 0"))
                .unwrap_or(formatted)
                .to_owned()
        }
    }

    pub(crate) fn format_stmt_header(&mut self, keyword: &str, code: &str) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            let wrapped = format!("{keyword} ({code}) {{}}");
            let formatted = self.format_with_external_formatter(
                wrapped,
                Hints {
                    print_width: self
                        .print_width
                        .saturating_sub(self.indent_level)
                        .saturating_sub(keyword.len() + 1), // this is technically wrong, just workaround
                    attr: false,
                    ext: "js",
                },
            );
            formatted
                .strip_prefix(keyword)
                .map(|s| s.trim_start())
                .and_then(|s| s.strip_prefix('('))
                .and_then(|s| s.trim_end().strip_suffix('}'))
                .and_then(|s| s.trim_end().strip_suffix('{'))
                .and_then(|s| s.trim_end().strip_suffix(')'))
                .unwrap_or(code)
                .to_owned()
        }
    }

    pub(crate) fn format_script<'a>(
        &mut self,
        code: &'a str,
        lang: &'b str,
        start: usize,
    ) -> Cow<'a, str> {
        self.format_with_external_formatter(
            self.source
                .get(0..start)
                .unwrap_or_default()
                .replace(|c: char| !c.is_ascii_whitespace(), " ")
                + code,
            Hints {
                print_width: self
                    .print_width
                    .saturating_sub(self.indent_level)
                    .saturating_sub(if self.script_indent() {
                        self.indent_width
                    } else {
                        0
                    }),
                attr: false,
                ext: lang,
            },
        )
    }

    pub(crate) fn format_style<'a>(
        &mut self,
        code: &'a str,
        lang: &'b str,
        start: usize,
    ) -> Cow<'a, str> {
        self.format_with_external_formatter(
            self.source
                .get(0..start)
                .unwrap_or_default()
                .replace(|c: char| !c.is_ascii_whitespace(), " ")
                + code,
            Hints {
                print_width: self
                    .print_width
                    .saturating_sub(self.indent_level)
                    .saturating_sub(if self.style_indent() {
                        self.indent_width
                    } else {
                        0
                    }),
                attr: false,
                ext: lang,
            },
        )
    }

    pub(crate) fn format_style_attr(&mut self, code: &str, start: usize) -> String {
        self.format_with_external_formatter(
            self.source
                .get(0..start)
                .unwrap_or_default()
                .replace(|c: char| !c.is_ascii_whitespace(), " ")
                + code,
            Hints {
                print_width: self
                    .print_width
                    .saturating_sub(self.indent_level)
                    .saturating_sub(if self.style_indent() {
                        self.indent_width
                    } else {
                        0
                    }),
                attr: true,
                ext: "css",
            },
        )
        .trim()
        .to_owned()
    }

    pub(crate) fn format_json<'a>(&mut self, code: &'a str, start: usize) -> Cow<'a, str> {
        self.format_with_external_formatter(
            self.source
                .get(0..start)
                .unwrap_or_default()
                .replace(|c: char| !c.is_ascii_whitespace(), " ")
                + code,
            Hints {
                print_width: self
                    .print_width
                    .saturating_sub(self.indent_level)
                    .saturating_sub(if self.script_indent() {
                        self.indent_width
                    } else {
                        0
                    }),
                attr: false,
                ext: "json",
            },
        )
    }

    fn format_with_external_formatter<'a>(
        &mut self,
        code: String,
        hints: Hints<'b>,
    ) -> Cow<'a, str> {
        match (self.external_formatter)(&code, hints) {
            Ok(Cow::Owned(formatted)) => Cow::from(formatted),
            Ok(Cow::Borrowed(..)) => Cow::from(code),
            Err(e) => {
                self.external_formatter_errors.push(e);
                code.into()
            }
        }
    }
}

/// Hints provide some useful additional information to the external formatter.
pub struct Hints<'s> {
    pub print_width: usize,
    /// Whether the code is inside attribute.
    pub attr: bool,
    /// Fake file extension.
    pub ext: &'s str,
}

pub(crate) trait NestWithCtx {
    fn nest_with_ctx<'b, E, F>(self, ctx: &mut Ctx<'b, E, F>) -> Self
    where
        F: for<'a> FnMut(&'a str, Hints<'b>) -> Result<Cow<'a, str>, E>;
}

impl NestWithCtx for Doc<'_> {
    fn nest_with_ctx<'b, E, F>(self, ctx: &mut Ctx<'b, E, F>) -> Self
    where
        F: for<'a> FnMut(&'a str, Hints<'b>) -> Result<Cow<'a, str>, E>,
    {
        ctx.indent_level += ctx.indent_width;
        let doc = self.nest(ctx.indent_width);
        ctx.indent_level -= ctx.indent_width;
        doc
    }
}
