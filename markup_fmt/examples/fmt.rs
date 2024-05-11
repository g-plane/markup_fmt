use markup_fmt::{config::FormatOptions, detect_language, format_text};
use std::{convert::Infallible, env, error::Error, fs, io};

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = env::args().nth(1).unwrap();
    let language = detect_language(&file_path).unwrap();
    let code = fs::read_to_string(file_path)?;
    let options = match fs::read_to_string("markup_fmt.toml") {
        Ok(s) => toml::from_str(&s)?,
        Err(error) => {
            if error.kind() == io::ErrorKind::NotFound {
                FormatOptions::default()
            } else {
                return Err(Box::new(error));
            }
        }
    };

    let formatted = format_text(&code, language, &options, |_, code, _| {
        Ok::<_, Infallible>(code.into())
    })?;
    print!("{formatted}");
    Ok(())
}
