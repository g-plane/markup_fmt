use crate::ast::*;
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
        fn is_attr_name_char(c: char) -> bool {
            !matches!(c, ' ' | '"' | '\'' | '>' | '/' | '=')
        }

        let Some((start, _)) = self.chars.next_if(|(_, c)| is_attr_name_char(*c)) else {
            return Err(SyntaxError::ExpectAttrName);
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
                return Err(SyntaxError::ExpectAttrValue);
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
            return Err(SyntaxError::ExpectComment);
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

    fn parse_element(&mut self) -> PResult<Element<'s>> {
        let Some(..) = self.chars.next_if(|(_, c)| *c == '<') else {
            return Err(SyntaxError::ExpectElement);
        };
        let tag_name = self.parse_tag_name()?;

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
                        });
                    } else {
                        return Err(SyntaxError::ExpectSelfCloseTag);
                    }
                }
                Some((_, '>')) => {
                    self.chars.next();
                    break;
                }
                _ => {
                    attrs.push(self.parse_attr()?);
                }
            }
        }

        let mut children = vec![];
        loop {
            match self.chars.peek() {
                Some((_, '<')) => {
                    let mut chars = self.chars.clone();
                    chars.next();
                    if chars.next_if(|(_, c)| *c == '/').is_some() {
                        self.chars = chars;
                        let close_tag_name = self.parse_tag_name()?;
                        if !close_tag_name.eq_ignore_ascii_case(tag_name) {
                            return Err(SyntaxError::ExpectCloseTag);
                        }
                        self.skip_ws();
                        if self.chars.next_if(|(_, c)| *c == '>').is_some() {
                            break;
                        } else {
                            return Err(SyntaxError::ExpectCloseTag);
                        }
                    } else {
                        children.push(
                            if tag_name.eq_ignore_ascii_case("script")
                                || tag_name.eq_ignore_ascii_case("style")
                            {
                                self.parse_raw_text_node().map(Node::TextNode)?
                            } else {
                                self.parse_node()?
                            },
                        );
                    }
                }
                Some(..) => {
                    children.push(
                        if tag_name.eq_ignore_ascii_case("script")
                            || tag_name.eq_ignore_ascii_case("style")
                        {
                            self.parse_raw_text_node().map(Node::TextNode)?
                        } else {
                            self.parse_node()?
                        },
                    );
                }
                None => return Err(SyntaxError::ExpectCloseTag),
            }
        }

        Ok(Element {
            tag_name,
            attrs,
            children,
            self_closing: false,
        })
    }

    fn parse_identifier(&mut self) -> PResult<&'s str> {
        fn is_identifier_char(c: char) -> bool {
            c.is_ascii_alphanumeric() || c == '-' || c == '_' || !c.is_ascii() || c == '\\'
        }

        let Some((start, _)) = self.chars.next_if(|(_, c)| is_identifier_char(*c)) else {
            return Err(SyntaxError::ExpectIdentifier);
        };
        let mut end = start;

        while let Some((i, _)) = self.chars.next_if(|(_, c)| is_identifier_char(*c)) {
            end = i;
        }

        unsafe { Ok(self.source.get_unchecked(start..=end)) }
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
                if chars.next_if(|(_, c)| *c == '!').is_some() {
                    self.parse_comment().map(Node::Comment)
                } else {
                    self.parse_element().map(Node::Element)
                }
            }
            Some((_, '{')) => {
                let mut chars = self.chars.clone();
                chars.next();
                match chars.next() {
                    Some((_, '{')) if matches!(self.language, Language::Vue) => {
                        self.parse_vue_interpolation().map(Node::VueInterpolation)
                    }
                    Some((_, '#')) if matches!(self.language, Language::Svelte) => self
                        .try_parse(Parser::parse_svelte_if_block)
                        .map(Node::SvelteIfBlock)
                        .map_err(|_| SyntaxError::UnknownSvelteBlock),
                    _ if matches!(self.language, Language::Svelte) => self
                        .parse_svelte_interpolation()
                        .map(Node::SvelteInterpolation),
                    _ => self.parse_text_node().map(Node::TextNode),
                }
            }
            Some(..) => self.parse_text_node().map(Node::TextNode),
            None => Err(SyntaxError::ExpectElement),
        }
    }

    fn parse_raw_text_node(&mut self) -> PResult<TextNode<'s>> {
        let start = self
            .chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.source.len());

        let end;
        loop {
            match self.chars.peek() {
                Some((i, '<')) => {
                    let i = *i;
                    let mut chars = self.chars.clone();
                    chars.next();
                    if chars.next_if(|(_, c)| *c == '/').is_some() {
                        end = i;
                        break;
                    } else {
                        self.chars.next();
                    }
                }
                Some(..) => {
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
        })
    }

    pub fn parse_root(&mut self) -> PResult<Root<'s>> {
        let mut children = vec![];
        while self.chars.peek().is_some() {
            children.push(self.parse_node()?);
        }

        Ok(Root { children })
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
                None => return Err(SyntaxError::ExpectSvelteBlockEnd),
            }
        }
        Ok(children)
    }

    fn parse_svelte_attr(&mut self) -> PResult<SvelteAttribute<'s>> {
        let name = self.parse_attr_name()?;
        self.skip_ws();
        let Some((start, _)) = self
            .chars
            .next_if(|(_, c)| *c == '=')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '{'))
        else {
            return Err(SyntaxError::ExpectSvelteAttr);
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

    /// This will consume `}`.
    fn parse_svelte_expr(&mut self, start: usize) -> PResult<&'s str> {
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
        let Some((start, _)) = self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '#'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'i'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'f'))
            .and_then(|_| self.chars.next_if(|(_, c)| c.is_ascii_whitespace()))
        else {
            return Err(SyntaxError::ExpectSvelteIfBlock);
        };

        let expr = self.parse_svelte_expr(start)?;
        let children = self.parse_svelte_block_children()?;

        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '/'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'i'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'f'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '}'))
            .is_none()
        {
            Err(SyntaxError::ExpectSvelteBlockEnd)
        } else {
            Ok(SvelteIfBlock {
                expr,
                children,
                else_children: None,
            })
        }
    }

    fn parse_svelte_interpolation(&mut self) -> PResult<SvelteInterpolation<'s>> {
        let Some((i, _)) = self.chars.next_if(|(_, c)| *c == '{') else {
            return Err(SyntaxError::ExpectSvelteInterpolation);
        };
        Ok(SvelteInterpolation {
            expr: self.parse_svelte_expr(i + 1)?,
        })
    }

    fn parse_tag_name(&mut self) -> PResult<&'s str> {
        fn is_tag_name_char(c: char) -> bool {
            c.is_ascii_alphanumeric()
                || c == '-'
                || c == '_'
                || c == '.'
                || c == ':'
                || !c.is_ascii()
                || c == '\\'
        }

        let Some((start, _)) = self.chars.next_if(|(_, c)| is_tag_name_char(*c)) else {
            return Err(SyntaxError::ExpectTagName);
        };
        let mut end = start;

        while let Some((i, _)) = self.chars.next_if(|(_, c)| is_tag_name_char(*c)) {
            end = i;
        }

        unsafe { Ok(self.source.get_unchecked(start..=end)) }
    }

    fn parse_text_node(&mut self) -> PResult<TextNode<'s>> {
        let Some((start, first_char)) = self.chars.next_if(|(_, c)| {
            *c != '<' && (matches!(self.language, Language::Vue | Language::Svelte) && *c != '{')
        }) else {
            return Err(SyntaxError::ExpectTextNode);
        };

        if matches!(self.language, Language::Vue)
            && first_char == '{'
            && matches!(self.chars.peek(), Some((_, '{')))
        {
            return Err(SyntaxError::ExpectTextNode);
        }

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
                    end = i;
                    break;
                }
                Some(..) => {
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
                    return Err(SyntaxError::ExpectVueDirective);
                }
            }
            _ => return Err(SyntaxError::ExpectVueDirective),
        };

        let arg_and_modifiers = if matches!(name, ":" | "@" | "#")
            || self.chars.next_if(|(_, c)| *c == ':').is_some()
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
            return Err(SyntaxError::ExpectVueInterpolation);
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

pub type PResult<T> = Result<T, SyntaxError>;

#[derive(Clone, Debug)]
pub enum SyntaxError {
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
