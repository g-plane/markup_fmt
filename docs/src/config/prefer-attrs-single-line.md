# `preferAttrsSingleLine`

Control whether attributes should be put on single line when possible.

This option conflicts with [`maxAttrsPerLine`](./max-attrs-per-line.md) option.

Default option is `false`.

## Example for `false`

This `<div>` is short enough to be put on single line,
but it won't because attributes in source code span multiple lines.

```html
<div
  data-a
  data-b data-c
></div>
```

will be formatted as:

```html
<div
  data-a
  data-b
  data-c
></div>
```

## Example for `true`

This `<div>` is short enough so it will be put on single line
though attributes span multiple lines in source code.

```html
<div data-a
  data-b
  data-c></div>
```

will be formatted as:

```html
<div data-a data-b data-c></div>
```
