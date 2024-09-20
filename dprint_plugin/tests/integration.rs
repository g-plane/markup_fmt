use anyhow::Error;
use insta::{assert_snapshot, glob, Settings};
use markup_fmt::{detect_language, format_text, FormatError};
use std::{borrow::Cow, fs, io, path::Path};

#[test]
fn integration_with_dprint_ts_snapshot() {
    fn format_with_dprint_ts(input: &str, path: &Path) -> Result<String, FormatError<Error>> {
        let options = match fs::read_to_string(path.with_extension("toml")) {
            Ok(file) => toml::from_str(&file).unwrap(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Default::default(),
            Err(e) => panic!("{e}"),
        };
        format_text(
            input,
            detect_language(path).unwrap(),
            &options,
            |code, hints| -> anyhow::Result<Cow<str>> {
                let ext = hints.ext;
                let additional_config =
                    dprint_plugin_markup::build_additional_config(hints, &options);
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
                            &Default::default(),
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
                } else {
                    dprint_plugin_typescript::format_text(
                        &Path::new("file").with_extension(ext),
                        code.to_owned(),
                        &dprint_plugin_typescript::configuration::resolve_config(
                            additional_config,
                            &Default::default(),
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

#[test]
fn integration_with_biome_snapshot() {
    fn format_with_biome(input: &str, path: &Path) -> Result<String, FormatError<Error>> {
        let options = match fs::read_to_string(path.with_extension("toml")) {
            Ok(file) => toml::from_str(&file).unwrap(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Default::default(),
            Err(e) => panic!("{e}"),
        };
        format_text(
            &input,
            detect_language(path).unwrap(),
            &options,
            |code, hints| -> anyhow::Result<Cow<str>> {
                let ext = hints.ext;
                let mut additional_config =
                    dprint_plugin_markup::build_additional_config(hints, &options);
                additional_config.insert("useTabs".into(), false.into());
                if let Some(syntax) = malva::detect_syntax(&Path::new("file").with_extension(ext)) {
                    malva::format_text(
                        code,
                        syntax,
                        &serde_json::to_value(additional_config)
                            .and_then(serde_json::from_value)?,
                    )
                    .map(Cow::from)
                    .map_err(Error::from)
                } else {
                    dprint_plugin_biome::format_text(
                        &Path::new("file").with_extension(ext),
                        code,
                        &serde_json::to_value(additional_config)
                            .and_then(serde_json::from_value)
                            .unwrap_or_default(),
                    )
                    .map(|formatted| {
                        if let Some(formatted) = formatted {
                            Cow::from(formatted)
                        } else {
                            Cow::from(code)
                        }
                    })
                }
            },
        )
    }

    glob!(
        "integration/**/*.{html,vue,svelte,astro,jinja,njk,vto}",
        |path| {
            let file_name = path.file_name().and_then(|file_name| file_name.to_str());
            if let Some("return.astro") = file_name {
                // dprint-plugin-biome doesn't support top-level return (but Biome supports),
                // so skip it.
                return;
            }

            let input = fs::read_to_string(path).unwrap();
            let output = format_with_biome(&input, path)
                .map_err(|err| format!("failed to format '{}': {:?}", path.display(), err))
                .unwrap();

            assert!(
                output.ends_with('\n'),
                "formatted output should contain trailing newline: {}",
                path.display()
            );

            let regression_format = format_with_biome(&output, path)
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

            build_settings(path.parent().unwrap().join("biome")).bind(|| {
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
