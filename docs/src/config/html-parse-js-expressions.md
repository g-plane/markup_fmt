# `htmlParseJsExpressions`

Control whether JS expressions should be parsed within html content or not.

This is particularly useful when formatting ``` html`` ``` tagged templates within a script.

Default option is `false`.

## Example for `false`

`${}` syntax is ignored, and treated as regular text.

```html
<div>An example of an invalid, unclosed expression ${</div>
```

## Example for `true`

`${}` syntax is parsed as a JS expression, and the contents are externally formatted as a script.

```html
<div>${foo}</div>
```
