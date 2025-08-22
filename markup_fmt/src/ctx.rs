use crate::{
    config::{LanguageOptions, Quotes, WhitespaceSensitivity},
    helpers,
    state::State,
    Language,
};
use memchr::memchr;
use std::borrow::Cow;

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
                        .clone()
                        .unwrap_or(self.options.whitespace_sensitivity.clone()),
                    WhitespaceSensitivity::Css | WhitespaceSensitivity::Strict
                )
            }
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

    pub(crate) fn format_expr(
        &mut self,
        code: &str,
        attr: bool,
        start: usize,
        state: &State,
    ) -> String {
        match self.try_format_expr(code, attr, start, state) {
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
        state: &State,
    ) -> Result<String, E> {
        if code.trim().is_empty() {
            Ok(String::new())
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
            let formatted = self.try_format_with_external_formatter(
                wrapped,
                Hints {
                    print_width: self
                        .print_width
                        .saturating_sub((state.indent_level as usize) * self.indent_width)
                        .saturating_sub(2), // this is technically wrong, just workaround
                    indent_level: state.indent_level,
                    attr,
                    ext: "tsx",
                },
            )?;
            let formatted = formatted.trim_matches(|c: char| c.is_ascii_whitespace() || c == ';');
            let formatted = formatted
                .strip_prefix("<>")
                .and_then(|s| s.strip_suffix("</>"))
                .unwrap_or(formatted);
            // The condition below detects these cases:
            // 1. Language is not Astro
            // 2. There's a line break after `{`
            //    ```
            //    {
            //        /*
            //        */
            //    }
            //    ```
            // 3. The indentation level of inner content is less than that of `{`
            //    ```
            //        {/*
            //    Hello
            //    */}
            //    ```
            let formatted = if self.language != Language::Astro
                || formatted
                    .trim_ascii_start()
                    .strip_prefix('{')
                    .is_some_and(|s| s.starts_with(['\n', '\r']))
                || formatted
                    .trim_start_matches(['\n', '\r'])
                    .find('{')
                    .is_some_and(|index| {
                        helpers::detect_indent(formatted.trim_start_matches(['\n', '\r'])) < index
                    }) {
                formatted
                    .trim_ascii()
                    .strip_prefix('{')
                    .and_then(|s| s.strip_suffix('}'))
                    .unwrap_or(formatted)
                    .trim_ascii_start()
                    .trim_matches(|c: char| c.is_ascii_whitespace() || c == ';')
                    .to_owned()
            } else {
                formatted
                    .replacen('{', "", 1)
                    .trim_ascii_end()
                    .strip_suffix('}')
                    .unwrap_or(formatted)
                    .trim_start_matches(['\n', '\r'])
                    .trim_end_matches(|c: char| c.is_ascii_whitespace() || c == ';')
                    .to_owned()
            };
            Ok(formatted)
        }
    }

    pub(crate) fn format_binding(&mut self, code: &str, start: usize, state: &State) -> String {
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
                        .saturating_sub((state.indent_level as usize) * self.indent_width)
                        .saturating_sub(2), // this is technically wrong, just workaround
                    indent_level: state.indent_level,
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

    pub(crate) fn format_type_params(&mut self, code: &str, start: usize, state: &State) -> String {
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
                        .saturating_sub((state.indent_level as usize) * self.indent_width)
                        .saturating_sub(TYPE_PARAMS_INDENT), // this is technically wrong, just workaround
                    indent_level: state.indent_level,
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

    pub(crate) fn format_stmt_header(
        &mut self,
        keyword: &str,
        code: &str,
        state: &State,
    ) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            let wrapped = format!("{keyword} ({code}) {{}}");
            let formatted = self.format_with_external_formatter(
                wrapped,
                Hints {
                    print_width: self
                        .print_width
                        .saturating_sub((state.indent_level as usize) * self.indent_width)
                        .saturating_sub(keyword.len() + 1), // this is technically wrong, just workaround
                    indent_level: state.indent_level,
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

    pub(crate) fn format_jinja<'a>(
        &mut self,
        code: &'a str,
        start: usize,
        ext: &'static str,
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
                    .saturating_sub((state.indent_level as usize) * self.indent_width),
                indent_level: state.indent_level,
                attr: false,
                ext,
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
