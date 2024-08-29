# `vSlotStyle`

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

## Example for `null`

Input:

```html
<template v-slot:default></template>
<template v-slot:header></template>
<template #default></template>
<template #header></template>
```

Output:

```html
<template v-slot:default></template>
<template v-slot:header></template>
<template #default></template>
<template #header></template>
```

## Example for `"short"`

Input:

```html
<template v-slot:default></template>
<template v-slot:header></template>
<template #default></template>
<template #header></template>
```

Output:

```html
<template #default></template>
<template #header></template>
<template #default></template>
<template #header></template>
```

## Example for `"long"`

Input:

```html
<template v-slot:default></template>
<template v-slot:header></template>
<template #default></template>
<template #header></template>
```

Output:

```html
<template v-slot:default></template>
<template v-slot:header></template>
<template v-slot:default></template>
<template v-slot:header></template>
```

## Example for `"vSlot"`

Input:

```html
<template v-slot:default></template>
<template v-slot:header></template>
<template #default></template>
<template #header></template>
```

Output:

```html
<template v-slot></template>
<template #header></template>
<template v-slot></template>
<template #header></template>
```
