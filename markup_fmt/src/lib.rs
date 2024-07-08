#![doc = include_str!("../README.md")]

mod ast;
pub mod config;
mod ctx;
mod error;
mod helpers;
mod parser;
mod printer;
mod state;

use crate::{config::FormatOptions, ctx::Ctx, parser::Parser, printer::DocGen, state::State};
pub use crate::{error::*, parser::Language};
use std::{borrow::Cow, path::Path};
use tiny_pretty::{IndentKind, PrintOptions};

/// Format the given source code.
///
/// An external formatter is required for formatting code
/// inside `<script>` or `<style>` tag.
/// If you don't need to format them or you don't have available formatters,
/// you can pass a closure that returns the original code. (see example below)
///
/// ```
/// use markup_fmt::{format_text, Language};
///
/// let code = r#"
/// <html>
///    <head>
///      <title>Example</title>
///      <style>button { outline: none; }</style>
///   </head>
///   <body><script>const a = 1;</script></body>
/// </html>"#;
///
/// let formatted = format_text(
///     code,
///     Language::Html,
///     &Default::default(),
///     |_, code, _| Ok::<_, std::convert::Infallible>(code.into()),
/// ).unwrap();
/// ```
///
/// For the external formatter closure,
///
/// - The first argument is file path.
/// Either script code or style code will be passed to this closure,
/// so you need to check file extension with the file path.
/// - The second argument is code that needs formatting.
/// - The third argument is print width that you may need to pass to formatter
/// if they support such option.
pub fn format_text<E, F>(
    code: &str,
    language: Language,
    options: &FormatOptions,
    external_formatter: F,
) -> Result<String, FormatError<E>>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    let mut parser = Parser::new(code, language.clone());
    let ast = parser.parse_root().map_err(FormatError::Syntax)?;
    let mut ctx = Ctx {
        language,
        indent_width: options.layout.indent_width,
        print_width: options.layout.print_width,
        options: &options.language,
        indent_level: 0,
        external_formatter,
        external_formatter_errors: Default::default(),
    };

    let doc = ast.doc(
        &mut ctx,
        &State {
            current_tag_name: None,
            is_root: true,
            in_svg: false,
        },
    );
    if !ctx.external_formatter_errors.is_empty() {
        return Err(FormatError::External(ctx.external_formatter_errors));
    }

    Ok(tiny_pretty::print(
        &doc,
        &PrintOptions {
            indent_kind: if options.layout.use_tabs {
                IndentKind::Tab
            } else {
                IndentKind::Space
            },
            line_break: options.layout.line_break.clone().into(),
            width: options.layout.print_width,
            tab_size: options.layout.indent_width,
        },
    ))
}

/// Detect language from file extension.
pub fn detect_language(path: impl AsRef<Path>) -> Option<Language> {
    let path = path.as_ref();
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("html") => {
            if path
                .file_stem()
                .map(|file_stem| file_stem.to_string_lossy().ends_with(".component"))
                .unwrap_or_default()
            {
                Some(Language::Angular)
            } else {
                Some(Language::Html)
            }
        }
        Some("vue") => Some(Language::Vue),
        Some("svelte") => Some(Language::Svelte),
        Some("astro") => Some(Language::Astro),
        Some("jinja" | "jinja2" | "twig" | "njk") => Some(Language::Jinja),
        Some("vto") => Some(Language::Vento),
        _ => None,
    }
}
