use insta::{assert_snapshot, glob, Settings};
use markup_fmt::{format_text, Language};
use std::fs;

#[test]
fn fmt_snapshot() {
    glob!("fmt/**/*.{html,vue,svelte}", |path| {
        let input = fs::read_to_string(path).unwrap();
        let language = match path.extension().unwrap().to_str().unwrap() {
            "html" => Language::Html,
            "vue" => Language::Vue,
            "svelte" => Language::Svelte,
            _ => unreachable!("unknown file extension"),
        };

        let options = fs::read_to_string(path.with_file_name("config.toml"))
            .map(|config_file| toml::from_str(&config_file).unwrap())
            .unwrap_or_default();

        let output = format_text(&input, language.clone(), &options, |_, code, _| {
            Ok::<_, ()>(code.into())
        })
        .map_err(|err| format!("failed to format '{}': {:?}", path.display(), err))
        .unwrap();
        let regression_format = format_text(&output, language, &options, |_, code, _| {
            Ok::<_, ()>(code.into())
        })
        .unwrap();
        assert_eq!(
            output,
            regression_format,
            "'{}' format is unstable",
            path.display()
        );

        let mut settings = Settings::clone_current();
        settings.set_snapshot_path(path.parent().unwrap());
        settings.remove_snapshot_suffix();
        settings.set_prepend_module_to_snapshot(false);
        settings.remove_input_file();
        settings.set_omit_expression(true);
        settings.remove_input_file();
        settings.remove_info();
        settings.bind(|| {
            let name = path.file_stem().unwrap().to_str().unwrap();
            assert_snapshot!(name, output);
        });
    });
}
