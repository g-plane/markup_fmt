# `vBindSameNameShortHand`

Control whether Vue attribute should be written in short-hand form or not if attribute name and value are same.
If this option is unset, attribute won't be changed.

> Available since v0.3.0.

Default value is `null`.

## Example for `null`

Input:

```html
<MyComponent :value />
<MyComponent :value="value" />
```

Output:

```html
<MyComponent :value />
<MyComponent :value="value" />
```

## Example for `true`

Input:

```html
<MyComponent :value />
<MyComponent :value="value" />
```

Output:

```html
<MyComponent :value />
<MyComponent :value />
```

## Example for `false`

Input:

```html
<MyComponent :value />
<MyComponent :value="value" />
```

Output:

```html
<MyComponent :value="value" />
<MyComponent :value="value" />
```
