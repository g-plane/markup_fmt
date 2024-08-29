# `doctypeKeywordCase`

Control the case of "doctype" keyword in `<!DOCTYPE>`.

Possible options:

- `"ignore"`: Keep the case as-is.
- `"upper"`: Print "DOCTYPE" in upper case.
- `"lower"`: Print "doctype" in lower case.

Default option is `"upper"`.

## Example for `"ignore"`

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

## Example for `"upper"`

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

## Example for `"lower"`

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
