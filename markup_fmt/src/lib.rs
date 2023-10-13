mod ast;
mod config;
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
use tiny_pretty::PrintOptions;

pub fn format_text(code: &str, language: Language) -> Result<String, SyntaxError> {
    let mut parser = Parser::new(code, language.clone());
    let ast = parser.parse_root()?;
    let ctx = Ctx {
        source: code,
        language,
        indent_width: 2,
        options: Default::default(),
    };
    Ok(tiny_pretty::print(
        &ast.doc(&ctx),
        &PrintOptions {
            tab_size: ctx.indent_width,
            ..Default::default()
        },
    ))
}
