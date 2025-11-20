use anyhow::Error;
use dprint_core::configuration::GlobalConfiguration;
use insta::{Settings, assert_snapshot, glob};
use markup_fmt::{
    FormatError,
    config::{FormatOptions, ScriptFormatter},
    detect_language, format_text,
};
use std::{borrow::Cow, fs, io, path::Path};

#[test]
fn integration_with_dprint_ts_snapshot() {
    fn format_with_dprint_ts(input: &str, path: &Path) -> Result<String, FormatError<Error>> {
        let mut options = match fs::read_to_string(path.with_extension("toml")) {
            Ok(file) => toml::from_str::<FormatOptions>(&file).unwrap(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Default::default(),
            Err(e) => panic!("{e}"),
        };
        let file_name = path.file_name().and_then(|file_name| file_name.to_str());
        if file_name.is_some_and(|file_name| file_name.starts_with("deno")) {
            options.language.script_indent = true;
            options.language.style_indent = true;
        }
        options.language.script_formatter = Some(ScriptFormatter::Dprint);
        format_text(
            input,
            detect_language(path).unwrap(),
            &options,
            |code, hints| -> anyhow::Result<Cow<str>> {
                let ext = hints.ext;
                let additional_config =
                    dprint_plugin_markup::build_additional_config(hints, &options);
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
                        &serde_json::to_value(additional_config)
                            .and_then(serde_json::from_value)?,
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
                    dprint_plugin_typescript::format_text(
                        dprint_plugin_typescript::FormatTextOptions {
                            path: &Path::new("file").with_extension(ext),
                            extension: Some(ext),
                            text: code.to_owned(),
                            config: &dprint_plugin_typescript::configuration::resolve_config(
                                additional_config,
                                &global_config,
                            )
                            .config,
                            external_formatter: None,
                        },
                    )
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
    }

    glob!(
        "integration/**/*.{html,vue,svelte,astro,jinja,njk,vto}",
        |path| {
            let input = fs::read_to_string(path).unwrap();
            let output = format_with_dprint_ts(&input, path)
                .map_err(|err| format!("failed to format '{}': {:?}", path.display(), err))
                .unwrap();

            assert!(
                output.ends_with('\n'),
                "formatted output should contain trailing newline: {}",
                path.display()
            );

            let regression_format = format_with_dprint_ts(&output, path)
                .map_err(|err| {
                    format!(
                        "syntax error in stability test '{}': {:?}",
                        path.display(),
                        err
                    )
                })
                .unwrap();
            similar_asserts::assert_eq!(
                output,
                regression_format,
                "'{}' format is unstable",
                path.display()
            );

            build_settings(path.parent().unwrap().join("dprint_ts")).bind(|| {
                let name = path.file_name().unwrap().to_str().unwrap();
                assert_snapshot!(name, output);
            });
        }
    );
}

fn build_settings(path: impl AsRef<Path>) -> Settings {
    let mut settings = Settings::clone_current();
    settings.set_snapshot_path(path);
    settings.remove_snapshot_suffix();
    settings.set_prepend_module_to_snapshot(false);
    settings.remove_input_file();
    settings.set_omit_expression(true);
    settings.remove_input_file();
    settings.remove_info();
    settings
}
