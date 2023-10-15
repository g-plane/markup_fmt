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
use config::LanguageOptions;
use std::{borrow::Cow, path::Path};
use tiny_pretty::PrintOptions;

pub fn format_text<F>(
    code: &str,
    language: Language,
    external_formatter: F,
) -> Result<String, SyntaxError>
where
    F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
{
    let mut parser = Parser::new(code, language.clone());
    let ast = parser.parse_root()?;
    let ctx = Ctx {
        language,
        indent_width: 2,
        options: LanguageOptions {
            ..Default::default()
        },
        external_formatter,
    };
    Ok(tiny_pretty::print(
        &ast.doc(&ctx),
        &PrintOptions {
            tab_size: ctx.indent_width,
            ..Default::default()
        },
    ))
}
