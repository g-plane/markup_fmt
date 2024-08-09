use anyhow::Error;
use insta::{assert_snapshot, glob, Settings};
use markup_fmt::{detect_language, format_text};
use std::{borrow::Cow, fs, path::Path};

#[test]
fn integration_with_dprint_ts_snapshot() {
    glob!(
        "integration/**/*.{html,vue,svelte,astro,jinja,njk,vto}",
        |path| {
            let input = fs::read_to_string(path).unwrap();
            let options = Default::default();

            let output = format_text(
                &input,
                detect_language(path).unwrap(),
                &options,
                |path, code, print_width| -> anyhow::Result<Cow<str>> {
                    let additional_config =
                        dprint_plugin_markup::build_additional_config(path, print_width, &options);
                    if let Some(syntax) = malva::detect_syntax(path) {
                        malva::format_text(
                            code,
                            syntax,
                            &serde_json::to_value(additional_config)
                                .and_then(serde_json::from_value)?,
                        )
                        .map(Cow::from)
                        .map_err(Error::from)
                    } else if path.extension().unwrap().to_str().unwrap() == "json" {
                        dprint_plugin_json::format_text(
                            path,
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
                            path,
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
            .map_err(|err| format!("failed to format '{}': {:?}", path.display(), err))
            .unwrap();

            assert!(
                output.ends_with('\n'),
                "formatted output should contain trailing newline: {}",
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
            let options = Default::default();

            let output = format_text(
                &input,
                detect_language(path).unwrap(),
                &options,
                |path, code, print_width| -> anyhow::Result<Cow<str>> {
                    let additional_config =
                        dprint_plugin_markup::build_additional_config(path, print_width, &options);
                    if let Some(syntax) = malva::detect_syntax(path) {
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
                            path,
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
            .map_err(|err| format!("failed to format '{}': {:?}", path.display(), err))
            .unwrap();

            assert!(
                output.ends_with('\n'),
                "formatted output should contain trailing newline: {}",
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
