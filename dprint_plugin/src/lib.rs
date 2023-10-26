use crate::config::resolve_config;
use anyhow::Result;
#[cfg(target_arch = "wasm32")]
use dprint_core::generate_plugin_code;
use dprint_core::{
    configuration::{ConfigKeyMap, GlobalConfiguration, ResolveConfigurationResult},
    plugins::{FileMatchingInfo, PluginInfo, SyncPluginHandler, SyncPluginInfo},
};
use markup_fmt::{config::FormatOptions, format_text, FormatError, Language};
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
                    "https://plugins.dprint.dev/g-plane/markup_fmt/{}/schema.json",
                    version
                ),
                update_url: Some(
                    "https://plugins.dprint.dev/g-plane/markup_fmt/latest.json".into(),
                ),
            },
            file_matching: FileMatchingInfo {
                file_extensions: vec!["html".into(), "vue".into(), "svelte".into()],
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
        file_text: &str,
        config: &FormatOptions,
        mut format_with_host: impl FnMut(&Path, String, &ConfigKeyMap) -> Result<Option<String>>,
    ) -> Result<Option<String>> {
        let language = match file_path.extension().and_then(|s| s.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("html") => Language::Html,
            Some(ext) if ext.eq_ignore_ascii_case("vue") => Language::Vue,
            Some(ext) if ext.eq_ignore_ascii_case("svelte") => Language::Svelte,
            _ => {
                return Err(anyhow::anyhow!(
                    "unknown file extension of file: {}",
                    file_path.display()
                ));
            }
        };

        let format_result = format_text(file_text, language, config, |path, code, print_width| {
            let mut additional_config = ConfigKeyMap::new();
            additional_config.insert("lineWidth".into(), (print_width as i32).into());
            additional_config.insert("printWidth".into(), (print_width as i32).into());
            if let Some("expr.ts" | "binding.ts" | "type_params.ts") =
                path.file_name().and_then(|s| s.to_str())
            {
                additional_config.insert("semiColons".into(), "asi".into());
            }

            format_with_host(path, code.into(), &additional_config).map(|result| match result {
                Some(code) => code.into(),
                None => code.into(),
            })
        });
        match format_result {
            Ok(code) => Ok(Some(code)),
            Err(FormatError::Syntax(err)) => Err(err.into()),
            Err(FormatError::External(err)) => Err(err),
        }
    }
}

#[cfg(target_arch = "wasm32")]
generate_plugin_code!(MarkupFmtPluginHandler, MarkupFmtPluginHandler {});
