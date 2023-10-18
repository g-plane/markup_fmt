use dprint_core::configuration::{
    get_nullable_value, get_unknown_property_diagnostics, get_value, ConfigKeyMap,
    ConfigurationDiagnostic, GlobalConfiguration, NewLineKind, ResolveConfigurationResult,
};
use markup_fmt::config::{
    ClosingTagLineBreakForEmpty, FormatOptions, LanguageOptions, LayoutOptions, LineBreak, Quotes,
    VBindStyle, VForDelimiterStyle, VOnStyle,
};

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
            style_indent: get_value(&mut config, "styleIndent", false, &mut diagnostics),
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
            v_bind_style: match get_nullable_value::<String>(
                &mut config,
                "vBindStyle",
                &mut diagnostics,
            )
            .as_deref()
            {
                Some("short") => Some(VBindStyle::Short),
                Some("long") => Some(VBindStyle::Long),
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "vBindStyle".into(),
                        message: "invalid value for config `vBindStyle`".into(),
                    });
                    Default::default()
                }
            },
            v_on_style: match get_nullable_value::<String>(
                &mut config,
                "vOnStyle",
                &mut diagnostics,
            )
            .as_deref()
            {
                Some("short") => Some(VOnStyle::Short),
                Some("long") => Some(VOnStyle::Long),
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "vOnStyle".into(),
                        message: "invalid value for config `vOnStyle`".into(),
                    });
                    Default::default()
                }
            },
            v_for_delimiter_style: match get_nullable_value::<String>(
                &mut config,
                "vForDelimiterStyle",
                &mut diagnostics,
            )
            .as_deref()
            {
                Some("in") => Some(VForDelimiterStyle::In),
                Some("of") => Some(VForDelimiterStyle::Of),
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "vForDelimiterStyle".into(),
                        message: "invalid value for config `vForDelimiterStyle`".into(),
                    });
                    Default::default()
                }
            },
        },
    };

    diagnostics.extend(get_unknown_property_diagnostics(config));

    ResolveConfigurationResult {
        config: markup_fmt_config,
        diagnostics,
    }
}
