use crate::{
    Language,
    ast::*,
    config::{Quotes, ScriptFormatter, VSlotStyle, VueComponentCase, WhitespaceSensitivity},
    ctx::{Ctx, Hints},
    helpers,
    parser::parse_as_interpolated,
    state::State,
};
use itertools::{EitherOrBoth, Itertools};
use std::borrow::Cow;
use tiny_pretty::Doc;

pub(super) trait DocGen<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>;
}

impl<'s> DocGen<'s> for AngularElseIf<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("@else if ("));
        docs.push(Doc::text(ctx.format_expr(self.expr.0, false, self.expr.1)));
        if let Some((reference, start)) = self.reference {
            docs.push(Doc::text("; as "));
            docs.push(Doc::text(ctx.format_binding(reference, start)));
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("@for ("));
        docs.push(Doc::text(
            ctx.format_binding(self.binding.0, self.binding.1),
        ));
        docs.push(Doc::text(" of "));
        docs.push(Doc::text(ctx.format_expr(self.expr.0, false, self.expr.1)));
        if let Some((track, start)) = self.track {
            docs.push(Doc::text("; track "));
            docs.push(Doc::text(ctx.format_expr(track, false, start)));
        }
        self.aliases.iter().for_each(|(aliases, start)| {
            docs.push(Doc::text("; "));
            docs.extend(reflow_with_indent(
                ctx.format_script(aliases, "js", *start, state)
                    .trim()
                    .trim_end_matches(';'),
                true,
            ));
        });
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

impl<'s> DocGen<'s> for AngularGenericBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("@"));
        docs.push(Doc::text(self.keyword));
        if let Some(header) = self.header {
            docs.push(Doc::space());
            docs.push(Doc::text(header));
        }
        docs.push(Doc::text(" {"));
        docs.push(format_control_structure_block_children(
            &self.children,
            ctx,
            state,
        ));
        docs.push(Doc::text("}"));
        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for Vec<AngularGenericBlock<'s>> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let next_block_ws = if ctx.options.angular_next_control_flow_same_line {
            Doc::space()
        } else {
            Doc::hard_line()
        };
        Doc::list(
            itertools::intersperse(
                self.iter().map(|block| block.doc(ctx, state)),
                next_block_ws,
            )
            .collect(),
        )
    }
}

impl<'s> DocGen<'s> for AngularIf<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let next_cf_ws = if ctx.options.angular_next_control_flow_same_line {
            Doc::space()
        } else {
            Doc::hard_line()
        };

        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("@if ("));
        docs.push(Doc::text(ctx.format_expr(self.expr.0, false, self.expr.1)));
        if let Some((reference, start)) = self.reference {
            docs.push(Doc::text("; as "));
            docs.push(Doc::text(ctx.format_binding(reference, start)));
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
                .flat_map(|block| [next_cf_ws.clone(), block.doc(ctx, state)]),
        );

        if let Some(children) = &self.else_children {
            docs.push(next_cf_ws);
            docs.push(Doc::text("@else {"));
            docs.push(format_control_structure_block_children(
                children, ctx, state,
            ));
            docs.push(Doc::text("}"));
        }

        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for AngularInterpolation<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(Doc::line_or_space())
            .concat(reflow_with_indent(
                ctx.try_format_expr(self.expr, false, self.start)
                    .as_deref()
                    .unwrap_or(self.expr),
                true,
            ))
            .nest(ctx.indent_width)
            .append(Doc::line_or_space())
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for AngularLet<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("@let ")
            .append(Doc::text(self.name))
            .append(Doc::text(" = "))
            .append(Doc::text(ctx.format_expr(self.expr.0, false, self.expr.1)))
            .append(Doc::text(";"))
    }
}

impl<'s> DocGen<'s> for AngularSwitch<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("@switch ("));
        docs.push(Doc::text(ctx.format_expr(self.expr.0, false, self.expr.1)));
        docs.push(Doc::text(") {"));
        docs.extend(
            self.arms
                .iter()
                .flat_map(|arm| [Doc::hard_line(), arm.doc(ctx, state)]),
        );
        Doc::list(docs)
            .nest(ctx.indent_width)
            .append(Doc::hard_line())
            .append(Doc::text("}"))
    }
}

impl<'s> DocGen<'s> for AngularSwitchArm<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text(format!("@{}", self.keyword)));
        if let Some(expr) = self.expr {
            docs.push(Doc::text(" ("));
            docs.push(Doc::text(ctx.format_expr(expr.0, false, expr.1)));
            docs.push(Doc::text(")"));
        }
        docs.push(Doc::text(" {"));
        docs.push(format_control_structure_block_children(
            &self.children,
            ctx,
            state,
        ));
        docs.push(Doc::text("}"));
        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for AstroAttribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let expr_code = ctx.format_expr(self.expr.0, false, self.expr.1);
        let expr = Doc::text("{")
            .concat(reflow_with_indent(&expr_code, true))
            .append(Doc::text("}"));
        if let Some(name) = self.name {
            if matches!(ctx.options.astro_attr_shorthand, Some(true)) && name == expr_code {
                expr
            } else {
                Doc::text(name).append(Doc::text("=")).append(expr)
            }
        } else if matches!(ctx.options.astro_attr_shorthand, Some(false))
            && !expr_code.starts_with("...")
        {
            Doc::text(expr_code.clone())
                .append(Doc::text("="))
                .append(expr)
        } else {
            expr
        }
    }
}

impl<'s> DocGen<'s> for AstroExpr<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let indent_width = ctx.indent_width;

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
        let formatted_script = ctx.format_expr(&script, false, self.start);

        let templates = self.children.iter().filter_map(|child| {
            if let AstroExprChild::Template(nodes) = child {
                Some(format_children_without_inserting_linebreak(
                    nodes, ctx, state,
                ))
            } else {
                None
            }
        });

        let mut docs = Vec::with_capacity(self.children.len() + 3);
        docs.push(Doc::text("{"));
        formatted_script
            .split(PLACEHOLDER)
            .zip_longest(templates)
            .for_each(|either| match either {
                EitherOrBoth::Both(script, template) => {
                    let extra_indent = script.split('\n').next_back().map_or(0, |line| {
                        line.chars().take_while(|c| c.is_ascii_whitespace()).count()
                    });
                    if script.contains('\n') {
                        docs.extend(reflow_with_indent(script, false));
                    } else {
                        docs.push(Doc::text(script.to_string()));
                    }
                    if script.trim_end().ends_with('(') {
                        docs.push(template.nest(extra_indent));
                    } else {
                        docs.push(
                            Doc::flat_or_break(Doc::nil(), Doc::text("("))
                                .append(Doc::line_or_nil())
                                .append(template)
                                .nest(indent_width)
                                .append(Doc::line_or_nil())
                                .append(Doc::flat_or_break(Doc::nil(), Doc::text(")")))
                                .group()
                                .nest(extra_indent),
                        );
                    }
                }
                EitherOrBoth::Left(script) => {
                    if script.contains('\n') {
                        docs.extend(reflow_with_indent(script, false));
                    } else {
                        docs.push(Doc::text(script.to_string()));
                    }
                }
                EitherOrBoth::Right(..) => {}
            });
        if self.has_line_comment {
            docs.push(Doc::hard_line());
        }
        docs.push(Doc::text("}"));
        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for Attribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        match self {
            Attribute::Native(native_attribute) => native_attribute.doc(ctx, state),
            Attribute::Svelte(svelte_attribute) => svelte_attribute.doc(ctx, state),
            Attribute::SvelteAttachment(svelte_attachment) => svelte_attachment.doc(ctx, state),
            Attribute::VueDirective(vue_directive) => vue_directive.doc(ctx, state),
            Attribute::Astro(astro_attribute) => astro_attribute.doc(ctx, state),
            Attribute::JinjaBlock(jinja_block) => jinja_block.doc(ctx, state),
            Attribute::JinjaComment(jinja_comment) => jinja_comment.doc(ctx, state),
            Attribute::JinjaTag(jinja_tag) => jinja_tag.doc(ctx, state),
            Attribute::VentoTagOrBlock(vento_tag_or_block) => vento_tag_or_block.doc(ctx, state),
        }
    }
}

impl<'s> DocGen<'s> for Cdata<'s> {
    fn doc<E, F>(&self, _: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("<![CDATA[")
            .concat(reflow_raw(self.raw))
            .append(Doc::text("]]>"))
    }
}

impl<'s> DocGen<'s> for Comment<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        if ctx.options.format_comments {
            Doc::text("<!--")
                .append(Doc::line_or_space())
                .concat(reflow_with_indent(self.raw.trim(), true))
                .nest(ctx.indent_width)
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let tag_name = self
            .tag_name
            .split_once(':')
            .and_then(|(namespace, name)| namespace.eq_ignore_ascii_case("html").then_some(name))
            .unwrap_or(self.tag_name);
        let formatted_tag_name = match ctx.language {
            Language::Html | Language::Jinja | Language::Vento | Language::Mustache
                if css_dataset::tags::STANDARD_HTML_TAGS
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(self.tag_name)) =>
            {
                Cow::from(self.tag_name.to_ascii_lowercase())
            }
            Language::Vue
                if !css_dataset::tags::SVG_TAGS
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(self.tag_name)) =>
            {
                match ctx.options.vue_component_case {
                    VueComponentCase::Ignore => Cow::from(self.tag_name),
                    VueComponentCase::PascalCase => helpers::kebab2pascal(self.tag_name),
                    VueComponentCase::KebabCase => helpers::pascal2kebab(self.tag_name),
                }
            }
            _ => Cow::from(self.tag_name),
        };
        let is_root = state.is_root;
        let mut state = State {
            current_tag_name: Some(tag_name),
            is_root: false,
            in_svg: tag_name.eq_ignore_ascii_case("svg"),
            indent_level: state.indent_level,
        };

        let self_closing = if helpers::is_void_element(tag_name, ctx.language) {
            ctx.options
                .html_void_self_closing
                .unwrap_or(self.self_closing)
        } else if helpers::is_html_tag(tag_name, ctx.language) {
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
        } else if helpers::is_svg_tag(self.tag_name, ctx.language) {
            ctx.options.svg_self_closing.unwrap_or(self.self_closing)
        } else if helpers::is_mathml_tag(self.tag_name, ctx.language) {
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
        docs.push(Doc::text(formatted_tag_name.clone()));

        match &*self.attrs {
            [] => {
                if self_closing && (self.void_element || is_empty) {
                    docs.push(Doc::text(" />"));
                    return Doc::list(docs).group();
                }
                if self.void_element {
                    docs.push(Doc::text(">"));
                    return Doc::list(docs).group();
                }
                if is_empty || !is_whitespace_sensitive {
                    docs.push(Doc::text(">"));
                } else {
                    docs.push(Doc::line_or_nil().append(Doc::text(">")).group());
                }
            }
            [attr]
                if ctx.options.single_attr_same_line
                    && !is_whitespace_sensitive
                    && !is_multi_line_attr(attr) =>
            {
                docs.push(Doc::space());
                docs.push(attr.doc(ctx, &state));
                if self_closing && is_empty {
                    docs.push(Doc::text(" />"));
                    return Doc::list(docs);
                } else {
                    docs.push(Doc::text(">"));
                };
                if self.void_element {
                    return Doc::list(docs);
                }
            }
            _ => {
                let attrs_sep = if self.first_attr_same_line {
                    Doc::line_or_space()
                } else if self.attrs.len() <= 1 {
                    if ctx.options.single_attr_same_line {
                        Doc::line_or_space()
                    } else {
                        Doc::hard_line()
                    }
                } else if !ctx.options.prefer_attrs_single_line
                    && ctx
                        .options
                        .max_attrs_per_line
                        .is_none_or(|value| value.get() <= 1)
                {
                    Doc::hard_line()
                } else {
                    Doc::line_or_space()
                };
                let attrs = if let Some(max) = ctx.options.max_attrs_per_line {
                    Doc::line_or_space()
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
                        .nest(ctx.indent_width)
                } else {
                    Doc::list(
                        self.attrs
                            .iter()
                            .flat_map(|attr| [attrs_sep.clone(), attr.doc(ctx, &state)].into_iter())
                            .collect(),
                    )
                    .nest(ctx.indent_width)
                };

                if self_closing && (self.void_element || is_empty) {
                    docs.push(attrs);
                    docs.push(Doc::line_or_space());
                    docs.push(Doc::text("/>"));
                    return Doc::list(docs).group();
                }
                if self.void_element {
                    docs.push(attrs);
                    if !ctx.options.closing_bracket_same_line {
                        docs.push(Doc::line_or_nil());
                    }
                    docs.push(Doc::text(">"));
                    return Doc::list(docs).group();
                }
                if ctx.options.closing_bracket_same_line {
                    docs.push(attrs.append(Doc::text(">")).group());
                } else {
                    // for #16
                    if is_whitespace_sensitive
                        && self.children.first().is_some_and(|child| {
                            if let NodeKind::Text(text_node) = &child.kind {
                                !text_node.raw.starts_with(|c: char| c.is_ascii_whitespace())
                            } else {
                                false
                            }
                        })
                        && self.children.last().is_some_and(|child| {
                            if let NodeKind::Text(text_node) = &child.kind {
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
            }
        }

        let has_two_more_non_text_children =
            has_two_more_non_text_children(&self.children, ctx.language);

        let (leading_ws, trailing_ws) = format_leading_trailing_ws(
            &self.children,
            is_empty,
            is_whitespace_sensitive,
            has_two_more_non_text_children,
            ctx,
        );

        if tag_name.eq_ignore_ascii_case("script") && ctx.language != Language::Xml {
            if let [
                Node {
                    kind: NodeKind::Text(text_node),
                    ..
                },
            ] = &*self.children
            {
                if text_node.raw.chars().all(|c| c.is_ascii_whitespace()) {
                    docs.push(Doc::hard_line());
                } else {
                    let type_attr = self.attrs.iter().find_map(|attr| match attr {
                        Attribute::Native(native) if native.name.eq_ignore_ascii_case("type") => {
                            native.value.map(|(value, _)| value.to_ascii_lowercase())
                        }
                        _ => None,
                    });
                    match type_attr.as_deref() {
                        Some(
                            "module"
                            | "application/javascript"
                            | "text/javascript"
                            | "application/ecmascript"
                            | "text/ecmascript"
                            | "application/x-javascript"
                            | "application/x-ecmascript"
                            | "text/x-javascript"
                            | "text/x-ecmascript"
                            | "text/jsx"
                            | "text/babel",
                        )
                        | None => {
                            let is_script_indent = ctx.script_indent();
                            if is_script_indent {
                                state.indent_level += 1;
                            }
                            let lang = self
                                .attrs
                                .iter()
                                .find_map(|attr| match attr {
                                    Attribute::Native(native)
                                        if native.name.eq_ignore_ascii_case("lang") =>
                                    {
                                        native.value.map(|(value, _)| value)
                                    }
                                    _ => None,
                                })
                                .unwrap_or(if matches!(ctx.language, Language::Astro) {
                                    "ts"
                                } else {
                                    "js"
                                });
                            let lang = if self.attrs.iter().any(|attr| match attr {
                                Attribute::Native(native)
                                    if native.name.eq_ignore_ascii_case("type") =>
                                {
                                    native.value.is_some_and(|(value, _)| value == "module")
                                }
                                _ => false,
                            }) {
                                match lang {
                                    "ts" => "mts",
                                    "js" => "mjs",
                                    lang => lang,
                                }
                            } else {
                                lang
                            };
                            let formatted =
                                ctx.format_script(text_node.raw, lang, text_node.start, &state);
                            let doc = if matches!(
                                ctx.options.script_formatter,
                                Some(ScriptFormatter::Dprint)
                            ) {
                                Doc::hard_line().concat(reflow_owned(formatted.trim()))
                            } else {
                                Doc::hard_line().concat(reflow_with_indent(formatted.trim(), true))
                            };
                            if is_script_indent {
                                docs.push(doc.nest(ctx.indent_width));
                            } else {
                                docs.push(doc);
                            }
                        }
                        Some(
                            "importmap"
                            | "application/json"
                            | "text/json"
                            | "application/ld+json"
                            | "speculationrules",
                        ) => {
                            let formatted = ctx.format_json(text_node.raw, text_node.start, &state);
                            docs.push(
                                Doc::hard_line().concat(reflow_with_indent(formatted.trim(), true)),
                            );
                        }
                        Some(..) => {
                            docs.push(Doc::hard_line());
                            docs.extend(reflow_raw(text_node.raw.trim_matches('\n')));
                        }
                    }
                    docs.push(Doc::hard_line());
                }
            }
        } else if tag_name.eq_ignore_ascii_case("style") && ctx.language != Language::Xml {
            if let [
                Node {
                    kind: NodeKind::Text(text_node),
                    ..
                },
            ] = &*self.children
            {
                if text_node.raw.chars().all(|c| c.is_ascii_whitespace()) {
                    docs.push(Doc::hard_line());
                } else {
                    let lang = self
                        .attrs
                        .iter()
                        .find_map(|attr| match attr {
                            Attribute::Native(native_attribute)
                                if native_attribute.name.eq_ignore_ascii_case("lang") =>
                            {
                                native_attribute.value.map(|(value, _)| value)
                            }
                            _ => None,
                        })
                        .unwrap_or("css");
                    let (statics, dynamics) =
                        parse_as_interpolated(text_node.raw, text_node.start, ctx.language, false);
                    const PLACEHOLDER: &str = "_saya0909_";
                    let masked = statics.join(PLACEHOLDER);
                    let formatted = ctx.format_style(&masked, lang, text_node.start, &state);
                    let doc = Doc::hard_line().concat(reflow_with_indent(
                        formatted
                            .split(PLACEHOLDER)
                            .map(Cow::from)
                            .interleave(dynamics.iter().map(|(expr, start)| match ctx.language {
                                Language::Jinja => Cow::from(format!(
                                    "{{{{ {} }}}}",
                                    ctx.format_jinja(expr, *start, true, &state),
                                )),
                                Language::Vento => Cow::from(format!(
                                    "{{{{ {} }}}}",
                                    ctx.format_expr(expr, false, *start),
                                )),
                                Language::Mustache => Cow::from(format!("{{{{{expr}}}}}")),
                                _ => unreachable!(),
                            }))
                            .collect::<String>()
                            .trim(),
                        lang != "sass",
                    ));
                    docs.push(
                        if ctx.style_indent() {
                            doc.nest(ctx.indent_width)
                        } else {
                            doc
                        }
                        .append(Doc::hard_line()),
                    );
                }
            }
        } else if tag_name.eq_ignore_ascii_case("pre") || tag_name.eq_ignore_ascii_case("textarea")
        {
            if let [
                Node {
                    kind: NodeKind::Text(text_node),
                    ..
                },
            ] = &self.children[..]
            {
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
            state.indent_level += 1;
            docs.push(leading_ws.nest(ctx.indent_width));
            docs.push(
                format_children_with_inserting_linebreak(&self.children, ctx, &state)
                    .nest(ctx.indent_width),
            );
            docs.push(trailing_ws);
        } else if is_whitespace_sensitive
            && matches!(&self.children[..], [Node { kind: NodeKind::Text(text_node), .. }] if is_all_ascii_whitespace(text_node.raw))
        {
            docs.push(Doc::line_or_space());
        } else {
            let should_not_indent = is_whitespace_sensitive
                && self.children.iter().all(|child| {
                    matches!(
                        &child.kind,
                        NodeKind::VueInterpolation(..)
                            | NodeKind::SvelteInterpolation(..)
                            | NodeKind::Comment(..)
                            | NodeKind::AstroExpr(..)
                            | NodeKind::JinjaInterpolation(..)
                            | NodeKind::VentoInterpolation(..)
                    )
                });
            if !should_not_indent {
                state.indent_level += 1;
            }
            let children_doc = leading_ws.append(format_children_without_inserting_linebreak(
                &self.children,
                ctx,
                &state,
            ));
            if should_not_indent {
                // This lets it format like this:
                // ```
                // <span>{{
                //    value
                // }}</span>
                // ```
                docs.push(children_doc);
            } else {
                docs.push(children_doc.nest(ctx.indent_width));
            }
            docs.push(trailing_ws);
        }

        docs.push(Doc::text(format!("</{formatted_tag_name}>")));

        Doc::list(docs).group()
    }
}

impl<'s> DocGen<'s> for FrontMatter<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        if matches!(ctx.language, Language::Astro) {
            let formatted = ctx.format_script(self.raw, "tsx", self.start, state);
            Doc::text("---")
                .append(Doc::hard_line())
                .concat(reflow_with_indent(formatted.trim(), true))
                .append(Doc::hard_line())
                .append(Doc::text("---"))
        } else {
            Doc::text("---")
                .concat(reflow_raw(self.raw))
                .append(Doc::text("---"))
        }
    }
}

impl<'s> DocGen<'s> for JinjaBlock<'s, Attribute<'s>> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::list(
            self.body
                .iter()
                .map(|child| match child {
                    JinjaTagOrChildren::Tag(tag) => tag.doc(ctx, state),
                    JinjaTagOrChildren::Children(children) => Doc::line_or_nil()
                        .concat(itertools::intersperse(
                            children.iter().map(|attr| attr.doc(ctx, state)),
                            Doc::line_or_space(),
                        ))
                        .nest(ctx.indent_width)
                        .append(Doc::line_or_nil()),
                })
                .collect(),
        )
    }
}

impl<'s> DocGen<'s> for JinjaBlock<'s, Node<'s>> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        if ctx.options.format_comments {
            Doc::text("{#")
                .append(Doc::line_or_space())
                .concat(reflow_with_indent(self.raw.trim(), true))
                .nest(ctx.indent_width)
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(if self.trim_prev {
                Doc::text("-")
            } else {
                Doc::nil()
            })
            .append(Doc::line_or_space())
            .concat(reflow_with_indent(
                ctx.format_jinja(self.expr, self.start, true, state).trim(),
                true,
            ))
            .nest(ctx.indent_width)
            .append(Doc::line_or_space())
            .append(if self.trim_next {
                Doc::text("-")
            } else {
                Doc::nil()
            })
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for JinjaTag<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let (prefix, content) = if let Some(content) = self.content.strip_prefix('-') {
            ("-", content)
        } else if let Some(content) = self.content.strip_prefix('+') {
            ("+", content)
        } else {
            ("", self.content)
        };
        let (content, suffix) = if let Some(content) = content.strip_suffix('-') {
            (content, "-")
        } else if let Some(content) = content.strip_suffix('+') {
            (content, "+")
        } else {
            (content, "")
        };

        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("{%"));
        docs.push(Doc::text(prefix));
        docs.push(Doc::line_or_space());
        docs.extend(reflow_with_indent(
            ctx.format_jinja(content, self.start + prefix.len(), false, state)
                .trim(),
            true,
        ));
        Doc::list(docs)
            .nest(ctx.indent_width)
            .append(Doc::line_or_space())
            .append(Doc::text(suffix))
            .append(Doc::text("%}"))
            .group()
    }
}

impl<'s> DocGen<'s> for MustacheBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::list(
            self.controls
                .iter()
                .map(|control| {
                    let mut docs = Vec::with_capacity(3);
                    docs.push(Doc::text("{{"));
                    if control.wc_before {
                        docs.push(Doc::text("~"));
                    }
                    docs.push(Doc::text(control.prefix));
                    docs.push(Doc::text(control.name));
                    if let Some(content) = control.content {
                        docs.push(Doc::space());
                        docs.extend(reflow_raw(content.trim_ascii()));
                    }
                    if control.wc_after {
                        docs.push(Doc::text("~"));
                    }
                    docs.push(Doc::text("}}"));
                    Doc::list(docs)
                })
                .interleave(
                    self.children
                        .iter()
                        .map(|nodes| format_control_structure_block_children(nodes, ctx, state)),
                )
                .collect(),
        )
    }
}

impl<'s> DocGen<'s> for MustacheInterpolation<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        if self.content.starts_with('!') {
            Doc::text("{{")
                .concat(reflow_raw(self.content))
                .append(Doc::text("}}"))
        } else {
            let content = if let Some(content) = self
                .content
                .strip_prefix('{')
                .and_then(|s| s.strip_suffix('}'))
            {
                Cow::from(format!("{{{}}}", content.trim()))
            } else {
                Cow::from(self.content.trim())
            };
            Doc::text("{{")
                .append(Doc::line_or_nil())
                .concat(reflow_owned(&content))
                .nest(ctx.indent_width)
                .append(Doc::line_or_nil())
                .append(Doc::text("}}"))
                .group()
        }
    }
}

impl<'s> DocGen<'s> for NativeAttribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let name = Doc::text(self.name);
        if let Some((value, value_start)) = self.value {
            let value = match ctx.language {
                Language::Vue => {
                    if state
                        .current_tag_name
                        .is_some_and(|name| name.eq_ignore_ascii_case("script"))
                        && self.name == "generic"
                    {
                        Cow::from(ctx.format_type_params(value, value_start))
                    } else {
                        Cow::from(value)
                    }
                }
                Language::Svelte
                    if state
                        .current_tag_name
                        .is_some_and(|name| name.eq_ignore_ascii_case("script"))
                        && self.name == "generics" =>
                {
                    Cow::from(ctx.format_type_params(value, value_start))
                }
                Language::Svelte if !ctx.options.strict_svelte_attr => {
                    if let Some(expr) = value
                        .strip_prefix('{')
                        .and_then(|s| s.strip_suffix('}'))
                        .filter(|s| !s.contains('{'))
                    {
                        let formatted_expr = ctx.format_expr(expr, false, value_start);
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
                                    .concat(reflow_with_indent(&formatted_expr, true))
                                    .append(Doc::text("}"))
                            }
                            _ => Doc::text(self.name.to_owned())
                                .append(Doc::text("={"))
                                .concat(reflow_with_indent(&formatted_expr, true))
                                .append(Doc::text("}")),
                        };
                    } else {
                        Cow::from(value)
                    }
                }
                Language::Angular
                    if self.name.starts_with(['[', '(']) && self.name.ends_with([']', ')']) =>
                {
                    Cow::from(ctx.format_expr(value, false, value_start))
                }
                _ => {
                    if !matches!(ctx.language, Language::Angular | Language::Xml)
                        && self.name.starts_with("on")
                    {
                        ctx.try_format_expr(value, true, value_start)
                            .map_or_else(|_| Cow::from(value), Cow::from)
                    } else {
                        Cow::from(value)
                    }
                }
            };
            let quote;
            let mut docs = Vec::with_capacity(5);
            docs.push(name);
            docs.push(Doc::text("="));
            if self.name.eq_ignore_ascii_case("class") {
                quote = compute_attr_value_quote(&value, self.quote, ctx);
                let value = value.trim();
                let maybe_line_break = if value.contains('\n') {
                    Doc::hard_line()
                } else {
                    Doc::nil()
                };
                docs.push(
                    maybe_line_break
                        .clone()
                        .concat(itertools::intersperse(
                            value
                                .trim()
                                .lines()
                                .filter(|line| !line.is_empty())
                                .map(|line| Doc::text(line.split_ascii_whitespace().join(" "))),
                            Doc::hard_line(),
                        ))
                        .nest(ctx.indent_width),
                );
                docs.push(maybe_line_break);
            } else if self.name.eq_ignore_ascii_case("style") {
                let (statics, dynamics) =
                    parse_as_interpolated(&value, value_start, ctx.language, true);
                const PLACEHOLDER: &str = "_mnk0430_";
                let formatted =
                    ctx.format_style_attr(&statics.join(PLACEHOLDER), value_start, state);
                quote = compute_attr_value_quote(&formatted, self.quote, ctx);
                docs.push(Doc::text(
                    formatted
                        .split(PLACEHOLDER)
                        .map(Cow::from)
                        .interleave(dynamics.iter().map(|(expr, start)| match ctx.language {
                            Language::Svelte => {
                                Cow::from(format!("{{{}}}", ctx.format_expr(expr, true, *start),))
                            }
                            Language::Jinja => Cow::from(format!(
                                "{{{{ {} }}}}",
                                ctx.format_jinja(expr, *start, true, state),
                            )),
                            Language::Vento => Cow::from(format!(
                                "{{{{ {} }}}}",
                                ctx.format_expr(expr, true, *start),
                            )),
                            Language::Mustache => Cow::from(format!("{{{{{expr}}}}}")),
                            _ => unreachable!(),
                        }))
                        .collect::<String>(),
                ));
            } else if self.name.eq_ignore_ascii_case("accept")
                && !matches!(ctx.language, Language::Xml)
                && state
                    .current_tag_name
                    .is_some_and(|name| name.eq_ignore_ascii_case("input"))
            {
                quote = compute_attr_value_quote(&value, self.quote, ctx);
                if helpers::has_template_interpolation(&value, ctx.language) {
                    docs.extend(reflow_owned(&value));
                } else {
                    docs.push(Doc::text(
                        value
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .join(", "),
                    ));
                }
            } else {
                quote = compute_attr_value_quote(&value, self.quote, ctx);
                docs.extend(reflow_owned(&value));
            }
            docs.insert(2, quote.clone());
            docs.push(quote);
            Doc::list(docs)
        } else if matches!(ctx.language, Language::Svelte)
            && matches!(ctx.options.svelte_directive_shorthand, Some(false))
        {
            if let Some((_, binding_name)) = self.name.split_once(':') {
                let value = format!("{{{binding_name}}}");
                name.append(Doc::text("="))
                    .append(if ctx.options.strict_svelte_attr {
                        format_attr_value(value, &ctx.options.quotes)
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

impl<'s> DocGen<'s> for NodeKind<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        match self {
            NodeKind::AngularFor(angular_for) => angular_for.doc(ctx, state),
            NodeKind::AngularGenericBlocks(blocks) => blocks.doc(ctx, state),
            NodeKind::AngularIf(angular_if) => angular_if.doc(ctx, state),
            NodeKind::AngularInterpolation(angular_interpolation) => {
                angular_interpolation.doc(ctx, state)
            }
            NodeKind::AngularLet(angular_let) => angular_let.doc(ctx, state),
            NodeKind::AngularSwitch(angular_switch) => angular_switch.doc(ctx, state),
            NodeKind::AstroExpr(astro_expr) => astro_expr.doc(ctx, state),
            NodeKind::Cdata(cdata) => cdata.doc(ctx, state),
            NodeKind::Comment(comment) => comment.doc(ctx, state),
            NodeKind::Doctype(doctype) => doctype.doc(ctx, state),
            NodeKind::Element(element) => element.doc(ctx, state),
            NodeKind::FrontMatter(front_matter) => front_matter.doc(ctx, state),
            NodeKind::JinjaBlock(jinja_block) => jinja_block.doc(ctx, state),
            NodeKind::JinjaComment(jinja_comment) => jinja_comment.doc(ctx, state),
            NodeKind::JinjaInterpolation(jinja_interpolation) => {
                jinja_interpolation.doc(ctx, state)
            }
            NodeKind::JinjaTag(jinja_tag) => jinja_tag.doc(ctx, state),
            NodeKind::MustacheBlock(mustache_block) => mustache_block.doc(ctx, state),
            NodeKind::MustacheInterpolation(mustache_interpolation) => {
                mustache_interpolation.doc(ctx, state)
            }
            NodeKind::SvelteAtTag(svelte_at_tag) => svelte_at_tag.doc(ctx, state),
            NodeKind::SvelteAwaitBlock(svelte_await_block) => svelte_await_block.doc(ctx, state),
            NodeKind::SvelteEachBlock(svelte_each_block) => svelte_each_block.doc(ctx, state),
            NodeKind::SvelteIfBlock(svelte_if_block) => svelte_if_block.doc(ctx, state),
            NodeKind::SvelteInterpolation(svelte_interpolation) => {
                svelte_interpolation.doc(ctx, state)
            }
            NodeKind::SvelteKeyBlock(svelte_key_block) => svelte_key_block.doc(ctx, state),
            NodeKind::SvelteSnippetBlock(svelte_snippet_block) => {
                svelte_snippet_block.doc(ctx, state)
            }
            NodeKind::Text(text_node) => text_node.doc(ctx, state),
            NodeKind::VentoBlock(vento_block) => vento_block.doc(ctx, state),
            NodeKind::VentoComment(vento_comment) => vento_comment.doc(ctx, state),
            NodeKind::VentoEval(vento_eval) => vento_eval.doc(ctx, state),
            NodeKind::VentoInterpolation(vento_interpolation) => {
                vento_interpolation.doc(ctx, state)
            }
            NodeKind::VentoTag(vento_tag) => vento_tag.doc(ctx, state),
            NodeKind::VueInterpolation(vue_interpolation) => vue_interpolation.doc(ctx, state),
            NodeKind::XmlDecl(xml_decl) => xml_decl.doc(ctx, state),
        }
    }
}

impl<'s> DocGen<'s> for Root<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let is_whole_document_like = self.children.iter().any(|child| match &child.kind {
            NodeKind::Doctype(..) => true,
            NodeKind::Element(element) => element.tag_name.eq_ignore_ascii_case("html"),
            _ => false,
        });
        let is_whitespace_sensitive = matches!(
            ctx.options.whitespace_sensitivity,
            WhitespaceSensitivity::Css | WhitespaceSensitivity::Strict
        );
        let has_two_more_non_text_children =
            has_two_more_non_text_children(&self.children, ctx.language);

        if is_whole_document_like
            && !matches!(
                ctx.options.whitespace_sensitivity,
                WhitespaceSensitivity::Strict
            )
            || !is_whitespace_sensitive && has_two_more_non_text_children
            || ctx.language == Language::Xml
        {
            format_children_with_inserting_linebreak(&self.children, ctx, state)
                .append(Doc::hard_line())
        } else {
            format_children_without_inserting_linebreak(&self.children, ctx, state)
                .append(Doc::hard_line())
        }
    }
}

impl<'s> DocGen<'s> for SvelteAtTag<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{@")
            .append(Doc::text(self.name))
            .append(Doc::space())
            .concat(reflow_with_indent(
                &ctx.format_expr(self.expr.0, false, self.expr.1),
                true,
            ))
            .append(Doc::text("}"))
    }
}

impl<'s> DocGen<'s> for SvelteAttribute<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let expr_code = ctx.format_expr(self.expr.0, false, self.expr.1);
        let expr = Doc::text("{")
            .concat(reflow_with_indent(&expr_code, true))
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
                ))
            } else {
                name.append(expr)
            }
        } else {
            expr
        }
    }
}

impl<'s> DocGen<'s> for SvelteAttachment<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let expr_code = ctx.format_expr(self.expr.0, false, self.expr.1);
        Doc::text("{@attach ")
            .concat(reflow_with_indent(&expr_code, true))
            .append(Doc::text("}"))
    }
}

impl<'s> DocGen<'s> for SvelteAwaitBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let mut head = Vec::with_capacity(5);
        head.push(Doc::text("{#await "));
        head.push(Doc::text(ctx.format_expr(self.expr.0, false, self.expr.1)));

        if let Some(then) = self.then_binding {
            head.push(Doc::line_or_space());
            head.push(Doc::text("then"));
            if let Some((binding, start)) = then {
                head.push(Doc::space());
                head.push(Doc::text(ctx.format_binding(binding, start)));
            }
        }

        if let Some(catch) = self.catch_binding {
            head.push(Doc::line_or_space());
            head.push(Doc::text("catch"));
            if let Some((binding, start)) = catch {
                head.push(Doc::space());
                head.push(Doc::text(ctx.format_binding(binding, start)));
            }
        }

        let mut docs = Vec::with_capacity(5);
        docs.push(
            Doc::list(head)
                .nest(ctx.indent_width)
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let children = format_control_structure_block_children(&self.children, ctx, state);
        if let Some((binding, start)) = self.binding {
            Doc::text("{:catch ")
                .append(Doc::text(ctx.format_binding(binding, start)))
                .append(Doc::text("}"))
                .append(children)
        } else {
            Doc::text("{:catch}").append(children)
        }
    }
}

impl<'s> DocGen<'s> for SvelteEachBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("{#each "));
        docs.extend(reflow_with_indent(
            &ctx.format_expr(self.expr.0, false, self.expr.1),
            false,
        ));
        if let Some(binding) = self.binding {
            docs.push(Doc::text(" as "));
            docs.push(Doc::text(ctx.format_binding(binding.0, binding.1)));
        }

        if let Some(index) = self.index {
            docs.push(Doc::text(", "));
            docs.push(Doc::text(index));
        }

        if let Some((key, start)) = self.key {
            docs.push(Doc::text(" ("));
            docs.push(Doc::text(ctx.format_expr(key, false, start)));
            docs.push(Doc::text(")"));
        }

        docs.push(Doc::text("}"));
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{:else if ")
            .append(Doc::text(ctx.format_expr(self.expr.0, false, self.expr.1)))
            .append(Doc::text("}"))
            .append(format_control_structure_block_children(
                &self.children,
                ctx,
                state,
            ))
    }
}

impl<'s> DocGen<'s> for SvelteIfBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("{#if "));
        docs.push(Doc::text(ctx.format_expr(self.expr.0, false, self.expr.1)));
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{")
            .append(Doc::line_or_nil())
            .concat(reflow_with_indent(
                &ctx.format_expr(self.expr.0, false, self.expr.1),
                true,
            ))
            .nest(ctx.indent_width)
            .append(Doc::line_or_nil())
            .append(Doc::text("}"))
            .group()
    }
}

impl<'s> DocGen<'s> for SvelteKeyBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{#key ")
            .append(Doc::text(ctx.format_expr(self.expr.0, false, self.expr.1)))
            .append(Doc::text("}"))
            .append(format_control_structure_block_children(
                &self.children,
                ctx,
                state,
            ))
            .append(Doc::text("{/key}"))
    }
}

impl<'s> DocGen<'s> for SvelteSnippetBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let wrapped = format!("function {}{{}}", self.signature.0);
        let formatted = ctx.format_script(&wrapped, "ts", self.signature.1, state);
        Doc::text("{#snippet ")
            .append(Doc::text(
                formatted
                    .trim()
                    .strip_prefix("function ")
                    .and_then(|s| s.trim_end().strip_suffix('}'))
                    .and_then(|s| s.trim_end().strip_suffix('{'))
                    .map(|s| s.trim())
                    .unwrap_or(&formatted)
                    .to_owned(),
            ))
            .append(Doc::text("}"))
            .append(format_control_structure_block_children(
                &self.children,
                ctx,
                state,
            ))
            .append(Doc::text("{/snippet}"))
    }
}

impl<'s> DocGen<'s> for SvelteThenBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        let mut docs = Vec::with_capacity(5);
        docs.push(Doc::text("{:then"));
        if let Some((binding, start)) = self.binding {
            docs.push(Doc::space());
            docs.push(Doc::text(ctx.format_binding(binding, start)));
        }
        docs.push(Doc::text("}"));
        docs.push(format_control_structure_block_children(
            &self.children,
            ctx,
            state,
        ));
        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for TextNode<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        if ctx.language == Language::Xml {
            if self.raw.chars().all(|c| c.is_ascii_whitespace()) {
                Doc::nil()
            } else {
                Doc::list(reflow_raw(self.raw.trim_ascii()).collect())
            }
        } else {
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
}

impl<'s> DocGen<'s> for VentoBlock<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        if ctx.options.format_comments {
            Doc::text("{{#")
                .append(Doc::line_or_space())
                .concat(reflow_with_indent(self.raw.trim(), true))
                .nest(ctx.indent_width)
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{>")
            .append(Doc::line_or_space())
            .concat(reflow_with_indent(
                ctx.format_script(self.raw, "js", self.start, state)
                    .trim()
                    .trim_end_matches(';'),
                true,
            ))
            .nest(ctx.indent_width)
            .append(Doc::line_or_space())
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for VentoInterpolation<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(if self.trim_prev {
                Doc::text("-")
            } else {
                Doc::nil()
            })
            .append(Doc::line_or_space())
            .concat(itertools::intersperse(
                self.expr.split("|>").map(|expr| {
                    Doc::list(
                        reflow_with_indent(&ctx.format_expr(expr, false, self.start), true)
                            .collect(),
                    )
                }),
                Doc::line_or_space()
                    .append(Doc::text("|>"))
                    .append(Doc::space()),
            ))
            .nest(ctx.indent_width)
            .append(Doc::line_or_space())
            .append(if self.trim_next {
                Doc::text("-")
            } else {
                Doc::nil()
            })
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for VentoTag<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(if self.trim_prev {
                Doc::text("-")
            } else {
                Doc::nil()
            })
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
                                .concat(reflow_with_indent(
                                    &ctx.format_expr(template, false, 0),
                                    true,
                                ))
                                .append(Doc::text(" "))
                                .concat(reflow_with_indent(&ctx.format_expr(data, false, 0), true))
                        } else {
                            Doc::text(tag_name.to_string()).append(Doc::space()).concat(
                                reflow_with_indent(&ctx.format_expr(parsed_tag.1, false, 0), true),
                            )
                        }
                    } else if parsed_tag.0 == "function" || parsed_tag.1.starts_with("function") {
                        // unsupported at present
                        Doc::list(reflow_with_indent(item.trim(), true).collect())
                    } else if let (tag_name @ ("set" | "export"), rest) = parsed_tag {
                        if let Some((binding, expr)) = rest.trim().split_once('=') {
                            Doc::text(tag_name.to_string())
                                .append(Doc::space())
                                .concat(reflow_with_indent(&ctx.format_binding(binding, 0), true))
                                .append(Doc::text(" = "))
                                .concat(reflow_with_indent(&ctx.format_expr(expr, false, 0), true))
                        } else {
                            Doc::text(tag_name.to_string())
                                .append(Doc::space())
                                .concat(reflow_with_indent(&ctx.format_binding(rest, 0), true))
                        }
                    } else if let ("import", _) = parsed_tag {
                        Doc::list(
                            reflow_with_indent(
                                ctx.format_script(item, "js", 0, state)
                                    .trim()
                                    .trim_end_matches(';'),
                                true,
                            )
                            .collect(),
                        )
                    } else {
                        Doc::list(reflow_with_indent(item.trim(), true).collect())
                    }
                }),
                Doc::line_or_space()
                    .append(Doc::text("|>"))
                    .append(Doc::space()),
            ))
            .nest(ctx.indent_width)
            .append(Doc::line_or_space())
            .append(if self.trim_next {
                Doc::text("-")
            } else {
                Doc::nil()
            })
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for VentoTagOrChildren<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
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
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
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
                        if arg_and_modifiers.starts_with(':') {
                            Doc::text("")
                        } else {
                            Doc::text("v-bind")
                        }
                    } else {
                        Doc::text("v-bind")
                    });
                    docs.push(Doc::text(arg_and_modifiers));
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

        if let Some((value, value_start)) = self.value {
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
                        format_v_for(left, delimiter, right, value_start, ctx)
                    } else if let Some((left, right)) = value.split_once(" of ") {
                        let delimiter = if let Some(VForDelimiterStyle::In) =
                            ctx.options.v_for_delimiter_style
                        {
                            "in"
                        } else {
                            "of"
                        };
                        format_v_for(left, delimiter, right, value_start, ctx)
                    } else {
                        ctx.with_escaping_quotes(value, |code, ctx| {
                            ctx.format_expr(&code, true, value_start)
                        })
                    }
                }
                "#" | "slot" => ctx.format_binding(value, value_start),
                _ => ctx.with_escaping_quotes(value, |code, ctx| {
                    ctx.try_format_expr(&code, true, value_start)
                        .unwrap_or_else(|_| {
                            let formatted = ctx
                                .format_script(&code, "ts", value_start, state)
                                .trim()
                                .to_owned();
                            if formatted.contains('\n') {
                                formatted
                            } else {
                                formatted.trim_end_matches(';').to_owned()
                            }
                        })
                }),
            };
            if !(matches!(ctx.options.v_bind_same_name_short_hand, Some(true))
                && is_v_bind
                && matches!(self.arg_and_modifiers, Some(arg_and_modifiers) if arg_and_modifiers == value))
            {
                docs.push(Doc::text("="));
                docs.push(format_attr_value(value, &ctx.options.quotes));
            }
        } else if matches!(ctx.options.v_bind_same_name_short_hand, Some(false))
            && is_v_bind
            && let Some(arg_and_modifiers) = self.arg_and_modifiers
        {
            docs.push(Doc::text("="));
            docs.push(format_attr_value(arg_and_modifiers, &ctx.options.quotes));
        }

        Doc::list(docs)
    }
}

impl<'s> DocGen<'s> for VueInterpolation<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, _: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("{{")
            .append(Doc::line_or_space())
            .concat(reflow_with_indent(
                &ctx.format_expr(self.expr, false, self.start),
                true,
            ))
            .nest(ctx.indent_width)
            .append(Doc::line_or_space())
            .append(Doc::text("}}"))
            .group()
    }
}

impl<'s> DocGen<'s> for XmlDecl<'s> {
    fn doc<E, F>(&self, ctx: &mut Ctx<'s, E, F>, state: &State<'s>) -> Doc<'s>
    where
        F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
    {
        Doc::text("<?xml")
            .concat(
                self.attrs
                    .iter()
                    .flat_map(|attr| [Doc::line_or_space(), attr.doc(ctx, state)].into_iter()),
            )
            .nest(ctx.indent_width)
            .append(Doc::text("?>"))
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

fn reflow_owned<'i, 'o: 'i>(s: &'i str) -> impl Iterator<Item = Doc<'o>> + 'i {
    itertools::intersperse(
        s.split('\n')
            .map(|s| Doc::text(s.strip_suffix('\r').unwrap_or(s).to_owned())),
        Doc::empty_line(),
    )
}

fn reflow_with_indent<'i, 'o: 'i>(
    s: &'i str,
    detect_indent: bool,
) -> impl Iterator<Item = Doc<'o>> + 'i {
    let indent = if detect_indent {
        helpers::detect_indent(s)
    } else {
        0
    };
    let mut pair_stack = vec![];
    s.split('\n').enumerate().flat_map(move |(i, s)| {
        let s = s.strip_suffix('\r').unwrap_or(s);
        let trimmed = if s.starts_with([' ', '\t']) {
            s.get(indent..).unwrap_or(s)
        } else {
            s
        };
        let should_keep_raw = matches!(pair_stack.last(), Some('`'));

        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '`' | '\'' | '"' => {
                    let last = pair_stack.last();
                    if last.is_some_and(|last| *last == c) {
                        pair_stack.pop();
                    } else if matches!(last, Some('$' | '{') | None) {
                        pair_stack.push(c);
                    }
                }
                '$' if matches!(pair_stack.last(), Some('`')) => {
                    if chars.next_if(|next| *next == '{').is_some() {
                        pair_stack.push('$');
                    }
                }
                '{' if !matches!(pair_stack.last(), Some('`' | '\'' | '"' | '/')) => {
                    pair_stack.push('{');
                }
                '}' if matches!(pair_stack.last(), Some('$' | '{')) => {
                    pair_stack.pop();
                }
                '/' if !matches!(pair_stack.last(), Some('\'' | '"' | '`')) => {
                    if chars.next_if(|next| *next == '*').is_some() {
                        pair_stack.push('*');
                    } else if chars.next_if(|next| *next == '/').is_some() {
                        break;
                    }
                }
                '*' => {
                    if chars.next_if(|next| *next == '/').is_some() {
                        pair_stack.pop();
                    }
                }
                '\\' if matches!(pair_stack.last(), Some('\'' | '"' | '`')) => {
                    chars.next();
                }
                _ => {}
            }
        }

        [
            if i == 0 {
                Doc::nil()
            } else if trimmed.trim().is_empty() || should_keep_raw {
                Doc::empty_line()
            } else {
                Doc::hard_line()
            },
            if should_keep_raw {
                Doc::text(s.to_owned())
            } else {
                Doc::text(trimmed.to_owned())
            },
        ]
        .into_iter()
    })
}

fn is_empty_element(children: &[Node], is_whitespace_sensitive: bool) -> bool {
    match &children {
        [] => true,
        [
            Node {
                kind: NodeKind::Text(text_node),
                ..
            },
        ] => {
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

fn is_multi_line_attr(attr: &Attribute) -> bool {
    match attr {
        Attribute::Native(attr) => attr
            .value
            .is_some_and(|(value, _)| value.trim().contains('\n')),
        Attribute::VueDirective(attr) => attr.value.is_some_and(|(value, _)| value.contains('\n')),
        Attribute::Astro(AstroAttribute {
            expr: (value, ..), ..
        })
        | Attribute::Svelte(SvelteAttribute {
            expr: (value, ..), ..
        })
        | Attribute::SvelteAttachment(SvelteAttachment {
            expr: (value, ..), ..
        })
        | Attribute::JinjaComment(JinjaComment { raw: value, .. })
        | Attribute::JinjaTag(JinjaTag { content: value, .. }) => value.contains('\n'),
        // Templating blocks usually span across multiple lines so let's just assume true.
        Attribute::JinjaBlock(..) | Attribute::VentoTagOrBlock(..) => true,
    }
}

fn should_ignore_node<'s, E, F>(index: usize, nodes: &[Node], ctx: &Ctx<'s, E, F>) -> bool
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    match index.checked_sub(1).and_then(|i| nodes.get(i)) {
        Some(Node {
            kind: NodeKind::Comment(comment),
            ..
        }) => has_ignore_directive(comment, ctx),
        Some(Node {
            kind: NodeKind::Text(text_node),
            ..
        }) if is_all_ascii_whitespace(text_node.raw) => {
            if let Some(Node {
                kind: NodeKind::Comment(comment),
                ..
            }) = index.checked_sub(2).and_then(|i| nodes.get(i))
            {
                has_ignore_directive(comment, ctx)
            } else {
                false
            }
        }
        _ => false,
    }
}
fn has_ignore_directive<'s, E, F>(comment: &Comment, ctx: &Ctx<'s, E, F>) -> bool
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    comment
        .raw
        .trim_start()
        .strip_prefix(&ctx.options.ignore_comment_directive)
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_whitespace()) || rest.is_empty())
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

fn has_two_more_non_text_children(children: &[Node], language: Language) -> bool {
    children
        .iter()
        .filter(|child| !is_text_like(child, language))
        .count()
        > 1
}

fn format_attr_value(value: impl AsRef<str>, quotes: &Quotes) -> Doc<'_> {
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
    quote
        .clone()
        .concat(reflow_with_indent(value, true))
        .append(quote)
}

fn format_children_with_inserting_linebreak<'s, E, F>(
    children: &[Node<'s>],
    ctx: &mut Ctx<'s, E, F>,
    state: &State<'s>,
) -> Doc<'s>
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    Doc::list(
        children
            .iter()
            .enumerate()
            .fold(
                (Vec::with_capacity(children.len() * 2), true),
                |(mut docs, is_prev_text_like), (i, child)| {
                    let is_current_text_like = is_text_like(child, ctx.language);
                    if should_ignore_node(i, children, ctx) {
                        let raw = child.raw.trim_end_matches([' ', '\t']);
                        let last_line_break_removed = raw.strip_suffix(['\n', '\r']);
                        docs.extend(reflow_raw(last_line_break_removed.unwrap_or(raw)));
                        if i < children.len() - 1 && last_line_break_removed.is_some() {
                            docs.push(Doc::hard_line());
                        }
                    } else {
                        let maybe_hard_line = if is_prev_text_like || is_current_text_like {
                            None
                        } else {
                            Some(Doc::hard_line())
                        };
                        match &child.kind {
                            NodeKind::Text(text_node) => {
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
                        }
                    }
                    (docs, is_current_text_like)
                },
            )
            .0,
    )
    .group()
}

/// Determines if a given node is "text-like".
/// Text-like nodes should remain on the same line whenever possible.
fn is_text_like(node: &Node, language: Language) -> bool {
    match &node.kind {
        NodeKind::Element(element) => {
            helpers::is_whitespace_sensitive_tag(element.tag_name, language)
        }
        NodeKind::Text(..)
        | NodeKind::VueInterpolation(..)
        | NodeKind::SvelteInterpolation(..)
        | NodeKind::AstroExpr(..)
        | NodeKind::JinjaInterpolation(..)
        | NodeKind::VentoInterpolation(..) => true,
        _ => false,
    }
}

fn format_children_without_inserting_linebreak<'s, E, F>(
    children: &[Node<'s>],
    ctx: &mut Ctx<'s, E, F>,
    state: &State<'s>,
) -> Doc<'s>
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    Doc::list(
        children
            .iter()
            .enumerate()
            .fold(
                (Vec::with_capacity(children.len() * 2), true),
                |(mut docs, is_prev_text_like), (i, child)| {
                    if should_ignore_node(i, children, ctx) {
                        let raw = child.raw.trim_end_matches([' ', '\t']);
                        let last_line_break_removed = raw.strip_suffix(['\n', '\r']);
                        docs.extend(reflow_raw(last_line_break_removed.unwrap_or(raw)));
                        if i < children.len() - 1 && last_line_break_removed.is_some() {
                            docs.push(Doc::hard_line());
                        }
                    } else if let NodeKind::Text(text_node) = &child.kind {
                        let is_first = i == 0;
                        let is_last = i + 1 == children.len();
                        if !is_first && !is_last && is_all_ascii_whitespace(text_node.raw) {
                            match text_node.line_breaks {
                                0 => {
                                    if !is_prev_text_like
                                        && children
                                            .get(i + 1)
                                            .is_some_and(|next| !is_text_like(next, ctx.language))
                                    {
                                        docs.push(Doc::line_or_space());
                                    } else {
                                        docs.push(Doc::soft_line());
                                    }
                                }
                                1 => docs.push(Doc::hard_line()),
                                _ => {
                                    docs.push(Doc::empty_line());
                                    docs.push(Doc::hard_line());
                                }
                            }
                            return (docs, true);
                        }

                        if let Some(doc) =
                            should_add_whitespace_before_text_node(text_node, is_first)
                        {
                            docs.push(doc);
                        }
                        docs.push(text_node.doc(ctx, state));
                        if let Some(doc) = should_add_whitespace_after_text_node(text_node, is_last)
                        {
                            docs.push(doc);
                        }
                    } else {
                        docs.push(child.kind.doc(ctx, state))
                    }
                    (docs, is_text_like(child, ctx.language))
                },
            )
            .0,
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
    ctx: &Ctx<'s, E, F>,
    state: &State<'s>,
) -> Option<VSlotStyle>
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    let option = if state
        .current_tag_name
        .is_some_and(|name| name.eq_ignore_ascii_case("template"))
    {
        if slot == "default" {
            ctx.options.default_v_slot_style
        } else {
            ctx.options.named_v_slot_style
        }
    } else {
        ctx.options.component_v_slot_style
    };
    option.or(ctx.options.v_slot_style)
}

fn format_v_slot(style: VSlotStyle, slot: &str) -> Doc<'_> {
    match style {
        VSlotStyle::Short => Doc::text(format!("#{slot}")),
        VSlotStyle::Long => Doc::text(format!("v-slot:{slot}")),
        VSlotStyle::VSlot => Doc::text("v-slot"),
    }
}

fn format_ws_sensitive_leading_ws<'s>(children: &[Node<'s>]) -> Doc<'s> {
    if let Some(Node {
        kind: NodeKind::Text(text_node),
        ..
    }) = children.first()
    {
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
    if let Some(Node {
        kind: NodeKind::Text(text_node),
        ..
    }) = children.last()
    {
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
        Some(Node {
            kind: NodeKind::Text(text_node),
            ..
        }) if text_node.line_breaks > 0 => Doc::hard_line(),
        _ => Doc::line_or_nil(),
    }
}
fn format_ws_insensitive_trailing_ws<'s>(children: &[Node<'s>]) -> Doc<'s> {
    match children.last() {
        Some(Node {
            kind: NodeKind::Text(text_node),
            ..
        }) if text_node.line_breaks > 0 => Doc::hard_line(),
        _ => Doc::line_or_nil(),
    }
}
fn format_v_for<'s, E, F>(
    left: &str,
    delimiter: &'static str,
    right: &str,
    start: usize,
    ctx: &mut Ctx<'s, E, F>,
) -> String
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    let left = ctx.format_expr(left, false, start);
    let right = ctx.format_expr(right, false, start + 4);
    // Add parentheses around tuple unpacking (e.g., `a, i` → `(a, i)`),
    // but not around destructuring patterns that are already wrapped
    // with brackets `[...]` or braces `{...}` or parentheses `(...)`.
    if left.contains(',') && !left.trim_start().starts_with(['(', '[', '{']) {
        format!("({left}) {delimiter} {right}")
    } else {
        format!("{left} {delimiter} {right}")
    }
}

fn format_leading_trailing_ws<'s, E, F>(
    children: &[Node<'s>],
    is_empty: bool,
    is_whitespace_sensitive: bool,
    has_two_more_non_text_children: bool,
    ctx: &Ctx<'s, E, F>,
) -> (Doc<'s>, Doc<'s>)
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    if is_empty
        || ctx.language == Language::Xml
            && matches!(
                children,
                [Node {
                    kind: NodeKind::Text(..),
                    ..
                }]
            )
    {
        (Doc::nil(), Doc::nil())
    } else if is_whitespace_sensitive {
        (
            format_ws_sensitive_leading_ws(children),
            format_ws_sensitive_trailing_ws(children),
        )
    } else if has_two_more_non_text_children {
        (Doc::hard_line(), Doc::hard_line())
    } else {
        (
            format_ws_insensitive_leading_ws(children),
            format_ws_insensitive_trailing_ws(children),
        )
    }
}

fn format_control_structure_block_children<'s, E, F>(
    children: &[Node<'s>],
    ctx: &mut Ctx<'s, E, F>,
    state: &State<'s>,
) -> Doc<'s>
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    let is_whitespace_sensitive = state
        .current_tag_name
        .is_none_or(|current_tag_name| ctx.is_whitespace_sensitive(current_tag_name));
    let (leading_ws, trailing_ws) = format_leading_trailing_ws(
        children,
        is_empty_element(children, is_whitespace_sensitive),
        is_whitespace_sensitive,
        has_two_more_non_text_children(children, ctx.language),
        ctx,
    );
    match children {
        [
            Node {
                kind: NodeKind::Text(text_node),
                ..
            },
        ] if is_all_ascii_whitespace(text_node.raw) => Doc::line_or_space(),
        _ => leading_ws
            .append(format_children_without_inserting_linebreak(
                children, ctx, state,
            ))
            .nest(ctx.indent_width)
            .append(trailing_ws),
    }
}

fn format_vento_stmt_header<'s, E, F>(
    tag_keyword: &'static str,
    fake_keyword: &'static str,
    code: &'s str,
    ctx: &mut Ctx<'s, E, F>,
) -> Doc<'s>
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    Doc::text(tag_keyword)
        .append(Doc::space())
        .concat(reflow_with_indent(
            &ctx.format_stmt_header(fake_keyword, code),
            true,
        ))
}

/// Computes the appropriate quote character (single or double) to use for an attribute value.
///
/// This will try to respect the configured `quotes` but might change it
/// to ensure the result is valid html.
fn compute_attr_value_quote<'s, E, F>(
    attr_value: &str,
    initial_quote: Option<char>,
    ctx: &mut Ctx<'s, E, F>,
) -> Doc<'s>
where
    F: for<'a> FnMut(&'a str, Hints) -> Result<Cow<'a, str>, E>,
{
    let has_single = attr_value.contains('\'');
    let has_double = attr_value.contains('"');
    if has_double && has_single {
        if let Some(quote) = initial_quote {
            Doc::text(quote.to_string())
        } else if let Quotes::Double = ctx.options.quotes {
            Doc::text("\"")
        } else {
            Doc::text("'")
        }
    } else if has_double {
        Doc::text("'")
    } else if has_single {
        Doc::text("\"")
    } else if let Quotes::Double = ctx.options.quotes {
        Doc::text("\"")
    } else {
        Doc::text("'")
    }
}
