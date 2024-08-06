# Configuration

Options name in this page are in camel case. If you're using markup_fmt as a Rust crate, please use snake case instead.

## `printWidth`

The line width limitation that markup_fmt should *(but not must)* avoid exceeding. markup_fmt will try its best to keep line width less than this value, but it may exceed for some cases, for example, a very very long single word.

Default option is `80`.

### Example for `80`

```html
<script src="very-very-very-very-long-name.js" async defer></script>
```

### Example for `40`

```html
<script
  src="very-very-very-very-long-name.js"
  async
  defer
></script>
```

## `useTabs`

Specify use space or tab for indentation.

Default option is `false`.

## `indentWidth`

Size of indentation. When enabled `useTabs`, this option may be disregarded,
since only one tab will be inserted when indented once.

Default option is `2`. This can't be zero.

### Example for `2`

```html
<script
  src="very-very-very-very-long-name.js"
  async
  defer
></script>
```

### Example for `4`

```html
<script
    src="very-very-very-very-long-name.js"
    async
    defer
></script>
```

## `lineBreak`

Specify whether use `\n` (LF) or `\r\n` (CRLF) for line break.

Default option is `"lf"`. Possible options are `"lf"` and `"crlf"`.

## `quotes`

Control the quotes of attribute value.

Possible options:

- `"double"`: Use double quotes as possible. However if there're double quotes in strings, quotes will be kept as-is.
- `"single"`: Use single quotes as possible. However if there're single quotes in strings, quotes will be kept as-is.

Default option is `"double"`.

### Example for `"double"`

```html
<div class=""></div>
```

### Example for `"single"`

```html
<div class=''></div>
```

## `formatComments`

Control whether whitespace should be inserted at the beginning and end of comments and
comments should be indented properly or not.

When this option is set to `false`, comments contain leading or trailing whitespace will still be kept as-is.

Default option is `false`.

### Example for `false`

```html
<!--comments-->
<!--very very very very very very very very very very very very very very very very long-->
```

### Example for `true`

```html
<!-- comments -->
<!--
  very very very very very very very very very very very very very very very very long
-->
```

## `scriptIndent`

Control whether the code block in the `<script>` tag should be indented or not.

Default option is `false`.

This global option can be overridden for individual languages by the following options:

- `html.scriptIndent`
- `vue.scriptIndent`
- `svelte.scriptIndent`
- `astro.scriptIndent`

### Example for `false`

```html
<script>
const a = 0
</script>
```

### Example for `true`

```html
<script>
  const a = 0
</script>
```

## `styleIndent`

Control whether the code block in the `<style>` tag should be indented or not.

Default option is `false`.

This global option can be overridden for individual languages by the following options:

- `html.styleIndent`
- `vue.styleIndent`
- `svelte.styleIndent`
- `astro.styleIndent`

### Example for `false`

```html
<style>
a { outline: none; }
</style>
```

### Example for `true`

```html
<style>
  a { outline: none; }
</style>
```

## `closingBracketSameLine`

Control the closing bracket (`>`) of a multi-line element should come at the end of the last line
or on the next line (with a line break).

Default option is `false`.

### Example for `false`

```html
<span
  class=""
  style=""
></span>
```

### Example for `true`

```html
<span
  class=""
  style=""></span>
```

## `closingTagLineBreakForEmpty`

When there're no children in an element,
this option controls whether to insert a line break before the closing tag or not.

Possible options:

- `"always"`: Always insert a line break before the closing tag.
- `"fit"`: Only insert a line break if it doesn't fit the `printWidth` option.
- `"never"`: Don't insert a line break.

Default option is `"fit"`.

### Example for `"always"`

```html
<div>
</div>
<div class="very-very-very-very-very-very-very-very-very-long-class-name">
</div>
```

### Example for `"fit"`

```html
<div></div>
<div class="very-very-very-very-very-very-very-very-very-long-class-name">
</div>
```

### Example for `"never"`

```html
<div></div>
<div class="very-very-very-very-very-very-very-very-very-long-class-name"></div>
```

## `maxAttrsPerLine`

Control the maximum number of attributes in one line.
If this option is unset, there won't be any limitations.

This option conflicts with `preferAttrsSingleLine` option.

Default option is `null`. This option can't be `0`.

### Example for `null`

```html
<div data-a data-b data-c></div>
```

### Example for `1`

```html
<div
  data-a
  data-b
  data-c
></div>
```

### Example for `2`

```html
<div
  data-a data-b
  data-c
></div>
```

## `preferAttrsSingleLine`

Control whether attributes should be put on single line when possible.

This option conflicts with `maxAttrsPerLine` option.

Default option is `false`.

### Example for `false`

This `<div>` is short enough to be put on single line,
but it won't because attributes in source code span multiple lines.

```html
<div
  data-a
  data-b
></div>
```

### Example for `true`

This `<div>` is short enough so it will be put on single line
though attributes span multiple lines in source code.

```html
<div data-a data-b></div>
```

## `*.selfClosing`

This group of options controls whether an element should be self-closed or not if
it doesn't have children.

There're several options:

- `html.normal.selfClosing`: This option affects on HTML normal elements.
- `html.void.selfClosing`: This option affects on HTML void elements.
- `component.selfClosing`: This option affects on Vue/Svelte/Astro/Angular components.
- `svg.selfClosing`: This option affects on SVG elements.
- `mathml.selfClosing`: This option affects on MathML elements.

All of these options can be set as `null`, `true` or `false`:

- `null`: Whether the element is self-closed or not won't be changed. Keep it as-is.
- `true`: Element will always be self-closed.
- `false`: Element will never be self-closed.

All of these options are default to `null`.

### Example for `null`

Input:

```html
<div />
<div></div>
```

Output:

```html
<div />
<div></div>
```

### Example for `true`

Input:

```html
<div />
<div></div>
```

Output:

```html
<div />
<div />
```

### Example for `false`

Input:

```html
<div />
<div></div>
```

Output:

```html
<div></div>
<div></div>
```

## `whitespaceSensitivity`

Control the whitespace sensitivity before and after the children of an element.
This is similar to Prettier, so you can read [its blog](https://prettier.io/blog/2018/11/07/1.15.0#whitespace-sensitive-formatting) for detail.

Possible options:

- `"css"`: Respect the default value of CSS `display` property.
- `"strict"`: Whitespace (or the lack of it) around all tags is considered significant.
- `"ignore"`: Whitespace (or the lack of it) around all tags is considered insignificant.

Default option is `"css"`.

This option can be overridden for Vue/Svelte/Astro/Angular components by the following option:

- `component.whitespaceSensitivity`

### Example for `"css"`

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

### Example for `"strict"`

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


### Example for `"ignore"`

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

## `doctypeKeywordCase`

Control the case of "doctype" keyword in `<!DOCTYPE>`.

Possible options:

- `"ignore"`: Keep the case as-is.
- `"upper"`: Print "DOCTYPE" in upper case.
- `"lower"`: Print "doctype" in lower case.

Default option is `"upper"`.

### Example for `"ignore"`

Input:

```html
<!DOCTYPE html>
<!doctype html>
```

Output:

```html
<!DOCTYPE html>
<!doctype html>
```

### Example for `"upper"`

Input:

```html
<!DOCTYPE html>
<!doctype html>
```

Output:

```html
<!DOCTYPE html>
<!DOCTYPE html>
```

### Example for `"lower"`

Input:

```html
<!DOCTYPE html>
<!doctype html>
```

Output:

```html
<!doctype html>
<!doctype html>
```

## `vBindStyle`

Control Vue `v-bind` directive style.

Possible options:

- `null`: Style of `v-bind` directive won't be changed.
- `"short"`: Use short-hand form like `:value`.
- `"long"`: Use long-hand form like `v-bind:value`.

Default option is `null`.

### Example for `null`

Input:

```vue
<input :value="">
<input v-bind:value="">
```

Output:

```vue
<input :value="">
<input v-bind:value="">
```

### Example for `"short"`

Input:

```vue
<input :value="">
<input v-bind:value="">
```

Output:

```vue
<input :value="">
<input :value="">
```

### Example for `"long"`

Input:

```vue
<input :value="">
<input v-bind:value="">
```

Output:

```vue
<input v-bind:value="">
<input v-bind:value="">
```

## `vOnStyle`

Control Vue `v-on` directive style.

Possible options:

- `null`: Style of `v-on` directive won't be changed.
- `"short"`: Use short-hand form like `@click`.
- `"long"`: Use long-hand form like `v-on:click`.

Default option is `null`.

### Example for `null`

Input:

```vue
<button @click=""></button>
<button v-on:click=""></button>
```

Output:

```vue
<button @click=""></button>
<button v-on:click=""></button>
```

### Example for `"short"`

Input:

```vue
<button @click=""></button>
<button v-on:click=""></button>
```

Output:

```vue
<button @click=""></button>
<button @click=""></button>
```

### Example for `"long"`

Input:

```vue
<button @click=""></button>
<button v-on:click=""></button>
```

Output:

```vue
<button v-on:click=""></button>
<button v-on:click=""></button>
```

## `vForDelimiterStyle`

Control Vue `v-for` directive delimiter style.

Possible options:

- `null`: Style of `v-for` directive delimiter won't be changed.
- `"in"`: Use `in` as `v-for` delimiter.
- `"of"`: Use `of` as `v-for` delimiter.

Default value is `null`.

### Example for `null`

Input:

```vue
<li v-for="item in list"></li>
<li v-for="item of list"></li>
```

Output:

```vue
<li v-for="item in list"></li>
<li v-for="item of list"></li>
```

### Example for `"in"`

Input:

```vue
<li v-for="item in list"></li>
<li v-for="item of list"></li>
```

Output:

```vue
<li v-for="item in list"></li>
<li v-for="item in list"></li>
```

### Example for `"of"`

Input:

```vue
<li v-for="item in list"></li>
<li v-for="item of list"></li>
```

Output:

```vue
<li v-for="item of list"></li>
<li v-for="item of list"></li>
```

## `vSlotStyle`

Control Vue `v-slot` directive style.

Possible options:

- `null`: Style of `v-slot` directive won't be changed.
- `"short"`: Use short-hand form like `#default` or `#named`.
- `"long"`: Use long-hand form like `v-slot:default` or `v-slot:named`.
- `"vSlot"`: For default slot, use `v-slot` (shorter than `#default`); otherwise, use short-hand form.

Default option is `null`.

This global option can be overridden by the following options:

- `component.vSlotStyle`: Control `v-slot` style at components.
- `default.vSlotStyle`: Control `v-slot` style of default slot at `<template>` tag.
- `named.vSlotStyle`: Control `v-slot` style of named slot at `<template>` tag.

### Example for `null`

Input:

```vue
<template v-slot:default></template>
<template v-slot:header></template>
<template #default></template>
<template #header></template>
```

Output:

```vue
<template v-slot:default></template>
<template v-slot:header></template>
<template #default></template>
<template #header></template>
```

### Example for `"short"`

Input:

```vue
<template v-slot:default></template>
<template v-slot:header></template>
<template #default></template>
<template #header></template>
```

Output:

```vue
<template #default></template>
<template #header></template>
<template #default></template>
<template #header></template>
```

### Example for `"long"`

Input:

```vue
<template v-slot:default></template>
<template v-slot:header></template>
<template #default></template>
<template #header></template>
```

Output:

```vue
<template v-slot:default></template>
<template v-slot:header></template>
<template v-slot:default></template>
<template v-slot:header></template>
```

### Example for `"vSlot"`

Input:

```vue
<template v-slot:default></template>
<template v-slot:header></template>
<template #default></template>
<template #header></template>
```

Output:

```vue
<template v-slot></template>
<template #header></template>
<template v-slot></template>
<template #header></template>
```

## `vBindSameNameShortHand`

Control whether Vue attribute should be written in short-hand form or not if attribute name and value are same.
If this option is unset, attribute won't be changed.

> Available since v0.3.0.

Default value is `null`.

### Example for `null`

Input:

```vue
<MyComponent :value />
<MyComponent :value="value" />
```

Output:

```vue
<MyComponent :value />
<MyComponent :value="value" />
```

### Example for `true`

Input:

```vue
<MyComponent :value />
<MyComponent :value="value" />
```

Output:

```vue
<MyComponent :value />
<MyComponent :value />
```

### Example for `false`

Input:

```vue
<MyComponent :value />
<MyComponent :value="value" />
```

Output:

```vue
<MyComponent :value="value" />
<MyComponent :value="value" />
```

## `strictSvelteAttr`

Control whether Svelte attribute value should be in strict mode or not.

Default option is `false`.

### Example for `false`

```svelte
<div class={class}></div>
```

### Example for `true`

```svelte
<div class="{class}"></div>
```

## `svelteAttrShorthand`

Control whether Svelte attribute should be written in short-hand form or not when possible.
If this option is unset, attribute won't be changed.

Default value is `null`.

### Example for `null`

Input:

```svelte
<MyComponent {value} />
<MyComponent value={value} />
```

Output:

```svelte
<MyComponent {value} />
<MyComponent value={value} />
```

### Example for `true`

Input:

```svelte
<MyComponent {value} />
<MyComponent value={value} />
```

Output:

```svelte
<MyComponent {value} />
<MyComponent {value} />
```

### Example for `false`

Input:

```svelte
<MyComponent {value} />
<MyComponent value={value} />
```

Output:

```svelte
<MyComponent value={value} />
<MyComponent value={value} />
```

## `svelteDirectiveShorthand`

Control whether Svelte directive should be written in short-hand form or not when possible.
If this option is unset, directive won't be changed.

Default value is `null`.

### Example for `null`

Input:

```svelte
<MyComponent bind:value />
<MyComponent bind:value={value} />
```

Output:

```svelte
<MyComponent bind:value />
<MyComponent bind:value={value} />
```

### Example for `true`

Input:

```svelte
<MyComponent bind:value />
<MyComponent bind:value={value} />
```

Output:

```svelte
<MyComponent bind:value />
<MyComponent bind:value />
```

### Example for `false`

Input:

```svelte
<MyComponent bind:value />
<MyComponent bind:value={value} />
```

Output:

```svelte
<MyComponent bind:value={value} />
<MyComponent bind:value={value} />
```

## `astroAttrShorthand`

Control whether Astro attribute should be written in short-hand form or not when possible.
If this option is unset, attribute won't be changed.

Default value is `null`.

### Example for `null`

Input:

```astro
<MyComponent {value} />
<MyComponent value={value} />
```

Output:

```astro
<MyComponent {value} />
<MyComponent value={value} />
```

### Example for `true`

Input:

```astro
<MyComponent {value} />
<MyComponent value={value} />
```

Output:

```astro
<MyComponent {value} />
<MyComponent {value} />
```

### Example for `false`

Input:

```astro
<MyComponent {value} />
<MyComponent value={value} />
```

Output:

```astro
<MyComponent value={value} />
<MyComponent value={value} />
```

## `ignoreCommentDirective`

Text directive for ignoring formatting specific element or node.

Default is `"markup-fmt-ignore"`.

### Example

```html
<div></div>
<!-- markup-fmt-ignore -->
<div  >  </div>
```
