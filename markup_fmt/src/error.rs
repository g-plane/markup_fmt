use std::{error::Error, fmt};

#[derive(Clone, Debug)]
pub struct SyntaxError {
    pub kind: SyntaxErrorKind,
    pub pos: usize,
}

#[derive(Clone, Debug)]
pub enum SyntaxErrorKind {
    ExpectAttrName,
    ExpectAttrValue,
    ExpectCloseTag,
    ExpectComment,
    ExpectElement,
    ExpectIdentifier,
    ExpectSelfCloseTag,
    ExpectSvelteAttr,
    ExpectSvelteBlockEnd,
    ExpectSvelteIfBlock,
    ExpectSvelteInterpolation,
    ExpectTagName,
    ExpectTextNode,
    ExpectVueDirective,
    ExpectVueInterpolation,
    UnknownSvelteBlock,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self.kind {
            SyntaxErrorKind::ExpectAttrName => "expect attribute name",
            SyntaxErrorKind::ExpectAttrValue => "expect attribute value",
            SyntaxErrorKind::ExpectCloseTag => "expect close tag",
            SyntaxErrorKind::ExpectComment => "expect comment",
            SyntaxErrorKind::ExpectElement => "expect element",
            SyntaxErrorKind::ExpectIdentifier => "expect identifier",
            SyntaxErrorKind::ExpectSelfCloseTag => "expect self close tag",
            SyntaxErrorKind::ExpectSvelteAttr => "expect Svelte attribute",
            SyntaxErrorKind::ExpectSvelteBlockEnd => "expect end of Svelte block",
            SyntaxErrorKind::ExpectSvelteIfBlock => "expect Svelte if block",
            SyntaxErrorKind::ExpectSvelteInterpolation => "expect Svelte interpolation",
            SyntaxErrorKind::ExpectTagName => "expect tag name",
            SyntaxErrorKind::ExpectTextNode => "expect text node",
            SyntaxErrorKind::ExpectVueDirective => "expect Vue directive",
            SyntaxErrorKind::ExpectVueInterpolation => "expect Vue interpolation",
            SyntaxErrorKind::UnknownSvelteBlock => "unknown Svelte block",
        };

        write!(f, "syntax error '{reason}' at position {}", self.pos)
    }
}

impl Error for SyntaxError {}

#[derive(Debug)]
pub enum FormatError<E> {
    Syntax(SyntaxError),
    External(E),
}

impl<E> fmt::Display for FormatError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Syntax(e) => e.fmt(f),
            FormatError::External(e) => e.fmt(f),
        }
    }
}

impl<E> Error for FormatError<E> where E: Error {}
