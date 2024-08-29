# `scriptIndent`

Control whether the code block in the `<script>` tag should be indented or not.

Default option is `false`.

This global option can be overridden for individual languages by the following options:

- `html.scriptIndent`
- `vue.scriptIndent`
- `svelte.scriptIndent`
- `astro.scriptIndent`

## Example for `false`

```html
<script>
const a = 0
</script>
```

## Example for `true`

```html
<script>
  const a = 0
</script>
```
