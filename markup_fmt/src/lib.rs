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
pub use crate::{ctx::Hints, error::*, parser::Language};
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
///     |code, _| Ok::<_, std::convert::Infallible>(code.into()),
/// ).unwrap();
/// ```
///
/// For the external formatter closure,
///
/// - The first argument is code that needs formatting.
/// - The second argument is hints which contains useful information for external formatters,
///   such as file extension and print width.
pub fn format_text<E, F>(
    code: &str,
    language: Language,
    options: &FormatOptions,
    external_formatter: F,
) -> Result<String, FormatError<E>>
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    let mut parser = Parser::new(code, language);
    let ast = parser.parse_root().map_err(FormatError::Syntax)?;

    if ast.children.first().is_some_and(|child| {
        if let ast::Node {
            kind: ast::NodeKind::Comment(ast::Comment { raw, .. }),
            ..
        } = child
        {
            raw.trim_start()
                .strip_prefix(&options.language.ignore_file_comment_directive)
                .is_some_and(|rest| {
                    rest.starts_with(|c: char| c.is_ascii_whitespace()) || rest.is_empty()
                })
        } else {
            false
        }
    }) {
        return Ok(code.into());
    }

    let mut ctx = Ctx {
        source: code,
        language,
        indent_width: options.layout.indent_width,
        print_width: options.layout.print_width,
        options: &options.language,
        external_formatter,
        external_formatter_errors: Default::default(),
    };

    let doc = ast.doc(
        &mut ctx,
        &State {
            current_tag_name: None,
            is_root: true,
            in_svg: false,
            indent_level: 0,
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
                .is_some_and(|file_stem| file_stem.to_string_lossy().ends_with(".component"))
            {
                Some(Language::Angular)
            } else {
                Some(Language::Html)
            }
        }
        Some("vue") => Some(Language::Vue),
        Some("svelte") => Some(Language::Svelte),
        Some("astro") => Some(Language::Astro),
        Some("jinja" | "jinja2" | "j2" | "twig" | "njk") => Some(Language::Jinja),
        Some("vto") => Some(Language::Vento),
        Some("mustache") => Some(Language::Mustache),
        Some("xml" | "svg" | "wsdl" | "xsd" | "xslt" | "xsl") => Some(Language::Xml),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;

    #[test]
    fn mjs() {
        let mut ext = None;
        let _ = format_text(
            "<script type=module>;</script>",
            Language::Html,
            &Default::default(),
            |code, hints| {
                ext = Some(hints.ext.to_owned());
                Ok::<_, Infallible>(Cow::from(code))
            },
        );
        assert_eq!(ext.as_deref(), Some("mjs"));
    }

    #[test]
    fn mts() {
        let mut ext = None;
        let _ = format_text(
            "<script type=\"module\" lang='ts'>;</script>",
            Language::Html,
            &Default::default(),
            |code, hints| {
                ext = Some(hints.ext.to_owned());
                Ok::<_, Infallible>(Cow::from(code))
            },
        );
        assert_eq!(ext.as_deref(), Some("mts"));
    }

    #[test]
    fn jsx_with_module() {
        let mut ext = None;
        let _ = format_text(
            "<script type=module lang=jsx>;</script>",
            Language::Html,
            &Default::default(),
            |code, hints| {
                ext = Some(hints.ext.to_owned());
                Ok::<_, Infallible>(Cow::from(code))
            },
        );
        assert_eq!(ext.as_deref(), Some("jsx"));
    }

    #[test]
    fn tsx_with_module() {
        let mut ext = None;
        let _ = format_text(
            "<script type=\"module\" lang='tsx'>;</script>",
            Language::Html,
            &Default::default(),
            |code, hints| {
                ext = Some(hints.ext.to_owned());
                Ok::<_, Infallible>(Cow::from(code))
            },
        );
        assert_eq!(ext.as_deref(), Some("tsx"));
    }
}
