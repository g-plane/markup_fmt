use crate::{
    config::{LanguageOptions, Quotes, WhitespaceSensitivity},
    helpers, Language,
};
use aho_corasick::AhoCorasick;
use memchr::memchr;
use once_cell::sync::Lazy;
use std::{borrow::Cow, path::Path};
use tiny_pretty::Doc;

const TYPE_PARAMS_INDENT: usize = "<script setup lang=\"ts\" generic=\"\">".len();

static UNESCAPING_AC: Lazy<AhoCorasick> =
    Lazy::new(|| AhoCorasick::new(["&quot;", "&#x22;", "&#x27;"]).unwrap());
const QUOTES: [&str; 3] = ["\"", "\"", "'"];

pub(crate) struct Ctx<'b, E, F>
where
    F: for<'a> FnMut(&Path, &'a str, FormattingMeta) -> Result<Cow<'a, str>, E>,
{
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
    F: for<'a> FnMut(&Path, &'a str, FormattingMeta) -> Result<Cow<'a, str>, E>,
{
    pub(crate) fn script_indent(&self) -> bool {
        match self.language {
            Language::Html | Language::Jinja | Language::Vento => self
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
            Language::Html | Language::Jinja | Language::Vento => self
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
            Language::Vue | Language::Svelte | Language::Astro
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

    pub(crate) fn format_general_expr(&mut self, code: &str, start_offset: usize) -> String {
        self.format_expr(code, Path::new("expr.tsx"), start_offset)
    }

    pub(crate) fn format_attr_expr(&mut self, code: &str, start_offset: usize) -> String {
        let code = UNESCAPING_AC.replace_all(code, &QUOTES);
        let formatted = self.format_expr(&code, Path::new("attr_expr.tsx"), start_offset);
        if memchr(b'\'', formatted.as_bytes()).is_some()
            && memchr(b'"', formatted.as_bytes()).is_some()
        {
            match self.options.quotes {
                Quotes::Double => formatted.replace('"', "&quot;"),
                Quotes::Single => formatted.replace('\'', "&#x27;"),
            }
        } else {
            formatted
        }
    }

    fn format_expr(&mut self, code: &str, path: &Path, start_offset: usize) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            const PREFIX: &str = "<>{";
            const SUFFIX: &str = "}</>";
            // Trim original code before sending it to the external formatter.
            // This makes sure the code will be trimmed
            // though external formatter isn't available.
            let wrapped = format!("{PREFIX}{}{SUFFIX}", code.trim());
            let formatted = self.format_with_external_formatter(
                path,
                &wrapped,
                code,
                FormattingMeta {
                    print_width: self
                        .print_width
                        .saturating_sub(self.indent_level)
                        .saturating_sub(2), // this is technically wrong, just workaround
                    start_offset: (start_offset - PREFIX.len()) as isize,
                },
            );
            let formatted =
                formatted.trim_end_matches(|c: char| c.is_ascii_whitespace() || c == ';');
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

    pub(crate) fn format_binding(&mut self, code: &str, start_offset: usize) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            const PREFIX: &str = "let ";
            let wrapped = format!("{PREFIX}{} = 0", code.trim());
            let formatted = self.format_with_external_formatter(
                Path::new("binding.ts"),
                &wrapped,
                code,
                FormattingMeta {
                    print_width: self
                        .print_width
                        .saturating_sub(self.indent_level)
                        .saturating_sub(2), // this is technically wrong, just workaround
                    start_offset: (start_offset - PREFIX.len()) as isize,
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

    pub(crate) fn format_type_params(&mut self, code: &str, start_offset: usize) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            const PREFIX: &str = "type T<";
            let wrapped = format!("{PREFIX}{}> = 0", code.trim());
            let formatted = self.format_with_external_formatter(
                Path::new("type_params.ts"),
                &wrapped,
                code,
                FormattingMeta {
                    print_width: self
                        .print_width
                        .saturating_sub(self.indent_level)
                        .saturating_sub(TYPE_PARAMS_INDENT), // this is technically wrong, just workaround
                    start_offset: (start_offset - PREFIX.len()) as isize,
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
        start_offset: usize,
    ) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            let wrapped = format!("{keyword} ({code}) {{}}");
            let formatted = self.format_with_external_formatter(
                Path::new("stmt_header.js"),
                &wrapped,
                code,
                FormattingMeta {
                    print_width: self
                        .print_width
                        .saturating_sub(self.indent_level)
                        .saturating_sub(keyword.len() + 1), // this is technically wrong, just workaround
                    start_offset: start_offset as isize,
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
        lang: &str,
        start_offset: usize,
    ) -> Cow<'a, str> {
        self.format_with_external_formatter(
            Path::new(&format!("script.{lang}")),
            code,
            code,
            FormattingMeta {
                print_width: self
                    .print_width
                    .saturating_sub(self.indent_level)
                    .saturating_sub(if self.script_indent() {
                        self.indent_width
                    } else {
                        0
                    }),
                start_offset: start_offset as isize,
            },
        )
    }

    pub(crate) fn format_style<'a>(
        &mut self,
        code: &'a str,
        lang: &str,
        start_offset: usize,
    ) -> Cow<'a, str> {
        self.format_with_external_formatter(
            Path::new(&format!("style.{lang}")),
            code,
            code,
            FormattingMeta {
                print_width: self
                    .print_width
                    .saturating_sub(self.indent_level)
                    .saturating_sub(if self.style_indent() {
                        self.indent_width
                    } else {
                        0
                    }),
                start_offset: start_offset as isize,
            },
        )
    }

    pub(crate) fn format_json<'a>(&mut self, code: &'a str, start_offset: usize) -> Cow<'a, str> {
        self.format_with_external_formatter(
            Path::new("code.json"),
            code,
            code,
            FormattingMeta {
                print_width: self
                    .print_width
                    .saturating_sub(self.indent_level)
                    .saturating_sub(if self.script_indent() {
                        self.indent_width
                    } else {
                        0
                    }),
                start_offset: start_offset as isize,
            },
        )
    }

    fn format_with_external_formatter<'a>(
        &mut self,
        path: &Path,
        code: &'a str,
        _original_code: &'a str,
        formatting_meta: FormattingMeta,
    ) -> Cow<'a, str> {
        match (self.external_formatter)(path, code, formatting_meta) {
            Ok(code) => code,
            Err(e) => {
                self.external_formatter_errors.push(e);
                code.into()
            }
        }
    }
}

pub(crate) trait NestWithCtx {
    fn nest_with_ctx<'b, E, F>(self, ctx: &mut Ctx<'b, E, F>) -> Self
    where
        F: for<'a> FnMut(&Path, &'a str, FormattingMeta) -> Result<Cow<'a, str>, E>;
}

impl NestWithCtx for Doc<'_> {
    fn nest_with_ctx<'b, E, F>(self, ctx: &mut Ctx<'b, E, F>) -> Self
    where
        F: for<'a> FnMut(&Path, &'a str, FormattingMeta) -> Result<Cow<'a, str>, E>,
    {
        ctx.indent_level += ctx.indent_width;
        let doc = self.nest(ctx.indent_width);
        ctx.indent_level -= ctx.indent_width;
        doc
    }
}

pub struct FormattingMeta {
    pub print_width: usize,
    pub start_offset: isize,
}
