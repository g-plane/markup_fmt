# `styleIndent`

Control whether the code block in the `<style>` tag should be indented or not.

Default option is `false`.

This global option can be overridden for individual languages by the following options:

- `html.styleIndent`
- `vue.styleIndent`
- `svelte.styleIndent`
- `astro.styleIndent`

## Example for `false`

```html
<style>
a { outline: none; }
</style>
```

## Example for `true`

```html
<style>
  a { outline: none; }
</style>
```
