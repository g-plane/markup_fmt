mod ast;
pub mod config;
mod ctx;
mod helpers;
mod parser;
mod printer;

pub use crate::parser::Language;
use crate::{
    ctx::Ctx,
    parser::{Parser, SyntaxError},
    printer::DocGen,
};
use config::FormatOptions;
use std::{borrow::Cow, path::Path};
use tiny_pretty::{IndentKind, PrintOptions};

pub fn format_text<F>(
    code: &str,
    language: Language,
    options: &FormatOptions,
    external_formatter: F,
) -> Result<String, SyntaxError>
where
    F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
{
    let mut parser = Parser::new(code, language.clone());
    let ast = parser.parse_root()?;
    let ctx = Ctx {
        language,
        indent_width: options.layout.indent_width,
        options: &options.language,
        external_formatter,
    };
    Ok(tiny_pretty::print(
        &ast.doc(&ctx),
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
