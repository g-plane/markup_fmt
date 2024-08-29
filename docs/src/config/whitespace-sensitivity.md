# `whitespaceSensitivity`

Control the whitespace sensitivity before and after the children of an element.
This is similar to Prettier, so you can read [its blog](https://prettier.io/blog/2018/11/07/1.15.0#whitespace-sensitive-formatting) for detail.

Possible options:

- `"css"`: Respect the default value of CSS [`display`](https://developer.mozilla.org/en-US/docs/Web/CSS/display) property.
- `"strict"`: Whitespace (or the lack of it) around all tags is considered significant.
- `"ignore"`: Whitespace (or the lack of it) around all tags is considered insignificant.

Default option is `"css"`.

This option can be overridden for Vue, Svelte, Astro and Angular components by the following option:

- `component.whitespaceSensitivity`

## Example for `"css"`

Input:

```html
<div> text </div>
<span> text </span>
```

Output:

```html
<div>text</div>
<span> text </span>
```

## Example for `"strict"`

Input:

```html
<div> text </div>
<span> text </span>
```

Output:

```html
<div> text </div>
<span> text </span>
```


## Example for `"ignore"`

Input:

```html
<div> text </div>
<span> text </span>
```

Output:

```html
<div>text</div>
<span>text</span>
```
