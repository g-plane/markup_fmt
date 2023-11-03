use crate::{
    config::{LanguageOptions, WhitespaceSensitivity},
    helpers, Language,
};
use std::{borrow::Cow, path::Path};
use tiny_pretty::Doc;

const TYPE_PARAMS_INDENT: usize = "<script setup lang=\"ts\" generic=\"\">".len();

pub(crate) struct Ctx<'b, 's, E, F>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    pub(crate) language: Language,
    pub(crate) indent_width: usize,
    pub(crate) print_width: usize,
    pub(crate) options: &'b LanguageOptions,
    pub(crate) current_tag_name: Option<&'s str>,
    pub(crate) is_root: bool,
    pub(crate) in_svg: bool,
    pub(crate) indent_level: usize,
    pub(crate) external_formatter: F,
    pub(crate) external_formatter_error: Option<E>,
}

impl<'b, 's, E, F> Ctx<'b, 's, E, F>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    pub(crate) fn script_indent(&self) -> bool {
        match self.language {
            Language::Html | Language::Jinja => self
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
        }
    }

    pub(crate) fn style_indent(&self) -> bool {
        match self.language {
            Language::Html | Language::Jinja => self
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
        }
    }

    pub(crate) fn is_whitespace_sensitive(&self, tag_name: &str) -> bool {
        if self.in_svg {
            return false;
        }

        match self.language {
            Language::Vue | Language::Svelte if helpers::is_component(tag_name) => {
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

    pub(crate) fn format_expr(&mut self, code: &str) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            // Trim original code before sending it to the external formatter.
            // This makes sure the code will be trimmed
            // though external formatter isn't available.
            let wrapped = format!("({})", code.trim());
            let formatted = self.format_with_external_formatter(
                Path::new("expr.ts"),
                &wrapped,
                self.print_width
                    .saturating_sub(self.indent_level)
                    .saturating_sub(2), // this is technically wrong, just workaround
            );
            let formatted = formatted.trim().trim_matches(';');
            formatted
                .strip_prefix('(')
                .and_then(|s| s.strip_suffix(')'))
                .unwrap_or(formatted)
                .to_owned()
        }
    }

    pub(crate) fn format_binding(&mut self, code: &str) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            let wrapped = format!("let {} = 0", code.trim());
            let formatted = self.format_with_external_formatter(
                Path::new("binding.ts"),
                &wrapped,
                self.print_width
                    .saturating_sub(self.indent_level)
                    .saturating_sub(2), // this is technically wrong, just workaround
            );
            let formatted = formatted.trim().trim_matches(';');
            formatted
                .strip_prefix("let ")
                .and_then(|s| s.strip_suffix(" = 0"))
                .unwrap_or(formatted)
                .to_owned()
        }
    }

    pub(crate) fn format_type_params(&mut self, code: &str) -> String {
        if code.trim().is_empty() {
            String::new()
        } else {
            let wrapped = format!("type T<{}> = 0", code.trim());
            let formatted = self.format_with_external_formatter(
                Path::new("type_params.ts"),
                &wrapped,
                self.print_width
                    .saturating_sub(self.indent_level)
                    .saturating_sub(TYPE_PARAMS_INDENT), // this is technically wrong, just workaround
            );
            let formatted = formatted.trim().trim_matches(';');
            formatted
                .strip_prefix("type T<")
                .and_then(|s| s.strip_suffix("> = 0"))
                .unwrap_or(formatted)
                .to_owned()
        }
    }

    pub(crate) fn format_script<'a>(&mut self, code: &'a str, lang: &str) -> Cow<'a, str> {
        self.format_with_external_formatter(
            Path::new(&format!("script.{lang}")),
            code,
            self.print_width
                .saturating_sub(self.indent_level)
                .saturating_sub(if self.script_indent() {
                    self.indent_width
                } else {
                    0
                }),
        )
    }

    pub(crate) fn format_style<'a>(&mut self, code: &'a str, lang: &str) -> Cow<'a, str> {
        self.format_with_external_formatter(
            Path::new(&format!("style.{lang}")),
            code,
            self.print_width
                .saturating_sub(self.indent_level)
                .saturating_sub(if self.style_indent() {
                    self.indent_width
                } else {
                    0
                }),
        )
    }

    fn format_with_external_formatter<'a>(
        &mut self,
        path: &Path,
        code: &'a str,
        print_width: usize,
    ) -> Cow<'a, str> {
        match (self.external_formatter)(path, code, print_width) {
            Ok(code) => code,
            Err(e) => {
                self.external_formatter_error = Some(e);
                code.into()
            }
        }
    }
}

pub(crate) trait NestWithCtx {
    fn nest_with_ctx<'b, 's, E, F>(self, ctx: &mut Ctx<'b, 's, E, F>) -> Self
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>;
}

impl NestWithCtx for Doc<'_> {
    fn nest_with_ctx<'b, 's, E, F>(self, ctx: &mut Ctx<'b, 's, E, F>) -> Self
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        ctx.indent_level += ctx.indent_width;
        let doc = self.nest(ctx.indent_width);
        ctx.indent_level -= ctx.indent_width;
        doc
    }
}
