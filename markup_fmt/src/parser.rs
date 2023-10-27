//! This parser is designed for internal use,
//! not generating general-purpose AST.
//!
//! Also, the parser consumes string then produces AST directly without tokenizing.
//! For a formal parser, it should be:
//! `source -> tokens (produced by lexer/tokenizer) -> AST (produced by parser)`.
//! So, if you're learning or looking for a parser,
//! this is not a good example and you should look for other projects.

use crate::{
    ast::*,
    error::{SyntaxError, SyntaxErrorKind},
    helpers,
};
use std::{iter::Peekable, str::CharIndices};

#[derive(Clone, Debug)]
pub enum Language {
    Html,
    Vue,
    Svelte,
}

pub struct Parser<'s> {
    source: &'s str,
    language: Language,
    chars: Peekable<CharIndices<'s>>,
}

impl<'s> Parser<'s> {
    pub fn new(source: &'s str, language: Language) -> Self {
        Self {
            source,
            language,
            chars: source.char_indices().peekable(),
        }
    }

    fn try_parse<F, R>(&mut self, f: F) -> PResult<R>
    where
        F: FnOnce(&mut Self) -> PResult<R>,
    {
        let chars = self.chars.clone();
        let result = f(self);
        if result.is_err() {
            self.chars = chars;
        }
        result
    }

    fn emit_error(&mut self, kind: SyntaxErrorKind) -> SyntaxError {
        SyntaxError {
            kind,
            pos: self
                .chars
                .peek()
                .map(|(pos, _)| *pos)
                .unwrap_or(self.source.len()),
        }
    }

    fn skip_ws(&mut self) {
        while self
            .chars
            .next_if(|(_, c)| c.is_ascii_whitespace())
            .is_some()
        {}
    }

    fn parse_attr(&mut self) -> PResult<Attribute<'s>> {
        match self.language {
            Language::Html => self.parse_native_attr().map(Attribute::NativeAttribute),
            Language::Vue => self
                .try_parse(Parser::parse_vue_directive)
                .map(Attribute::VueDirective)
                .or_else(|_| self.parse_native_attr().map(Attribute::NativeAttribute)),
            Language::Svelte => self
                .try_parse(Parser::parse_svelte_attr)
                .map(Attribute::SvelteAttribute)
                .or_else(|_| self.parse_native_attr().map(Attribute::NativeAttribute)),
        }
    }

    fn parse_attr_name(&mut self) -> PResult<&'s str> {
        let Some((start, _)) = self.chars.next_if(|(_, c)| is_attr_name_char(*c)) else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectAttrName));
        };
        let mut end = start;

        while let Some((i, _)) = self.chars.next_if(|(_, c)| is_attr_name_char(*c)) {
            end = i;
        }

        unsafe { Ok(self.source.get_unchecked(start..=end)) }
    }

    fn parse_attr_value(&mut self) -> PResult<&'s str> {
        let quote = self.chars.next_if(|(_, c)| *c == '"' || *c == '\'');

        if let Some((start, quote)) = quote {
            let start = start + 1;
            let mut end = start;
            loop {
                match self.chars.next() {
                    Some((i, c)) if c == quote => {
                        end = i;
                        break;
                    }
                    Some(..) => continue,
                    None => break,
                }
            }
            Ok(unsafe { self.source.get_unchecked(start..end) })
        } else {
            fn is_unquoted_attr_value_char(c: char) -> bool {
                !c.is_ascii_whitespace() && !matches!(c, '"' | '\'' | '=' | '<' | '>' | '`')
            }

            let Some((start, _)) = self.chars.next_if(|(_, c)| is_unquoted_attr_value_char(*c))
            else {
                return Err(self.emit_error(SyntaxErrorKind::ExpectAttrValue));
            };
            let mut end = start;

            while let Some((i, _)) = self.chars.next_if(|(_, c)| is_unquoted_attr_value_char(*c)) {
                end = i;
            }

            unsafe { Ok(self.source.get_unchecked(start..=end)) }
        }
    }

    fn parse_comment(&mut self) -> PResult<Comment<'s>> {
        let Some((start, _)) = self
            .chars
            .next_if(|(_, c)| *c == '<')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '!'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '-'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '-'))
        else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectComment));
        };
        let start = start + 1;

        let mut end = start;
        loop {
            match self.chars.next() {
                Some((i, '-')) => {
                    let mut chars = self.chars.clone();
                    if chars
                        .next_if(|(_, c)| *c == '-')
                        .and_then(|_| chars.next_if(|(_, c)| *c == '>'))
                        .is_some()
                    {
                        end = i;
                        self.chars = chars;
                        break;
                    } else {
                        continue;
                    }
                }
                Some(..) => continue,
                None => break,
            }
        }

        Ok(Comment {
            raw: unsafe { self.source.get_unchecked(start..end) },
        })
    }

    fn parse_doctype(&mut self) -> PResult<()> {
        if self
            .chars
            .next_if(|(_, c)| *c == '<')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '!'))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'d')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'o')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'c')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'t')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'y')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'p')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'e')))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectDoctype));
        };
        self.skip_ws();

        if self
            .chars
            .next_if(|(_, c)| c.eq_ignore_ascii_case(&'h'))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'t')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'m')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'l')))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectDoctype));
        }
        self.skip_ws();

        if self.chars.next_if(|(_, c)| *c == '>').is_some() {
            Ok(())
        } else {
            Err(self.emit_error(SyntaxErrorKind::ExpectDoctype))
        }
    }

    fn parse_element(&mut self) -> PResult<Element<'s>> {
        let Some(..) = self.chars.next_if(|(_, c)| *c == '<') else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectElement));
        };
        let tag_name = self.parse_tag_name()?;
        let void_element = helpers::is_void_element(tag_name, self.language.clone());

        let mut attrs = vec![];
        loop {
            self.skip_ws();
            match self.chars.peek() {
                Some((_, '/')) => {
                    self.chars.next();
                    if self.chars.next_if(|(_, c)| *c == '>').is_some() {
                        return Ok(Element {
                            tag_name,
                            attrs,
                            children: vec![],
                            self_closing: true,
                            void_element,
                        });
                    } else {
                        return Err(self.emit_error(SyntaxErrorKind::ExpectSelfCloseTag));
                    }
                }
                Some((_, '>')) => {
                    self.chars.next();
                    if void_element {
                        return Ok(Element {
                            tag_name,
                            attrs,
                            children: vec![],
                            self_closing: false,
                            void_element,
                        });
                    }
                    break;
                }
                _ => {
                    attrs.push(self.parse_attr()?);
                }
            }
        }

        let mut children = vec![];
        if tag_name.eq_ignore_ascii_case("script")
            || tag_name.eq_ignore_ascii_case("style")
            || tag_name.eq_ignore_ascii_case("pre")
            || tag_name.eq_ignore_ascii_case("textarea")
        {
            let text_node = self.parse_raw_text_node(tag_name)?;
            if !text_node.raw.is_empty() {
                children.push(Node::TextNode(text_node));
            }
        }

        loop {
            match self.chars.peek() {
                Some((_, '<')) => {
                    let mut chars = self.chars.clone();
                    chars.next();
                    if let Some((pos, _)) = chars.next_if(|(_, c)| *c == '/') {
                        self.chars = chars;
                        let close_tag_name = self.parse_tag_name()?;
                        if !close_tag_name.eq_ignore_ascii_case(tag_name) {
                            return Err(SyntaxError {
                                kind: SyntaxErrorKind::ExpectCloseTag,
                                pos,
                            });
                        }
                        self.skip_ws();
                        if self.chars.next_if(|(_, c)| *c == '>').is_some() {
                            break;
                        } else {
                            return Err(self.emit_error(SyntaxErrorKind::ExpectCloseTag));
                        }
                    } else {
                        children.push(self.parse_node()?);
                    }
                }
                Some(..) => {
                    children.push(
                        if tag_name.eq_ignore_ascii_case("script")
                            || tag_name.eq_ignore_ascii_case("style")
                            || tag_name.eq_ignore_ascii_case("pre")
                            || tag_name.eq_ignore_ascii_case("textarea")
                        {
                            self.parse_raw_text_node(tag_name).map(Node::TextNode)?
                        } else {
                            self.parse_node()?
                        },
                    );
                }
                None => return Err(self.emit_error(SyntaxErrorKind::ExpectCloseTag)),
            }
        }

        Ok(Element {
            tag_name,
            attrs,
            children,
            self_closing: false,
            void_element,
        })
    }

    fn parse_identifier(&mut self) -> PResult<&'s str> {
        fn is_identifier_char(c: char) -> bool {
            c.is_ascii_alphanumeric() || c == '-' || c == '_' || !c.is_ascii() || c == '\\'
        }

        let Some((start, _)) = self.chars.next_if(|(_, c)| is_identifier_char(*c)) else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectIdentifier));
        };
        let mut end = start;

        while let Some((i, _)) = self.chars.next_if(|(_, c)| is_identifier_char(*c)) {
            end = i;
        }

        unsafe { Ok(self.source.get_unchecked(start..=end)) }
    }

    /// This will consume the open and close char.
    fn parse_inside(&mut self, open: char, close: char, inclusive: bool) -> PResult<&'s str> {
        let Some(start) = self.chars.next_if(|(_, c)| *c == open).map(|(i, c)| {
            if inclusive {
                i
            } else {
                i + c.len_utf8()
            }
        }) else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar(open)));
        };
        let mut end = start;
        let mut stack = 0u8;
        for (i, c) in self.chars.by_ref() {
            if c == open {
                stack += 1;
            } else if c == close {
                if stack == 0 {
                    end = if inclusive { i + close.len_utf8() } else { i };
                    break;
                } else {
                    stack -= 1;
                }
            }
        }
        Ok(unsafe { self.source.get_unchecked(start..end) })
    }

    fn parse_native_attr(&mut self) -> PResult<NativeAttribute<'s>> {
        let name = self.parse_attr_name()?;
        self.skip_ws();
        let value = if self.chars.next_if(|(_, c)| *c == '=').is_some() {
            self.skip_ws();
            Some(self.parse_attr_value()?)
        } else {
            None
        };
        Ok(NativeAttribute { name, value })
    }

    fn parse_node(&mut self) -> PResult<Node<'s>> {
        match self.chars.peek() {
            Some((_, '<')) => {
                let mut chars = self.chars.clone();
                chars.next();
                match chars.next() {
                    Some((_, c)) if is_tag_name_char(c) => self.parse_element().map(Node::Element),
                    Some((_, '!')) => {
                        if let Language::Html = self.language {
                            self.try_parse(Parser::parse_comment)
                                .map(Node::Comment)
                                .or_else(|_| {
                                    self.try_parse(Parser::parse_doctype).map(|_| Node::Doctype)
                                })
                                .or_else(|_| self.parse_text_node().map(Node::TextNode))
                        } else {
                            self.parse_comment().map(Node::Comment)
                        }
                    }
                    _ => self.parse_text_node().map(Node::TextNode),
                }
            }
            Some((_, '{')) => {
                let mut chars = self.chars.clone();
                chars.next();
                match chars.next() {
                    Some((_, '{')) if matches!(self.language, Language::Vue) => {
                        self.parse_vue_interpolation().map(Node::VueInterpolation)
                    }
                    Some((pos, '#')) if matches!(self.language, Language::Svelte) => self
                        .try_parse(Parser::parse_svelte_if_block)
                        .map(Node::SvelteIfBlock)
                        .or_else(|_| {
                            self.try_parse(Parser::parse_svelte_each_block)
                                .map(Node::SvelteEachBlock)
                        })
                        .or_else(|_| {
                            self.try_parse(Parser::parse_svelte_await_block)
                                .map(Node::SvelteAwaitBlock)
                        })
                        .or_else(|_| {
                            self.try_parse(Parser::parse_svelte_key_block)
                                .map(Node::SvelteKeyBlock)
                        })
                        .map_err(|_| SyntaxError {
                            kind: SyntaxErrorKind::UnknownSvelteBlock,
                            pos,
                        }),
                    _ => {
                        if matches!(self.language, Language::Svelte) {
                            self.parse_svelte_interpolation()
                                .map(Node::SvelteInterpolation)
                        } else {
                            self.parse_text_node().map(Node::TextNode)
                        }
                    }
                }
            }
            Some(..) => self.parse_text_node().map(Node::TextNode),
            None => Err(self.emit_error(SyntaxErrorKind::ExpectElement)),
        }
    }

    fn parse_raw_text_node(&mut self, tag_name: &str) -> PResult<TextNode<'s>> {
        let start = self
            .chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.source.len());

        let mut line_breaks = 0;
        let end;
        loop {
            match self.chars.peek() {
                Some((i, '<')) => {
                    let i = *i;
                    let mut chars = self.chars.clone();
                    chars.next();
                    if chars
                        .next_if(|(_, c)| *c == '/')
                        .map(|_| {
                            chars
                                .zip(tag_name.chars())
                                .all(|((_, a), b)| a.eq_ignore_ascii_case(&b))
                        })
                        .unwrap_or_default()
                    {
                        end = i;
                        break;
                    } else {
                        self.chars.next();
                    }
                }
                Some((_, c)) => {
                    if *c == '\n' {
                        line_breaks += 1;
                    }
                    self.chars.next();
                }
                None => {
                    end = self.source.len();
                    break;
                }
            }
        }

        Ok(TextNode {
            raw: unsafe { self.source.get_unchecked(start..end) },
            line_breaks,
        })
    }

    pub fn parse_root(&mut self) -> PResult<Root<'s>> {
        let mut children = vec![];
        while self.chars.peek().is_some() {
            children.push(self.parse_node()?);
        }

        Ok(Root { children })
    }

    fn parse_svelte_attr(&mut self) -> PResult<SvelteAttribute<'s>> {
        let (name, start) = if let Some((start, _)) = self.chars.next_if(|(_, c)| *c == '{') {
            (None, start)
        } else {
            let name = self.parse_attr_name()?;
            self.skip_ws();
            let Some((start, _)) = self
                .chars
                .next_if(|(_, c)| *c == '=')
                .and_then(|_| self.chars.next_if(|(_, c)| *c == '{'))
            else {
                return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteAttr));
            };
            (Some(name), start)
        };

        let start = start + 1;
        let mut end = start;
        loop {
            match self.chars.next() {
                Some((i, '}')) => {
                    end = i;
                    break;
                }
                Some(..) => continue,
                None => break,
            }
        }
        Ok(SvelteAttribute {
            name,
            expr: unsafe { self.source.get_unchecked(start..end) },
        })
    }

    fn parse_svelte_await_block(&mut self) -> PResult<SvelteAwaitBlock<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '#'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'w'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'i'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 't'))
            .and_then(|_| self.chars.next_if(|(_, c)| c.is_ascii_whitespace()))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteIfBlock));
        };
        self.skip_ws();

        let expr = {
            let start = self
                .chars
                .peek()
                .map(|(i, _)| *i)
                .unwrap_or(self.source.len());
            let mut end = start;
            let mut braces_stack = 0u8;
            loop {
                match self.chars.peek() {
                    Some((i, c)) if c.is_ascii_whitespace() => {
                        let i = *i;
                        self.skip_ws();
                        let mut chars = self.chars.clone();
                        match chars.next() {
                            Some((_, 't')) => {
                                if chars
                                    .next_if(|(_, c)| *c == 'h')
                                    .and_then(|_| chars.next_if(|(_, c)| *c == 'e'))
                                    .and_then(|_| chars.next_if(|(_, c)| *c == 'n'))
                                    .is_some()
                                {
                                    end = i;
                                    break;
                                }
                            }
                            Some((_, 'c')) => {
                                if chars
                                    .next_if(|(_, c)| *c == 'a')
                                    .and_then(|_| chars.next_if(|(_, c)| *c == 't'))
                                    .and_then(|_| chars.next_if(|(_, c)| *c == 'c'))
                                    .and_then(|_| chars.next_if(|(_, c)| *c == 'h'))
                                    .is_some()
                                {
                                    end = i;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    Some((i, '{')) => {
                        braces_stack += 1;
                        end = *i;
                        self.chars.next();
                    }
                    Some((i, '}')) => {
                        end = *i;
                        if braces_stack == 0 {
                            break;
                        } else {
                            self.chars.next();
                            braces_stack -= 1;
                        }
                    }
                    Some((i, _)) => {
                        end = *i;
                        self.chars.next();
                    }
                    None => break,
                }
            }
            unsafe { self.source.get_unchecked(start..end) }
        };

        self.skip_ws();
        let then_binding = if self
            .chars
            .next_if(|(_, c)| *c == 't')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'h'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'e'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'n'))
            .is_some()
        {
            self.skip_ws();
            Some(self.parse_svelte_binding()?)
        } else {
            None
        };

        self.skip_ws();
        let catch_binding = if self
            .chars
            .next_if(|(_, c)| *c == 'c')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 't'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'c'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'h'))
            .is_some()
        {
            self.skip_ws();
            Some(self.parse_svelte_binding()?)
        } else {
            None
        };

        self.skip_ws();
        if self.chars.next_if(|(_, c)| *c == '}').is_none() {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar('}')));
        }

        let children = self.parse_svelte_block_children()?;

        let then_block = if self
            .try_parse(|parser| {
                parser
                    .chars
                    .next_if(|(_, c)| *c == '{')
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == ':'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 't'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 'h'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 'e'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 'n'))
                    .ok_or_else(|| parser.emit_error(SyntaxErrorKind::ExpectSvelteThenBlock))
            })
            .is_ok()
        {
            self.skip_ws();
            let binding = self.parse_svelte_binding()?;
            self.skip_ws();
            if self.chars.next_if(|(_, c)| *c == '}').is_none() {
                return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteThenBlock));
            }
            let children = self.parse_svelte_block_children()?;
            Some(SvelteThenBlock { binding, children })
        } else {
            None
        };

        let catch_block = if self
            .try_parse(|parser| {
                parser
                    .chars
                    .next_if(|(_, c)| *c == '{')
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == ':'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 'c'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 'a'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 't'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 'c'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 'h'))
                    .ok_or_else(|| parser.emit_error(SyntaxErrorKind::ExpectSvelteCatchBlock))
            })
            .is_ok()
        {
            self.skip_ws();
            let binding = self.parse_svelte_binding()?;
            self.skip_ws();
            if self.chars.next_if(|(_, c)| *c == '}').is_none() {
                return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteCatchBlock));
            }
            let children = self.parse_svelte_block_children()?;
            Some(SvelteCatchBlock { binding, children })
        } else {
            None
        };

        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '/'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'w'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'i'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 't'))
            .map(|_| self.skip_ws())
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '}'))
            .is_some()
        {
            Ok(SvelteAwaitBlock {
                expr,
                then_binding,
                catch_binding,
                children,
                then_block,
                catch_block,
            })
        } else {
            Err(self.emit_error(SyntaxErrorKind::ExpectSvelteBlockEnd))
        }
    }

    fn parse_svelte_binding(&mut self) -> PResult<&'s str> {
        match self.chars.peek() {
            Some((_, '{')) => self.parse_inside('{', '}', true),
            Some((_, '[')) => self.parse_inside('[', ']', true),
            _ => self.parse_identifier(),
        }
    }

    fn parse_svelte_block_children(&mut self) -> PResult<Vec<Node<'s>>> {
        let mut children = vec![];
        loop {
            match self.chars.peek() {
                Some((_, '{')) => {
                    let mut chars = self.chars.clone();
                    chars.next();
                    if chars.next_if(|(_, c)| *c == '/' || *c == ':').is_some() {
                        break;
                    } else {
                        children.push(self.parse_node()?);
                    }
                }
                Some(..) => {
                    children.push(self.parse_node()?);
                }
                None => return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteBlockEnd)),
            }
        }
        Ok(children)
    }

    fn parse_svelte_each_block(&mut self) -> PResult<SvelteEachBlock<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '#'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'e'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'c'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'h'))
            .and_then(|_| self.chars.next_if(|(_, c)| c.is_ascii_whitespace()))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteIfBlock));
        };
        self.skip_ws();

        let expr = {
            let start = self
                .chars
                .peek()
                .map(|(i, _)| *i)
                .unwrap_or(self.source.len());
            let mut end = start;
            loop {
                match self.chars.peek() {
                    Some((i, c)) if c.is_ascii_whitespace() => {
                        end = *i;
                        self.skip_ws();
                        let mut chars = self.chars.clone();
                        if chars
                            .next_if(|(_, c)| *c == 'a')
                            .and_then(|_| chars.next_if(|(_, c)| *c == 's'))
                            .is_some()
                        {
                            self.chars = chars;
                            break;
                        }
                    }
                    Some((i, _)) => {
                        end = *i;
                        self.chars.next();
                    }
                    None => break,
                }
            }
            unsafe { self.source.get_unchecked(start..end) }
        };

        self.skip_ws();
        let binding = self.parse_svelte_binding()?;

        self.skip_ws();
        let index = if self.chars.next_if(|(_, c)| *c == ',').is_some() {
            self.skip_ws();
            Some(self.parse_identifier()?)
        } else {
            None
        };

        self.skip_ws();
        let key = if matches!(self.chars.peek(), Some((_, '('))) {
            Some(self.parse_inside('(', ')', false)?)
        } else {
            None
        };

        self.skip_ws();
        if self.chars.next_if(|(_, c)| *c == '}').is_none() {
            return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteEachBlock));
        }

        let children = self.parse_svelte_block_children()?;

        let else_children = if self
            .try_parse(|parser| {
                parser
                    .chars
                    .next_if(|(_, c)| *c == '{')
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == ':'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 'e'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 'l'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 's'))
                    .and_then(|_| parser.chars.next_if(|(_, c)| *c == 'e'))
                    .and_then(|_| {
                        parser.skip_ws();
                        parser.chars.next_if(|(_, c)| *c == '}')
                    })
                    .ok_or_else(|| parser.emit_error(SyntaxErrorKind::ExpectSvelteEachBlock))
            })
            .is_ok()
        {
            Some(self.parse_svelte_block_children()?)
        } else {
            None
        };

        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '/'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'e'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'c'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'h'))
            .map(|_| self.skip_ws())
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '}'))
            .is_some()
        {
            Ok(SvelteEachBlock {
                expr,
                binding,
                index,
                key,
                children,
                else_children,
            })
        } else {
            Err(self.emit_error(SyntaxErrorKind::ExpectSvelteBlockEnd))
        }
    }

    /// This will consume `}`.
    fn parse_svelte_expr(&mut self) -> PResult<&'s str> {
        self.skip_ws();

        let start = self
            .chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.source.len());
        let mut end = start;
        let mut braces_stack = 0u8;
        loop {
            match self.chars.next() {
                Some((_, '{')) => {
                    braces_stack += 1;
                }
                Some((i, '}')) => {
                    if braces_stack == 0 {
                        end = i;
                        break;
                    } else {
                        braces_stack -= 1;
                    }
                }
                Some(..) => continue,
                None => break,
            }
        }
        Ok(unsafe { self.source.get_unchecked(start..end) })
    }

    fn parse_svelte_if_block(&mut self) -> PResult<SvelteIfBlock<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '#'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'i'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'f'))
            .and_then(|_| self.chars.next_if(|(_, c)| c.is_ascii_whitespace()))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteIfBlock));
        };

        let expr = self.parse_svelte_expr()?;
        let children = self.parse_svelte_block_children()?;

        let mut else_if_blocks = vec![];
        let mut else_children = None;
        loop {
            if self.chars.next_if(|(_, c)| *c == '{').is_none() {
                return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteBlockEnd));
            }
            match self.chars.next() {
                Some((_, ':')) => {
                    if self
                        .chars
                        .next_if(|(_, c)| *c == 'e')
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 'l'))
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 's'))
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 'e'))
                        .is_none()
                    {
                        return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteElseIfBlock));
                    }
                    self.skip_ws();
                    match self.chars.next() {
                        Some((_, 'i')) => {
                            if self.chars.next_if(|(_, c)| *c == 'f').is_none() {
                                return Err(
                                    self.emit_error(SyntaxErrorKind::ExpectSvelteElseIfBlock)
                                );
                            }
                            let expr = self.parse_svelte_expr()?;
                            let children = self.parse_svelte_block_children()?;
                            else_if_blocks.push(SvelteElseIfBlock { expr, children });
                        }
                        Some((_, '}')) => {
                            else_children = Some(self.parse_svelte_block_children()?);
                        }
                        _ => return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteElseIfBlock)),
                    }
                }
                Some((_, '/')) => break,
                _ => return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteBlockEnd)),
            }
        }
        if self
            .chars
            .next_if(|(_, c)| *c == 'i')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'f'))
            .map(|_| self.skip_ws())
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '}'))
            .is_some()
        {
            Ok(SvelteIfBlock {
                expr,
                children,
                else_if_blocks,
                else_children,
            })
        } else {
            Err(self.emit_error(SyntaxErrorKind::ExpectSvelteBlockEnd))
        }
    }

    fn parse_svelte_interpolation(&mut self) -> PResult<SvelteInterpolation<'s>> {
        if self.chars.next_if(|(_, c)| *c == '{').is_none() {
            return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteInterpolation));
        };
        Ok(SvelteInterpolation {
            expr: self.parse_svelte_expr()?,
        })
    }

    fn parse_svelte_key_block(&mut self) -> PResult<SvelteKeyBlock<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '#'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'k'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'e'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'y'))
            .and_then(|_| self.chars.next_if(|(_, c)| c.is_ascii_whitespace()))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteKeyBlock));
        };

        let expr = self.parse_svelte_expr()?;
        let children = self.parse_svelte_block_children()?;

        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '/'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'k'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'e'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'y'))
            .map(|_| self.skip_ws())
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '}'))
            .is_some()
        {
            Ok(SvelteKeyBlock { expr, children })
        } else {
            Err(self.emit_error(SyntaxErrorKind::ExpectSvelteBlockEnd))
        }
    }

    fn parse_tag_name(&mut self) -> PResult<&'s str> {
        let Some((start, _)) = self.chars.next_if(|(_, c)| is_tag_name_char(*c)) else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectTagName));
        };
        let mut end = start;

        while let Some((i, _)) = self.chars.next_if(|(_, c)| is_tag_name_char(*c)) {
            end = i;
        }

        unsafe { Ok(self.source.get_unchecked(start..=end)) }
    }

    fn parse_text_node(&mut self) -> PResult<TextNode<'s>> {
        let Some((start, first_char)) = self.chars.next_if(|(_, c)| {
            if let Language::Vue | Language::Svelte = self.language {
                *c != '{'
            } else {
                true
            }
        }) else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectTextNode));
        };

        if matches!(self.language, Language::Vue)
            && first_char == '{'
            && matches!(self.chars.peek(), Some((_, '{')))
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectTextNode));
        }

        let mut line_breaks = if first_char == '\n' { 1 } else { 0 };
        let end;
        loop {
            match self.chars.peek() {
                Some((i, '{')) => match self.language {
                    Language::Vue => {
                        let i = *i;
                        let mut chars = self.chars.clone();
                        chars.next();
                        if chars.next_if(|(_, c)| *c == '{').is_some() {
                            end = i;
                            break;
                        } else {
                            self.chars.next();
                        }
                    }
                    Language::Svelte => {
                        end = *i;
                        break;
                    }
                    _ => {
                        self.chars.next();
                    }
                },
                Some((i, '<')) => {
                    let i = *i;
                    let mut chars = self.chars.clone();
                    chars.next();
                    match chars.next() {
                        Some((_, c)) if is_tag_name_char(c) || c == '/' || c == '!' => {
                            end = i;
                            break;
                        }
                        _ => {
                            self.chars.next();
                        }
                    }
                }
                Some((_, c)) => {
                    if *c == '\n' {
                        line_breaks += 1;
                    }
                    self.chars.next();
                }
                None => {
                    end = self.source.len();
                    break;
                }
            }
        }

        Ok(TextNode {
            raw: unsafe { self.source.get_unchecked(start..end) },
            line_breaks,
        })
    }

    fn parse_vue_directive(&mut self) -> PResult<VueDirective<'s>> {
        let name = match self.chars.peek() {
            Some((_, ':')) => {
                self.chars.next();
                ":"
            }
            Some((_, '@')) => {
                self.chars.next();
                "@"
            }
            Some((_, '#')) => {
                self.chars.next();
                "#"
            }
            Some((_, 'v')) => {
                let mut chars = self.chars.clone();
                chars.next();
                if chars.next_if(|(_, c)| *c == '-').is_some() {
                    self.chars = chars;
                    self.parse_identifier()?
                } else {
                    return Err(self.emit_error(SyntaxErrorKind::ExpectVueDirective));
                }
            }
            _ => return Err(self.emit_error(SyntaxErrorKind::ExpectVueDirective)),
        };

        let arg_and_modifiers = if matches!(name, ":" | "@" | "#")
            || self
                .chars
                .peek()
                .map(|(_, c)| is_attr_name_char(*c))
                .unwrap_or_default()
        {
            Some(self.parse_attr_name()?)
        } else {
            None
        };

        self.skip_ws();
        let value = if self.chars.next_if(|(_, c)| *c == '=').is_some() {
            self.skip_ws();
            Some(self.parse_attr_value()?)
        } else {
            None
        };

        Ok(VueDirective {
            name,
            arg_and_modifiers,
            value,
        })
    }

    fn parse_vue_interpolation(&mut self) -> PResult<VueInterpolation<'s>> {
        let Some((start, _)) = self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '{'))
        else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectVueInterpolation));
        };
        let start = start + 1;

        let mut end = start;
        loop {
            match self.chars.next() {
                Some((i, '}')) => {
                    if self.chars.next_if(|(_, c)| *c == '}').is_some() {
                        end = i;
                        break;
                    } else {
                        continue;
                    }
                }
                Some(..) => continue,
                None => break,
            }
        }

        Ok(VueInterpolation {
            expr: unsafe { self.source.get_unchecked(start..end) },
        })
    }
}

fn is_tag_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || c == '-'
        || c == '_'
        || c == '.'
        || c == ':'
        || !c.is_ascii()
        || c == '\\'
}

fn is_attr_name_char(c: char) -> bool {
    !matches!(c, '"' | '\'' | '>' | '/' | '=') && !c.is_ascii_whitespace()
}

pub type PResult<T> = Result<T, SyntaxError>;
