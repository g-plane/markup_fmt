use anyhow::Error;
use dprint_core::configuration::GlobalConfiguration;
use markup_fmt::{config::FormatOptions, detect_language, format_text};
use std::{borrow::Cow, env, fs, io, path::Path};

fn main() {
    let file_path = env::args().nth(1).unwrap();
    let code = fs::read_to_string(&file_path).unwrap();
    let mut options = match fs::read_to_string("markup_fmt.toml") {
        Ok(s) => toml::from_str(&s).unwrap(),
        Err(error) => {
            if error.kind() == io::ErrorKind::NotFound {
                FormatOptions::default()
            } else {
                panic!("{error}");
            }
        }
    };
    options.language.script_formatter = Some(markup_fmt::config::ScriptFormatter::Dprint);

    let formatted = format_text(
        &code,
        detect_language(&file_path).unwrap(),
        &options,
        |code, hints| -> anyhow::Result<Cow<str>> {
            let ext = hints.ext;
            let additional_config = dprint_plugin_markup::build_additional_config(hints, &options);
            let global_config = GlobalConfiguration {
                line_width: Some(options.layout.print_width as u32),
                use_tabs: Some(options.layout.use_tabs),
                indent_width: Some(options.layout.indent_width as u8),
                ..Default::default()
            };
            if let Some(syntax) = malva::detect_syntax(&Path::new("file").with_extension(ext)) {
                malva::format_text(
                    code,
                    syntax,
                    &serde_json::to_value(additional_config).and_then(serde_json::from_value)?,
                )
                .map(Cow::from)
                .map_err(Error::from)
            } else if ext == "json" {
                dprint_plugin_json::format_text(
                    &Path::new("file").with_extension(ext),
                    code,
                    &dprint_plugin_json::configuration::resolve_config(
                        additional_config,
                        &global_config,
                    )
                    .config,
                )
                .map(|formatted| {
                    if let Some(formatted) = formatted {
                        Cow::from(formatted)
                    } else {
                        Cow::from(code)
                    }
                })
            } else if matches!(ext, "tsx" | "ts" | "mts" | "jsx" | "js" | "mjs") {
                dprint_plugin_typescript::format_text(dprint_plugin_typescript::FormatTextOptions {
                    path: &Path::new("file").with_extension(ext),
                    extension: Some(ext),
                    text: code.to_owned(),
                    config: &dprint_plugin_typescript::configuration::resolve_config(
                        additional_config,
                        &global_config,
                    )
                    .config,
                    external_formatter: None,
                })
                .map(|formatted| {
                    if let Some(formatted) = formatted {
                        Cow::from(formatted)
                    } else {
                        Cow::from(code)
                    }
                })
            } else {
                Ok(Cow::from(code))
            }
        },
    )
    .unwrap();
    print!("{formatted}");
}
