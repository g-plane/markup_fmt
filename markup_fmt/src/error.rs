use std::{borrow::Cow, error::Error, fmt};

#[derive(Clone, Debug)]
pub struct SyntaxError {
    pub kind: SyntaxErrorKind,
    pub pos: usize,
}

#[derive(Clone, Debug)]
pub enum SyntaxErrorKind {
    ExpectAttrName,
    ExpectAttrValue,
    ExpectChar(char),
    ExpectCloseTag,
    ExpectComment,
    ExpectDoctype,
    ExpectElement,
    ExpectIdentifier,
    ExpectKeyword(&'static str),
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
    ExpectVueDirective,
    ExpectVueInterpolation,
    UnknownSvelteBlock,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason: Cow<_> = match self.kind {
            SyntaxErrorKind::ExpectAttrName => "expect attribute name".into(),
            SyntaxErrorKind::ExpectAttrValue => "expect attribute value".into(),
            SyntaxErrorKind::ExpectChar(c) => format!("expect char '{c}'").into(),
            SyntaxErrorKind::ExpectCloseTag => "expect close tag".into(),
            SyntaxErrorKind::ExpectComment => "expect comment".into(),
            SyntaxErrorKind::ExpectDoctype => "expect HTML doctype".into(),
            SyntaxErrorKind::ExpectElement => "expect element".into(),
            SyntaxErrorKind::ExpectIdentifier => "expect identifier".into(),
            SyntaxErrorKind::ExpectKeyword(keyword) => {
                format!("expect keyword '{}'", keyword).into()
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
            SyntaxErrorKind::ExpectVueDirective => "expect Vue directive".into(),
            SyntaxErrorKind::ExpectVueInterpolation => "expect Vue interpolation".into(),
            SyntaxErrorKind::UnknownSvelteBlock => "unknown Svelte block".into(),
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
