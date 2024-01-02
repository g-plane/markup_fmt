use markup_fmt::{detect_language, format_text};
use std::{env, fs};

fn main() {
    let file_path = env::args().nth(1).unwrap();
    let language = detect_language(&file_path).unwrap();
    let code = fs::read_to_string(file_path).unwrap();

    let formatted = format_text(&code, language, &Default::default(), |_, code, _| {
        Ok::<_, ()>(code.into())
    })
    .unwrap();
    print!("{formatted}");
}
