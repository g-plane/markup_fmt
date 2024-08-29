# `vBindStyle`

Control Vue `v-bind` directive style.

Possible options:

- `null`: Style of `v-bind` directive won't be changed.
- `"short"`: Use short-hand form like `:value`.
- `"long"`: Use long-hand form like `v-bind:value`.

Default option is `null`.

## Example for `null`

Input:

```html
<input :value="">
<input v-bind:value="">
```

Output:

```html
<input :value="">
<input v-bind:value="">
```

## Example for `"short"`

Input:

```html
<input :value="">
<input v-bind:value="">
```

Output:

```html
<input :value="">
<input :value="">
```

## Example for `"long"`

Input:

```html
<input :value="">
<input v-bind:value="">
```

Output:

```html
<input v-bind:value="">
<input v-bind:value="">
```
