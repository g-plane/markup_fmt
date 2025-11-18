use crate::config::resolve_config;
use anyhow::Result;
use dprint_core::{
    configuration::{ConfigKeyMap, GlobalConfiguration},
    plugins::{
        CheckConfigUpdatesMessage, ConfigChange, FormatResult, PluginInfo,
        PluginResolveConfigurationResult, SyncFormatRequest, SyncHostFormatRequest,
        SyncPluginHandler,
    },
};
use markup_fmt::{
    FormatError, Hints,
    config::{FormatOptions, Quotes, ScriptFormatter},
    detect_language, format_text,
};

mod config;

pub struct MarkupFmtPluginHandler;

impl SyncPluginHandler<FormatOptions> for MarkupFmtPluginHandler {
    fn plugin_info(&mut self) -> PluginInfo {
        let version = env!("CARGO_PKG_VERSION").to_string();
        PluginInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: version.clone(),
            config_key: "markup".to_string(),
            help_url: "https://github.com/g-plane/markup_fmt".to_string(),
            config_schema_url: format!(
                "https://plugins.dprint.dev/g-plane/markup_fmt/v{version}/schema.json",
            ),
            update_url: Some("https://plugins.dprint.dev/g-plane/markup_fmt/latest.json".into()),
        }
    }

    fn license_text(&mut self) -> String {
        include_str!("../../LICENSE").into()
    }

    fn resolve_config(
        &mut self,
        config: ConfigKeyMap,
        global_config: &GlobalConfiguration,
    ) -> PluginResolveConfigurationResult<FormatOptions> {
        resolve_config(config, global_config)
    }

    fn check_config_updates(&self, _: CheckConfigUpdatesMessage) -> Result<Vec<ConfigChange>> {
        Ok(Vec::new())
    }

    fn format(
        &mut self,
        request: SyncFormatRequest<FormatOptions>,
        mut format_with_host: impl FnMut(SyncHostFormatRequest) -> FormatResult,
    ) -> FormatResult {
        // falling back to HTML allows to format files with unknown extensions, such as .svg
        let language = detect_language(request.file_path).unwrap_or(markup_fmt::Language::Html);

        let format_result = format_text(
            std::str::from_utf8(&request.file_bytes)?,
            language,
            request.config,
            |code, hints| {
                let mut file_name = request
                    .file_path
                    .file_name()
                    .expect("missing file name")
                    .to_owned();
                file_name.push("#.");
                file_name.push(hints.ext);
                let additional_config = build_additional_config(hints, request.config);
                format_with_host(SyncHostFormatRequest {
                    file_path: &request.file_path.with_file_name(file_name),
                    file_bytes: code.as_bytes(),
                    range: None,
                    override_config: &additional_config,
                })
                .and_then(|result| match result {
                    Some(code) => String::from_utf8(code)
                        .map(|s| s.into())
                        .map_err(anyhow::Error::from),
                    None => Ok(code.into()),
                })
            },
        );
        match format_result {
            Ok(code) => Ok(Some(code.into_bytes())),
            Err(FormatError::Syntax(err)) => Err(err.into()),
            Err(FormatError::External(errors)) => {
                let msg = errors.into_iter().fold(
                    String::from("failed to format code with external formatter:\n"),
                    |mut msg, error| {
                        msg.push_str(&format!("{error}\n"));
                        msg
                    },
                );
                Err(anyhow::anyhow!(msg))
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
dprint_core::generate_plugin_code!(
    MarkupFmtPluginHandler,
    MarkupFmtPluginHandler,
    FormatOptions
);

#[doc(hidden)]
pub fn build_additional_config(hints: Hints, config: &FormatOptions) -> ConfigKeyMap {
    let mut additional_config = ConfigKeyMap::new();
    additional_config.insert("lineWidth".into(), (hints.print_width as i32).into());
    additional_config.insert("printWidth".into(), (hints.print_width as i32).into());
    additional_config.insert("fileIndentLevel".into(), (hints.indent_level as i32).into());

    if hints.attr {
        match config.language.quotes {
            Quotes::Double => {
                if matches!(
                    config.language.script_formatter,
                    Some(ScriptFormatter::Biome)
                ) {
                    additional_config.insert("javascriptQuoteStyle".into(), "single".into());
                } else {
                    additional_config.insert("quoteStyle".into(), "alwaysSingle".into());
                }
            }
            Quotes::Single => {
                if matches!(
                    config.language.script_formatter,
                    Some(ScriptFormatter::Biome)
                ) {
                    additional_config.insert("javascriptQuoteStyle".into(), "double".into());
                } else {
                    additional_config.insert("quoteStyle".into(), "alwaysDouble".into());
                }
            }
        }
        if hints.ext == "css" {
            additional_config.insert("singleLineTopLevelDeclarations".into(), true.into());
        }
    }

    additional_config
}
