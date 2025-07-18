# `vueComponentCase`

Control the component naming style in template.

This option only formats component name that has two or more words.

If you're heavily using custom elements, you shouldn't use this option.

Available option values:

- `"ignore"`: Component names will not be changed.
- `"pascalCase"`: Component names will be converted to PascalCase.
- `"kebabCase"`: Component names will be converted to kebab-case.

Default value is `"ignore"`.

## Example for `"ignore"`

Input:

```html
<template>
  <component></component>
  <Component></Component>
  <MyComponent></MyComponent>
  <my-component></my-component>
  <My-Component></My-Component>
  <myComponent></myComponent>
</template>
```

Output:

```html
<template>
  <component></component>
  <Component></Component>
  <MyComponent></MyComponent>
  <my-component></my-component>
  <My-Component></My-Component>
  <myComponent></myComponent>
</template>
```

## Example for `"pascalCase"`

Input:

```html
<template>
  <component></component>
  <Component></Component>
  <MyComponent></MyComponent>
  <my-component></my-component>
  <My-Component></My-Component>
  <myComponent></myComponent>
</template>
```

Output:

```html
<template>
  <component></component>
  <Component></Component>
  <MyComponent></MyComponent>
  <MyComponent></MyComponent>
  <MyComponent></MyComponent>
  <MyComponent></MyComponent>
</template>
```

## Example for `"kebabCase"`

Input:

```html
<template>
  <component></component>
  <Component></Component>
  <MyComponent></MyComponent>
  <my-component></my-component>
  <My-Component></My-Component>
  <myComponent></myComponent>
</template>
```

Output:

```html
<template>
  <component></component>
  <Component></Component>
  <my-component></my-component>
  <my-component></my-component>
  <my-component></my-component>
  <my-component></my-component>
</template>
```
