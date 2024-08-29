# `*.selfClosing`

This group of options controls whether an element should be self-closed or not if
it doesn't have children.

There're several options:

- `html.normal.selfClosing`: This option affects on HTML normal elements.
- `html.void.selfClosing`: This option affects on HTML void elements.
- `component.selfClosing`: This option affects on Vue, Svelte, Astro and Angular components.
- `svg.selfClosing`: This option affects on SVG elements.
- `mathml.selfClosing`: This option affects on MathML elements.

All of these options can be set as `null`, `true` or `false`:

- `null`: Whether the element is self-closed or not won't be changed. Keep it as-is.
- `true`: Element will always be self-closed.
- `false`: Element will never be self-closed.

All of these options are default to `null`.

## Example for `null`

Input:

```html
<div />
<div></div>
```

Output:

```html
<div />
<div></div>
```

## Example for `true`

Input:

```html
<div />
<div></div>
```

Output:

```html
<div />
<div />
```

## Example for `false`

Input:

```html
<div />
<div></div>
```

Output:

```html
<div></div>
<div></div>
```
