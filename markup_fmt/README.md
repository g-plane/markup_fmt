markup_fmt is a configurable HTML, Vue, Svelte, Astro, Angular, Jinja, Twig, Nunjucks, Vento, Mustache, Handlebars and XML formatter.

## Basic Usage

You can format source code string by using [`format_text`] function.

```rust
use markup_fmt::{config::FormatOptions, format_text, Language};

let options = FormatOptions::default();
assert_eq!("<div class=\"container\"></div>\n", &format_text(
    "<div class=container></div>",
    Language::Html,
    &options,
    |code, _| Ok::<_, std::convert::Infallible>(code.into()),
).unwrap());
```

For detailed documentation of configuration,
please refer to [Configuration](https://markup-fmt.netlify.app/) on GitHub.

If there're syntax errors in source code, it will return [`Err`]:

```rust
use markup_fmt::{config::FormatOptions, format_text, FormatError, Language, SyntaxError};

let options = FormatOptions::default();
assert!(matches!(
    format_text(
        "<div>",
        Language::Html,
        &options,
        |code, _| Ok::<_, std::convert::Infallible>(code.into()),
    ).unwrap_err(),
    FormatError::Syntax(SyntaxError { .. })
));
```

External formatter can return [`Err`] as well.
This error will be aggregated and returned in [`FormatError::External`]:

```rust
use markup_fmt::{config::FormatOptions, format_text, FormatError, Language};

struct ExternalFormatterError;

let options = FormatOptions::default();
assert!(matches!(
    format_text(
        "<script>a</script>",
        Language::Html,
        &options,
        |_, _| Err(ExternalFormatterError),
    ).unwrap_err(),
    FormatError::External(errors) if !errors.is_empty()
));
```
