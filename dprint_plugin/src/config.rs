use dprint_core::configuration::{
    get_unknown_property_diagnostics, get_value, ConfigKeyMap, ConfigurationDiagnostic,
    GlobalConfiguration, NewLineKind, ResolveConfigurationResult,
};
use markup_fmt::config::{FormatOptions, LayoutOptions, LineBreak};

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
        language: Default::default(), // TODO
    };

    diagnostics.extend(get_unknown_property_diagnostics(config));

    ResolveConfigurationResult {
        config: markup_fmt_config,
        diagnostics,
    }
}
