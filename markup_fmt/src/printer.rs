use crate::{
    ast::*,
    config::{Quotes, VSlotStyle, WhitespaceSensitivity},
    ctx::{Ctx, NestWithCtx},
    helpers, Language,
};
use itertools::Itertools;
use std::{borrow::Cow, mem, path::Path};
use tiny_pretty::Doc;

pub(super) trait DocGen<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>;
}

impl<'s> DocGen<'s> for AstroAttribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let expr_code = ctx.format_expr(self.expr);
        let expr = Doc::text("{")
            .concat(reflow_with_indent(&expr_code))
            .append(Doc::text("}"));
        if let Some(name) = self.name {
            if (matches!(ctx.options.astro_attr_shorthand, Some(true))) && name == expr_code {
                expr
            } else {
                Doc::text(name).append(Doc::text("=")).append(expr)
            }
        } else if matches!(ctx.options.astro_attr_shorthand, Some(false)) {
            Doc::text(expr_code.clone())
                .append(Doc::text("="))
                .append(expr)
        } else {
            expr
        }
    }
}

impl<'s> DocGen<'s> for AstroExpr<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        const PLACEHOLDER: &str = "$AstroTpl$";
        let script = self
            .children
            .iter()
            .filter_map(|child| {
                if let AstroExprChild::Script(script) = child {
                    Some(*script)
                } else {
                    None
                }
            })
            .join(PLACEHOLDER);
        let formatted_script = ctx.format_expr(&script);

        let templates = self.children.iter().filter_map(|child| {
            if let AstroExprChild::Template(nodes) = child {
                Some(
                    Doc::flat_or_break(Doc::nil(), Doc::text("("))
                        .append(
                            Doc::line_or_nil()
                                .append(format_children_without_inserting_linebreak(
                                    &nodes,
                                    has_two_more_non_text_children(&nodes),
                                    ctx,
                                ))
                                .nest_with_ctx(ctx),
                        )
                        .append(Doc::line_or_nil())
                        .append(Doc::flat_or_break(Doc::nil(), Doc::text(")")))
                        .group(),
                )
            } else {
                None
            }
        });

        let doc = Doc::text("{")
            .append(
                Doc::line_or_nil()
                    .concat(
                        formatted_script
                            .split(PLACEHOLDER)
                            .map(|script| {
                                if script.contains('\n') {
                                    Doc::list(reflow_with_indent(script).collect())
                                } else {
                                    Doc::text(script.to_string())
                                }
                            })
                            .interleave(templates),
                    )
                    .nest_with_ctx(ctx),
            )
            .append(Doc::line_or_nil())
            .append(Doc::text("}"));
        if script.contains("//") {
            doc
        } else {
            doc.group()
        }
    }
}

impl<'s> DocGen<'s> for AstroFrontMatter<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let formatted = ctx.format_script(self.raw, "tsx");
        Doc::text("---")
            .append(Doc::hard_line())
            .concat(reflow_with_indent(formatted.trim()))
            .append(Doc::hard_line())
            .append(Doc::text("---"))
    }
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
            Attribute::AstroAttribute(astro_attribute) => astro_attribute.doc(ctx),
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
                        .concat(reflow_with_indent(self.raw.trim()))
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
        let should_lower_cased = matches!(ctx.language, Language::Html | Language::Jinja)
            && css_dataset::tags::STANDARD_HTML_TAGS
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(self.tag_name));

        let self_closing = if helpers::is_void_element(tag_name, ctx.language.clone()) {
            ctx.options
                .html_void_self_closing
                .unwrap_or(self.self_closing)
        } else if helpers::is_html_tag(tag_name, ctx.language.clone()) {
            ctx.options
                .html_normal_self_closing
                .unwrap_or(self.self_closing)
        } else if matches!(ctx.language, Language::Vue | Language::Svelte)
            && helpers::is_component(self.tag_name)
        {
            ctx.options
                .component_self_closing
                .unwrap_or(self.self_closing)
        } else if helpers::is_svg_tag(self.tag_name, ctx.language.clone()) {
            ctx.options.svg_self_closing.unwrap_or(self.self_closing)
        } else if helpers::is_mathml_tag(self.tag_name, ctx.language.clone()) {
            ctx.options.mathml_self_closing.unwrap_or(self.self_closing)
        } else {
            self.self_closing
        };

        let mut docs = Vec::with_capacity(5);

        docs.push(Doc::text("<"));
        docs.push(Doc::text(if should_lower_cased {
            Cow::from(self.tag_name.to_ascii_lowercase())
        } else {
            Cow::from(self.tag_name)
        }));

        let attrs = if let Some(max) = ctx.options.max_attrs_per_line {
            // fix #2
            if self.attrs.is_empty() {
                Doc::line_or_nil()
            } else {
                Doc::line_or_space()
            }
            .concat(itertools::intersperse(
                self.attrs.chunks(max.into()).map(|chunk| {
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
            if self_closing {
                docs.push(Doc::line_or_space());
                docs.push(Doc::text("/>"));
            } else {
                if !ctx.options.closing_bracket_same_line {
                    docs.push(Doc::line_or_nil());
                }
                docs.push(Doc::text(">"));
            }
            return Doc::list(docs).group();
        } else if self_closing && self.children.is_empty() {
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

        let is_whitespace_sensitive = !(matches!(ctx.language, Language::Vue)
            && ctx.is_root
            && self.tag_name.eq_ignore_ascii_case("template"))
            && ctx.is_whitespace_sensitive(tag_name);
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
            format_ws_sensitive_leading_ws(&self.children)
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
            format_ws_sensitive_trailing_ws(&self.children)
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

        let is_root = mem::replace(&mut ctx.is_root, false);
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
                        .unwrap_or(if matches!(ctx.language, Language::Astro) {
                            "ts"
                        } else {
                            "js"
                        }),
                );
                let doc = Doc::hard_line().concat(reflow_with_indent(formatted.trim()));
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
                let doc = Doc::hard_line().concat(reflow_with_indent(formatted.trim()));
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
                    Node::VueInterpolation(..)
                        | Node::SvelteInterpolation(..)
                        | Node::Comment(..)
                        | Node::AstroExpr(..)
                        | Node::JinjaInterpolation(..)
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
        ctx.is_root = is_root;

        Doc::list(docs).group()
    }
}

impl<'s> DocGen<'s> for JinjaBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::list(
            self.body
                .iter()
                .map(|child| match child {
                    JinjaTagOrChildren::Tag(tag) => tag.doc(ctx),
                    JinjaTagOrChildren::Children(children) => {
                        format_control_structure_block_children(children, ctx)
                    }
                })
                .collect(),
        )
    }
}

impl<'s> DocGen<'s> for JinjaComment<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        if ctx.options.format_comments {
            Doc::text("{#")
                .append(
                    Doc::line_or_space()
                        .concat(reflow_with_indent(self.raw.trim()))
                        .nest_with_ctx(ctx),
                )
                .append(Doc::line_or_space())
                .append(Doc::text("#}"))
                .group()
        } else {
            Doc::text("{#")
                .concat(reflow_raw(self.raw))
                .append(Doc::text("#}"))
        }
    }
}

impl<'s> DocGen<'s> for JinjaInterpolation<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(
                Doc::line_or_space()
                    .append(Doc::text(self.expr.trim()))
                    .nest(ctx.indent_width),
            )
            .append(Doc::line_or_space())
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for JinjaTag<'s> {
    fn doc<E, F>(&self, _: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{%")
            .append(Doc::text(self.content))
            .append(Doc::text("%}"))
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
                Language::Svelte if !ctx.options.strict_svelte_attr => {
                    if let Some(expr) = value.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                        let formatted_expr = ctx.format_expr(expr);
                        return match self.name.split_once(':') {
                            Some((_, name))
                                if matches!(ctx.options.svelte_directive_shorthand, Some(true))
                                    && name == formatted_expr =>
                            {
                                Doc::text(self.name)
                            }
                            None if matches!(ctx.options.svelte_attr_shorthand, Some(true))
                                && self.name == formatted_expr =>
                            {
                                Doc::text("{")
                                    .concat(reflow_with_indent(&formatted_expr))
                                    .append(Doc::text("}"))
                            }
                            _ => Doc::text(self.name.to_owned())
                                .append(Doc::text("={"))
                                .concat(reflow_with_indent(&formatted_expr))
                                .append(Doc::text("}")),
                        };
                    } else {
                        Cow::from(value)
                    }
                }
                _ => Cow::from(value),
            };
            name.append(Doc::text("=")).append(format_attr_value(
                value,
                &ctx.options.quotes,
                self.name.eq_ignore_ascii_case("class"),
                false,
            ))
        } else if matches!(ctx.language, Language::Svelte)
            && matches!(ctx.options.svelte_directive_shorthand, Some(false))
        {
            if let Some((_, binding_name)) = self.name.split_once(':') {
                let value = format!("{{{binding_name}}}");
                name.append(Doc::text("="))
                    .append(if ctx.options.strict_svelte_attr {
                        format_attr_value(value, &ctx.options.quotes, false, true)
                    } else {
                        Doc::text(value)
                    })
            } else {
                name
            }
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
            Node::AstroExpr(astro_expr) => astro_expr.doc(ctx),
            Node::AstroFrontMatter(astro_front_matter) => astro_front_matter.doc(ctx),
            Node::Comment(comment) => comment.doc(ctx),
            Node::Doctype => Doc::text("<!DOCTYPE html>"),
            Node::Element(element) => element.doc(ctx),
            Node::JinjaBlock(jinja_block) => jinja_block.doc(ctx),
            Node::JinjaComment(jinja_comment) => jinja_comment.doc(ctx),
            Node::JinjaInterpolation(jinja_interpolation) => jinja_interpolation.doc(ctx),
            Node::JinjaTag(jinja_tag) => jinja_tag.doc(ctx),
            Node::SvelteAtTag(svelte_at_tag) => svelte_at_tag.doc(ctx),
            Node::SvelteAwaitBlock(svelte_await_block) => svelte_await_block.doc(ctx),
            Node::SvelteEachBlock(svelte_each_block) => svelte_each_block.doc(ctx),
            Node::SvelteIfBlock(svelte_if_block) => svelte_if_block.doc(ctx),
            Node::SvelteInterpolation(svelte_interpolation) => svelte_interpolation.doc(ctx),
            Node::SvelteKeyBlock(svelte_key_block) => svelte_key_block.doc(ctx),
            Node::TextNode(text_node) => text_node.doc(ctx),
            Node::VueInterpolation(vue_interpolation) => vue_interpolation.doc(ctx),
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

impl<'s> DocGen<'s> for SvelteAtTag<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{@")
            .append(Doc::text(self.name))
            .append(Doc::space())
            .append(Doc::text(ctx.format_expr(self.expr)))
            .append(Doc::text("}"))
    }
}

impl<'s> DocGen<'s> for SvelteAttribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let expr_code = ctx.format_expr(self.expr);
        let expr = Doc::text("{")
            .concat(reflow_with_indent(&expr_code))
            .append(Doc::text("}"));
        if let Some(name) = self.name {
            match name.split_once(':') {
                Some((_, binding_name))
                    if matches!(ctx.options.svelte_directive_shorthand, Some(true))
                        && binding_name == expr_code =>
                {
                    Doc::text(name)
                }
                None if (matches!(ctx.options.svelte_attr_shorthand, Some(true)))
                    && name == expr_code =>
                {
                    expr
                }
                _ => {
                    let name = Doc::text(name).append(Doc::text("="));
                    if ctx.options.strict_svelte_attr {
                        name.append(format_attr_value(
                            format!("{{{expr_code}}}"),
                            &ctx.options.quotes,
                            false,
                            true,
                        ))
                    } else {
                        name.append(expr)
                    }
                }
            }
        } else if matches!(ctx.options.svelte_attr_shorthand, Some(false)) {
            let name = Doc::text(expr_code.clone()).append(Doc::text("="));
            if ctx.options.strict_svelte_attr {
                name.append(format_attr_value(
                    format!("{{{expr_code}}}"),
                    &ctx.options.quotes,
                    false,
                    true,
                ))
            } else {
                name.append(expr)
            }
        } else {
            expr
        }
    }
}

impl<'s> DocGen<'s> for SvelteAwaitBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let mut head = Vec::with_capacity(5);
        head.push(Doc::text("{#await "));
        head.push(Doc::text(ctx.format_expr(self.expr)));

        if let Some(then_binding) = self.then_binding {
            head.push(Doc::line_or_space());
            head.push(Doc::text("then "));
            head.push(Doc::text(ctx.format_binding(then_binding)));
        }

        if let Some(catch_binding) = self.catch_binding {
            head.push(Doc::line_or_space());
            head.push(Doc::text("catch "));
            head.push(Doc::text(ctx.format_binding(catch_binding)));
        }

        let mut docs = Vec::with_capacity(5);
        docs.push(
            Doc::list(head)
                .nest_with_ctx(ctx)
                .append(Doc::line_or_nil())
                .append(Doc::text("}"))
                .group(),
        );
        docs.push(format_control_structure_block_children(&self.children, ctx));

        if let Some(then_block) = &self.then_block {
            docs.push(then_block.doc(ctx));
        }

        if let Some(catch_block) = &self.catch_block {
            docs.push(catch_block.doc(ctx));
        }

        docs.push(Doc::text("{/await}"));
        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for SvelteCatchBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let children = format_control_structure_block_children(&self.children, ctx);
        if let Some(binding) = self.binding {
            Doc::text("{:catch ")
                .append(Doc::text(ctx.format_binding(binding)))
                .append(Doc::text("}"))
                .append(children)
        } else {
            Doc::text("{:catch}").append(children)
        }
    }
}

impl<'s> DocGen<'s> for SvelteEachBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let mut head = Vec::with_capacity(5);
        head.push(Doc::text("{#each "));
        head.push(Doc::text(ctx.format_expr(self.expr)));
        head.push(Doc::text(" as"));
        head.push(Doc::line_or_space());
        head.push(Doc::text(ctx.format_binding(self.binding)));

        if let Some(index) = self.index {
            head.push(Doc::text(","));
            head.push(Doc::line_or_space());
            head.push(Doc::text(ctx.format_binding(index)));
        }

        if let Some(key) = self.key {
            head.push(Doc::line_or_space());
            head.push(Doc::text("("));
            head.push(Doc::text(ctx.format_expr(key)));
            head.push(Doc::text(")"));
        }

        let mut docs = Vec::with_capacity(5);
        docs.push(
            Doc::list(head)
                .nest_with_ctx(ctx)
                .append(Doc::line_or_nil())
                .append(Doc::text("}"))
                .group(),
        );
        docs.push(format_control_structure_block_children(&self.children, ctx));

        if let Some(children) = &self.else_children {
            docs.push(Doc::text("{:else}"));
            docs.push(format_control_structure_block_children(children, ctx));
        }

        docs.push(Doc::text("{/each}"));
        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for SvelteElseIfBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{:else if ")
            .append(Doc::text(ctx.format_expr(self.expr)))
            .append(Doc::text("}"))
            .append(format_control_structure_block_children(&self.children, ctx))
    }
}

impl<'s> DocGen<'s> for SvelteIfBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("{#if "));
        docs.push(Doc::text(ctx.format_expr(self.expr)));
        docs.push(Doc::text("}"));
        docs.push(format_control_structure_block_children(&self.children, ctx));

        docs.extend(self.else_if_blocks.iter().map(|block| block.doc(ctx)));

        if let Some(children) = &self.else_children {
            docs.push(Doc::text("{:else}"));
            docs.push(format_control_structure_block_children(children, ctx));
        }

        docs.push(Doc::text("{/if}"));
        Doc::list(docs)
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
                    .concat(reflow_with_indent(&ctx.format_expr(self.expr)))
                    .nest_with_ctx(ctx),
            )
            .append(Doc::line_or_nil())
            .append(Doc::text("}"))
            .group()
    }
}

impl<'s> DocGen<'s> for SvelteKeyBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{#key ")
            .append(Doc::text(ctx.format_expr(self.expr)))
            .append(Doc::text("}"))
            .append(format_control_structure_block_children(&self.children, ctx))
            .append(Doc::text("{/key}"))
    }
}

impl<'s> DocGen<'s> for SvelteThenBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, 's, E, F>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{:then ")
            .append(Doc::text(ctx.format_binding(self.binding)))
            .append(Doc::text("}"))
            .append(format_control_structure_block_children(&self.children, ctx))
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
        let mut is_v_bind = false;

        match self.name {
            ":" => {
                is_v_bind = true;
                docs.push(if let Some(VBindStyle::Long) = ctx.options.v_bind_style {
                    Doc::text("v-bind:")
                } else {
                    Doc::text(":")
                });
                if let Some(arg_and_modifiers) = self.arg_and_modifiers {
                    docs.push(Doc::text(arg_and_modifiers.trim_start_matches(':')));
                }
            }
            "bind" => {
                is_v_bind = true;
                if let Some(arg_and_modifiers) = self.arg_and_modifiers {
                    docs.push(if let Some(VBindStyle::Short) = ctx.options.v_bind_style {
                        Doc::text(":")
                    } else {
                        Doc::text("v-bind:")
                    });
                    docs.push(Doc::text(arg_and_modifiers.trim_start_matches(':')));
                } else {
                    docs.push(Doc::text("v-bind"));
                }
            }
            "@" => {
                docs.push(if let Some(VOnStyle::Long) = ctx.options.v_on_style {
                    Doc::text("v-on:")
                } else {
                    Doc::text("@")
                });
                if let Some(arg_and_modifiers) = self.arg_and_modifiers {
                    docs.push(Doc::text(arg_and_modifiers.trim_start_matches(':')));
                }
            }
            "on" => {
                if let Some(arg_and_modifiers) = self.arg_and_modifiers {
                    docs.push(if let Some(VOnStyle::Short) = ctx.options.v_on_style {
                        Doc::text("@")
                    } else {
                        Doc::text("v-on:")
                    });
                    docs.push(Doc::text(arg_and_modifiers.trim_start_matches(':')));
                } else {
                    docs.push(Doc::text("v-on"));
                }
            }
            "#" => {
                let slot = extract_slot_name(self.arg_and_modifiers);
                let style = match get_v_slot_style_option(slot, ctx) {
                    Some(VSlotStyle::Short) | None => VSlotStyle::Short,
                    Some(VSlotStyle::VSlot) if slot == "default" => VSlotStyle::VSlot,
                    Some(VSlotStyle::Long | VSlotStyle::VSlot) => VSlotStyle::Long,
                };
                docs.push(format_v_slot(style, slot));
            }
            "slot" => {
                let slot = extract_slot_name(self.arg_and_modifiers);
                let style = match get_v_slot_style_option(slot, ctx) {
                    Some(VSlotStyle::Short) => VSlotStyle::Short,
                    Some(VSlotStyle::VSlot) if slot == "default" => VSlotStyle::VSlot,
                    Some(VSlotStyle::Long | VSlotStyle::VSlot) => VSlotStyle::Long,
                    None if self.arg_and_modifiers.is_some() => VSlotStyle::Long,
                    None => VSlotStyle::VSlot,
                };
                docs.push(format_v_slot(style, slot));
            }
            name => {
                docs.push(Doc::text(format!("v-{name}")));
                if let Some(arg_and_modifiers) = self.arg_and_modifiers {
                    docs.push(Doc::text(arg_and_modifiers));
                }
            }
        };

        if let Some(value) = self.value {
            let value = match self.name {
                "for" => {
                    use crate::config::VForDelimiterStyle;
                    if let Some((left, right)) = value.split_once(" in ") {
                        let delimiter = if let Some(VForDelimiterStyle::Of) =
                            ctx.options.v_for_delimiter_style
                        {
                            "of"
                        } else {
                            "in"
                        };
                        format_v_for(left, delimiter, right, ctx)
                    } else if let Some((left, right)) = value.split_once(" of ") {
                        let delimiter = if let Some(VForDelimiterStyle::In) =
                            ctx.options.v_for_delimiter_style
                        {
                            "in"
                        } else {
                            "of"
                        };
                        format_v_for(left, delimiter, right, ctx)
                    } else {
                        ctx.format_expr(value)
                    }
                }
                "#" | "slot" => ctx.format_binding(value),
                _ => ctx.format_expr(value),
            };
            if !(matches!(ctx.options.v_bind_same_name_short_hand, Some(true))
                && is_v_bind
                && matches!(self.arg_and_modifiers, Some(arg_and_modifiers) if arg_and_modifiers == value))
            {
                docs.push(Doc::text("="));
                docs.push(format_attr_value(value, &ctx.options.quotes, false, true));
            }
        } else if matches!(ctx.options.v_bind_same_name_short_hand, Some(false)) && is_v_bind {
            if let Some(arg_and_modifiers) = self.arg_and_modifiers {
                docs.push(Doc::text("="));
                docs.push(format_attr_value(
                    arg_and_modifiers,
                    &ctx.options.quotes,
                    false,
                    true,
                ));
            }
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
                    .concat(reflow_with_indent(&ctx.format_expr(self.expr)))
                    .nest_with_ctx(ctx),
            )
            .append(Doc::line_or_space())
            .append(Doc::text("}}"))
            .group()
    }
}

fn reflow_raw(s: &str) -> impl Iterator<Item = Doc<'_>> {
    itertools::intersperse(
        s.split('\n')
            .map(|s| Doc::text(s.strip_suffix('\r').unwrap_or(s))),
        Doc::empty_line(),
    )
}

fn reflow_owned(s: &str) -> impl Iterator<Item = Doc<'static>> + '_ {
    itertools::intersperse(
        s.split('\n')
            .map(|s| Doc::text(s.strip_suffix('\r').unwrap_or(s).to_owned())),
        Doc::empty_line(),
    )
}

fn reflow_with_indent(s: &str) -> impl Iterator<Item = Doc<'static>> + '_ {
    let indent = s
        .lines()
        .skip(if s.starts_with([' ', '\t']) { 0 } else { 1 })
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.as_bytes()
                .iter()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count()
        })
        .min()
        .unwrap_or_default();
    s.split('\n').enumerate().flat_map(move |(i, s)| {
        let s = s.strip_suffix('\r').unwrap_or(s);
        let s = if s.starts_with([' ', '\t']) {
            &s[indent..]
        } else {
            s
        };
        [
            if i == 0 {
                Doc::nil()
            } else if s.trim().is_empty() {
                Doc::empty_line()
            } else {
                Doc::hard_line()
            },
            Doc::text(s.to_owned()),
        ]
        .into_iter()
    })
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

fn format_attr_value(
    value: impl AsRef<str>,
    quotes: &Quotes,
    split_whitespaces: bool,
    indent: bool,
) -> Doc<'static> {
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
    if split_whitespaces {
        quote
            .clone()
            .append(Doc::text(value.split_ascii_whitespace().join(" ")))
            .append(quote)
    } else {
        quote
            .clone()
            .append(if indent {
                Doc::list(reflow_with_indent(value).collect())
            } else {
                Doc::list(reflow_owned(value).collect())
            })
            .append(quote)
    }
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
                            | Node::SvelteInterpolation(..)
                            | Node::AstroExpr(..)
                            | Node::JinjaInterpolation(..) => true,
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

fn extract_slot_name(arg_and_modifiers: Option<&str>) -> &str {
    arg_and_modifiers
        .map(|arg| arg.strip_prefix(':').unwrap_or(arg))
        .unwrap_or("default")
}

fn get_v_slot_style_option<E, F>(slot: &str, ctx: &Ctx<E, F>) -> Option<VSlotStyle>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    let option = if ctx
        .current_tag_name
        .map(|name| name.eq_ignore_ascii_case("template"))
        .unwrap_or_default()
    {
        if slot == "default" {
            ctx.options.default_v_slot_style.clone()
        } else {
            ctx.options.named_v_slot_style.clone()
        }
    } else {
        ctx.options.component_v_slot_style.clone()
    };
    option.or(ctx.options.v_slot_style.clone())
}

fn format_v_slot(style: VSlotStyle, slot: &str) -> Doc<'_> {
    match style {
        VSlotStyle::Short => Doc::text(format!("#{slot}")),
        VSlotStyle::Long => Doc::text(format!("v-slot:{slot}")),
        VSlotStyle::VSlot => Doc::text("v-slot"),
    }
}

fn format_ws_sensitive_leading_ws<'s>(children: &[Node<'s>]) -> Doc<'s> {
    if let Some(Node::TextNode(text_node)) = children.first() {
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
}

fn format_ws_sensitive_trailing_ws<'s>(children: &[Node<'s>]) -> Doc<'s> {
    if let Some(Node::TextNode(text_node)) = children.last() {
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
}

fn format_v_for<'s, E, F>(
    left: &str,
    delimiter: &'static str,
    right: &str,
    ctx: &mut Ctx<'_, 's, E, F>,
) -> String
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    let left = ctx.format_expr(left);
    let right = ctx.format_expr(right);
    if left.contains(',') && !left.contains('(') {
        format!("({left}) {delimiter} {right}")
    } else {
        format!("{left} {delimiter} {right}")
    }
}

fn format_control_structure_block_children<'s, E, F>(
    children: &[Node<'s>],
    ctx: &mut Ctx<'_, 's, E, F>,
) -> Doc<'s>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    match children {
        [Node::TextNode(text_node)] if is_all_ascii_whitespace(text_node.raw) => {
            Doc::line_or_space()
        }
        _ => format_ws_sensitive_leading_ws(children)
            .append(format_children_without_inserting_linebreak(
                children,
                has_two_more_non_text_children(children),
                ctx,
            ))
            .nest_with_ctx(ctx)
            .append(format_ws_sensitive_trailing_ws(children)),
    }
}
