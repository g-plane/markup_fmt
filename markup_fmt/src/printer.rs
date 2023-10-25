use crate::{
    ast::*,
    config::{Quotes, WhitespaceSensitivity},
    ctx::{Ctx, NestWithCtx},
    Language,
};
use std::{borrow::Cow, path::Path};
use tiny_pretty::Doc;

pub(super) trait DocGen<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>;
}

impl<'s> DocGen<'s> for Attribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        match self {
            Attribute::NativeAttribute(native_attribute) => native_attribute.doc(ctx),
            Attribute::SvelteAttribute(svelte_attribute) => svelte_attribute.doc(ctx),
            Attribute::VueDirective(vue_directive) => vue_directive.doc(ctx),
        }
    }
}

impl<'s> DocGen<'s> for Comment<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        if ctx.options.format_comments {
            Doc::text("<!--")
                .append(
                    Doc::line_or_space()
                        .concat(reflow(self.raw.trim()))
                        .nest_with_ctx(ctx),
                )
                .append(Doc::line_or_space())
                .append(Doc::text("-->"))
                .group()
        } else {
            Doc::text("<!--")
                .concat(reflow_raw(self.raw))
                .append(Doc::text("-->"))
        }
    }
}

impl<'s> DocGen<'s> for Element<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let tag_name = self
            .tag_name
            .split_once(':')
            .and_then(|(namespace, name)| namespace.eq_ignore_ascii_case("html").then_some(name))
            .unwrap_or(self.tag_name);
        ctx.current_tag_name = Some(tag_name);
        ctx.in_svg = tag_name.eq_ignore_ascii_case("svg");
        let should_lower_cased = css_dataset::tags::STANDARD_HTML_TAGS
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(self.tag_name));

        let mut docs = Vec::with_capacity(5);

        docs.push(Doc::text("<"));
        docs.push(Doc::text(if should_lower_cased {
            Cow::from(self.tag_name.to_ascii_lowercase())
        } else {
            Cow::from(self.tag_name)
        }));

        let attrs = if let Some(max) = ctx.options.max_attrs_per_line {
            Doc::line_or_space()
                .concat(itertools::intersperse(
                    self.attrs.chunks(max).map(|chunk| {
                        Doc::list(
                            itertools::intersperse(
                                chunk.iter().map(|attr| attr.doc(ctx)),
                                Doc::line_or_space(),
                            )
                            .collect(),
                        )
                        .group()
                    }),
                    Doc::hard_line(),
                ))
                .nest_with_ctx(ctx)
        } else {
            Doc::list(
                self.attrs
                    .iter()
                    .flat_map(|attr| [Doc::line_or_space(), attr.doc(ctx)].into_iter())
                    .collect(),
            )
            .nest_with_ctx(ctx)
        };

        if self.void_element {
            docs.push(attrs);
            if !ctx.options.closing_bracket_same_line {
                docs.push(Doc::line_or_nil());
            }
            docs.push(Doc::text(">"));
            return Doc::list(docs).group();
        } else if self.self_closing {
            docs.push(attrs);
            docs.push(Doc::line_or_space());
            docs.push(Doc::text("/>"));
            return Doc::list(docs).group();
        } else if ctx.options.closing_bracket_same_line {
            docs.push(attrs.append(Doc::text(">")).group());
        } else {
            docs.push(
                attrs
                    .append(Doc::line_or_nil())
                    .append(Doc::text(">"))
                    .group(),
            );
        }

        let is_whitespace_sensitive = ctx.is_whitespace_sensitive(tag_name);
        let is_empty = match &self.children[..] {
            [] => true,
            [Node::TextNode(text_node)] => {
                !is_whitespace_sensitive
                    && text_node
                        .raw
                        .trim_matches(|c: char| c.is_ascii_whitespace())
                        .is_empty()
            }
            _ => false,
        };
        let has_two_more_non_text_children = has_two_more_non_text_children(&self.children);

        let leading_ws = if is_whitespace_sensitive {
            if let Some(Node::TextNode(text_node)) = self.children.first() {
                if text_node.raw.starts_with(|c: char| c.is_ascii_whitespace()) {
                    if text_node.line_breaks > 0 {
                        Doc::hard_line()
                    } else {
                        Doc::line_or_space()
                    }
                } else {
                    Doc::nil()
                }
            } else {
                Doc::nil()
            }
        } else if has_two_more_non_text_children
            || self
                .children
                .first()
                .map(|child| match child {
                    Node::TextNode(text_node) => text_node.line_breaks > 0,
                    _ => false,
                })
                .unwrap_or_default()
        {
            Doc::hard_line()
        } else if is_empty {
            Doc::nil()
        } else {
            Doc::line_or_nil()
        };
        let trailing_ws = if is_whitespace_sensitive {
            if let Some(Node::TextNode(text_node)) = self.children.last() {
                if text_node.raw.ends_with(|c: char| c.is_ascii_whitespace()) {
                    if text_node.line_breaks > 0 {
                        Doc::hard_line()
                    } else {
                        Doc::line_or_space()
                    }
                } else {
                    Doc::nil()
                }
            } else {
                Doc::nil()
            }
        } else if has_two_more_non_text_children
            || self
                .children
                .last()
                .map(|child| match child {
                    Node::TextNode(text_node) => text_node.line_breaks > 0,
                    _ => false,
                })
                .unwrap_or_default()
        {
            Doc::hard_line()
        } else if is_empty {
            Doc::nil()
        } else {
            Doc::line_or_nil()
        };

        if tag_name.eq_ignore_ascii_case("script") {
            if let [Node::TextNode(text_node)] = &self.children[..] {
                let formatted = ctx.format_script(
                    text_node.raw,
                    self.attrs
                        .iter()
                        .find_map(|attr| match attr {
                            Attribute::NativeAttribute(native_attribute)
                                if native_attribute.name.eq_ignore_ascii_case("lang") =>
                            {
                                native_attribute.value
                            }
                            _ => None,
                        })
                        .unwrap_or("js"),
                );
                let doc = Doc::hard_line().concat(reflow_raw_owned(formatted.trim()));
                docs.push(
                    if ctx.script_indent() {
                        doc.nest_with_ctx(ctx)
                    } else {
                        doc
                    }
                    .append(Doc::hard_line()),
                );
            }
        } else if tag_name.eq_ignore_ascii_case("style") {
            if let [Node::TextNode(text_node)] = &self.children[..] {
                let formatted = ctx.format_style(
                    text_node.raw,
                    self.attrs
                        .iter()
                        .find_map(|attr| match attr {
                            Attribute::NativeAttribute(native_attribute)
                                if native_attribute.name.eq_ignore_ascii_case("lang") =>
                            {
                                native_attribute.value
                            }
                            _ => None,
                        })
                        .unwrap_or("css"),
                );
                let doc = Doc::hard_line().concat(reflow_raw_owned(formatted.trim()));
                docs.push(
                    if ctx.style_indent() {
                        doc.nest_with_ctx(ctx)
                    } else {
                        doc
                    }
                    .append(Doc::hard_line()),
                );
            }
        } else if tag_name.eq_ignore_ascii_case("pre") || tag_name.eq_ignore_ascii_case("textarea")
        {
            if let [Node::TextNode(text_node)] = &self.children[..] {
                if text_node.raw.contains('\n')
                    && !text_node.raw.starts_with('\n')
                    && !text_node.raw.starts_with("\r\n")
                {
                    docs.push(Doc::empty_line());
                }
                docs.extend(reflow_raw(text_node.raw));
            }
        } else if is_empty {
            use crate::config::ClosingTagLineBreakForEmpty;
            if !is_whitespace_sensitive {
                match ctx.options.closing_tag_line_break_for_empty {
                    ClosingTagLineBreakForEmpty::Always => docs.push(Doc::hard_line()),
                    ClosingTagLineBreakForEmpty::Fit => docs.push(Doc::line_or_nil()),
                    ClosingTagLineBreakForEmpty::Never => {}
                };
            }
        } else if !is_whitespace_sensitive && has_two_more_non_text_children {
            docs.push(leading_ws.nest_with_ctx(ctx));
            docs.push(
                format_children_with_inserting_linebreak(&self.children, ctx).nest_with_ctx(ctx),
            );
            docs.push(trailing_ws);
        } else if is_whitespace_sensitive
            && matches!(&self.children[..], [Node::TextNode(text_node)] if is_all_ascii_whitespace(text_node.raw))
        {
            docs.push(Doc::line_or_space());
        } else {
            let children_doc = leading_ws.append(format_children_without_inserting_linebreak(
                &self.children,
                has_two_more_non_text_children,
                ctx,
            ));
            if self.children.iter().all(|child| {
                matches!(
                    child,
                    Node::VueInterpolation(..) | Node::SvelteInterpolation(..) | Node::Comment(..)
                )
            }) {
                // This lets it format like this:
                // ```
                // <span>{{
                //    value
                // }}</span>
                // ```
                docs.push(children_doc);
            } else {
                docs.push(children_doc.nest_with_ctx(ctx));
            }
            docs.push(trailing_ws);
        }

        docs.push(
            Doc::text("</")
                .append(Doc::text(if should_lower_cased {
                    Cow::from(self.tag_name.to_ascii_lowercase())
                } else {
                    Cow::from(self.tag_name)
                }))
                .append(Doc::line_or_nil())
                .append(Doc::text(">"))
                .group(),
        );
        ctx.current_tag_name = None;
        ctx.in_svg = false;

        Doc::list(docs).group()
    }
}

impl<'s> DocGen<'s> for NativeAttribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let name = Doc::text(self.name);
        if let Some(value) = self.value {
            let value = match ctx.language {
                Language::Vue => {
                    if ctx
                        .current_tag_name
                        .map(|name| name.eq_ignore_ascii_case("script"))
                        .unwrap_or_default()
                        && self.name == "generic"
                    {
                        Cow::from(ctx.format_type_params(value))
                    } else {
                        Cow::from(value)
                    }
                }
                Language::Svelte => {
                    if let Some(expr) = value.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                        return name
                            .append(Doc::text("={"))
                            .append(Doc::text(expr))
                            .append(Doc::text("}"));
                    } else {
                        Cow::from(value)
                    }
                }
                _ => Cow::from(value),
            };
            name.append(Doc::text("="))
                .append(format_attr_value(value, &ctx.options.quotes))
        } else {
            name
        }
    }
}

impl<'s> DocGen<'s> for Node<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        match self {
            Node::Comment(comment) => comment.doc(ctx),
            Node::Doctype => Doc::text("<!DOCTYPE html>"),
            Node::Element(element) => element.doc(ctx),
            Node::SvelteInterpolation(svelte_interpolation) => svelte_interpolation.doc(ctx),
            Node::TextNode(text_node) => text_node.doc(ctx),
            Node::VueInterpolation(vue_interpolation) => vue_interpolation.doc(ctx),
            _ => todo!(),
        }
    }
}

impl<'s> DocGen<'s> for Root<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let is_whole_document_like = self.children.iter().any(|child| match child {
            Node::Doctype => true,
            Node::Element(element) => element.tag_name.eq_ignore_ascii_case("html"),
            _ => false,
        });
        let is_whitespace_sensitive = matches!(
            ctx.options.whitespace_sensitivity,
            WhitespaceSensitivity::Css | WhitespaceSensitivity::Strict
        );
        let has_two_more_non_text_children = has_two_more_non_text_children(&self.children);

        if is_whole_document_like
            && !matches!(
                ctx.options.whitespace_sensitivity,
                WhitespaceSensitivity::Strict
            )
            || !is_whitespace_sensitive && has_two_more_non_text_children
        {
            format_children_with_inserting_linebreak(&self.children, ctx).append(Doc::hard_line())
        } else {
            format_children_without_inserting_linebreak(
                &self.children,
                has_two_more_non_text_children,
                ctx,
            )
            .append(Doc::hard_line())
        }
    }
}

impl<'s> DocGen<'s> for SvelteAttribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let name = Doc::text(self.name.to_owned());
        name.append(Doc::text("={"))
            .concat(reflow_raw_owned(&ctx.format_expr(self.expr)))
            .append(Doc::text("}"))
    }
}

impl<'s> DocGen<'s> for SvelteInterpolation<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{")
            .append(
                Doc::line_or_nil()
                    .concat(reflow_raw_owned(&ctx.format_expr(self.expr)))
                    .nest_with_ctx(ctx),
            )
            .append(Doc::line_or_nil())
            .append(Doc::text("}"))
            .group()
    }
}

impl<'s> DocGen<'s> for TextNode<'s> {
    fn doc<E, F>(&self, _: &mut Ctx<E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let docs = itertools::intersperse(
            self.raw.split_ascii_whitespace().map(Doc::text),
            Doc::soft_line(),
        )
        .collect::<Vec<_>>();

        if docs.is_empty() {
            Doc::nil()
        } else {
            Doc::list(docs)
        }
    }
}

impl<'s> DocGen<'s> for VueDirective<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        use crate::config::{VBindStyle, VOnStyle};

        let mut docs = Vec::with_capacity(5);

        docs.push(match self.name {
            ":" => {
                if let Some(VBindStyle::Long) = ctx.options.v_bind_style {
                    Doc::text("v-bind")
                } else {
                    Doc::text(":")
                }
            }
            "bind" if self.arg_and_modifiers.is_some() => {
                if let Some(VBindStyle::Short) = ctx.options.v_bind_style {
                    Doc::text(":")
                } else {
                    Doc::text("v-bind")
                }
            }
            "@" => {
                if let Some(VOnStyle::Long) = ctx.options.v_on_style {
                    Doc::text("v-on")
                } else {
                    Doc::text("@")
                }
            }
            "on" => {
                if let Some(VOnStyle::Short) = ctx.options.v_on_style {
                    Doc::text("@")
                } else {
                    Doc::text("v-on")
                }
            }
            "#" => Doc::text("#"),
            name => Doc::text(format!("v-{name}")),
        });

        if let Some(arg_and_modifiers) = self.arg_and_modifiers {
            docs.push(Doc::text(arg_and_modifiers));
        }

        if let Some(value) = self.value {
            docs.push(Doc::text("="));

            let value = if self.name == "for" {
                use crate::config::VForDelimiterStyle;
                if let Some((left, right)) = value.split_once(" in ") {
                    let delimiter =
                        if let Some(VForDelimiterStyle::Of) = ctx.options.v_for_delimiter_style {
                            "of"
                        } else {
                            "in"
                        };
                    format!(
                        "{} {delimiter} {}",
                        ctx.format_expr(left),
                        ctx.format_expr(right)
                    )
                } else if let Some((left, right)) = value.split_once(" of ") {
                    let delimiter =
                        if let Some(VForDelimiterStyle::In) = ctx.options.v_for_delimiter_style {
                            "in"
                        } else {
                            "of"
                        };
                    format!(
                        "{} {delimiter} {}",
                        ctx.format_expr(left),
                        ctx.format_expr(right)
                    )
                } else {
                    ctx.format_expr(value)
                }
            } else {
                ctx.format_expr(value)
            };
            docs.push(format_attr_value(value, &ctx.options.quotes));
        }

        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for VueInterpolation<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(
                Doc::line_or_space()
                    .concat(reflow_raw_owned(&ctx.format_expr(self.expr)))
                    .nest_with_ctx(ctx),
            )
            .append(Doc::line_or_space())
            .append(Doc::text("}}"))
            .group()
    }
}

fn reflow(s: &str) -> impl Iterator<Item = Doc<'static>> + '_ {
    itertools::intersperse(
        s.split('\n')
            .map(|s| Doc::text(s.strip_suffix('\r').unwrap_or(s).to_owned())),
        Doc::hard_line(),
    )
}

fn reflow_raw(s: &str) -> impl Iterator<Item = Doc<'_>> {
    itertools::intersperse(
        s.split('\n')
            .map(|s| Doc::text(s.strip_suffix('\r').unwrap_or(s))),
        Doc::empty_line(),
    )
}

fn reflow_raw_owned(s: &str) -> impl Iterator<Item = Doc<'static>> + '_ {
    itertools::intersperse(
        s.split('\n')
            .map(|s| Doc::text(s.strip_suffix('\r').unwrap_or(s).to_owned())),
        Doc::empty_line(),
    )
}

fn is_all_ascii_whitespace(s: &str) -> bool {
    !s.is_empty() && s.as_bytes().iter().all(|byte| byte.is_ascii_whitespace())
}

fn should_add_whitespace_before_text_node<'s>(
    text_node: &TextNode<'s>,
    is_first: bool,
) -> Option<Doc<'s>> {
    let trimmed = text_node
        .raw
        .trim_end_matches(|c: char| c.is_ascii_whitespace());
    if !is_first && trimmed.starts_with(|c: char| c.is_ascii_whitespace()) {
        let line_breaks_count = text_node
            .raw
            .chars()
            .take_while(|c| c.is_ascii_whitespace())
            .filter(|c| *c == '\n')
            .count();
        match line_breaks_count {
            0 => Some(Doc::soft_line()),
            1 => Some(Doc::hard_line()),
            _ => Some(Doc::empty_line().append(Doc::hard_line())),
        }
    } else {
        None
    }
}

fn should_add_whitespace_after_text_node<'s>(
    text_node: &TextNode<'s>,
    is_last: bool,
) -> Option<Doc<'s>> {
    let trimmed = text_node
        .raw
        .trim_start_matches(|c: char| c.is_ascii_whitespace());
    if !is_last && trimmed.ends_with(|c: char| c.is_ascii_whitespace()) {
        let line_breaks_count = text_node
            .raw
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_whitespace())
            .filter(|c| *c == '\n')
            .count();
        match line_breaks_count {
            0 => Some(Doc::soft_line()),
            1 => Some(Doc::hard_line()),
            _ => Some(Doc::empty_line().append(Doc::hard_line())),
        }
    } else {
        None
    }
}

fn has_two_more_non_text_children(children: &[Node]) -> bool {
    children
        .iter()
        .filter(|child| !matches!(child, Node::TextNode(_)))
        .count()
        > 1
}

fn format_attr_value(value: impl AsRef<str>, quotes: &Quotes) -> Doc<'static> {
    let value = value.as_ref();
    let quote = if value.contains('"') {
        Doc::text("'")
    } else if value.contains('\'') {
        Doc::text("\"")
    } else if let Quotes::Double = quotes {
        Doc::text("\"")
    } else {
        Doc::text("'")
    };
    quote.clone().concat(reflow_raw_owned(value)).append(quote)
}

fn format_children_with_inserting_linebreak<'s, E, F>(
    children: &[Node<'s>],
    ctx: &mut Ctx<'_, 's, E, F>,
) -> Doc<'s>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    Doc::list(
        children
            .iter()
            .enumerate()
            .fold(
                (Vec::with_capacity(children.len() * 2), true),
                |(mut docs, is_prev_text_like), (i, child)| {
                    let maybe_hard_line = if is_prev_text_like {
                        None
                    } else {
                        Some(Doc::hard_line())
                    };
                    match child {
                        Node::TextNode(text_node) => {
                            let is_first = i == 0;
                            let is_last = i + 1 == children.len();
                            if is_all_ascii_whitespace(text_node.raw) {
                                if !is_first && !is_last {
                                    if text_node.line_breaks > 1 {
                                        docs.push(Doc::empty_line());
                                    }
                                    docs.push(Doc::hard_line());
                                }
                            } else {
                                if let Some(hard_line) = maybe_hard_line {
                                    docs.push(hard_line);
                                } else if let Some(doc) =
                                    should_add_whitespace_before_text_node(text_node, is_first)
                                {
                                    docs.push(doc);
                                }
                                docs.push(text_node.doc(ctx));
                                if let Some(doc) =
                                    should_add_whitespace_after_text_node(text_node, is_last)
                                {
                                    docs.push(doc);
                                }
                            }
                        }
                        child => {
                            if let Some(hard_line) = maybe_hard_line {
                                docs.push(hard_line);
                            }
                            docs.push(child.doc(ctx));
                        }
                    };
                    (
                        docs,
                        match child {
                            Node::TextNode(..)
                            | Node::VueInterpolation(..)
                            | Node::SvelteInterpolation(..) => true,
                            Node::Element(element) => {
                                element.tag_name.eq_ignore_ascii_case("label")
                            }
                            _ => false,
                        },
                    )
                },
            )
            .0,
    )
    .group()
}

fn format_children_without_inserting_linebreak<'s, E, F>(
    children: &[Node<'s>],
    has_two_more_non_text_children: bool,
    ctx: &mut Ctx<'_, 's, E, F>,
) -> Doc<'s>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    Doc::list(
        children
            .iter()
            .enumerate()
            .map(|(i, child)| match child {
                Node::TextNode(text_node) => {
                    let is_first = i == 0;
                    let is_last = i + 1 == children.len();
                    if !is_first && !is_last && is_all_ascii_whitespace(text_node.raw) {
                        return if text_node.line_breaks > 1 {
                            Doc::empty_line().append(Doc::hard_line())
                        } else if has_two_more_non_text_children {
                            Doc::hard_line()
                        } else {
                            Doc::line_or_space()
                        };
                    }

                    let mut docs = Vec::with_capacity(3);
                    if let Some(doc) = should_add_whitespace_before_text_node(text_node, is_first) {
                        docs.push(doc);
                    }
                    docs.push(text_node.doc(ctx));
                    if let Some(doc) = should_add_whitespace_after_text_node(text_node, is_last) {
                        docs.push(doc);
                    }
                    Doc::list(docs)
                }
                child => child.doc(ctx),
            })
            .collect(),
    )
    .group()
}
