use crate::{ast::*, ctx::Ctx, helpers, Language};
use std::{borrow::Cow, path::Path};
use tiny_pretty::Doc;

pub(super) trait DocGen<'s> {
    fn doc<F>(&self, ctx: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>;
}

impl<'s> DocGen<'s> for Node<'s> {
    fn doc<F>(&self, ctx: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
    {
        match self {
            Node::Comment(comment) => comment.doc(ctx),
            Node::Element(element) => element.doc(ctx),
            Node::SvelteInterpolation(svelte_interpolation) => svelte_interpolation.doc(ctx),
            Node::TextNode(text_node) => text_node.doc(ctx),
            Node::VueInterpolation(vue_interpolation) => vue_interpolation.doc(ctx),
            _ => todo!(),
        }
    }
}

impl<'s> DocGen<'s> for Attribute<'s> {
    fn doc<F>(&self, ctx: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
    {
        match self {
            Attribute::NativeAttribute(native_attribute) => native_attribute.doc(ctx),
            Attribute::SvelteAttribute(svelte_attribute) => svelte_attribute.doc(ctx),
            Attribute::VueDirective(vue_directive) => vue_directive.doc(ctx),
        }
    }
}

impl<'s> DocGen<'s> for Comment<'s> {
    fn doc<F>(&self, _: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
    {
        Doc::text("<!--")
            .concat(reflow(self.raw))
            .append(Doc::text("-->"))
    }
}

impl<'s> DocGen<'s> for Element<'s> {
    fn doc<F>(&self, ctx: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
    {
        let mut docs = Vec::with_capacity(5);

        docs.push(Doc::text("<"));
        docs.push(Doc::text(self.tag_name));

        let attrs = Doc::list(
            self.attrs
                .iter()
                .map(|prop| Doc::line_or_space().append(prop.doc(ctx)))
                .collect(),
        )
        .nest(ctx.indent_width);

        if self.void_element {
            docs.push(attrs);
            docs.push(Doc::line_or_nil());
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

        let is_whitespace_sensitive = match ctx.language {
            Language::Html => helpers::is_whitespace_sensitive_tag(self.tag_name),
            Language::Vue | Language::Svelte => {
                !helpers::is_component(self.tag_name)
                    && helpers::is_whitespace_sensitive_tag(self.tag_name)
            }
        };
        let has_two_more_non_text_children = self
            .children
            .iter()
            .filter(|child| !matches!(child, Node::TextNode(_)))
            .count()
            > 1;

        let leading_ws = if is_whitespace_sensitive {
            if let Some(Node::TextNode(text_node)) = self.children.first() {
                if text_node.raw.starts_with(|c: char| c.is_ascii_whitespace()) {
                    Doc::line_or_space()
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
                    Node::TextNode(text_node) => text_node.raw.contains('\n'),
                    _ => false,
                })
                .unwrap_or_default()
        {
            Doc::hard_line()
        } else {
            Doc::line_or_nil()
        };
        let trailing_ws = if is_whitespace_sensitive {
            if let Some(Node::TextNode(text_node)) = self.children.last() {
                if text_node.raw.ends_with(|c: char| c.is_ascii_whitespace()) {
                    Doc::line_or_space()
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
                    Node::TextNode(text_node) => text_node.raw.contains('\n'),
                    _ => false,
                })
                .unwrap_or_default()
        {
            Doc::hard_line()
        } else {
            Doc::line_or_nil()
        };

        if self.tag_name.eq_ignore_ascii_case("script") {
            if let [Node::TextNode(text_node)] = &self.children[..] {
                let formatted = ctx.format_with_external_formatter(
                    match self.attrs.iter().find_map(|attr| match attr {
                        Attribute::NativeAttribute(native_attribute)
                            if native_attribute.name.eq_ignore_ascii_case("lang") =>
                        {
                            native_attribute.value
                        }
                        _ => None,
                    }) {
                        Some("ts") => Path::new("script.ts"),
                        Some("tsx") => Path::new("script.tsx"),
                        Some("jsx") => Path::new("script.jsx"),
                        _ => Path::new("style.js"),
                    },
                    text_node.raw,
                );
                let doc = Doc::hard_line()
                    .concat(reflow(formatted.trim()))
                    .append(Doc::hard_line());
                docs.push(if ctx.options.script_indent {
                    doc.nest(ctx.indent_width)
                } else {
                    doc
                });
            }
        } else if self.tag_name.eq_ignore_ascii_case("style") {
            if let [Node::TextNode(text_node)] = &self.children[..] {
                let formatted = ctx.format_with_external_formatter(
                    match self.attrs.iter().find_map(|attr| match attr {
                        Attribute::NativeAttribute(native_attribute)
                            if native_attribute.name.eq_ignore_ascii_case("lang") =>
                        {
                            native_attribute.value
                        }
                        _ => None,
                    }) {
                        Some("scss") => Path::new("style.scss"),
                        Some("sass") => Path::new("style.sass"),
                        Some("less") => Path::new("style.less"),
                        _ => Path::new("style.css"),
                    },
                    text_node.raw,
                );
                let doc = Doc::hard_line()
                    .concat(reflow(formatted.trim()))
                    .append(Doc::hard_line());
                docs.push(if ctx.options.style_indent {
                    doc.nest(ctx.indent_width)
                } else {
                    doc
                });
            }
        } else if !is_whitespace_sensitive && has_two_more_non_text_children {
            docs.push(
                leading_ws
                    .append(
                        Doc::list(
                            itertools::intersperse(
                                self.children.iter().filter_map(|child| match child {
                                    Node::TextNode(text_node) => {
                                        if text_node
                                            .raw
                                            .as_bytes()
                                            .iter()
                                            .filter(|byte| **byte == b'\n')
                                            .count()
                                            > 1
                                        {
                                            // line break will be inserted later
                                            // by `itertools::intersperse`
                                            Some(Doc::nil())
                                        } else if text_node.raw.trim().is_empty() {
                                            None
                                        } else {
                                            Some(text_node.doc(ctx))
                                        }
                                    }
                                    node => Some(node.doc(ctx)),
                                }),
                                Doc::hard_line(),
                            )
                            .collect(),
                        )
                        .group(),
                    )
                    .nest(ctx.indent_width),
            );
            docs.push(trailing_ws);
        } else {
            docs.push(
                leading_ws
                    .append(
                        Doc::list(self.children.iter().map(|child| child.doc(ctx)).collect())
                            .group(),
                    )
                    .nest(ctx.indent_width),
            );
            docs.push(trailing_ws);
        }

        docs.push(
            Doc::text("</")
                .append(Doc::text(self.tag_name))
                .append(Doc::line_or_nil())
                .append(Doc::text(">"))
                .group(),
        );

        Doc::list(docs).group()
    }
}

impl<'s> DocGen<'s> for NativeAttribute<'s> {
    fn doc<F>(&self, ctx: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
    {
        let name = Doc::text(self.name);
        if let Some(value) = self.value {
            if matches!(ctx.language, Language::Svelte) {
                if let Some(expr) = value.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                    return name
                        .append(Doc::text("={"))
                        .append(Doc::text(expr))
                        .append(Doc::text("}"));
                }
            }
            name.append(Doc::text("="))
                .append(Doc::text("\""))
                .append(Doc::text(value))
                .append(Doc::text("\""))
        } else {
            name
        }
    }
}

impl<'s> DocGen<'s> for Root<'s> {
    fn doc<F>(&self, ctx: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
    {
        Doc::list(
            itertools::intersperse(
                self.children.iter().filter_map(|child| match child {
                    Node::TextNode(text_node) => {
                        if text_node
                            .raw
                            .as_bytes()
                            .iter()
                            .filter(|byte| **byte == b'\n')
                            .count()
                            > 1
                        {
                            // line break will be inserted later
                            // by `itertools::intersperse`
                            Some(Doc::nil())
                        } else if text_node.raw.trim().is_empty() {
                            None
                        } else {
                            Some(text_node.doc(ctx))
                        }
                    }
                    node => Some(node.doc(ctx)),
                }),
                Doc::hard_line(),
            )
            .collect(),
        )
        .append(Doc::hard_line())
    }
}

impl<'s> DocGen<'s> for SvelteAttribute<'s> {
    fn doc<F>(&self, _: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
    {
        let name = Doc::text(self.name);
        name.append(Doc::text("={"))
            .append(Doc::text(self.expr))
            .append(Doc::text("}"))
    }
}

impl<'s> DocGen<'s> for SvelteInterpolation<'s> {
    fn doc<F>(&self, ctx: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
    {
        Doc::text("{")
            .append(
                Doc::line_or_nil()
                    .append(Doc::text(self.expr.trim()))
                    .nest(ctx.indent_width),
            )
            .append(Doc::line_or_nil())
            .append(Doc::text("}"))
            .group()
    }
}

impl<'s> DocGen<'s> for TextNode<'s> {
    fn doc<F>(&self, _: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
    {
        let docs = self
            .raw
            .split('\n')
            .map(|s| s.strip_suffix('\r').unwrap_or(s))
            .flat_map(|line| {
                itertools::intersperse(
                    line.split_ascii_whitespace().map(Doc::text),
                    Doc::line_or_space(),
                )
            })
            .collect::<Vec<_>>();

        if docs.is_empty() {
            Doc::nil()
        } else {
            Doc::list(docs).group()
        }
    }
}

impl<'s> DocGen<'s> for VueDirective<'s> {
    fn doc<F>(&self, ctx: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
    {
        use crate::config::{VBindStyle, VOnStyle};

        let mut docs = Vec::with_capacity(5);

        let mut is_short_hand = false;
        docs.push(match self.name {
            ":" => {
                if let Some(VBindStyle::Long) = ctx.options.v_bind_style {
                    Doc::text("v-bind")
                } else {
                    is_short_hand = true;
                    Doc::text(":")
                }
            }
            "bind" if self.arg_and_modifiers.is_some() => {
                if let Some(VBindStyle::Short) = ctx.options.v_bind_style {
                    is_short_hand = true;
                    Doc::text(":")
                } else {
                    Doc::text("v-bind")
                }
            }
            "@" => {
                if let Some(VOnStyle::Long) = ctx.options.v_on_style {
                    Doc::text("v-on")
                } else {
                    is_short_hand = true;
                    Doc::text("@")
                }
            }
            "on" => {
                if let Some(VOnStyle::Short) = ctx.options.v_on_style {
                    is_short_hand = true;
                    Doc::text("@")
                } else {
                    Doc::text("v-on")
                }
            }
            "#" => {
                is_short_hand = true;
                Doc::text("#")
            }
            name => Doc::text(format!("v-{name}")),
        });

        if let Some(arg_and_modifiers) = self.arg_and_modifiers {
            if !is_short_hand {
                docs.push(Doc::text(":"));
            }
            docs.push(Doc::text(arg_and_modifiers));
        }

        if let Some(value) = self.value {
            // TODO: should be formatted as JS
            docs.push(Doc::text("=\""));
            docs.push(Doc::text(value));
            docs.push(Doc::text("\""));
        }

        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for VueInterpolation<'s> {
    fn doc<F>(&self, ctx: &Ctx<F>) -> Doc<'s>
    where
        F: for<'a> Fn(&Path, &'a str) -> Cow<'a, str>,
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

fn reflow(s: &str) -> impl Iterator<Item = Doc<'static>> + '_ {
    itertools::intersperse(
        s.split('\n')
            .map(|s| Doc::text(s.strip_suffix('\r').unwrap_or(s).to_owned())),
        Doc::hard_line(),
    )
}
