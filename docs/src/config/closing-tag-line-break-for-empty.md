# `closingTagLineBreakForEmpty`

When there're no children in an element,
this option controls whether to insert a line break before the closing tag or not.

Possible options:

- `"always"`: Always insert a line break before the closing tag.
- `"fit"`: Only insert a line break if it doesn't fit the [`printWidth`](./print-width.md) option.
- `"never"`: Don't insert a line break.

Default option is `"fit"`.

## Example for `"always"`

```html
<div>
</div>
<div class="very-very-very-very-very-very-very-very-very-long-class-name">
</div>
```

## Example for `"fit"`

```html
<div></div>
<div class="very-very-very-very-very-very-very-very-very-long-class-name">
</div>
```

## Example for `"never"`

```html
<div></div>
<div class="very-very-very-very-very-very-very-very-very-long-class-name"></div>
```
