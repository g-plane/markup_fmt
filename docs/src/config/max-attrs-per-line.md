# `maxAttrsPerLine`

Control the maximum number of attributes in one line.
If this option is unset, there won't be any limitations.

This option conflicts with [`preferAttrsSingleLine`](./prefer-attrs-single-line.md) option.

Default option is `null`. This option can't be `0`.

## Example for `null`

```html
<div data-a data-b data-c></div>
```

## Example for `1`

```html
<div
  data-a
  data-b
  data-c
></div>
```

## Example for `2`

```html
<div
  data-a data-b
  data-c
></div>
```
