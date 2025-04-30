# `singleAttrSameLine`

Control whether single attribute should be put on the same line with tag name.

Default option is `true`.

## Example for `false`

Input:

```html
<div class="app"></div>
<div
  class="app"
></div>
```

Output:

```html
<div class="app"></div>
<div
  class="app"
></div>
```

## Example for `true`

Input:

```html
<div class="app"></div>
<div
  class="app"
></div>
```

Output:

```html
<div class="app"></div>
<div class="app"></div>
```
