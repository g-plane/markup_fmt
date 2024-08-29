# `printWidth`

The line width limitation that markup_fmt should *(but not must)* avoid exceeding. markup_fmt will try its best to keep line width less than this value, but it may exceed for some cases, for example, a very very long single word.

Default option is `80`.

## Example for `80`

```html
<script src="very-very-very-very-long-name.js" async defer></script>
```

## Example for `40`

```html
<script
  src="very-very-very-very-long-name.js"
  async
  defer
></script>
```
