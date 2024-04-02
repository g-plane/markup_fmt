use insta::{assert_snapshot, glob, Settings};
use markup_fmt::{config::FormatOptions, detect_language, format_text, Language};
use std::{collections::HashMap, fs, path::Path};

#[test]
fn fmt_snapshot() {
    glob!("fmt/**/*.{html,vue,svelte,astro,jinja,njk,vto}", |path| {
        let input = fs::read_to_string(path).unwrap();
        let language = detect_language(path).unwrap();

        let options = fs::read_to_string(path.with_file_name("config.toml"))
            .map(|config_file| {
                toml::from_str::<HashMap<String, FormatOptions>>(&config_file).unwrap()
            })
            .ok();

        if let Some(options) = options {
            options.into_iter().for_each(|(option_name, options)| {
                let output = run_format_test(path, &input, &options, language.clone());
                build_settings(path).bind(|| {
                    let name = path.file_stem().unwrap().to_str().unwrap();
                    assert_snapshot!(format!("{name}.{option_name}"), output);
                });
            })
        } else {
            let output = run_format_test(path, &input, &Default::default(), language);
            build_settings(path).bind(|| {
                let name = path.file_stem().unwrap().to_str().unwrap();
                assert_snapshot!(name, output);
            });
        }
    });
}

fn run_format_test(
    path: &Path,
    input: &str,
    options: &FormatOptions,
    language: Language,
) -> String {
    let output = format_text(&input, language.clone(), &options, |_, code, _| {
        Ok::<_, ()>(code.into())
    })
    .map_err(|err| format!("failed to format '{}': {:?}", path.display(), err))
    .unwrap();
    let regression_format = format_text(&output, language, &options, |_, code, _| {
        Ok::<_, ()>(code.into())
    })
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

    output
}

fn build_settings(path: &Path) -> Settings {
    let mut settings = Settings::clone_current();
    settings.set_snapshot_path(path.parent().unwrap());
    settings.remove_snapshot_suffix();
    settings.set_prepend_module_to_snapshot(false);
    settings.remove_input_file();
    settings.set_omit_expression(true);
    settings.remove_input_file();
    settings.remove_info();
    settings
}
