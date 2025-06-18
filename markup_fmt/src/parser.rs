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
use std::{cmp::Ordering, iter::Peekable, ops::ControlFlow, str::CharIndices};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Supported languages.
pub enum Language {
    Html,
    Vue,
    Svelte,
    Astro,
    Angular,
    Jinja,
    Vento,
    Mustache,
    Xml,
}

pub struct Parser<'s> {
    source: &'s str,
    language: Language,
    chars: Peekable<CharIndices<'s>>,
    state: ParserState,
}

#[derive(Default)]
struct ParserState {
    has_front_matter: bool,
}

impl<'s> Parser<'s> {
    pub fn new(source: &'s str, language: Language) -> Self {
        Self {
            source,
            language,
            chars: source.char_indices().peekable(),
            state: Default::default(),
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
        let pos = self
            .chars
            .peek()
            .map(|(pos, _)| *pos)
            .unwrap_or(self.source.len());
        self.emit_error_with_pos(kind, pos)
    }

    fn emit_error_with_pos(&self, kind: SyntaxErrorKind, pos: usize) -> SyntaxError {
        let (line, column) = self.pos_to_line_col(pos);
        SyntaxError {
            kind,
            pos,
            line,
            column,
        }
    }
    fn pos_to_line_col(&self, pos: usize) -> (usize, usize) {
        let search = memchr::memchr_iter(b'\n', self.source.as_bytes()).try_fold(
            (1, 0),
            |(line, prev_offset), offset| match pos.cmp(&offset) {
                Ordering::Less => ControlFlow::Break((line, prev_offset)),
                Ordering::Equal => ControlFlow::Break((line, prev_offset)),
                Ordering::Greater => ControlFlow::Continue((line + 1, offset)),
            },
        );
        match search {
            ControlFlow::Break((line, offset)) => (line, pos - offset + 1),
            ControlFlow::Continue((line, _)) => (line, 0),
        }
    }

    fn skip_ws(&mut self) {
        while self
            .chars
            .next_if(|(_, c)| c.is_ascii_whitespace())
            .is_some()
        {}
    }

    fn with_taken<T, F>(&mut self, parser: F) -> PResult<(T, &'s str)>
    where
        F: FnOnce(&mut Self) -> PResult<T>,
    {
        let start = self
            .chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.source.len());
        let parsed = parser(self)?;
        let end = self
            .chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.source.len());
        Ok((parsed, unsafe { self.source.get_unchecked(start..end) }))
    }

    fn parse_angular_control_flow_children(&mut self) -> PResult<Vec<Node<'s>>> {
        if self.chars.next_if(|(_, c)| *c == '{').is_none() {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar('{')));
        }

        let mut children = vec![];
        while let Some((_, c)) = self.chars.peek() {
            if *c == '}' {
                self.chars.next();
                break;
            } else {
                children.push(self.parse_node()?);
            }
        }
        Ok(children)
    }

    fn parse_angular_for(&mut self) -> PResult<AngularFor<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '@')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'f'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'o'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'r'))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectAngularFor));
        }
        self.skip_ws();

        let Some((start, _)) = self.chars.next_if(|(_, c)| *c == '(') else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar('(')));
        };

        let (header, header_start) = self.parse_angular_inline_script(start + 1)?;
        let Some((binding, expr)) = header.split_once(" of ").map(|(binding, expr)| {
            (
                (binding.trim_end(), header_start),
                (expr.trim_start(), header_start + 4),
            )
        }) else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectKeyword("of")));
        };

        let mut track = None;
        if self.chars.next_if(|(_, c)| *c == ';').is_some() {
            self.skip_ws();
            if self
                .chars
                .next_if(|(_, c)| *c == 't')
                .and_then(|_| self.chars.next_if(|(_, c)| *c == 'r'))
                .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
                .and_then(|_| self.chars.next_if(|(_, c)| *c == 'c'))
                .and_then(|_| self.chars.next_if(|(_, c)| *c == 'k'))
                .is_some()
            {
                self.skip_ws();
                if let Some((start, _)) = self.chars.peek() {
                    let start = *start;
                    track = Some(self.parse_angular_inline_script(start)?);
                }
            }
        }

        let mut aliases = vec![];
        while self.chars.next_if(|(_, c)| *c == ';').is_some() {
            self.skip_ws();
            let mut chars = self.chars.clone();
            if chars
                .next_if(|(_, c)| *c == 'l')
                .and_then(|_| chars.next_if(|(_, c)| *c == 'e'))
                .and_then(|_| chars.next_if(|(_, c)| *c == 't'))
                .is_some()
            {
                if let Some((start, _)) = self.chars.peek() {
                    let start = *start;
                    aliases.push(self.parse_angular_inline_script(start)?);
                }
            }
        }

        self.chars.next_if(|(_, c)| *c == ';');
        self.skip_ws();
        if self.chars.next_if(|(_, c)| *c == ')').is_none() {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar(')')));
        };
        self.skip_ws();
        let children = self.parse_angular_control_flow_children()?;

        let mut empty = None;
        let mut chars = self.chars.clone();
        while chars.next_if(|(_, c)| c.is_ascii_whitespace()).is_some() {}
        if chars
            .next_if(|(_, c)| *c == '@')
            .and_then(|_| chars.next_if(|(_, c)| *c == 'e'))
            .and_then(|_| chars.next_if(|(_, c)| *c == 'm'))
            .and_then(|_| chars.next_if(|(_, c)| *c == 'p'))
            .and_then(|_| chars.next_if(|(_, c)| *c == 't'))
            .and_then(|_| chars.next_if(|(_, c)| *c == 'y'))
            .is_some()
        {
            self.chars = chars;
            self.skip_ws();
            empty = Some(self.parse_angular_control_flow_children()?);
        }

        Ok(AngularFor {
            binding,
            expr,
            track,
            aliases,
            children,
            empty,
        })
    }

    fn parse_angular_if(&mut self) -> PResult<AngularIf<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '@')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'i'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'f'))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectAngularIf));
        }
        self.skip_ws();

        let (expr, reference) = self.parse_angular_if_cond()?;
        self.skip_ws();
        let children = self.parse_angular_control_flow_children()?;

        let mut else_if_blocks = vec![];
        let mut else_children = None;
        'alter: loop {
            let mut chars = self.chars.clone();
            'peek: loop {
                match chars.next() {
                    Some((_, c)) if c.is_ascii_whitespace() => continue 'peek,
                    Some((_, '@')) => {
                        if chars
                            .next_if(|(_, c)| *c == 'e')
                            .and_then(|_| chars.next_if(|(_, c)| *c == 'l'))
                            .and_then(|_| chars.next_if(|(_, c)| *c == 's'))
                            .and_then(|_| chars.next_if(|(_, c)| *c == 'e'))
                            .is_some()
                        {
                            self.chars = chars;
                            break 'peek;
                        } else {
                            break 'alter;
                        }
                    }
                    _ => break 'alter,
                }
            }
            self.skip_ws();

            if self
                .chars
                .next_if(|(_, c)| *c == 'i')
                .and_then(|_| self.chars.next_if(|(_, c)| *c == 'f'))
                .is_some()
            {
                self.skip_ws();
                let (expr, reference) = self.parse_angular_if_cond()?;
                self.skip_ws();
                let children = self.parse_angular_control_flow_children()?;
                else_if_blocks.push(AngularElseIf {
                    expr,
                    reference,
                    children,
                });
            } else {
                else_children = Some(self.parse_angular_control_flow_children()?);
                break;
            }
        }

        Ok(AngularIf {
            expr,
            reference,
            children,
            else_if_blocks,
            else_children,
        })
    }

    fn parse_angular_if_cond(&mut self) -> PResult<AngularIfCond<'s>> {
        let Some((start, _)) = self.chars.next_if(|(_, c)| *c == '(') else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar('(')));
        };

        let expr = self.parse_angular_inline_script(start + 1)?;

        let mut reference = None;
        if self.chars.next_if(|(_, c)| *c == ';').is_some() {
            self.skip_ws();
            if self
                .chars
                .next_if(|(_, c)| *c == 'a')
                .and_then(|_| self.chars.next_if(|(_, c)| *c == 's'))
                .is_none()
            {
                return Err(self.emit_error(SyntaxErrorKind::ExpectKeyword("as")));
            }
            self.skip_ws();
            if let Some((start, _)) = self.chars.peek() {
                let start = *start;
                reference = Some(self.parse_angular_inline_script(start)?);
            }
        }

        if self.chars.next_if(|(_, c)| *c == ')').is_none() {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar(')')));
        }

        Ok((expr, reference))
    }

    fn parse_angular_inline_script(&mut self, start: usize) -> PResult<(&'s str, usize)> {
        let end;
        let mut chars_stack = vec![];
        loop {
            match self.chars.peek() {
                Some((_, c @ '\'' | c @ '"' | c @ '`')) => {
                    if chars_stack.last().is_some_and(|last| last == c) {
                        chars_stack.pop();
                    } else {
                        chars_stack.push(*c);
                    }
                    self.chars.next();
                }
                Some((_, '(')) => {
                    chars_stack.push('(');
                    self.chars.next();
                }
                Some((i, ')')) => {
                    if chars_stack.is_empty() {
                        end = *i;
                        break;
                    } else if chars_stack.last().is_some_and(|last| *last == '(') {
                        chars_stack.pop();
                        self.chars.next();
                    }
                }
                Some((i, ';')) if chars_stack.is_empty() => {
                    end = *i;
                    break;
                }
                Some(..) => {
                    self.chars.next();
                }
                None => {
                    end = start;
                    break;
                }
            }
        }
        Ok((unsafe { self.source.get_unchecked(start..end) }, start))
    }

    fn parse_angular_let(&mut self) -> PResult<AngularLet<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '@')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'l'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'e'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 't'))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectAngularLet));
        }
        self.skip_ws();

        let name = self.parse_identifier()?;
        self.skip_ws();
        if self.chars.next_if(|(_, c)| *c == '=').is_none() {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar('=')));
        }
        self.skip_ws();
        let start = self
            .chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.source.len());
        let expr = self.parse_angular_inline_script(start)?;
        if self.chars.next_if(|(_, c)| *c == ';').is_none() {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar(';')));
        }

        Ok(AngularLet { name, expr })
    }

    fn parse_angular_switch(&mut self) -> PResult<AngularSwitch<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '@')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 's'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'w'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'i'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 't'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'c'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'h'))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectAngularSwitch));
        }
        self.skip_ws();

        let Some((start, _)) = self.chars.next_if(|(_, c)| *c == '(') else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar('(')));
        };
        let expr = self.parse_angular_inline_script(start + 1)?;
        if self.chars.next_if(|(_, c)| *c == ')').is_none() {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar(')')));
        }

        self.skip_ws();
        if self.chars.next_if(|(_, c)| *c == '{').is_none() {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar('{')));
        }
        self.skip_ws();

        let mut arms = Vec::with_capacity(2);
        while let Some((_, '@')) = self.chars.peek() {
            self.chars.next();
            match self.chars.peek() {
                Some((_, 'c')) => {
                    if self
                        .chars
                        .next_if(|(_, c)| *c == 'c')
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 's'))
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 'e'))
                        .is_none()
                    {
                        return Err(self.emit_error(SyntaxErrorKind::ExpectKeyword("case")));
                    }
                    self.skip_ws();
                    let Some((start, _)) = self.chars.next_if(|(_, c)| *c == '(') else {
                        return Err(self.emit_error(SyntaxErrorKind::ExpectChar('(')));
                    };
                    let expr = self.parse_angular_inline_script(start + 1)?;
                    if self.chars.next_if(|(_, c)| *c == ')').is_none() {
                        return Err(self.emit_error(SyntaxErrorKind::ExpectChar(')')));
                    }
                    self.skip_ws();
                    let children = self.parse_angular_control_flow_children()?;
                    arms.push(AngularSwitchArm {
                        keyword: "case",
                        expr: Some(expr),
                        children,
                    });
                    self.skip_ws();
                }
                Some((_, 'd')) => {
                    if self
                        .chars
                        .next_if(|(_, c)| *c == 'd')
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 'e'))
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 'f'))
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 'u'))
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 'l'))
                        .and_then(|_| self.chars.next_if(|(_, c)| *c == 't'))
                        .is_none()
                    {
                        return Err(self.emit_error(SyntaxErrorKind::ExpectKeyword("default")));
                    }
                    self.skip_ws();
                    arms.push(AngularSwitchArm {
                        keyword: "default",
                        expr: None,
                        children: self.parse_angular_control_flow_children()?,
                    });
                    self.skip_ws();
                }
                _ => return Err(self.emit_error(SyntaxErrorKind::ExpectKeyword("case"))),
            }
        }

        self.skip_ws();
        if self.chars.next_if(|(_, c)| *c == '}').is_none() {
            return Err(self.emit_error(SyntaxErrorKind::ExpectChar('}')));
        }

        Ok(AngularSwitch { expr, arms })
    }

    fn parse_astro_attr(&mut self) -> PResult<AstroAttribute<'s>> {
        let name = if self.chars.next_if(|(_, c)| *c == '{').is_some() {
            None
        } else {
            let name = self.parse_attr_name()?;
            self.skip_ws();
            if self
                .chars
                .next_if(|(_, c)| *c == '=')
                .map(|_| self.skip_ws())
                .and_then(|_| self.chars.next_if(|(_, c)| *c == '{'))
                .is_some()
            {
                Some(name)
            } else {
                return Err(self.emit_error(SyntaxErrorKind::ExpectAstroAttr));
            }
        };

        self.parse_svelte_or_astro_expr()
            .map(|expr| AstroAttribute { name, expr })
    }

    fn parse_astro_expr(&mut self) -> PResult<AstroExpr<'s>> {
        let Some((start, _)) = self.chars.next_if(|(_, c)| *c == '{') else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectAstroExpr));
        };

        let mut children = Vec::with_capacity(1);
        let mut has_line_comment = false;
        let mut pair_stack = vec![];
        let mut pos = self
            .chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.source.len());
        while let Some((i, c)) = self.chars.peek() {
            match c {
                '{' => {
                    pair_stack.push('{');
                    self.chars.next();
                }
                '}' => {
                    let i = *i;
                    self.chars.next();
                    if pair_stack.is_empty() {
                        debug_assert!(matches!(
                            children.last(),
                            Some(AstroExprChild::Template(..)) | None
                        ));
                        children.push(AstroExprChild::Script(unsafe {
                            self.source.get_unchecked(pos..i)
                        }));
                        break;
                    }
                    pair_stack.pop();
                }
                '<' if !matches!(pair_stack.last(), Some('/' | '*' | '\'' | '"' | '`')) => {
                    let i = *i;
                    let mut chars = self.chars.clone();
                    chars.next();
                    if chars
                        .next_if(|(_, c)| is_html_tag_name_char(*c) || *c == '!' || *c == '>')
                        .is_some()
                    {
                        let prev = unsafe { self.source.get_unchecked(pos..i) };
                        if prev.is_empty() {
                            // do nothing
                        } else if prev.chars().all(|c| c.is_ascii_whitespace()) {
                            if let Some(AstroExprChild::Template(nodes)) = children.last_mut() {
                                nodes.push(Node {
                                    kind: NodeKind::Text(TextNode {
                                        raw: prev,
                                        line_breaks: prev.chars().filter(|c| *c == '\n').count(),
                                        start: pos,
                                    }),
                                    raw: prev,
                                });
                            }
                        } else {
                            children.push(AstroExprChild::Script(prev));
                        }

                        let node = self.parse_node()?;
                        if let Some(AstroExprChild::Template(nodes)) = children.last_mut() {
                            nodes.push(node);
                        } else {
                            debug_assert!(matches!(
                                children.last(),
                                Some(AstroExprChild::Script(..)) | None
                            ));
                            children.push(AstroExprChild::Template(vec![node]));
                        }
                        pos = self
                            .chars
                            .peek()
                            .map(|(i, _)| *i)
                            .unwrap_or(self.source.len());
                    } else {
                        self.chars.next();
                    }
                }
                '\'' | '"' | '`' => {
                    let last = pair_stack.last();
                    if last.is_some_and(|last| last == c) {
                        pair_stack.pop();
                    } else if matches!(last, Some('$' | '{') | None) {
                        pair_stack.push(*c);
                    }
                    self.chars.next();
                }
                '$' if matches!(pair_stack.last(), Some('`')) => {
                    self.chars.next();
                    if self.chars.next_if(|(_, c)| *c == '{').is_some() {
                        pair_stack.push('$');
                    }
                }
                '/' if !matches!(pair_stack.last(), Some('\'' | '"' | '`' | '/' | '*')) => {
                    self.chars.next();
                    match self.chars.peek() {
                        Some((_, '/')) => {
                            pair_stack.push('/');
                            has_line_comment = true;
                            self.chars.next();
                        }
                        Some((_, '*')) => {
                            pair_stack.push('*');
                            self.chars.next();
                        }
                        _ => {}
                    }
                }
                '\n' => {
                    self.chars.next();
                    if let Some('/') = pair_stack.last() {
                        pair_stack.pop();
                    }
                }
                '*' => {
                    self.chars.next();
                    if self
                        .chars
                        .next_if(|(_, c)| *c == '/' && matches!(pair_stack.last(), Some('*')))
                        .is_some()
                    {
                        pair_stack.pop();
                    }
                }
                '\\' if matches!(pair_stack.last(), Some('\'' | '"' | '`')) => {
                    self.chars.next();
                }
                _ => {
                    self.chars.next();
                }
            }
        }

        Ok(AstroExpr {
            children,
            has_line_comment,
            start: start + 1,
        })
    }

    fn parse_attr(&mut self) -> PResult<Attribute<'s>> {
        match self.language {
            Language::Html | Language::Angular | Language::Mustache | Language::Xml => {
                self.parse_native_attr().map(Attribute::Native)
            }
            Language::Vue => self
                .try_parse(Parser::parse_vue_directive)
                .map(Attribute::VueDirective)
                .or_else(|_| self.parse_native_attr().map(Attribute::Native)),
            Language::Svelte => self
                .try_parse(Parser::parse_svelte_attachment)
                .map(Attribute::SvelteAttachment)
                .or_else(|_| {
                    self.try_parse(Parser::parse_svelte_attr)
                        .map(Attribute::Svelte)
                })
                .or_else(|_| self.parse_native_attr().map(Attribute::Native)),
            Language::Astro => self
                .try_parse(Parser::parse_astro_attr)
                .map(Attribute::Astro)
                .or_else(|_| self.parse_native_attr().map(Attribute::Native)),
            Language::Jinja => {
                self.skip_ws();
                let result = if matches!(self.chars.peek(), Some((_, '{'))) {
                    let mut chars = self.chars.clone();
                    chars.next();
                    match chars.next() {
                        Some((_, '{')) => self.parse_native_attr().map(Attribute::Native),
                        Some((_, '#')) => self.parse_jinja_comment().map(Attribute::JinjaComment),
                        _ => self.parse_jinja_tag_or_block(None, &mut Parser::parse_attr),
                    }
                } else {
                    self.parse_native_attr().map(Attribute::Native)
                };
                if result.is_ok() {
                    self.skip_ws();
                }
                result
            }
            Language::Vento => self
                .try_parse(|parser| parser.parse_vento_tag_or_block(None))
                .map(Attribute::VentoTagOrBlock)
                .or_else(|_| self.parse_native_attr().map(Attribute::Native)),
        }
    }

    fn parse_attr_name(&mut self) -> PResult<&'s str> {
        if matches!(
            self.language,
            Language::Jinja | Language::Vento | Language::Mustache
        ) {
            let Some((start, mut end)) = (match self.chars.peek() {
                Some((i, '{')) => {
                    let start = *i;
                    let mut chars = self.chars.clone();
                    chars.next();
                    if let Some((_, '{')) = chars.next() {
                        let end =
                            start + self.parse_mustache_interpolation()?.0.len() + "{{}}".len();
                        Some((start, end))
                    } else {
                        None
                    }
                }
                Some((_, c)) if is_attr_name_char(*c) => self
                    .chars
                    .next()
                    .map(|(start, c)| (start, start + c.len_utf8())),
                _ => None,
            }) else {
                return Err(self.emit_error(SyntaxErrorKind::ExpectAttrName));
            };

            while let Some((_, c)) = self.chars.peek() {
                if is_attr_name_char(*c) && *c != '{' {
                    end += c.len_utf8();
                    self.chars.next();
                } else if *c == '{' {
                    let mut chars = self.chars.clone();
                    chars.next();
                    match chars.next() {
                        Some((_, '%')) => {
                            break;
                        }
                        Some((_, '{')) => {
                            end += self.parse_mustache_interpolation()?.0.len() + "{{}}".len();
                        }
                        Some((_, c)) => {
                            end += c.len_utf8();
                            self.chars.next();
                        }
                        None => break,
                    }
                } else {
                    break;
                }
            }

            unsafe { Ok(self.source.get_unchecked(start..end)) }
        } else {
            let Some((start, start_char)) = self.chars.next_if(|(_, c)| is_attr_name_char(*c))
            else {
                return Err(self.emit_error(SyntaxErrorKind::ExpectAttrName));
            };
            let mut end = start + start_char.len_utf8();

            while let Some((_, c)) = self.chars.next_if(|(_, c)| is_attr_name_char(*c)) {
                end += c.len_utf8();
            }

            unsafe { Ok(self.source.get_unchecked(start..end)) }
        }
    }

    fn parse_attr_value(&mut self) -> PResult<(&'s str, usize)> {
        let quote = self.chars.next_if(|(_, c)| *c == '"' || *c == '\'');

        if let Some((start, quote)) = quote {
            let can_interpolate = matches!(
                self.language,
                Language::Jinja | Language::Vento | Language::Mustache
            );
            let start = start + 1;
            let mut end = start;
            let mut chars_stack = vec![];
            loop {
                match self.chars.next() {
                    Some((i, c)) if c == quote => {
                        if chars_stack.is_empty() || !can_interpolate {
                            end = i;
                            break;
                        } else if chars_stack.last().is_some_and(|last| *last == c) {
                            chars_stack.pop();
                        } else {
                            chars_stack.push(c);
                        }
                    }
                    Some((_, '{')) if can_interpolate => {
                        chars_stack.push('{');
                    }
                    Some((_, '}'))
                        if can_interpolate
                            && chars_stack.last().is_some_and(|last| *last == '{') =>
                    {
                        chars_stack.pop();
                    }
                    Some(..) => continue,
                    None => break,
                }
            }
            Ok((unsafe { self.source.get_unchecked(start..end) }, start))
        } else {
            fn is_unquoted_attr_value_char(c: char) -> bool {
                !c.is_ascii_whitespace() && !matches!(c, '"' | '\'' | '=' | '<' | '>' | '`')
            }

            let start = match self.chars.peek() {
                Some((i, c)) if is_unquoted_attr_value_char(*c) => *i,
                _ => return Err(self.emit_error(SyntaxErrorKind::ExpectAttrValue)),
            };

            let mut end = start;
            loop {
                match self.chars.peek() {
                    Some((i, '{'))
                        if matches!(
                            self.language,
                            Language::Jinja | Language::Vento | Language::Mustache
                        ) =>
                    {
                        end = *i;
                        let mut chars = self.chars.clone();
                        chars.next();
                        match chars.peek() {
                            Some((_, '%')) => {
                                if self
                                    .parse_jinja_tag_or_block(None, &mut Parser::parse_node)
                                    .is_ok()
                                {
                                    end =
                                        self.chars.peek().map(|(i, _)| i - 1).ok_or_else(|| {
                                            self.emit_error(SyntaxErrorKind::ExpectAttrValue)
                                        })?;
                                } else {
                                    self.chars.next();
                                }
                            }
                            Some((_, '{')) => {
                                chars.next();
                                // We use inclusive range when returning string,
                                // so we need to substract 1 here.
                                let (interpolation, _) = self.parse_mustache_interpolation()?;
                                end += interpolation.len() + "{{}}".len() - 1;
                            }
                            _ => {
                                self.chars.next();
                            }
                        }
                    }
                    Some((i, c)) if is_unquoted_attr_value_char(*c) => {
                        end = *i;
                        self.chars.next();
                    }
                    _ => break,
                }
            }

            Ok((unsafe { self.source.get_unchecked(start..=end) }, start))
        }
    }

    fn parse_cdata(&mut self) -> PResult<Cdata<'s>> {
        let Some((start, _)) = self
            .chars
            .next_if(|(_, c)| *c == '<')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '!'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '['))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'C'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'D'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'A'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'T'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'A'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '['))
        else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectCdata));
        };
        let start = start + 1;

        let mut end = start;
        loop {
            match self.chars.next() {
                Some((i, ']')) => {
                    let mut chars = self.chars.clone();
                    if chars
                        .next_if(|(_, c)| *c == ']')
                        .and_then(|_| chars.next_if(|(_, c)| *c == '>'))
                        .is_some()
                    {
                        end = i;
                        self.chars = chars;
                        break;
                    }
                }
                Some(..) => continue,
                None => break,
            }
        }

        Ok(Cdata {
            raw: unsafe { self.source.get_unchecked(start..end) },
        })
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

    fn parse_doctype(&mut self) -> PResult<Doctype<'s>> {
        let keyword_start = if let Some((start, _)) = self
            .chars
            .next_if(|(_, c)| *c == '<')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '!'))
        {
            start + 1
        } else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectDoctype));
        };
        let keyword = if let Some((end, _)) = self
            .chars
            .next_if(|(_, c)| c.eq_ignore_ascii_case(&'d'))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'o')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'c')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'t')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'y')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'p')))
            .and_then(|_| self.chars.next_if(|(_, c)| c.eq_ignore_ascii_case(&'e')))
        {
            unsafe { self.source.get_unchecked(keyword_start..end + 1) }
        } else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectDoctype));
        };
        self.skip_ws();

        let value_start = if let Some((start, _)) = self.chars.peek() {
            *start
        } else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectDoctype));
        };
        while self.chars.next_if(|(_, c)| *c != '>').is_some() {}

        if let Some((value_end, _)) = self.chars.next_if(|(_, c)| *c == '>') {
            Ok(Doctype {
                keyword,
                value: unsafe { self.source.get_unchecked(value_start..value_end) }.trim_end(),
            })
        } else {
            Err(self.emit_error(SyntaxErrorKind::ExpectDoctype))
        }
    }

    fn parse_element(&mut self) -> PResult<Element<'s>> {
        let Some((element_start, _)) = self.chars.next_if(|(_, c)| *c == '<') else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectElement));
        };
        let tag_name = self.parse_tag_name()?;
        let void_element = helpers::is_void_element(tag_name, self.language);

        let mut attrs = vec![];
        let mut first_attr_same_line = true;
        loop {
            match self.chars.peek() {
                Some((_, '/')) => {
                    self.chars.next();
                    if self.chars.next_if(|(_, c)| *c == '>').is_some() {
                        return Ok(Element {
                            tag_name,
                            attrs,
                            first_attr_same_line,
                            children: vec![],
                            self_closing: true,
                            void_element,
                        });
                    }
                    return Err(self.emit_error(SyntaxErrorKind::ExpectSelfCloseTag));
                }
                Some((_, '>')) => {
                    self.chars.next();
                    if void_element {
                        return Ok(Element {
                            tag_name,
                            attrs,
                            first_attr_same_line,
                            children: vec![],
                            self_closing: false,
                            void_element,
                        });
                    }
                    break;
                }
                Some((_, '\n')) => {
                    if attrs.is_empty() {
                        first_attr_same_line = false;
                    }
                    self.chars.next();
                }
                Some((_, c)) if c.is_ascii_whitespace() => {
                    self.chars.next();
                }
                _ => {
                    attrs.push(self.parse_attr()?);
                }
            }
        }

        let mut children = vec![];
        let should_parse_raw = self.language != Language::Xml
            && (tag_name.eq_ignore_ascii_case("script")
                || tag_name.eq_ignore_ascii_case("style")
                || tag_name.eq_ignore_ascii_case("pre")
                || tag_name.eq_ignore_ascii_case("textarea"));
        if should_parse_raw {
            let text_node = self.parse_raw_text_node(tag_name)?;
            let raw = text_node.raw;
            if !raw.is_empty() {
                children.push(Node {
                    kind: NodeKind::Text(text_node),
                    raw,
                });
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
                            let (line, column) = self.pos_to_line_col(element_start);
                            return Err(self.emit_error_with_pos(
                                SyntaxErrorKind::ExpectCloseTag {
                                    tag_name: tag_name.into(),
                                    line,
                                    column,
                                },
                                pos,
                            ));
                        }
                        self.skip_ws();
                        if self.chars.next_if(|(_, c)| *c == '>').is_some() {
                            break;
                        }
                        let (line, column) = self.pos_to_line_col(element_start);
                        return Err(self.emit_error(SyntaxErrorKind::ExpectCloseTag {
                            tag_name: tag_name.into(),
                            line,
                            column,
                        }));
                    }
                    children.push(self.parse_node()?);
                }
                Some(..) => {
                    if should_parse_raw {
                        let text_node = self.parse_raw_text_node(tag_name)?;
                        let raw = text_node.raw;
                        if !raw.is_empty() {
                            children.push(Node {
                                kind: NodeKind::Text(text_node),
                                raw,
                            });
                        }
                    } else {
                        children.push(self.parse_node()?);
                    }
                }
                None => {
                    let (line, column) = self.pos_to_line_col(element_start);
                    return Err(self.emit_error(SyntaxErrorKind::ExpectCloseTag {
                        tag_name: tag_name.into(),
                        line,
                        column,
                    }));
                }
            }
        }

        Ok(Element {
            tag_name,
            attrs,
            first_attr_same_line,
            children,
            self_closing: false,
            void_element,
        })
    }

    fn parse_front_matter(&mut self) -> PResult<FrontMatter<'s>> {
        let Some((start, _)) = self
            .chars
            .next_if(|(_, c)| *c == '-')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '-'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '-'))
        else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectFrontMatter));
        };
        let start = start + 1;

        let mut pair_stack = vec![];
        let mut end = start;
        loop {
            match self.chars.next() {
                Some((i, '-')) if pair_stack.is_empty() => {
                    let mut chars = self.chars.clone();
                    if chars
                        .next_if(|(_, c)| *c == '-')
                        .and_then(|_| chars.next_if(|(_, c)| *c == '-'))
                        .is_some()
                    {
                        end = i;
                        self.chars = chars;
                        break;
                    }
                }
                Some((_, c @ '\'' | c @ '"' | c @ '`')) => {
                    let last = pair_stack.last();
                    if last.is_some_and(|last| *last == c) {
                        pair_stack.pop();
                    } else if matches!(last, Some('$' | '{') | None) {
                        pair_stack.push(c);
                    }
                }
                Some((_, '$')) if matches!(pair_stack.last(), Some('`')) => {
                    if self.chars.next_if(|(_, c)| *c == '{').is_some() {
                        pair_stack.push('$');
                    }
                }
                Some((_, '{')) if matches!(pair_stack.last(), Some('$' | '{') | None) => {
                    pair_stack.push('{');
                }
                Some((_, '}')) if matches!(pair_stack.last(), Some('$' | '{')) => {
                    pair_stack.pop();
                }
                Some((_, '/'))
                    if !matches!(pair_stack.last(), Some('\'' | '"' | '`' | '/' | '*')) =>
                {
                    if let Some((_, c)) = self.chars.next_if(|(_, c)| *c == '/' || *c == '*') {
                        pair_stack.push(c);
                    }
                }
                Some((_, '\n')) => {
                    if let Some('/') = pair_stack.last() {
                        pair_stack.pop();
                    }
                }
                Some((_, '*')) => {
                    if self
                        .chars
                        .next_if(|(_, c)| *c == '/' && matches!(pair_stack.last(), Some('*')))
                        .is_some()
                    {
                        pair_stack.pop();
                    }
                }
                Some((_, '\\')) if matches!(pair_stack.last(), Some('\'' | '"' | '`')) => {
                    self.chars.next();
                }
                Some(..) => continue,
                None => break,
            }
        }

        self.state.has_front_matter = true;
        Ok(FrontMatter {
            raw: unsafe { self.source.get_unchecked(start..end) },
            start,
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
                }
                stack -= 1;
            }
        }
        Ok(unsafe { self.source.get_unchecked(start..end) })
    }

    fn parse_jinja_block_children<T, F>(&mut self, children_parser: &mut F) -> PResult<Vec<T>>
    where
        T: HasJinjaFlowControl<'s>,
        F: FnMut(&mut Self) -> PResult<T>,
    {
        let mut children = vec![];
        loop {
            match self.chars.peek() {
                Some((_, '{')) => {
                    let mut chars = self.chars.clone();
                    chars.next();
                    if chars.next_if(|(_, c)| *c == '%').is_some() {
                        break;
                    }
                    children.push(children_parser(self)?);
                }
                Some(..) => {
                    children.push(children_parser(self)?);
                }
                None => return Err(self.emit_error(SyntaxErrorKind::ExpectJinjaBlockEnd)),
            }
        }
        Ok(children)
    }

    fn parse_jinja_comment(&mut self) -> PResult<JinjaComment<'s>> {
        let Some((start, _)) = self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '#'))
        else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectComment));
        };
        let start = start + 1;

        let mut end = start;
        loop {
            match self.chars.next() {
                Some((i, '#')) => {
                    let mut chars = self.chars.clone();
                    if chars.next_if(|(_, c)| *c == '}').is_some() {
                        end = i;
                        self.chars = chars;
                        break;
                    }
                }
                Some(..) => continue,
                None => break,
            }
        }

        Ok(JinjaComment {
            raw: unsafe { self.source.get_unchecked(start..end) },
        })
    }

    fn parse_jinja_tag(&mut self) -> PResult<JinjaTag<'s>> {
        let Some((start, _)) = self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '%'))
        else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectJinjaTag));
        };
        let start = start + 1;

        let mut end = start;
        loop {
            match self.chars.next() {
                Some((i, '%')) => {
                    if self.chars.next_if(|(_, c)| *c == '}').is_some() {
                        end = i;
                        break;
                    }
                }
                Some(..) => continue,
                None => break,
            }
        }

        Ok(JinjaTag {
            content: unsafe { self.source.get_unchecked(start..end) },
        })
    }

    fn parse_jinja_tag_or_block<T, F>(
        &mut self,
        first_tag: Option<JinjaTag<'s>>,
        children_parser: &mut F,
    ) -> PResult<T::Intermediate>
    where
        T: HasJinjaFlowControl<'s>,
        F: FnMut(&mut Self) -> PResult<T>,
    {
        let first_tag = if let Some(first_tag) = first_tag {
            first_tag
        } else {
            self.parse_jinja_tag()?
        };
        let tag_name = parse_jinja_tag_name(&first_tag);

        if matches!(
            tag_name,
            "for"
                | "if"
                | "macro"
                | "call"
                | "filter"
                | "block"
                | "apply"
                | "autoescape"
                | "embed"
                | "with"
                | "trans"
                | "raw"
        ) || tag_name == "set" && !first_tag.content.contains('=')
        {
            let mut body = vec![JinjaTagOrChildren::Tag(first_tag)];

            loop {
                let mut children = self.parse_jinja_block_children(children_parser)?;
                if !children.is_empty() {
                    if let Some(JinjaTagOrChildren::Children(nodes)) = body.last_mut() {
                        nodes.append(&mut children);
                    } else {
                        body.push(JinjaTagOrChildren::Children(children));
                    }
                }
                if let Ok(next_tag) = self.parse_jinja_tag() {
                    let next_tag_name = parse_jinja_tag_name(&next_tag);
                    if next_tag_name
                        .strip_prefix("end")
                        .map(|name| name == tag_name)
                        .unwrap_or_default()
                    {
                        body.push(JinjaTagOrChildren::Tag(next_tag));
                        break;
                    }
                    if (tag_name == "if" || tag_name == "for")
                        && matches!(next_tag_name, "elif" | "elseif" | "else")
                    {
                        body.push(JinjaTagOrChildren::Tag(next_tag));
                    } else if let Some(JinjaTagOrChildren::Children(nodes)) = body.last_mut() {
                        nodes.push(
                            self.with_taken(|parser| {
                                parser.parse_jinja_tag_or_block(Some(next_tag), children_parser)
                            })
                            .map(|(kind, raw)| T::build(kind, raw))?,
                        );
                    } else {
                        body.push(JinjaTagOrChildren::Children(vec![self
                            .with_taken(|parser| {
                                parser.parse_jinja_tag_or_block(Some(next_tag), children_parser)
                            })
                            .map(|(kind, raw)| T::build(kind, raw))?]));
                    }
                } else {
                    break;
                }
            }
            Ok(T::from_block(JinjaBlock { body }))
        } else {
            Ok(T::from_tag(first_tag))
        }
    }

    fn parse_mustache_block_or_interpolation(&mut self) -> PResult<NodeKind<'s>> {
        let (content, _) = self.parse_mustache_interpolation()?;
        if let Some((prefix, rest)) = content
            .split_at_checked(1)
            .filter(|(c, _)| matches!(*c, "#" | "^" | "$" | "<"))
        {
            let trimmed_rest = rest.trim_ascii();
            let mut children = vec![];
            loop {
                let chars = self.chars.clone();
                if self
                    .parse_mustache_interpolation()
                    .ok()
                    .and_then(|(content, _)| content.strip_prefix('/'))
                    .is_some_and(|s| s.trim_ascii() == trimmed_rest)
                {
                    break;
                } else {
                    self.chars = chars;
                }
                children.push(self.parse_node()?);
            }
            Ok(NodeKind::MustacheBlock(MustacheBlock {
                prefix,
                content: rest,
                children,
            }))
        } else {
            Ok(NodeKind::MustacheInterpolation(MustacheInterpolation {
                content,
            }))
        }
    }

    fn parse_mustache_interpolation(&mut self) -> PResult<(&'s str, usize)> {
        let Some((start, _)) = self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '{'))
        else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectMustacheInterpolation));
        };
        let start = start + 1;

        let mut braces_stack = 0usize;
        let mut end = start;
        loop {
            match self.chars.next() {
                Some((_, '{')) => braces_stack += 1,
                Some((i, '}')) => {
                    if braces_stack == 0 {
                        if self.chars.next_if(|(_, c)| *c == '}').is_some() {
                            end = i;
                            break;
                        }
                    } else {
                        braces_stack -= 1;
                    }
                }
                Some(..) => continue,
                None => break,
            }
        }

        Ok((unsafe { self.source.get_unchecked(start..end) }, start))
    }

    fn parse_native_attr(&mut self) -> PResult<NativeAttribute<'s>> {
        let name = self.parse_attr_name()?;
        self.skip_ws();
        let mut quote = None;
        let value = if self.chars.next_if(|(_, c)| *c == '=').is_some() {
            self.skip_ws();
            quote = self
                .chars
                .peek()
                .and_then(|(_, c)| (*c == '\'' || *c == '"').then_some(*c));
            Some(self.parse_attr_value()?)
        } else {
            None
        };
        Ok(NativeAttribute { name, value, quote })
    }

    fn parse_node(&mut self) -> PResult<Node<'s>> {
        let (kind, raw) = self.with_taken(Parser::parse_node_kind)?;
        Ok(Node { kind, raw })
    }

    fn parse_node_kind(&mut self) -> PResult<NodeKind<'s>> {
        match self.chars.peek() {
            Some((_, '<')) => {
                let mut chars = self.chars.clone();
                chars.next();
                match chars.next() {
                    Some((_, c))
                        if is_html_tag_name_char(c)
                            || is_special_tag_name_char(c, self.language) =>
                    {
                        self.parse_element().map(NodeKind::Element)
                    }
                    Some((_, '!')) => {
                        if matches!(
                            self.language,
                            Language::Html
                                | Language::Astro
                                | Language::Jinja
                                | Language::Vento
                                | Language::Mustache
                                | Language::Xml
                        ) {
                            self.try_parse(Parser::parse_comment)
                                .map(NodeKind::Comment)
                                .or_else(|_| {
                                    self.try_parse(Parser::parse_doctype).map(NodeKind::Doctype)
                                })
                                .or_else(|_| {
                                    self.try_parse(Parser::parse_cdata).map(NodeKind::Cdata)
                                })
                                .or_else(|_| self.parse_text_node().map(NodeKind::Text))
                        } else {
                            self.parse_comment().map(NodeKind::Comment)
                        }
                    }
                    Some((_, '?')) if self.language == Language::Xml => {
                        self.parse_xml_decl().map(NodeKind::XmlDecl)
                    }
                    _ => self.parse_text_node().map(NodeKind::Text),
                }
            }
            Some((_, '{')) => {
                let mut chars = self.chars.clone();
                chars.next();
                match chars.next() {
                    Some((_, '{'))
                        if matches!(
                            self.language,
                            Language::Vue | Language::Jinja | Language::Angular
                        ) =>
                    {
                        self.parse_mustache_interpolation().map(|(expr, start)| {
                            match self.language {
                                Language::Vue => {
                                    NodeKind::VueInterpolation(VueInterpolation { expr, start })
                                }
                                Language::Jinja => {
                                    NodeKind::JinjaInterpolation(JinjaInterpolation { expr })
                                }
                                Language::Angular => {
                                    NodeKind::AngularInterpolation(AngularInterpolation {
                                        expr,
                                        start,
                                    })
                                }
                                _ => unreachable!(),
                            }
                        })
                    }
                    Some((_, '{')) if matches!(self.language, Language::Vento) => {
                        self.parse_vento_tag_or_block(None)
                    }
                    Some((_, '{')) if matches!(self.language, Language::Mustache) => {
                        self.parse_mustache_block_or_interpolation()
                    }
                    Some((_, '#')) if matches!(self.language, Language::Svelte) => {
                        match chars.next() {
                            Some((_, 'i')) => {
                                self.parse_svelte_if_block().map(NodeKind::SvelteIfBlock)
                            }
                            Some((_, 'e')) => self
                                .parse_svelte_each_block()
                                .map(NodeKind::SvelteEachBlock),
                            Some((_, 'a')) => self
                                .parse_svelte_await_block()
                                .map(NodeKind::SvelteAwaitBlock),
                            Some((_, 'k')) => {
                                self.parse_svelte_key_block().map(NodeKind::SvelteKeyBlock)
                            }
                            Some((_, 's')) => self
                                .parse_svelte_snippet_block()
                                .map(NodeKind::SvelteSnippetBlock),
                            _ => self.parse_text_node().map(NodeKind::Text),
                        }
                    }
                    Some((_, '#')) if matches!(self.language, Language::Jinja) => {
                        self.parse_jinja_comment().map(NodeKind::JinjaComment)
                    }
                    Some((_, '@')) => self.parse_svelte_at_tag().map(NodeKind::SvelteAtTag),
                    Some((_, '%')) if matches!(self.language, Language::Jinja) => {
                        self.parse_jinja_tag_or_block(None, &mut Parser::parse_node)
                    }
                    _ => match self.language {
                        Language::Svelte => self
                            .parse_svelte_interpolation()
                            .map(NodeKind::SvelteInterpolation),
                        Language::Astro => self.parse_astro_expr().map(NodeKind::AstroExpr),
                        _ => self.parse_text_node().map(NodeKind::Text),
                    },
                }
            }
            Some((_, '-'))
                if matches!(
                    self.language,
                    Language::Astro | Language::Jinja | Language::Vento | Language::Mustache
                ) && !self.state.has_front_matter =>
            {
                let mut chars = self.chars.clone();
                chars.next();
                if let Some(((_, '-'), (_, '-'))) = chars.next().zip(chars.next()) {
                    self.parse_front_matter().map(NodeKind::FrontMatter)
                } else {
                    self.parse_text_node().map(NodeKind::Text)
                }
            }
            Some((_, '@')) if matches!(self.language, Language::Angular) => {
                let mut chars = self.chars.clone();
                chars.next();
                match chars.next() {
                    Some((_, 'i')) => self.parse_angular_if().map(NodeKind::AngularIf),
                    Some((_, 'f')) => self.parse_angular_for().map(NodeKind::AngularFor),
                    Some((_, 's')) => self.parse_angular_switch().map(NodeKind::AngularSwitch),
                    Some((_, 'l')) => self.parse_angular_let().map(NodeKind::AngularLet),
                    _ => self.parse_text_node().map(NodeKind::Text),
                }
            }
            Some(..) => self.parse_text_node().map(NodeKind::Text),
            None => Err(self.emit_error(SyntaxErrorKind::ExpectElement)),
        }
    }

    fn parse_raw_text_node(&mut self, tag_name: &str) -> PResult<TextNode<'s>> {
        let start = self
            .chars
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or(self.source.len());

        let allow_nested = tag_name.eq_ignore_ascii_case("pre");
        let mut nested = 0u16;
        let mut line_breaks = 0;
        let end;
        loop {
            match self.chars.peek() {
                Some((i, '<')) => {
                    let i = *i;
                    let mut chars = self.chars.clone();
                    chars.next();
                    if chars.next_if(|(_, c)| *c == '/').is_some()
                        && chars
                            .by_ref()
                            .zip(tag_name.chars())
                            .all(|((_, a), b)| a.eq_ignore_ascii_case(&b))
                    {
                        if nested == 0 {
                            end = i;
                            break;
                        } else {
                            nested -= 1;
                            self.chars = chars;
                            continue;
                        }
                    } else if allow_nested
                        && chars
                            .by_ref()
                            .zip(tag_name.chars())
                            .all(|((_, a), b)| a.eq_ignore_ascii_case(&b))
                    {
                        nested += 1;
                        self.chars = chars;
                        continue;
                    }
                    self.chars.next();
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
            start,
        })
    }

    pub fn parse_root(&mut self) -> PResult<Root<'s>> {
        let mut children = vec![];
        while self.chars.peek().is_some() {
            children.push(self.parse_node()?);
        }

        Ok(Root { children })
    }

    fn parse_svelte_at_tag(&mut self) -> PResult<SvelteAtTag<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '@'))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteAtTag));
        };
        let name = self.parse_identifier()?;
        self.skip_ws();
        let expr = self.parse_svelte_or_astro_expr()?;
        Ok(SvelteAtTag { name, expr })
    }

    fn parse_svelte_attachment(&mut self) -> PResult<SvelteAttachment<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .map(|_| self.skip_ws())
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '@'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 't'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 't'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'a'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'c'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'h'))
            .is_some()
        {
            self.parse_svelte_or_astro_expr()
                .map(|expr| SvelteAttachment { expr })
        } else {
            Err(self.emit_error(SyntaxErrorKind::ExpectSvelteAttachment))
        }
    }

    fn parse_svelte_attr(&mut self) -> PResult<SvelteAttribute<'s>> {
        let name = if self.chars.next_if(|(_, c)| *c == '{').is_some() {
            None
        } else {
            let name = self.parse_attr_name()?;
            self.skip_ws();
            if self
                .chars
                .next_if(|(_, c)| *c == '=')
                .map(|_| self.skip_ws())
                .and_then(|_| self.chars.next_if(|(_, c)| *c == '{'))
                .is_some()
            {
                Some(name)
            } else {
                return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteAttr));
            }
        };

        self.parse_svelte_or_astro_expr()
            .map(|expr| SvelteAttribute { name, expr })
    }

    fn parse_svelte_await_block(&mut self) -> PResult<Box<SvelteAwaitBlock<'s>>> {
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
                        }
                        self.chars.next();
                        braces_stack -= 1;
                    }
                    Some((i, _)) => {
                        end = *i;
                        self.chars.next();
                    }
                    None => break,
                }
            }
            (unsafe { self.source.get_unchecked(start..end) }, start)
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
            Some(match self.chars.peek() {
                Some((_, '}')) => None,
                _ => Some(self.parse_svelte_binding()?),
            })
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
            Some(match self.chars.peek() {
                Some((_, '}')) => None,
                _ => Some(self.parse_svelte_binding()?),
            })
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
            let binding = match self.chars.peek() {
                Some((_, '}')) => None,
                _ => {
                    let binding = self.parse_svelte_binding()?;
                    self.skip_ws();
                    Some(binding)
                }
            };
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
            let binding = match self.chars.peek() {
                Some((_, '}')) => None,
                _ => {
                    let binding = self.parse_svelte_binding()?;
                    self.skip_ws();
                    Some(binding)
                }
            };
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
            .map(|_| self.skip_ws())
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
            Ok(Box::new(SvelteAwaitBlock {
                expr,
                then_binding,
                catch_binding,
                children,
                then_block,
                catch_block,
            }))
        } else {
            Err(self.emit_error(SyntaxErrorKind::ExpectSvelteBlockEnd))
        }
    }

    fn parse_svelte_binding(&mut self) -> PResult<(&'s str, usize)> {
        match self.chars.peek() {
            Some((start, '{')) => {
                let start = start + 1;
                self.parse_inside('{', '}', true)
                    .map(|binding| (binding, start))
            }
            Some((start, '[')) => {
                let start = start + 1;
                self.parse_inside('[', ']', true)
                    .map(|binding| (binding, start))
            }
            Some((start, _)) => {
                let start = *start;
                self.parse_identifier().map(|ident| (ident, start))
            }
            _ => Err(self.emit_error(SyntaxErrorKind::ExpectIdentifier)),
        }
    }

    fn parse_svelte_block_children(&mut self) -> PResult<Vec<Node<'s>>> {
        let mut children = vec![];
        loop {
            match self.chars.peek() {
                Some((_, '{')) => {
                    let mut chars = self.chars.clone();
                    chars.next();
                    while chars.next_if(|(_, c)| c.is_ascii_whitespace()).is_some() {}
                    if chars.next_if(|(_, c)| *c == '/' || *c == ':').is_some() {
                        break;
                    }
                    children.push(self.parse_node()?);
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

        let mut binding = None;
        let expr = {
            let start = self
                .chars
                .peek()
                .map(|(i, _)| *i)
                .unwrap_or(self.source.len());
            let mut end = start;
            let mut pair_stack = vec![];
            loop {
                match self.chars.peek() {
                    Some((i, c)) if c.is_ascii_whitespace() => {
                        end = *i;
                        self.skip_ws();
                        let mut chars = self.chars.clone();
                        if chars
                            .next_if(|(_, c)| *c == 'a')
                            .and_then(|_| chars.next_if(|(_, c)| *c == 's'))
                            .and_then(|_| chars.next_if(|(_, c)| c.is_ascii_whitespace()))
                            .is_some()
                        {
                            self.chars = chars;
                            self.skip_ws();
                            binding = Some(self.parse_svelte_binding()?);

                            // fix for #127
                            let mut chars = self.chars.clone();
                            while chars.next_if(|(_, c)| c.is_ascii_whitespace()).is_some() {}
                            if matches!(chars.peek(), Some((_, '}' | '(' | ','))) {
                                break;
                            }
                        }
                    }
                    Some((_, '(')) => {
                        pair_stack.push('(');
                        self.chars.next();
                    }
                    Some((i, ')')) if matches!(pair_stack.last(), Some('(')) => {
                        pair_stack.pop();
                        end = *i;
                        self.chars.next();
                    }
                    Some((_, '[')) => {
                        pair_stack.push('[');
                        self.chars.next();
                    }
                    Some((i, ']')) if matches!(pair_stack.last(), Some('[')) => {
                        pair_stack.pop();
                        end = *i;
                        self.chars.next();
                    }
                    Some((_, '{')) => {
                        pair_stack.push('{');
                        self.chars.next();
                    }
                    Some((i, '}')) => {
                        end = *i;
                        if matches!(pair_stack.last(), Some('{')) {
                            pair_stack.pop();
                            self.chars.next();
                        } else {
                            break;
                        }
                    }
                    Some((i, ',')) => {
                        end = *i;
                        if pair_stack.is_empty() {
                            break;
                        } else {
                            self.chars.next();
                        }
                    }
                    Some((i, _)) => {
                        end = *i;
                        self.chars.next();
                    }
                    None => break,
                }
            }
            (unsafe { self.source.get_unchecked(start..end) }, start)
        };

        self.skip_ws();
        let index = if self.chars.next_if(|(_, c)| *c == ',').is_some() {
            self.skip_ws();
            Some(self.parse_identifier()?)
        } else {
            None
        };

        self.skip_ws();
        let key = if let Some((start, '(')) = self.chars.peek() {
            let start = start + 1;
            Some((self.parse_inside('(', ')', false)?, start))
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
            .map(|_| self.skip_ws())
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

        let expr = self.parse_svelte_or_astro_expr()?;
        let children = self.parse_svelte_block_children()?;

        let mut else_if_blocks = vec![];
        let mut else_children = None;
        loop {
            if self.chars.next_if(|(_, c)| *c == '{').is_none() {
                return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteBlockEnd));
            }
            self.skip_ws();
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
                            let expr = self.parse_svelte_or_astro_expr()?;
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
        if self.chars.next_if(|(_, c)| *c == '{').is_some() {
            Ok(SvelteInterpolation {
                expr: self.parse_svelte_or_astro_expr()?,
            })
        } else {
            Err(self.emit_error(SyntaxErrorKind::ExpectSvelteInterpolation))
        }
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

        let expr = self.parse_svelte_or_astro_expr()?;
        let children = self.parse_svelte_block_children()?;

        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .map(|_| self.skip_ws())
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

    /// This will consume `}`.
    fn parse_svelte_or_astro_expr(&mut self) -> PResult<(&'s str, usize)> {
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
                    }
                    braces_stack -= 1;
                }
                Some(..) => continue,
                None => break,
            }
        }
        Ok((unsafe { self.source.get_unchecked(start..end) }, start))
    }

    fn parse_svelte_snippet_block(&mut self) -> PResult<SvelteSnippetBlock<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '#'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 's'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'n'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'i'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'p'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'p'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'e'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 't'))
            .and_then(|_| self.chars.next_if(|(_, c)| c.is_ascii_whitespace()))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectSvelteSnippetBlock));
        };

        let signature = self.parse_svelte_or_astro_expr()?;
        let children = self.parse_svelte_block_children()?;

        if self
            .chars
            .next_if(|(_, c)| *c == '{')
            .map(|_| self.skip_ws())
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '/'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 's'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'n'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'i'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'p'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'p'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'e'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 't'))
            .map(|_| self.skip_ws())
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '}'))
            .is_some()
        {
            Ok(SvelteSnippetBlock {
                signature,
                children,
            })
        } else {
            Err(self.emit_error(SyntaxErrorKind::ExpectSvelteBlockEnd))
        }
    }

    fn parse_tag_name(&mut self) -> PResult<&'s str> {
        let (start, mut end) = match self.chars.peek() {
            Some((i, c)) if is_html_tag_name_char(*c) => {
                let c = *c;
                let start = *i;
                self.chars.next();
                (start, start + c.len_utf8())
            }
            Some((i, '{')) if matches!(self.language, Language::Jinja) => (*i, *i + 1),
            Some((_, '>')) if matches!(self.language, Language::Astro) => {
                // Astro allows fragment
                return Ok("");
            }
            _ => return Err(self.emit_error(SyntaxErrorKind::ExpectTagName)),
        };

        while let Some((i, c)) = self.chars.peek() {
            if is_html_tag_name_char(*c) {
                end = *i + c.len_utf8();
                self.chars.next();
            } else if *c == '{' && matches!(self.language, Language::Jinja) {
                let current_i = *i;
                let mut chars = self.chars.clone();
                chars.next();
                if chars.next_if(|(_, c)| *c == '{').is_some() {
                    end = current_i + self.parse_mustache_interpolation()?.0.len() + "{{}}".len();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        unsafe { Ok(self.source.get_unchecked(start..end)) }
    }

    fn parse_text_node(&mut self) -> PResult<TextNode<'s>> {
        let Some((start, first_char)) = self.chars.next_if(|(_, c)| {
            if matches!(
                self.language,
                Language::Vue
                    | Language::Svelte
                    | Language::Jinja
                    | Language::Vento
                    | Language::Mustache
                    | Language::Angular
            ) {
                *c != '{'
            } else {
                true
            }
        }) else {
            return Err(self.emit_error(SyntaxErrorKind::ExpectTextNode));
        };

        if matches!(
            self.language,
            Language::Vue
                | Language::Jinja
                | Language::Vento
                | Language::Angular
                | Language::Mustache
        ) && first_char == '{'
            && matches!(self.chars.peek(), Some((_, '{')))
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectTextNode));
        }

        let mut line_breaks = if first_char == '\n' { 1 } else { 0 };
        let end;
        loop {
            match self.chars.peek() {
                Some((i, '{')) => match self.language {
                    Language::Html | Language::Xml => {
                        self.chars.next();
                    }
                    Language::Vue | Language::Vento | Language::Angular | Language::Mustache => {
                        let i = *i;
                        let mut chars = self.chars.clone();
                        chars.next();
                        if chars.next_if(|(_, c)| *c == '{').is_some() {
                            end = i;
                            break;
                        }
                        self.chars.next();
                    }
                    Language::Svelte | Language::Astro => {
                        end = *i;
                        break;
                    }
                    Language::Jinja => {
                        let i = *i;
                        let mut chars = self.chars.clone();
                        chars.next();
                        if chars
                            .next_if(|(_, c)| *c == '%' || *c == '{' || *c == '#')
                            .is_some()
                        {
                            end = i;
                            break;
                        }
                        self.chars.next();
                    }
                },
                Some((i, '<')) => {
                    let i = *i;
                    let mut chars = self.chars.clone();
                    chars.next();
                    match chars.next() {
                        Some((_, c))
                            if is_html_tag_name_char(c)
                                || is_special_tag_name_char(c, self.language)
                                || c == '/'
                                || c == '!' =>
                        {
                            end = i;
                            break;
                        }
                        _ => {
                            self.chars.next();
                        }
                    }
                }
                Some((i, '-'))
                    if matches!(self.language, Language::Astro) && !self.state.has_front_matter =>
                {
                    let i = *i;
                    let mut chars = self.chars.clone();
                    chars.next();
                    if let Some(((_, '-'), (_, '-'))) = chars.next().zip(chars.next()) {
                        end = i;
                        break;
                    }
                    self.chars.next();
                }
                Some((i, '}' | '@')) if matches!(self.language, Language::Angular) => {
                    end = *i;
                    break;
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
            start,
        })
    }

    fn parse_vento_block_children(&mut self) -> PResult<Vec<Node<'s>>> {
        let mut children = vec![];
        loop {
            match self.chars.peek() {
                Some((_, '{')) => {
                    let mut chars = self.chars.clone();
                    chars.next();
                    if chars.next_if(|(_, c)| *c == '{').is_some() {
                        break;
                    }
                    children.push(self.parse_node()?);
                }
                Some(..) => {
                    children.push(self.parse_node()?);
                }
                None => return Err(self.emit_error(SyntaxErrorKind::ExpectVentoBlockEnd)),
            }
        }
        Ok(children)
    }

    fn parse_vento_tag_or_block(
        &mut self,
        first_tag: Option<(&'s str, bool, bool, usize)>,
    ) -> PResult<NodeKind<'s>> {
        let (first_tag, trim_prev, trim_next, first_tag_start) = if let Some(first_tag) = first_tag
        {
            first_tag
        } else {
            let (mut first_tag, mut start) = self.parse_mustache_interpolation()?;
            let mut trim_prev = false;
            let mut trim_next = false;
            if let Some(tag) = first_tag.strip_prefix('-') {
                first_tag = tag;
                trim_prev = true;
                start += 1;
            }
            if let Some(tag) = first_tag.strip_suffix('-') {
                first_tag = tag;
                trim_next = true;
            }
            (first_tag, trim_prev, trim_next, start)
        };

        if let Some(raw) = first_tag
            .strip_prefix('#')
            .and_then(|s| s.strip_suffix('#'))
        {
            return Ok(NodeKind::VentoComment(VentoComment { raw }));
        } else if let Some(raw) = first_tag.strip_prefix('>') {
            return Ok(NodeKind::VentoEval(VentoEval {
                raw,
                start: first_tag_start,
            }));
        }

        let (tag_name, tag_rest) = helpers::parse_vento_tag(first_tag);

        let is_function = tag_name == "function"
            || matches!(tag_name, "async" | "export") && tag_rest.starts_with("function");
        if matches!(tag_name, "for" | "if" | "layout")
            || matches!(tag_name, "set" | "export") && !first_tag.contains('=')
            || is_function
        {
            let mut body = vec![VentoTagOrChildren::Tag(VentoTag {
                tag: first_tag,
                trim_prev,
                trim_next,
            })];

            loop {
                let mut children = self.parse_vento_block_children()?;
                if !children.is_empty() {
                    if let Some(VentoTagOrChildren::Children(nodes)) = body.last_mut() {
                        nodes.append(&mut children);
                    } else {
                        body.push(VentoTagOrChildren::Children(children));
                    }
                }
                if let Ok((mut next_tag, mut next_tag_start)) = self.parse_mustache_interpolation()
                {
                    let mut trim_prev = false;
                    let mut trim_next = false;
                    if let Some(tag) = next_tag.strip_prefix('-') {
                        next_tag = tag;
                        trim_prev = true;
                        next_tag_start += 1;
                    };
                    if let Some(tag) = next_tag.strip_suffix('-') {
                        next_tag = tag;
                        trim_next = true;
                    };
                    let (next_tag_name, _) = helpers::parse_vento_tag(next_tag);
                    if next_tag_name
                        .trim()
                        .strip_prefix('/')
                        .is_some_and(|name| name == tag_name || is_function && name == "function")
                    {
                        body.push(VentoTagOrChildren::Tag(VentoTag {
                            tag: next_tag,
                            trim_prev,
                            trim_next,
                        }));
                        break;
                    }
                    if tag_name == "if" && next_tag_name == "else" {
                        body.push(VentoTagOrChildren::Tag(VentoTag {
                            tag: next_tag,
                            trim_prev,
                            trim_next,
                        }));
                    } else {
                        let node = self
                            .with_taken(|parser| {
                                parser.parse_vento_tag_or_block(Some((
                                    next_tag,
                                    trim_prev,
                                    trim_next,
                                    next_tag_start,
                                )))
                            })
                            .map(|(kind, raw)| Node { kind, raw })?;
                        if let Some(VentoTagOrChildren::Children(nodes)) = body.last_mut() {
                            nodes.push(node);
                        } else {
                            body.push(VentoTagOrChildren::Children(vec![node]));
                        }
                    }
                } else {
                    break;
                }
            }
            Ok(NodeKind::VentoBlock(VentoBlock { body }))
        } else if is_vento_interpolation(tag_name) {
            Ok(NodeKind::VentoInterpolation(VentoInterpolation {
                expr: first_tag,
                start: first_tag_start,
            }))
        } else {
            Ok(NodeKind::VentoTag(VentoTag {
                tag: first_tag,
                trim_prev,
                trim_next,
            }))
        }
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

    fn parse_xml_decl(&mut self) -> PResult<XmlDecl<'s>> {
        if self
            .chars
            .next_if(|(_, c)| *c == '<')
            .and_then(|_| self.chars.next_if(|(_, c)| *c == '?'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'x'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'm'))
            .and_then(|_| self.chars.next_if(|(_, c)| *c == 'l'))
            .and_then(|_| self.chars.next_if(|(_, c)| c.is_ascii_whitespace()))
            .is_none()
        {
            return Err(self.emit_error(SyntaxErrorKind::ExpectXmlDecl));
        };

        let mut attrs = vec![];
        loop {
            match self.chars.peek() {
                Some((_, '?')) => {
                    self.chars.next();
                    if self.chars.next_if(|(_, c)| *c == '>').is_some() {
                        break;
                    }
                    return Err(self.emit_error(SyntaxErrorKind::ExpectChar('>')));
                }
                Some((_, c)) if c.is_ascii_whitespace() => {
                    self.chars.next();
                }
                _ => {
                    attrs.push(self.parse_native_attr()?);
                }
            }
        }
        Ok(XmlDecl { attrs })
    }
}

/// Returns true if the provided character is a valid HTML tag name character.
fn is_html_tag_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || c == '-'
        || c == '_'
        || c == '.'
        || c == ':'
        || !c.is_ascii()
        || c == '\\'
}

/// Checks whether a character is valid in an HTML tag name, for specific template languages.
///
/// For example:
/// - Astro allows '>' in tag names (for fragments)
/// - Jinja allows '{' for template expressions like <{{ tag_name }}>
fn is_special_tag_name_char(c: char, language: Language) -> bool {
    match language {
        Language::Astro => c == '>',
        Language::Jinja => c == '{',
        _ => false,
    }
}

fn is_attr_name_char(c: char) -> bool {
    !matches!(c, '"' | '\'' | '>' | '/' | '=') && !c.is_ascii_whitespace()
}

fn parse_jinja_tag_name<'s>(tag: &JinjaTag<'s>) -> &'s str {
    let trimmed = tag.content.trim_start_matches(['+', '-']).trim_start();
    trimmed
        .split_once(|c: char| c.is_ascii_whitespace())
        .map(|(name, _)| name)
        .unwrap_or(trimmed)
}

fn is_vento_interpolation(tag_name: &str) -> bool {
    !matches!(
        tag_name,
        "if" | "else"
            | "for"
            | "set"
            | "include"
            | "layout"
            | "async"
            | "function"
            | "import"
            | "export"
    )
}

pub type PResult<T> = Result<T, SyntaxError>;
type AngularIfCond<'s> = ((&'s str, usize), Option<(&'s str, usize)>);

trait HasJinjaFlowControl<'s>: Sized {
    type Intermediate;

    fn build(intermediate: Self::Intermediate, raw: &'s str) -> Self;
    fn from_tag(tag: JinjaTag<'s>) -> Self::Intermediate;
    fn from_block(block: JinjaBlock<'s, Self>) -> Self::Intermediate;
}

impl<'s> HasJinjaFlowControl<'s> for Node<'s> {
    type Intermediate = NodeKind<'s>;

    fn build(intermediate: Self::Intermediate, raw: &'s str) -> Self {
        Node {
            kind: intermediate,
            raw,
        }
    }

    fn from_tag(tag: JinjaTag<'s>) -> Self::Intermediate {
        NodeKind::JinjaTag(tag)
    }

    fn from_block(block: JinjaBlock<'s, Self>) -> Self::Intermediate {
        NodeKind::JinjaBlock(block)
    }
}

impl<'s> HasJinjaFlowControl<'s> for Attribute<'s> {
    type Intermediate = Attribute<'s>;

    fn build(intermediate: Self::Intermediate, _: &'s str) -> Self {
        intermediate
    }

    fn from_tag(tag: JinjaTag<'s>) -> Self::Intermediate {
        Attribute::JinjaTag(tag)
    }

    fn from_block(block: JinjaBlock<'s, Self>) -> Self::Intermediate {
        Attribute::JinjaBlock(block)
    }
}
