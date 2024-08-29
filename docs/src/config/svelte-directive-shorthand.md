# `svelteDirectiveShorthand`

Control whether Svelte directive should be written in short-hand form or not when possible.
If this option is unset, directive won't be changed.

Default value is `null`.

## Example for `null`

Input:

```html
<MyComponent bind:value />
<MyComponent bind:value={value} />
```

Output:

```html
<MyComponent bind:value />
<MyComponent bind:value={value} />
```

## Example for `true`

Input:

```html
<MyComponent bind:value />
<MyComponent bind:value={value} />
```

Output:

```html
<MyComponent bind:value />
<MyComponent bind:value />
```

## Example for `false`

Input:

```html
<MyComponent bind:value />
<MyComponent bind:value={value} />
```

Output:

```html
<MyComponent bind:value={value} />
<MyComponent bind:value={value} />
```
