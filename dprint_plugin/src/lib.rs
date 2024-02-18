use crate::config::resolve_config;
use anyhow::Result;
#[cfg(target_arch = "wasm32")]
use dprint_core::generate_plugin_code;
use dprint_core::{
    configuration::{ConfigKeyMap, GlobalConfiguration, ResolveConfigurationResult},
    plugins::{FileMatchingInfo, PluginInfo, SyncPluginHandler, SyncPluginInfo},
};
use markup_fmt::{config::FormatOptions, detect_language, format_text, FormatError};
use std::path::Path;

mod config;

#[cfg(target_arch = "wasm32")]
type Configuration = FormatOptions;

pub struct MarkupFmtPluginHandler {}

impl SyncPluginHandler<FormatOptions> for MarkupFmtPluginHandler {
    fn plugin_info(&mut self) -> SyncPluginInfo {
        let version = env!("CARGO_PKG_VERSION").to_string();
        SyncPluginInfo {
            info: PluginInfo {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: version.clone(),
                config_key: "markup".to_string(),
                help_url: "https://github.com/g-plane/markup_fmt".to_string(),
                config_schema_url: format!(
                    "https://plugins.dprint.dev/g-plane/markup_fmt/v{}/schema.json",
                    version
                ),
                update_url: Some(
                    "https://plugins.dprint.dev/g-plane/markup_fmt/latest.json".into(),
                ),
            },
            file_matching: FileMatchingInfo {
                file_extensions: ["html", "vue", "svelte", "astro", "jinja", "jinja2", "twig"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                file_names: vec![],
            },
        }
    }

    fn license_text(&mut self) -> String {
        include_str!("../../LICENSE").into()
    }

    fn resolve_config(
        &mut self,
        config: ConfigKeyMap,
        global_config: &GlobalConfiguration,
    ) -> ResolveConfigurationResult<FormatOptions> {
        resolve_config(config, global_config)
    }

    fn format(
        &mut self,
        file_path: &Path,
        file_text: Vec<u8>,
        config: &FormatOptions,
        mut format_with_host: impl FnMut(&Path, Vec<u8>, &ConfigKeyMap) -> Result<Option<Vec<u8>>>,
    ) -> Result<Option<Vec<u8>>> {
        // falling back to HTML allows to format files with unknown extensions, such as .svg
        let language = detect_language(file_path).unwrap_or(markup_fmt::Language::Html);

        let format_result = format_text(
            std::str::from_utf8(&file_text)?,
            language,
            config,
            |path, code, print_width| {
                let mut additional_config = ConfigKeyMap::new();
                additional_config.insert("lineWidth".into(), (print_width as i32).into());
                additional_config.insert("printWidth".into(), (print_width as i32).into());
                if let Some("expr.ts" | "binding.ts" | "type_params.ts") =
                    path.file_name().and_then(|s| s.to_str())
                {
                    // dprint-plugin-typescript
                    additional_config.insert("semiColons".into(), "asi".into());
                    // Biome
                    additional_config.insert("semicolons".into(), "asNeeded".into());
                }

                format_with_host(path, code.into(), &additional_config).and_then(|result| {
                    match result {
                        Some(code) => String::from_utf8(code)
                            .map(|s| s.into())
                            .map_err(anyhow::Error::from),
                        None => Ok(code.into()),
                    }
                })
            },
        );
        match format_result {
            Ok(code) => Ok(Some(code.into_bytes())),
            Err(FormatError::Syntax(err)) => Err(err.into()),
            Err(FormatError::External(err, code)) => Err(anyhow::anyhow!(
                "[markup_fmt] failed to format code with external formatter: `{code}`:\n{err}"
            )),
        }
    }
}

#[cfg(target_arch = "wasm32")]
generate_plugin_code!(MarkupFmtPluginHandler, MarkupFmtPluginHandler {});
