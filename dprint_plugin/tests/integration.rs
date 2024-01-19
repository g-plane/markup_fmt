use insta::{assert_snapshot, glob, Settings};
use std::{fs, path::Path, process::Command};

#[test]
fn integration_snapshot() {
    glob!("integration/**/*.{html,vue,svelte,jinja}", |path| {
        let file = fs::File::open(path).unwrap();

        let output = Command::new("../node_modules/.bin/dprint")
            .arg("fmt")
            .arg("--stdin")
            .arg(path)
            .arg("--plugins")
            .arg("../target/wasm32-unknown-unknown/debug/dprint_plugin_markup.wasm")
            .arg("https://plugins.dprint.dev/g-plane/malva-v0.1.4.wasm")
            .arg("https://plugins.dprint.dev/typescript-0.88.9.wasm")
            .stdin(file)
            .output()
            .unwrap()
            .stdout;

        build_settings(path).bind(|| {
            let name = path.file_name().unwrap().to_str().unwrap();
            assert_snapshot!(name, String::from_utf8(output).unwrap());
        });
    });
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
