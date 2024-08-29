# `astroAttrShorthand`

Control whether Astro attribute should be written in short-hand form or not when possible.
If this option is unset, attribute won't be changed.

Default value is `null`.

## Example for `null`

Input:

```html
<MyComponent {value} />
<MyComponent value={value} />
```

Output:

```html
<MyComponent {value} />
<MyComponent value={value} />
```

## Example for `true`

Input:

```html
<MyComponent {value} />
<MyComponent value={value} />
```

Output:

```html
<MyComponent {value} />
<MyComponent {value} />
```

## Example for `false`

Input:

```html
<MyComponent {value} />
<MyComponent value={value} />
```

Output:

```html
<MyComponent value={value} />
<MyComponent value={value} />
```
