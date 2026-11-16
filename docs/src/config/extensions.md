# `extensions`

Adds or overrides extensions with specified languages.

This should be configured as an object whose keys are file extensions and values are languages.
For example:

```json
{
  "tpl": "jinja"
}
```

This will treat files whose extensions are `tpl` (such as `page.tpl`) as Jinja templates.

Available languages:

- `"html"`
- `"vue"`
- `"svelte"`
- `"astro"`
- `"angular"`
- `"jinja"`
- `"vento"`
- `"mustache"`
- `"xml"`

Default value is `{}`, an empty object.

This option is only available for dprint plugin, not Rust API.
