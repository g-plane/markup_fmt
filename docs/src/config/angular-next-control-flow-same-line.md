# `angularNextControlFlowSameLine`

Control whether the next Angular control flow code should be on the same line with previous `}` or not.

Default value is `true`.

## Example for `true`

Input:

```html
@if (cond) {
    <div></div>
}
@else {
    <div></div>
}
```

Output:

```html
@if (cond) {
    <div></div>
} @else {
    <div></div>
}
```

## Example for `false`

Input:

```html
@if (cond) {
    <div></div>
} @else {
    <div></div>
}
```

Output:

```html
@if (cond) {
    <div></div>
}
@else {
    <div></div>
}
```
