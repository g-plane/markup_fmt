use markup_fmt::{format_text, Language};
use std::{env, fs, path::Path};

fn main() {
    let file_path = env::args().nth(1).unwrap();
    let language = match Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        Some("html") => Language::Html,
        Some("vue") => Language::Vue,
        Some("svelte") => Language::Svelte,
        _ => panic!("Unsupported file extension"),
    };
    let code = fs::read_to_string(file_path).unwrap();

    let formatted = format_text(&code, language, &Default::default(), |_, code, _| {
        Ok::<_, ()>(code.into())
    })
    .unwrap();
    print!("{formatted}");
}
