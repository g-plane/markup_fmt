use crate::{config::LanguageOptions, Language};
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
    pub(crate) indent_level: usize,
    pub(crate) external_formatter: F,
    pub(crate) external_formatter_error: Option<E>,
}

impl<'b, 's, E, F> Ctx<'b, 's, E, F>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
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
                self.print_width - self.indent_level - 2, // this is technically wrong, just workaround
            );
            let formatted = formatted.trim().trim_matches(';');
            formatted
                .strip_prefix('(')
                .and_then(|s| s.strip_suffix(')'))
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
                self.print_width - self.indent_level - TYPE_PARAMS_INDENT, // this is technically wrong, just workaround
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
                - self.indent_level
                - if self.options.script_indent {
                    self.indent_width
                } else {
                    0
                },
        )
    }

    pub(crate) fn format_style<'a>(&mut self, code: &'a str, lang: &str) -> Cow<'a, str> {
        self.format_with_external_formatter(
            Path::new(&format!("style.{lang}")),
            code,
            self.print_width
                - self.indent_level
                - if self.options.style_indent {
                    self.indent_width
                } else {
                    0
                },
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
        self.nest(ctx.indent_width)
    }
}
