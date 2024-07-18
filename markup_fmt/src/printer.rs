use crate::{
    ast::*,
    config::{Quotes, VSlotStyle, WhitespaceSensitivity},
    ctx::{Ctx, NestWithCtx},
    helpers,
    state::State,
    Language,
};
use itertools::Itertools;
use std::{borrow::Cow, path::Path};
use tiny_pretty::Doc;

pub(super) trait DocGen<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>;
}

impl<'s> DocGen<'s> for AngularCase<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("@case (")
            .append(Doc::text(ctx.format_general_expr(self.expr)))
            .append(Doc::text(") {"))
            .append(format_control_structure_block_children(
                &self.children,
                ctx,
                state,
            ))
            .append(Doc::text("}"))
    }
}

impl<'s> DocGen<'s> for AngularElseIf<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("@else if ("));
        docs.push(Doc::text(ctx.format_general_expr(self.expr)));
        if let Some(reference) = self.reference {
            docs.push(Doc::text("; as "));
            docs.push(Doc::text(ctx.format_binding(reference)));
        }
        docs.push(Doc::text(") {"));
        docs.push(format_control_structure_block_children(
            &self.children,
            ctx,
            state,
        ));
        docs.push(Doc::text("}"));
        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for AngularFor<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("@for ("));
        docs.push(Doc::text(ctx.format_binding(self.binding)));
        docs.push(Doc::text(" of "));
        docs.push(Doc::text(ctx.format_general_expr(self.expr)));
        if let Some(track) = self.track {
            docs.push(Doc::text("; track "));
            docs.push(Doc::text(ctx.format_general_expr(track)));
        }
        if let Some(aliases) = self.aliases {
            docs.push(Doc::text("; "));
            docs.extend(reflow_with_indent(
                ctx.format_script(aliases, "js")
                    .trim()
                    .trim_end_matches(';'),
            ));
        }
        docs.push(Doc::text(") {"));
        docs.push(format_control_structure_block_children(
            &self.children,
            ctx,
            state,
        ));
        docs.push(Doc::text("}"));

        if let Some(children) = &self.empty {
            docs.push(Doc::text(" @empty {"));
            docs.push(format_control_structure_block_children(
                children, ctx, state,
            ));
            docs.push(Doc::text("}"));
        }

        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for AngularIf<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("@if ("));
        docs.push(Doc::text(ctx.format_general_expr(self.expr)));
        if let Some(reference) = self.reference {
            docs.push(Doc::text("; as "));
            docs.push(Doc::text(ctx.format_binding(reference)));
        }
        docs.push(Doc::text(") {"));
        docs.push(format_control_structure_block_children(
            &self.children,
            ctx,
            state,
        ));
        docs.push(Doc::text("}"));

        docs.extend(
            self.else_if_blocks
                .iter()
                .flat_map(|block| [Doc::space(), block.doc(ctx, state)]),
        );

        if let Some(children) = &self.else_children {
            docs.push(Doc::text(" @else {"));
            docs.push(format_control_structure_block_children(
                children, ctx, state,
            ));
            docs.push(Doc::text("}"));
        }

        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for AngularInterpolation<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(Doc::line_or_space())
            .concat(reflow_with_indent(&ctx.format_general_expr(self.expr)))
            .nest_with_ctx(ctx)
            .append(Doc::line_or_space())
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for AngularLet<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("@let ")
            .append(Doc::text(self.name))
            .append(Doc::text(" = "))
            .append(Doc::text(ctx.format_general_expr(self.expr)))
            .append(Doc::text(";"))
    }
}

impl<'s> DocGen<'s> for AngularSwitch<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("@switch ("));
        docs.push(Doc::text(ctx.format_general_expr(self.expr)));
        docs.push(Doc::text(") {"));

        docs.extend(
            self.cases
                .iter()
                .flat_map(|case| [Doc::hard_line(), case.doc(ctx, state)]),
        );

        if let Some(default) = self.default.as_ref() {
            docs.push(Doc::hard_line());
            docs.push(Doc::text("@default {"));
            docs.push(format_control_structure_block_children(default, ctx, state));
            docs.push(Doc::text("}"));
        }

        Doc::list(docs)
            .nest_with_ctx(ctx)
            .append(Doc::hard_line())
            .append(Doc::text("}"))
    }
}

impl<'s> DocGen<'s> for AstroAttribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let expr_code = ctx.format_general_expr(self.expr);
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
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
        let formatted_script = ctx.format_general_expr(&script);

        let templates = self.children.iter().filter_map(|child| {
            if let AstroExprChild::Template(nodes) = child {
                Some(
                    Doc::flat_or_break(Doc::nil(), Doc::text("("))
                        .append(Doc::line_or_nil())
                        .append(format_children_without_inserting_linebreak(
                            nodes,
                            has_two_more_non_text_children(nodes),
                            ctx,
                            state,
                        ))
                        .nest_with_ctx(ctx)
                        .append(Doc::line_or_nil())
                        .append(Doc::flat_or_break(Doc::nil(), Doc::text(")")))
                        .group(),
                )
            } else {
                None
            }
        });

        let doc = Doc::text("{")
            .append(Doc::line_or_nil())
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
            .nest_with_ctx(ctx)
            .append(Doc::line_or_nil())
            .append(Doc::text("}"));
        if script.contains("//") {
            doc
        } else {
            doc.group()
        }
    }
}

impl<'s> DocGen<'s> for Attribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        match self {
            Attribute::Native(native_attribute) => native_attribute.doc(ctx, state),
            Attribute::Svelte(svelte_attribute) => svelte_attribute.doc(ctx, state),
            Attribute::VueDirective(vue_directive) => vue_directive.doc(ctx, state),
            Attribute::Astro(astro_attribute) => astro_attribute.doc(ctx, state),
        }
    }
}

impl<'s> DocGen<'s> for Comment<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        if ctx.options.format_comments {
            Doc::text("<!--")
                .append(Doc::line_or_space())
                .concat(reflow_with_indent(self.raw.trim()))
                .nest_with_ctx(ctx)
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

impl<'s> DocGen<'s> for Doctype<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        use crate::config::DoctypeKeywordCase;

        Doc::text("<!")
            .append(match ctx.options.doctype_keyword_case {
                DoctypeKeywordCase::Ignore => Doc::text(self.keyword),
                DoctypeKeywordCase::Upper => Doc::text("DOCTYPE"),
                DoctypeKeywordCase::Lower => Doc::text("doctype"),
            })
            .append(Doc::space())
            .append(Doc::text(if self.value.eq_ignore_ascii_case("html") {
                "html"
            } else {
                self.value
            }))
            .append(Doc::text(">"))
    }
}

impl<'s> DocGen<'s> for Element<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let tag_name = self
            .tag_name
            .split_once(':')
            .and_then(|(namespace, name)| namespace.eq_ignore_ascii_case("html").then_some(name))
            .unwrap_or(self.tag_name);
        let is_root = state.is_root;
        let state = State {
            current_tag_name: Some(tag_name),
            is_root: false,
            in_svg: tag_name.eq_ignore_ascii_case("svg"),
        };
        let should_lower_cased = matches!(
            ctx.language,
            Language::Html | Language::Jinja | Language::Vento
        ) && css_dataset::tags::STANDARD_HTML_TAGS
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
        } else if matches!(
            ctx.language,
            Language::Vue | Language::Svelte | Language::Angular
        ) && helpers::is_component(self.tag_name)
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
        let is_whitespace_sensitive = !(matches!(ctx.language, Language::Vue)
            && is_root
            && self.tag_name.eq_ignore_ascii_case("template")
            || state.in_svg)
            && ctx.is_whitespace_sensitive(tag_name);
        let is_empty = is_empty_element(&self.children, is_whitespace_sensitive);

        let mut docs = Vec::with_capacity(5);

        docs.push(Doc::text("<"));
        docs.push(Doc::text(if should_lower_cased {
            Cow::from(self.tag_name.to_ascii_lowercase())
        } else {
            Cow::from(self.tag_name)
        }));

        let attrs_sep = if !self.first_attr_same_line
            && !ctx.options.prefer_attrs_single_line
            && self.attrs.len() > 1
            && !ctx
                .options
                .max_attrs_per_line
                .map(|value| value.get() > 1)
                .unwrap_or_default()
        {
            Doc::hard_line()
        } else {
            Doc::line_or_space()
        };
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
                            chunk.iter().map(|attr| attr.doc(ctx, &state)),
                            attrs_sep.clone(),
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
                    .flat_map(|attr| [attrs_sep.clone(), attr.doc(ctx, &state)].into_iter())
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
        }
        if self_closing && is_empty {
            docs.push(attrs);
            docs.push(Doc::line_or_space());
            docs.push(Doc::text("/>"));
            return Doc::list(docs).group();
        }
        if ctx.options.closing_bracket_same_line {
            docs.push(attrs.append(Doc::text(">")).group());
        } else {
            // for #16
            if is_whitespace_sensitive
                && !self.attrs.is_empty() // there're no attributes, so don't insert line break
                && self
                    .children
                    .first()
                    .is_some_and(|child| {
                        if let Node::Text(text_node) = child {
                            !text_node.raw.starts_with(|c: char| c.is_ascii_whitespace())
                        } else {
                            false
                        }
                    })
                && self
                    .children
                    .last()
                    .is_some_and(|child| {
                        if let Node::Text(text_node) = child {
                            !text_node.raw.ends_with(|c: char| c.is_ascii_whitespace())
                        } else {
                            false
                        }
                    })
            {
                docs.push(
                    attrs
                        .group()
                        .append(Doc::line_or_nil())
                        .append(Doc::text(">")),
                );
            } else {
                docs.push(
                    attrs
                        .append(Doc::line_or_nil())
                        .append(Doc::text(">"))
                        .group(),
                );
            }
        }

        let has_two_more_non_text_children = has_two_more_non_text_children(&self.children);

        let (leading_ws, trailing_ws) = if is_empty {
            (Doc::nil(), Doc::nil())
        } else if is_whitespace_sensitive {
            (
                format_ws_sensitive_leading_ws(&self.children),
                format_ws_sensitive_trailing_ws(&self.children),
            )
        } else if has_two_more_non_text_children {
            (Doc::hard_line(), Doc::hard_line())
        } else {
            (
                format_ws_insensitive_leading_ws(&self.children),
                format_ws_insensitive_trailing_ws(&self.children),
            )
        };

        if tag_name.eq_ignore_ascii_case("script") {
            if let [Node::Text(text_node)] = &self.children[..] {
                if text_node.raw.chars().all(|c| c.is_ascii_whitespace()) {
                    docs.push(Doc::hard_line());
                } else {
                    let is_json = self.attrs.iter().any(|attr| {
                        if let Attribute::Native(native_attr) = attr {
                            native_attr.name.eq_ignore_ascii_case("type")
                                && native_attr
                                    .value
                                    .map(|value| {
                                        value == "importmap" || value == "application/json"
                                    })
                                    .unwrap_or_default()
                        } else {
                            false
                        }
                    });
                    let formatted = if is_json {
                        ctx.format_json(text_node.raw)
                    } else {
                        ctx.format_script(
                            text_node.raw,
                            self.attrs
                                .iter()
                                .find_map(|attr| match attr {
                                    Attribute::Native(native_attribute)
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
                        )
                    };
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
            }
        } else if tag_name.eq_ignore_ascii_case("style") {
            if let [Node::Text(text_node)] = &self.children[..] {
                if text_node.raw.chars().all(|c| c.is_ascii_whitespace()) {
                    docs.push(Doc::hard_line());
                } else {
                    let formatted = ctx.format_style(
                        text_node.raw,
                        self.attrs
                            .iter()
                            .find_map(|attr| match attr {
                                Attribute::Native(native_attribute)
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
            }
        } else if tag_name.eq_ignore_ascii_case("pre") || tag_name.eq_ignore_ascii_case("textarea")
        {
            if let [Node::Text(text_node)] = &self.children[..] {
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
                format_children_with_inserting_linebreak(&self.children, ctx, &state)
                    .nest_with_ctx(ctx),
            );
            docs.push(trailing_ws);
        } else if is_whitespace_sensitive
            && matches!(&self.children[..], [Node::Text(text_node)] if is_all_ascii_whitespace(text_node.raw))
        {
            docs.push(Doc::line_or_space());
        } else {
            let children_doc = leading_ws.append(format_children_without_inserting_linebreak(
                &self.children,
                has_two_more_non_text_children,
                ctx,
                &state,
            ));
            if is_whitespace_sensitive
                && self.children.iter().all(|child| {
                    matches!(
                        child,
                        Node::VueInterpolation(..)
                            | Node::SvelteInterpolation(..)
                            | Node::Comment(..)
                            | Node::AstroExpr(..)
                            | Node::JinjaInterpolation(..)
                            | Node::VentoInterpolation(..)
                    )
                })
            {
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

        Doc::list(docs).group()
    }
}

impl<'s> DocGen<'s> for FrontMatter<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        if matches!(ctx.language, Language::Astro) {
            let formatted = ctx.format_script(self.raw, "tsx");
            Doc::text("---")
                .append(Doc::hard_line())
                .concat(reflow_with_indent(formatted.trim()))
                .append(Doc::hard_line())
                .append(Doc::text("---"))
        } else {
            Doc::text("---")
                .concat(reflow_raw(self.raw))
                .append(Doc::text("---"))
        }
    }
}

impl<'s> DocGen<'s> for JinjaBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::list(
            self.body
                .iter()
                .map(|child| match child {
                    JinjaTagOrChildren::Tag(tag) => tag.doc(ctx, state),
                    JinjaTagOrChildren::Children(children) => {
                        format_control_structure_block_children(children, ctx, state)
                    }
                })
                .collect(),
        )
    }
}

impl<'s> DocGen<'s> for JinjaComment<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        if ctx.options.format_comments {
            Doc::text("{#")
                .append(Doc::line_or_space())
                .concat(reflow_with_indent(self.raw.trim()))
                .nest_with_ctx(ctx)
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(Doc::line_or_space())
            .append(Doc::text(self.expr.trim()))
            .nest(ctx.indent_width)
            .append(Doc::line_or_space())
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for JinjaTag<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let (prefix, content) = self
            .content
            .strip_prefix('-')
            .map(|content| ("-", content))
            .unwrap_or(("", self.content));
        let (content, suffix) = self
            .content
            .strip_suffix('-')
            .map(|content| (content, "-"))
            .unwrap_or((content, ""));
        Doc::text("{%")
            .append(Doc::text(prefix))
            .append(Doc::line_or_space())
            .append(Doc::text(content.trim()))
            .nest(ctx.indent_width)
            .append(Doc::line_or_space())
            .append(Doc::text(suffix))
            .append(Doc::text("%}"))
            .group()
    }
}

impl<'s> DocGen<'s> for NativeAttribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let name = Doc::text(self.name);
        if let Some(value) = self.value {
            let value = match ctx.language {
                Language::Vue => {
                    if state
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
                        let formatted_expr = ctx.format_general_expr(expr);
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
                Language::Angular
                    if self.name.starts_with(['[', '(']) && self.name.ends_with([']', ')']) =>
                {
                    Cow::from(ctx.format_general_expr(value))
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        match self {
            Node::AngularFor(angular_for) => angular_for.doc(ctx, state),
            Node::AngularIf(angular_if) => angular_if.doc(ctx, state),
            Node::AngularInterpolation(angular_interpolation) => {
                angular_interpolation.doc(ctx, state)
            }
            Node::AngularLet(angular_let) => angular_let.doc(ctx, state),
            Node::AngularSwitch(angular_switch) => angular_switch.doc(ctx, state),
            Node::AstroExpr(astro_expr) => astro_expr.doc(ctx, state),
            Node::Comment(comment) => comment.doc(ctx, state),
            Node::Doctype(doctype) => doctype.doc(ctx, state),
            Node::Element(element) => element.doc(ctx, state),
            Node::FrontMatter(front_matter) => front_matter.doc(ctx, state),
            Node::JinjaBlock(jinja_block) => jinja_block.doc(ctx, state),
            Node::JinjaComment(jinja_comment) => jinja_comment.doc(ctx, state),
            Node::JinjaInterpolation(jinja_interpolation) => jinja_interpolation.doc(ctx, state),
            Node::JinjaTag(jinja_tag) => jinja_tag.doc(ctx, state),
            Node::SvelteAtTag(svelte_at_tag) => svelte_at_tag.doc(ctx, state),
            Node::SvelteAwaitBlock(svelte_await_block) => svelte_await_block.doc(ctx, state),
            Node::SvelteEachBlock(svelte_each_block) => svelte_each_block.doc(ctx, state),
            Node::SvelteIfBlock(svelte_if_block) => svelte_if_block.doc(ctx, state),
            Node::SvelteInterpolation(svelte_interpolation) => svelte_interpolation.doc(ctx, state),
            Node::SvelteKeyBlock(svelte_key_block) => svelte_key_block.doc(ctx, state),
            Node::Text(text_node) => text_node.doc(ctx, state),
            Node::VentoBlock(vento_block) => vento_block.doc(ctx, state),
            Node::VentoComment(vento_comment) => vento_comment.doc(ctx, state),
            Node::VentoEval(vento_eval) => vento_eval.doc(ctx, state),
            Node::VentoInterpolation(vento_interpolation) => vento_interpolation.doc(ctx, state),
            Node::VentoTag(vento_tag) => vento_tag.doc(ctx, state),
            Node::VueInterpolation(vue_interpolation) => vue_interpolation.doc(ctx, state),
        }
    }
}

impl<'s> DocGen<'s> for Root<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let is_whole_document_like = self.children.iter().any(|child| match child {
            Node::Doctype(..) => true,
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
            format_children_with_inserting_linebreak(&self.children, ctx, state)
                .append(Doc::hard_line())
        } else {
            format_children_without_inserting_linebreak(
                &self.children,
                has_two_more_non_text_children,
                ctx,
                state,
            )
            .append(Doc::hard_line())
        }
    }
}

impl<'s> DocGen<'s> for SvelteAtTag<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{@")
            .append(Doc::text(self.name))
            .append(Doc::space())
            .append(Doc::text(ctx.format_general_expr(self.expr)))
            .append(Doc::text("}"))
    }
}

impl<'s> DocGen<'s> for SvelteAttribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let expr_code = ctx.format_general_expr(self.expr);
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let mut head = Vec::with_capacity(5);
        head.push(Doc::text("{#await "));
        head.push(Doc::text(ctx.format_general_expr(self.expr)));

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
        docs.push(format_control_structure_block_children(
            &self.children,
            ctx,
            state,
        ));

        if let Some(then_block) = &self.then_block {
            docs.push(then_block.doc(ctx, state));
        }

        if let Some(catch_block) = &self.catch_block {
            docs.push(catch_block.doc(ctx, state));
        }

        docs.push(Doc::text("{/await}"));
        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for SvelteCatchBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let children = format_control_structure_block_children(&self.children, ctx, state);
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let mut head = Vec::with_capacity(5);
        head.push(Doc::text("{#each "));
        head.push(Doc::text(ctx.format_general_expr(self.expr)));
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
            head.push(Doc::text(ctx.format_general_expr(key)));
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
        docs.push(format_control_structure_block_children(
            &self.children,
            ctx,
            state,
        ));

        if let Some(children) = &self.else_children {
            docs.push(Doc::text("{:else}"));
            docs.push(format_control_structure_block_children(
                children, ctx, state,
            ));
        }

        docs.push(Doc::text("{/each}"));
        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for SvelteElseIfBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{:else if ")
            .append(Doc::text(ctx.format_general_expr(self.expr)))
            .append(Doc::text("}"))
            .append(format_control_structure_block_children(
                &self.children,
                ctx,
                state,
            ))
    }
}

impl<'s> DocGen<'s> for SvelteIfBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("{#if "));
        docs.push(Doc::text(ctx.format_general_expr(self.expr)));
        docs.push(Doc::text("}"));
        docs.push(format_control_structure_block_children(
            &self.children,
            ctx,
            state,
        ));

        docs.extend(
            self.else_if_blocks
                .iter()
                .map(|block| block.doc(ctx, state)),
        );

        if let Some(children) = &self.else_children {
            docs.push(Doc::text("{:else}"));
            docs.push(format_control_structure_block_children(
                children, ctx, state,
            ));
        }

        docs.push(Doc::text("{/if}"));
        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for SvelteInterpolation<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{")
            .append(Doc::line_or_nil())
            .concat(reflow_with_indent(&ctx.format_general_expr(self.expr)))
            .nest_with_ctx(ctx)
            .append(Doc::line_or_nil())
            .append(Doc::text("}"))
            .group()
    }
}

impl<'s> DocGen<'s> for SvelteKeyBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{#key ")
            .append(Doc::text(ctx.format_general_expr(self.expr)))
            .append(Doc::text("}"))
            .append(format_control_structure_block_children(
                &self.children,
                ctx,
                state,
            ))
            .append(Doc::text("{/key}"))
    }
}

impl<'s> DocGen<'s> for SvelteThenBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{:then ")
            .append(Doc::text(ctx.format_binding(self.binding)))
            .append(Doc::text("}"))
            .append(format_control_structure_block_children(
                &self.children,
                ctx,
                state,
            ))
    }
}

impl<'s> DocGen<'s> for TextNode<'s> {
    fn doc<E, F>(&self, _: &mut Ctx<E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        // for #16
        Doc::flat_or_break(Doc::text(self.raw.split_ascii_whitespace().join(" ")), {
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
        })
    }
}

impl<'s> DocGen<'s> for VentoBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::list(
            self.body
                .iter()
                .map(|child| child.doc(ctx, state))
                .collect(),
        )
    }
}

impl<'s> DocGen<'s> for VentoComment<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        if ctx.options.format_comments {
            Doc::text("{{#")
                .append(Doc::line_or_space())
                .concat(reflow_with_indent(self.raw.trim()))
                .nest_with_ctx(ctx)
                .append(Doc::line_or_space())
                .append(Doc::text("#}}"))
                .group()
        } else {
            Doc::text("{{#")
                .concat(reflow_raw(self.raw))
                .append(Doc::text("#}}"))
        }
    }
}

impl<'s> DocGen<'s> for VentoEval<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{>")
            .append(Doc::line_or_space())
            .concat(reflow_with_indent(
                ctx.format_script(self.raw, "js")
                    .trim()
                    .trim_end_matches(';'),
            ))
            .nest_with_ctx(ctx)
            .append(Doc::line_or_space())
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for VentoInterpolation<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(Doc::line_or_space())
            .concat(itertools::intersperse(
                self.expr.split("|>").map(|expr| {
                    Doc::list(reflow_with_indent(&ctx.format_general_expr(expr)).collect())
                }),
                Doc::line_or_space()
                    .append(Doc::text("|>"))
                    .append(Doc::space()),
            ))
            .nest_with_ctx(ctx)
            .append(Doc::line_or_space())
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for VentoTag<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(Doc::line_or_space())
            .concat(itertools::intersperse(
                self.tag.split("|>").map(|item| {
                    let parsed_tag = helpers::parse_vento_tag(item);
                    if let ("if", rest) = parsed_tag {
                        format_vento_stmt_header("if", "if", rest, ctx)
                    } else if let ("else", rest) = parsed_tag {
                        if let ("if", rest) = helpers::parse_vento_tag(rest) {
                            format_vento_stmt_header("else if", "if", rest, ctx)
                        } else {
                            Doc::text("else")
                        }
                    } else if let ("for", rest) = parsed_tag {
                        let (keyword, rest) =
                            if let ("await", rest) = helpers::parse_vento_tag(rest) {
                                ("for await", rest)
                            } else {
                                ("for", rest)
                            };
                        format_vento_stmt_header(keyword, keyword, rest, ctx)
                    } else if let (tag_name @ ("include" | "layout"), rest) = parsed_tag {
                        let mut brace_index = None;
                        let mut quotes_stack = vec![];
                        for (index, char) in rest.char_indices() {
                            match char {
                                '\'' | '"' | '`' => {
                                    if quotes_stack.last().is_some_and(|last| *last == char) {
                                        quotes_stack.pop();
                                    } else {
                                        quotes_stack.push(char);
                                    }
                                }
                                '{' => {
                                    if quotes_stack.is_empty() {
                                        brace_index = Some(index);
                                        break;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if let Some(index) = brace_index {
                            let (template, data) = rest.split_at(index);
                            Doc::text(tag_name.to_string())
                                .append(Doc::space())
                                .concat(reflow_with_indent(&ctx.format_general_expr(template)))
                                .append(Doc::text(" "))
                                .concat(reflow_with_indent(&ctx.format_general_expr(data)))
                        } else {
                            Doc::text(tag_name.to_string())
                                .append(Doc::space())
                                .concat(reflow_with_indent(&ctx.format_general_expr(parsed_tag.1)))
                        }
                    } else if parsed_tag.0 == "function" || parsed_tag.1.starts_with("function") {
                        // unsupported at present
                        Doc::list(reflow_with_indent(item.trim()).collect())
                    } else if let (tag_name @ ("set" | "export"), rest) = parsed_tag {
                        if let Some((binding, expr)) = rest.trim().split_once('=') {
                            Doc::text(tag_name.to_string())
                                .append(Doc::space())
                                .concat(reflow_with_indent(&ctx.format_binding(binding)))
                                .append(Doc::text(" = "))
                                .concat(reflow_with_indent(&ctx.format_general_expr(expr)))
                        } else {
                            Doc::text(tag_name.to_string())
                                .append(Doc::space())
                                .concat(reflow_with_indent(&ctx.format_binding(rest)))
                        }
                    } else if let ("import", _) = parsed_tag {
                        Doc::list(
                            reflow_with_indent(
                                ctx.format_script(item, "js").trim().trim_end_matches(';'),
                            )
                            .collect(),
                        )
                    } else {
                        Doc::list(reflow_with_indent(item.trim()).collect())
                    }
                }),
                Doc::line_or_space()
                    .append(Doc::text("|>"))
                    .append(Doc::space()),
            ))
            .nest_with_ctx(ctx)
            .append(Doc::line_or_space())
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for VentoTagOrChildren<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'_, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        match self {
            VentoTagOrChildren::Tag(tag) => tag.doc(ctx, state),
            VentoTagOrChildren::Children(children) => {
                format_control_structure_block_children(children, ctx, state)
            }
        }
    }
}

impl<'s> DocGen<'s> for VueDirective<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>, state: &State<'s>) -> Doc<'s>
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
                let style = match get_v_slot_style_option(slot, ctx, state) {
                    Some(VSlotStyle::Short) | None => VSlotStyle::Short,
                    Some(VSlotStyle::VSlot) if slot == "default" => VSlotStyle::VSlot,
                    Some(VSlotStyle::Long | VSlotStyle::VSlot) => VSlotStyle::Long,
                };
                docs.push(format_v_slot(style, slot));
            }
            "slot" => {
                let slot = extract_slot_name(self.arg_and_modifiers);
                let style = match get_v_slot_style_option(slot, ctx, state) {
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
                        ctx.format_attr_expr(value)
                    }
                }
                "#" | "slot" => ctx.format_binding(value),
                _ => ctx.format_attr_expr(value),
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
    fn doc<E, F>(&self, ctx: &mut Ctx<E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(Doc::line_or_space())
            .concat(reflow_with_indent(&ctx.format_general_expr(self.expr)))
            .nest_with_ctx(ctx)
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

fn reflow_owned<'i, 'o: 'i>(s: &'i str) -> impl Iterator<Item = Doc<'o>> + '_ {
    itertools::intersperse(
        s.split('\n')
            .map(|s| Doc::text(s.strip_suffix('\r').unwrap_or(s).to_owned())),
        Doc::empty_line(),
    )
}

fn reflow_with_indent<'i, 'o: 'i>(s: &'i str) -> impl Iterator<Item = Doc<'o>> + '_ {
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

fn is_empty_element(children: &[Node], is_whitespace_sensitive: bool) -> bool {
    match &children {
        [] => true,
        [Node::Text(text_node)] => {
            !is_whitespace_sensitive
                && text_node
                    .raw
                    .trim_matches(|c: char| c.is_ascii_whitespace())
                    .is_empty()
        }
        _ => false,
    }
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
    children.iter().filter(|child| !is_text_like(child)).count() > 1
}

fn format_attr_value<'a>(
    value: impl AsRef<str>,
    quotes: &Quotes,
    split_whitespaces: bool,
    indent: bool,
) -> Doc<'a> {
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
    ctx: &mut Ctx<'_, E, F>,
    state: &State<'s>,
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
                    let is_current_text_like = is_text_like(child);
                    let maybe_hard_line = if is_prev_text_like || is_current_text_like {
                        None
                    } else {
                        Some(Doc::hard_line())
                    };
                    match child {
                        Node::Text(text_node) => {
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
                                docs.push(text_node.doc(ctx, state));
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
                            docs.push(child.doc(ctx, state));
                        }
                    };
                    (docs, is_current_text_like)
                },
            )
            .0,
    )
    .group()
}

/// Determines if a given node is "text-like".
/// Text-like nodes should remain on the same line whenever possible.
fn is_text_like(node: &Node) -> bool {
    match node {
        Node::Text(..)
        | Node::VueInterpolation(..)
        | Node::SvelteInterpolation(..)
        | Node::AstroExpr(..)
        | Node::JinjaInterpolation(..)
        | Node::VentoInterpolation(..) => true,
        Node::Element(element) => element.tag_name.eq_ignore_ascii_case("label"),
        _ => false,
    }
}

fn format_children_without_inserting_linebreak<'s, E, F>(
    children: &[Node<'s>],
    has_two_more_non_text_children: bool,
    ctx: &mut Ctx<'_, E, F>,
    state: &State<'s>,
) -> Doc<'s>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    Doc::list(
        children
            .iter()
            .enumerate()
            .map(|(i, child)| match child {
                Node::Text(text_node) => {
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
                    docs.push(text_node.doc(ctx, state));
                    if let Some(doc) = should_add_whitespace_after_text_node(text_node, is_last) {
                        docs.push(doc);
                    }
                    Doc::list(docs)
                }
                child => child.doc(ctx, state),
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

fn get_v_slot_style_option<'s, E, F>(
    slot: &'s str,
    ctx: &Ctx<E, F>,
    state: &State<'s>,
) -> Option<VSlotStyle>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    let option = if state
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
    if let Some(Node::Text(text_node)) = children.first() {
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
    if let Some(Node::Text(text_node)) = children.last() {
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
fn format_ws_insensitive_leading_ws<'s>(children: &[Node<'s>]) -> Doc<'s> {
    match children.first() {
        Some(Node::Text(text_node)) if text_node.line_breaks > 0 => Doc::hard_line(),
        _ => Doc::line_or_nil(),
    }
}
fn format_ws_insensitive_trailing_ws<'s>(children: &[Node<'s>]) -> Doc<'s> {
    match children.last() {
        Some(Node::Text(text_node)) if text_node.line_breaks > 0 => Doc::hard_line(),
        _ => Doc::line_or_nil(),
    }
}
fn format_v_for<'s, E, F>(
    left: &str,
    delimiter: &'static str,
    right: &str,
    ctx: &mut Ctx<'_, E, F>,
) -> String
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    let left = ctx.format_general_expr(left);
    let right = ctx.format_general_expr(right);
    if left.contains(',') && !left.contains('(') {
        format!("({left}) {delimiter} {right}")
    } else {
        format!("{left} {delimiter} {right}")
    }
}

fn format_control_structure_block_children<'s, E, F>(
    children: &[Node<'s>],
    ctx: &mut Ctx<'_, E, F>,
    state: &State<'s>,
) -> Doc<'s>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    match children {
        [Node::Text(text_node)] if is_all_ascii_whitespace(text_node.raw) => Doc::line_or_space(),
        _ => format_ws_sensitive_leading_ws(children)
            .append(format_children_without_inserting_linebreak(
                children,
                has_two_more_non_text_children(children),
                ctx,
                state,
            ))
            .nest_with_ctx(ctx)
            .append(format_ws_sensitive_trailing_ws(children)),
    }
}

fn format_vento_stmt_header<'s, E, F>(
    tag_keyword: &'static str,
    fake_keyword: &'static str,
    code: &'s str,
    ctx: &mut Ctx<'_, E, F>,
) -> Doc<'s>
where
    F: for<'a> FnMut(&Path, &'a str, usize) -> Result<Cow<'a, str>, E>,
{
    Doc::text(tag_keyword)
        .append(Doc::space())
        .concat(reflow_with_indent(
            &ctx.format_stmt_header(fake_keyword, code),
        ))
}
