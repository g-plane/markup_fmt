use dprint_core::configuration::{
    get_nullable_value, get_unknown_property_diagnostics, get_value, ConfigKeyMap,
    ConfigurationDiagnostic, GlobalConfiguration, NewLineKind, ResolveConfigurationResult,
};
use markup_fmt::config::*;

pub(crate) fn resolve_config(
    mut config: ConfigKeyMap,
    global_config: &GlobalConfiguration,
) -> ResolveConfigurationResult<FormatOptions> {
    let mut diagnostics = Vec::new();
    let markup_fmt_config = FormatOptions {
        layout: LayoutOptions {
            print_width: get_value(
                &mut config,
                "printWidth",
                global_config.line_width.unwrap_or(80),
                &mut diagnostics,
            ) as usize,
            use_tabs: get_value(
                &mut config,
                "useTabs",
                global_config.use_tabs.unwrap_or_default(),
                &mut diagnostics,
            ),
            indent_width: get_value(
                &mut config,
                "indentWidth",
                global_config.indent_width.unwrap_or(2),
                &mut diagnostics,
            ) as usize,
            line_break: match &*get_value(
                &mut config,
                "lineBreak",
                match global_config.new_line_kind {
                    Some(NewLineKind::LineFeed) => "lf",
                    Some(NewLineKind::CarriageReturnLineFeed) => "crlf",
                    _ => "lf",
                }
                .to_string(),
                &mut diagnostics,
            ) {
                "lf" => LineBreak::Lf,
                "crlf" => LineBreak::Crlf,
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "lineBreak".into(),
                        message: "invalid value for config `lineBreak`".into(),
                    });
                    LineBreak::Lf
                }
            },
        },
        language: LanguageOptions {
            quotes: match &*get_value(
                &mut config,
                "quotes",
                "double".to_string(),
                &mut diagnostics,
            ) {
                "double" => Quotes::Double,
                "single" => Quotes::Single,
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "quotes".into(),
                        message: "invalid value for config `quotes`".into(),
                    });
                    Default::default()
                }
            },
            format_comments: get_value(&mut config, "formatComments", false, &mut diagnostics),
            script_indent: get_value(&mut config, "scriptIndent", false, &mut diagnostics),
            html_script_indent: get_nullable_value(
                &mut config,
                "html.scriptIndent",
                &mut diagnostics,
            ),
            vue_script_indent: get_nullable_value(
                &mut config,
                "vue.scriptIndent",
                &mut diagnostics,
            ),
            svelte_script_indent: get_nullable_value(
                &mut config,
                "svelte.scriptIndent",
                &mut diagnostics,
            ),
            astro_script_indent: get_nullable_value(
                &mut config,
                "astro.scriptIndent",
                &mut diagnostics,
            ),
            style_indent: get_value(&mut config, "styleIndent", false, &mut diagnostics),
            html_style_indent: get_nullable_value(
                &mut config,
                "html.styleIndent",
                &mut diagnostics,
            ),
            vue_style_indent: get_nullable_value(&mut config, "vue.styleIndent", &mut diagnostics),
            svelte_style_indent: get_nullable_value(
                &mut config,
                "svelte.styleIndent",
                &mut diagnostics,
            ),
            astro_style_indent: get_nullable_value(
                &mut config,
                "astro.styleIndent",
                &mut diagnostics,
            ),
            closing_bracket_same_line: get_value(
                &mut config,
                "closingBracketSameLine",
                false,
                &mut diagnostics,
            ),
            closing_tag_line_break_for_empty: match &*get_value(
                &mut config,
                "closingTagLineBreakForEmpty",
                "fit".to_string(),
                &mut diagnostics,
            ) {
                "always" => ClosingTagLineBreakForEmpty::Always,
                "fit" => ClosingTagLineBreakForEmpty::Fit,
                "never" => ClosingTagLineBreakForEmpty::Never,
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "closingTagLineBreakForEmpty".into(),
                        message: "invalid value for config `closingTagLineBreakForEmpty`".into(),
                    });
                    Default::default()
                }
            },
            max_attrs_per_line: get_nullable_value(
                &mut config,
                "maxAttrsPerLine",
                &mut diagnostics,
            ),
            prefer_attrs_single_line: get_value(
                &mut config,
                "preferAttrsSingleLine",
                false,
                &mut diagnostics,
            ),
            prefer_single_line_opening_tag: get_value(
                &mut config,
                "preferSingleLineOpeningTag",
                false,
                &mut diagnostics,
            ),
            html_normal_self_closing: get_nullable_value(
                &mut config,
                "html.normal.selfClosing",
                &mut diagnostics,
            ),
            html_void_self_closing: get_nullable_value(
                &mut config,
                "html.void.selfClosing",
                &mut diagnostics,
            ),
            component_self_closing: get_nullable_value(
                &mut config,
                "component.selfClosing",
                &mut diagnostics,
            ),
            svg_self_closing: get_nullable_value(&mut config, "svg.selfClosing", &mut diagnostics),
            mathml_self_closing: get_nullable_value(
                &mut config,
                "mathml.selfClosing",
                &mut diagnostics,
            ),
            whitespace_sensitivity: match &*get_value(
                &mut config,
                "whitespaceSensitivity",
                "css".to_string(),
                &mut diagnostics,
            ) {
                "css" => WhitespaceSensitivity::Css,
                "strict" => WhitespaceSensitivity::Strict,
                "ignore" => WhitespaceSensitivity::Ignore,
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "whitespaceSensitivity".into(),
                        message: "invalid value for config `whitespaceSensitivity`".into(),
                    });
                    Default::default()
                }
            },
            component_whitespace_sensitivity: get_nullable_value::<String>(
                &mut config,
                "component.whitespaceSensitivity",
                &mut diagnostics,
            )
            .as_deref()
            .and_then(|option_value| match option_value {
                "css" => Some(WhitespaceSensitivity::Css),
                "strict" => Some(WhitespaceSensitivity::Strict),
                "ignore" => Some(WhitespaceSensitivity::Ignore),
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "component.whitespaceSensitivity".into(),
                        message: "invalid value for config `component.whitespaceSensitivity`"
                            .into(),
                    });
                    Default::default()
                }
            }),
            doctype_keyword_case: match &*get_value(
                &mut config,
                "doctypeKeywordCase",
                "upper".to_string(),
                &mut diagnostics,
            ) {
                "ignore" => DoctypeKeywordCase::Ignore,
                "upper" => DoctypeKeywordCase::Upper,
                "lower" => DoctypeKeywordCase::Lower,
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "doctypeKeywordCase".into(),
                        message: "invalid value for config `doctypeKeywordCase`".into(),
                    });
                    Default::default()
                }
            },
            v_bind_style: get_nullable_value::<String>(&mut config, "vBindStyle", &mut diagnostics)
                .as_deref()
                .and_then(|option_value| match option_value {
                    "short" => Some(VBindStyle::Short),
                    "long" => Some(VBindStyle::Long),
                    _ => {
                        diagnostics.push(ConfigurationDiagnostic {
                            property_name: "vBindStyle".into(),
                            message: "invalid value for config `vBindStyle`".into(),
                        });
                        Default::default()
                    }
                }),
            v_on_style: get_nullable_value::<String>(&mut config, "vOnStyle", &mut diagnostics)
                .as_deref()
                .and_then(|option_value| match option_value {
                    "short" => Some(VOnStyle::Short),
                    "long" => Some(VOnStyle::Long),
                    _ => {
                        diagnostics.push(ConfigurationDiagnostic {
                            property_name: "vOnStyle".into(),
                            message: "invalid value for config `vOnStyle`".into(),
                        });
                        Default::default()
                    }
                }),
            v_for_delimiter_style: get_nullable_value::<String>(
                &mut config,
                "vForDelimiterStyle",
                &mut diagnostics,
            )
            .as_deref()
            .and_then(|option_value| match option_value {
                "in" => Some(VForDelimiterStyle::In),
                "of" => Some(VForDelimiterStyle::Of),
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "vForDelimiterStyle".into(),
                        message: "invalid value for config `vForDelimiterStyle`".into(),
                    });
                    Default::default()
                }
            }),
            v_slot_style: get_nullable_value::<String>(&mut config, "vSlotStyle", &mut diagnostics)
                .as_deref()
                .and_then(|option_value| match option_value {
                    "short" => Some(VSlotStyle::Short),
                    "long" => Some(VSlotStyle::Long),
                    "vSlot" => Some(VSlotStyle::VSlot),
                    _ => {
                        diagnostics.push(ConfigurationDiagnostic {
                            property_name: "vSlotStyle".into(),
                            message: "invalid value for config `vSlotStyle`".into(),
                        });
                        Default::default()
                    }
                }),
            component_v_slot_style: get_nullable_value::<String>(
                &mut config,
                "component.vSlotStyle",
                &mut diagnostics,
            )
            .as_deref()
            .and_then(|option_value| match option_value {
                "short" => Some(VSlotStyle::Short),
                "long" => Some(VSlotStyle::Long),
                "vSlot" => Some(VSlotStyle::VSlot),
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "component.vSlotStyle".into(),
                        message: "invalid value for config `component.vSlotStyle`".into(),
                    });
                    Default::default()
                }
            }),
            default_v_slot_style: get_nullable_value::<String>(
                &mut config,
                "default.vSlotStyle",
                &mut diagnostics,
            )
            .as_deref()
            .and_then(|option_value| match option_value {
                "short" => Some(VSlotStyle::Short),
                "long" => Some(VSlotStyle::Long),
                "vSlot" => Some(VSlotStyle::VSlot),
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "default.vSlotStyle".into(),
                        message: "invalid value for config `default.vSlotStyle`".into(),
                    });
                    Default::default()
                }
            }),
            named_v_slot_style: get_nullable_value::<String>(
                &mut config,
                "named.vSlotStyle",
                &mut diagnostics,
            )
            .as_deref()
            .and_then(|option_value| match option_value {
                "short" => Some(VSlotStyle::Short),
                "long" => Some(VSlotStyle::Long),
                "vSlot" => Some(VSlotStyle::VSlot),
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "named.vSlotStyle".into(),
                        message: "invalid value for config `named.vSlotStyle`".into(),
                    });
                    Default::default()
                }
            }),
            v_bind_same_name_short_hand: get_nullable_value(
                &mut config,
                "vBindSameNameShortHand",
                &mut diagnostics,
            ),
            strict_svelte_attr: get_value(&mut config, "strictSvelteAttr", false, &mut diagnostics),
            svelte_attr_shorthand: get_nullable_value(
                &mut config,
                "svelteAttrShorthand",
                &mut diagnostics,
            ),
            svelte_directive_shorthand: get_nullable_value(
                &mut config,
                "svelteDirectiveShorthand",
                &mut diagnostics,
            ),
            astro_attr_shorthand: get_nullable_value(
                &mut config,
                "astroAttrShorthand",
                &mut diagnostics,
            ),
            script_formatter: get_nullable_value::<String>(
                &mut config,
                "scriptFormatter",
                &mut diagnostics,
            )
            .as_deref()
            .map(|option_value| match option_value {
                "dprint" => ScriptFormatter::Dprint,
                "biome" => ScriptFormatter::Biome,
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "scriptFormatter".into(),
                        message: "invalid value for config `scriptFormatter`".into(),
                    });
                    ScriptFormatter::Dprint
                }
            })
            .or(Some(ScriptFormatter::Dprint)),
            ignore_comment_directive: get_value(
                &mut config,
                "ignoreCommentDirective",
                "markup-fmt-ignore".into(),
                &mut diagnostics,
            ),
            ignore_file_comment_directive: get_value(
                &mut config,
                "ignoreFileCommentDirective",
                "dprint-ignore-file".into(),
                &mut diagnostics,
            ),
        },
    };

    diagnostics.extend(get_unknown_property_diagnostics(config));

    ResolveConfigurationResult {
        config: markup_fmt_config,
        diagnostics,
    }
}
