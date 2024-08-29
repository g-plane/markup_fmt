# `vOnStyle`

Control Vue `v-on` directive style.

Possible options:

- `null`: Style of `v-on` directive won't be changed.
- `"short"`: Use short-hand form like `@click`.
- `"long"`: Use long-hand form like `v-on:click`.

Default option is `null`.

## Example for `null`

Input:

```html
<button @click=""></button>
<button v-on:click=""></button>
```

Output:

```html
<button @click=""></button>
<button v-on:click=""></button>
```

## Example for `"short"`

Input:

```html
<button @click=""></button>
<button v-on:click=""></button>
```

Output:

```html
<button @click=""></button>
<button @click=""></button>
```

## Example for `"long"`

Input:

```html
<button @click=""></button>
<button v-on:click=""></button>
```

Output:

```html
<button v-on:click=""></button>
<button v-on:click=""></button>
```
