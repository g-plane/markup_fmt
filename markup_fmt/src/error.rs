use std::{borrow::Cow, error::Error, fmt};

#[derive(Clone, Debug)]
/// Syntax error when parsing tags, not `<script>` or `<style>` tag.
pub struct SyntaxError {
    pub kind: SyntaxErrorKind,
    pub pos: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug)]
pub enum SyntaxErrorKind {
    ExpectAngularFor,
    ExpectAngularIf,
    ExpectAngularSwitch,
    ExpectAstroAttr,
    ExpectAstroExpr,
    ExpectAttrName,
    ExpectAttrValue,
    ExpectChar(char),
    ExpectCloseTag,
    ExpectComment,
    ExpectDoctype,
    ExpectElement,
    ExpectFrontMatter,
    ExpectIdentifier,
    ExpectJinjaBlockEnd,
    ExpectJinjaTag,
    ExpectKeyword(&'static str),
    ExpectMustacheInterpolation,
    ExpectSelfCloseTag,
    ExpectSvelteAtTag,
    ExpectSvelteAttr,
    ExpectSvelteAwaitBlock,
    ExpectSvelteBlockEnd,
    ExpectSvelteCatchBlock,
    ExpectSvelteEachBlock,
    ExpectSvelteElseIfBlock,
    ExpectSvelteIfBlock,
    ExpectSvelteInterpolation,
    ExpectSvelteKeyBlock,
    ExpectSvelteThenBlock,
    ExpectTagName,
    ExpectTextNode,
    ExpectVentoBlockEnd,
    ExpectVueDirective,
    UnknownSvelteBlock,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason: Cow<_> = match self.kind {
            SyntaxErrorKind::ExpectAngularFor => "expect Angular `@for`".into(),
            SyntaxErrorKind::ExpectAngularIf => "expect Angular `@if`".into(),
            SyntaxErrorKind::ExpectAngularSwitch => "expect Angular `@switch`".into(),
            SyntaxErrorKind::ExpectAstroAttr => "expect Astro attribute".into(),
            SyntaxErrorKind::ExpectAstroExpr => "expect Astro expression".into(),
            SyntaxErrorKind::ExpectAttrName => "expect attribute name".into(),
            SyntaxErrorKind::ExpectAttrValue => "expect attribute value".into(),
            SyntaxErrorKind::ExpectChar(c) => format!("expect char '{c}'").into(),
            SyntaxErrorKind::ExpectCloseTag => "expect close tag".into(),
            SyntaxErrorKind::ExpectComment => "expect comment".into(),
            SyntaxErrorKind::ExpectDoctype => "expect HTML doctype".into(),
            SyntaxErrorKind::ExpectElement => "expect element".into(),
            SyntaxErrorKind::ExpectFrontMatter => "expect front matter".into(),
            SyntaxErrorKind::ExpectIdentifier => "expect identifier".into(),
            SyntaxErrorKind::ExpectJinjaBlockEnd => "expect Jinja block end".into(),
            SyntaxErrorKind::ExpectJinjaTag => "expect Jinja tag".into(),
            SyntaxErrorKind::ExpectKeyword(keyword) => {
                format!("expect keyword '{}'", keyword).into()
            }
            SyntaxErrorKind::ExpectMustacheInterpolation => {
                "expect mustache-like interpolation".into()
            }
            SyntaxErrorKind::ExpectSelfCloseTag => "expect self close tag".into(),
            SyntaxErrorKind::ExpectSvelteAtTag => "expect Svelte `{@` tag".into(),
            SyntaxErrorKind::ExpectSvelteAttr => "expect Svelte attribute".into(),
            SyntaxErrorKind::ExpectSvelteAwaitBlock => "expect Svelte await block".into(),
            SyntaxErrorKind::ExpectSvelteBlockEnd => "expect end of Svelte block".into(),
            SyntaxErrorKind::ExpectSvelteCatchBlock => "expect Svelte catch block".into(),
            SyntaxErrorKind::ExpectSvelteEachBlock => "expect Svelte each block".into(),
            SyntaxErrorKind::ExpectSvelteElseIfBlock => "expect Svelte else if block".into(),
            SyntaxErrorKind::ExpectSvelteIfBlock => "expect Svelte if block".into(),
            SyntaxErrorKind::ExpectSvelteInterpolation => "expect Svelte interpolation".into(),
            SyntaxErrorKind::ExpectSvelteKeyBlock => "expect Svelte key block".into(),
            SyntaxErrorKind::ExpectSvelteThenBlock => "expect Svelte then block".into(),
            SyntaxErrorKind::ExpectTagName => "expect tag name".into(),
            SyntaxErrorKind::ExpectTextNode => "expect text node".into(),
            SyntaxErrorKind::ExpectVentoBlockEnd => "expect Vento block end".into(),
            SyntaxErrorKind::ExpectVueDirective => "expect Vue directive".into(),
            SyntaxErrorKind::UnknownSvelteBlock => "unknown Svelte block".into(),
        };

        write!(
            f,
            "syntax error '{reason}' at line {}, column {}",
            self.line, self.column
        )
    }
}

impl Error for SyntaxError {}

#[derive(Debug)]
/// The error type for markup_fmt.
pub enum FormatError<E> {
    /// Syntax error when parsing tags.
    Syntax(SyntaxError),
    /// Error from external formatter, for example,
    /// there're errors when formatting the `<script>` or `<style>` tag.
    External(Vec<E>),
}

impl<E> fmt::Display for FormatError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Syntax(e) => e.fmt(f),
            FormatError::External(errors) => {
                writeln!(f, "failed to format code with external formatter:")?;
                for error in errors {
                    writeln!(f, "{error}")?;
                }
                Ok(())
            }
        }
    }
}

impl<E> Error for FormatError<E> where E: Error {}
