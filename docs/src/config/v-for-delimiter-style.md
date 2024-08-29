# `vForDelimiterStyle`

Control Vue `v-for` directive delimiter style.

Possible options:

- `null`: Style of `v-for` directive delimiter won't be changed.
- `"in"`: Use `in` as `v-for` delimiter.
- `"of"`: Use `of` as `v-for` delimiter.

Default value is `null`.

## Example for `null`

Input:

```html
<li v-for="item in list"></li>
<li v-for="item of list"></li>
```

Output:

```html
<li v-for="item in list"></li>
<li v-for="item of list"></li>
```

## Example for `"in"`

Input:

```html
<li v-for="item in list"></li>
<li v-for="item of list"></li>
```

Output:

```html
<li v-for="item in list"></li>
<li v-for="item in list"></li>
```

## Example for `"of"`

Input:

```html
<li v-for="item in list"></li>
<li v-for="item of list"></li>
```

Output:

```html
<li v-for="item of list"></li>
<li v-for="item of list"></li>
```
