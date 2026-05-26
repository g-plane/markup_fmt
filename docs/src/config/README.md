# Configuration

Options name in this page are in camel case. If you're using markup_fmt as a Rust crate, please use snake case instead.

If you're using in dprint, please note that the key name for configuration is `"markup"`, not `"markup_fmt"`:

```jsonc
{
  // ...
  "markup": { // <-- the key name here is "markup", not "markup_fmt"
    // config comes here
  }
}
```
