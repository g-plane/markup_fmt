## `preferSingleLineOpeningTag`

Control whether opening tags that don't fit within the `printWidth` limit
should be kept on a single line. This option only applies to tags with 0 or 1 attributes.

Default option is `false`.

### Example for `false`

```html

<div
  class="very-very-very-very-very-very-very-very-very-long-class-name"
>
  text
</div>
```

### Example for `true`

```html

<div class="very-very-very-very-very-very-very-very-very-long-class-name">
  text
</div>
```

