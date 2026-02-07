use crate::{
    Language,
    config::{LanguageOptions, Quotes, WhitespaceSensitivity},
    helpers,
    state::State,
};
use memchr::memchr;
use std::borrow::Cow;

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
    pub(crate) external_formatter: F,
    pub(crate) external_formatter_errors: Vec<E>,
}

impl<'b, E, F> Ctx<'b, E, F>
where
    F: for<'a> FnMut(&'a str, Hints<'b>) -> Result<Cow<'a, str>, E>,
{
    pub(crate) fn script_indent(&self) -> bool {
        match self.language {
            Language::Html
            | Language::Jinja
            | Language::Vento
            | Language::Angular
            | Language::Mustache => self
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
            Language::Xml => false,
        }
    }

    pub(crate) fn style_indent(&self) -> bool {
        match self.language {
            Language::Html
            | Language::Jinja
            | Language::Vento
            | Language::Angular
            | Language::Mustache => self
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
            Language::Xml => false,
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
                        .unwrap_or(self.options.whitespace_sensitivity),
                    WhitespaceSensitivity::Css | WhitespaceSensitivity::Strict
                )
            }
            Language::Xml => false,
            _ => match self.options.whitespace_sensitivity {
                WhitespaceSensitivity::Css => {
                    helpers::is_whitespace_sensitive_tag(tag_name, self.language)
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
        match self.try_format_expr(code, attr, start) {
            Ok(formatted) => formatted,
            Err(e) => {
                self.external_formatter_errors.push(e);
                code.to_owned()
            }
        }
    }

    pub(crate) fn try_format_expr(
        &mut self,
        code: &str,
        attr: bool,
        start: usize,
    ) -> Result<String, E> {
        if code.trim().is_empty() {
            Ok(String::new())
        } else {
            // Trim original code before sending it to the external formatter.
            // This makes sure the code will be trimmed
            // though external formatter isn't available.
            let preprocessed = code.trim_start();
            let will_add_brackets =
                preprocessed.starts_with('{') || preprocessed.starts_with("...");
            let wrapped = if will_add_brackets {
                self.source
                    .get(0..start.saturating_sub(1))
                    .unwrap_or_default()
                    .replace(|c: char| !c.is_ascii_whitespace(), " ")
                    + "["
                    + code.trim()
                    + "]"
            } else {
                self.source
                    .get(0..start)
                    .unwrap_or_default()
                    .replace(|c: char| !c.is_ascii_whitespace(), " ")
                    + code
            };
            let formatted = self.try_format_with_external_formatter(
                wrapped,
                Hints {
                    print_width: self.print_width,
                    indent_level: 0,
                    attr,
                    ext: "tsx",
                },
            )?;
            let mut formatted =
                formatted.trim_matches(|c: char| c.is_ascii_whitespace() || c == ';');
            formatted = trim_delim(preprocessed, formatted, '[', ']');
            formatted = trim_delim(preprocessed, formatted, '(', ')');
            if will_add_brackets {
                formatted = formatted.trim_ascii_end().trim_end_matches(',');
            }
            Ok(formatted.trim_ascii().to_owned())
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
                    print_width: self.print_width,
                    indent_level: 0,
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
                    print_width: self.print_width,
                    indent_level: 0,
                    attr: true,
                    ext: "ts",
                },
            );
            let formatted = formatted.trim_matches(|c: char| c.is_ascii_whitespace() || c == ';');
            formatted
                .strip_prefix("type T<")
                .and_then(|s| s.strip_suffix("> = 0"))
                .map(|s| s.trim())
                .map(|s| s.strip_suffix(',').unwrap_or(s))
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
                    print_width: self.print_width,
                    indent_level: 0,
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
        state: &State,
    ) -> Cow<'a, str> {
        self.format_with_external_formatter(
            self.source
                .get(0..start)
                .unwrap_or_default()
                .replace(|c: char| !c.is_ascii_whitespace(), " ")
                + code,
            Hints {
                print_width: self.print_width,
                indent_level: state.indent_level,
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
        state: &State,
    ) -> Cow<'a, str> {
        self.format_with_external_formatter(
            "\n".repeat(
                self.source
                    .get(0..start)
                    .unwrap_or_default()
                    .lines()
                    .count()
                    .saturating_sub(1),
            ) + code,
            Hints {
                print_width: self
                    .print_width
                    .saturating_sub((state.indent_level as usize) * self.indent_width)
                    .saturating_sub(if self.style_indent() {
                        self.indent_width
                    } else {
                        0
                    }),
                indent_level: state.indent_level,
                attr: false,
                ext: if lang == "postcss" { "css" } else { lang },
            },
        )
    }

    pub(crate) fn format_style_attr(&mut self, code: &str, start: usize, state: &State) -> String {
        self.format_with_external_formatter(
            self.source
                .get(0..start)
                .unwrap_or_default()
                .replace(|c: char| !c.is_ascii_whitespace(), " ")
                + code,
            Hints {
                print_width: u16::MAX as usize,
                indent_level: state.indent_level,
                attr: true,
                ext: "css",
            },
        )
        .trim()
        .to_owned()
    }

    pub(crate) fn format_json<'a>(
        &mut self,
        code: &'a str,
        start: usize,
        state: &State,
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
                    .saturating_sub((state.indent_level as usize) * self.indent_width)
                    .saturating_sub(if self.script_indent() {
                        self.indent_width
                    } else {
                        0
                    }),
                indent_level: state.indent_level,
                attr: false,
                ext: "json",
            },
        )
    }

    pub(crate) fn format_jinja(
        &mut self,
        code: &str,
        start: usize,
        expr: bool,
        state: &State,
    ) -> String {
        self.format_with_external_formatter(
            self.source
                .get(0..start)
                .unwrap_or_default()
                .replace(|c: char| !c.is_ascii_whitespace(), " ")
                + code,
            Hints {
                print_width: self
                    .print_width
                    .saturating_sub((state.indent_level as usize) * self.indent_width),
                indent_level: state.indent_level,
                attr: false,
                ext: if expr {
                    "markup-fmt-jinja-expr"
                } else {
                    "markup-fmt-jinja-stmt"
                },
            },
        )
        .trim_ascii()
        .to_owned()
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

    fn try_format_with_external_formatter<'a>(
        &mut self,
        code: String,
        hints: Hints<'b>,
    ) -> Result<Cow<'a, str>, E> {
        match (self.external_formatter)(&code, hints) {
            Ok(Cow::Owned(formatted)) => Ok(Cow::from(formatted)),
            Ok(Cow::Borrowed(..)) => Ok(Cow::from(code)),
            Err(e) => Err(e),
        }
    }
}

/// Hints provide some useful additional information to the external formatter.
pub struct Hints<'s> {
    pub print_width: usize,
    /// current indent width = indent width in config * indent level
    pub indent_level: u16,
    /// Whether the code is inside attribute.
    pub attr: bool,
    /// Fake file extension.
    pub ext: &'s str,
}

fn trim_delim<'a>(user_input: &str, formatted: &'a str, start: char, end: char) -> &'a str {
    if user_input
        .trim_start()
        .chars()
        .take_while(|c| *c == start)
        .count()
        < formatted.chars().take_while(|c| *c == start).count()
        && user_input
            .trim_end()
            .chars()
            .rev()
            .take_while(|c| *c == end)
            .count()
            < formatted.chars().rev().take_while(|c| *c == end).count()
    {
        formatted
            .trim_ascii()
            .strip_prefix(start)
            .and_then(|s| s.strip_suffix(end))
            .unwrap_or(formatted)
    } else {
        formatted
    }
}
