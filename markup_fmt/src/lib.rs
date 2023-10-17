mod ast;
pub mod config;
mod ctx;
mod error;
mod helpers;
mod parser;
mod printer;

use crate::{config::FormatOptions, ctx::Ctx, parser::Parser, printer::DocGen};
pub use crate::{error::FormatError, parser::Language};
use std::{borrow::Cow, path::Path};
use tiny_pretty::{IndentKind, PrintOptions};

pub fn format_text<E, F>(
    code: &str,
    language: Language,
    options: &FormatOptions,
    external_formatter: F,
) -> Result<String, FormatError<E>>
where
    F: for<'a> FnMut(&Path, &'a str) -> Result<Cow<'a, str>, E>,
{
    let mut parser = Parser::new(code, language.clone());
    let ast = parser.parse_root().map_err(FormatError::Syntax)?;
    let mut ctx = Ctx {
        language,
        indent_width: options.layout.indent_width,
        options: &options.language,
        current_tag_name: None,
        external_formatter,
        external_formatter_error: None,
    };

    let doc = ast.doc(&mut ctx);
    if let Some(error) = ctx.external_formatter_error {
        return Err(FormatError::External(error));
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
