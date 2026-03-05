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
    ExpectAngularBlock(&'static str),
    ExpectAngularLet,
    ExpectAngularSwitch,
    ExpectAstroAttr,
    ExpectAstroExpr,
    ExpectAttrName,
    ExpectAttrValue,
    ExpectCdata,
    ExpectChar(char),
    ExpectCloseTag {
        tag_name: String,
        line: usize,
        column: usize,
    },
    ExpectComment,
    ExpectDoctype,
    ExpectElement,
    ExpectFrontMatter,
    ExpectIdentifier,
    ExpectJinjaBlockEnd {
        tag_name: String,
        line: usize,
        column: usize,
    },
    ExpectJinjaTag,
    ExpectKeyword(&'static str),
    ExpectMustacheInterpolation,
    ExpectSelfCloseTag,
    ExpectSvelteAttachment,
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
    ExpectSvelteSnippetBlock,
    ExpectSvelteThenBlock,
    ExpectTagName,
    ExpectTextNode,
    ExpectVentoBlockEnd,
    ExpectVueDirective,
    ExpectXmlDecl,
}

impl fmt::Display for SyntaxErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason: Cow<_> = match self {
            SyntaxErrorKind::ExpectAngularBlock(keyword) => {
                format!("expected Angular `@{keyword}` block").into()
            }
            SyntaxErrorKind::ExpectAngularLet => "expected Angular `@let`".into(),
            SyntaxErrorKind::ExpectAngularSwitch => "expected Angular `@switch`".into(),
            SyntaxErrorKind::ExpectAstroAttr => "expected Astro attribute".into(),
            SyntaxErrorKind::ExpectAstroExpr => "expected Astro expression".into(),
            SyntaxErrorKind::ExpectAttrName => "expected attribute name".into(),
            SyntaxErrorKind::ExpectAttrValue => "expected attribute value".into(),
            SyntaxErrorKind::ExpectCdata => "expected CDATA section".into(),
            SyntaxErrorKind::ExpectChar(c) => format!("expected char '{c}'").into(),
            SyntaxErrorKind::ExpectCloseTag {
                tag_name,
                line,
                column,
            } => format!(
                "expected close tag for opening tag <{tag_name}> from line {line}, column {column}"
            )
            .into(),
            SyntaxErrorKind::ExpectComment => "expected comment".into(),
            SyntaxErrorKind::ExpectDoctype => "expected HTML doctype".into(),
            SyntaxErrorKind::ExpectElement => "expected element".into(),
            SyntaxErrorKind::ExpectFrontMatter => "expected front matter".into(),
            SyntaxErrorKind::ExpectIdentifier => "expected identifier".into(),
            SyntaxErrorKind::ExpectJinjaBlockEnd {
                tag_name,
                line,
                column,
            } => format!(
                "expected end tag for opening Jinja block {{% {tag_name} %}} from line {line}, column {column}"
            )
            .into(),
            SyntaxErrorKind::ExpectJinjaTag => "expected Jinja tag".into(),
            SyntaxErrorKind::ExpectKeyword(keyword) => {
                format!("expected keyword '{keyword}'").into()
            }
            SyntaxErrorKind::ExpectMustacheInterpolation => {
                "expected mustache-like interpolation".into()
            }
            SyntaxErrorKind::ExpectSelfCloseTag => "expected self close tag".into(),
            SyntaxErrorKind::ExpectSvelteAttachment => "expected Svelte attachment".into(),
            SyntaxErrorKind::ExpectSvelteAtTag => "expected Svelte `{@` tag".into(),
            SyntaxErrorKind::ExpectSvelteAttr => "expected Svelte attribute".into(),
            SyntaxErrorKind::ExpectSvelteAwaitBlock => "expected Svelte await block".into(),
            SyntaxErrorKind::ExpectSvelteBlockEnd => "expected end of Svelte block".into(),
            SyntaxErrorKind::ExpectSvelteCatchBlock => "expected Svelte catch block".into(),
            SyntaxErrorKind::ExpectSvelteEachBlock => "expected Svelte each block".into(),
            SyntaxErrorKind::ExpectSvelteElseIfBlock => "expected Svelte else if block".into(),
            SyntaxErrorKind::ExpectSvelteIfBlock => "expected Svelte if block".into(),
            SyntaxErrorKind::ExpectSvelteInterpolation => "expected Svelte interpolation".into(),
            SyntaxErrorKind::ExpectSvelteKeyBlock => "expected Svelte key block".into(),
            SyntaxErrorKind::ExpectSvelteSnippetBlock => "expected Svelte snippet block".into(),
            SyntaxErrorKind::ExpectSvelteThenBlock => "expected Svelte then block".into(),
            SyntaxErrorKind::ExpectTagName => "expected tag name".into(),
            SyntaxErrorKind::ExpectTextNode => "expected text node".into(),
            SyntaxErrorKind::ExpectVentoBlockEnd => "expected Vento block end".into(),
            SyntaxErrorKind::ExpectVueDirective => "expected Vue directive".into(),
            SyntaxErrorKind::ExpectXmlDecl => "expected XML declaration".into(),
        };

        write!(f, "{reason}")
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "syntax error '{}' at line {}, column {}",
            self.kind, self.line, self.column
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
