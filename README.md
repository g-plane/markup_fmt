# markup_fmt

markup_fmt is a configurable HTML/Vue/Svelte formatter.

## Notes for Vue and Svelte Users

This formatter provides some options such as `vBindStyle`, `vOnStyle` and more for Vue and
`svelteAttrShorthand` and `svelteDirectiveShorthand` for Svelte.

It's recommended to enable these options in this formatter and disable the corresponding
rules in [eslint-plugin-vue](https://eslint.vuejs.org) and [eslint-plugin-svelte](https://sveltejs.github.io/eslint-plugin-svelte) if you used.
This will make ESLint faster because less rules will be executed.

## Getting Started

### dprint

We've provided [dprint](https://dprint.dev/) integration.

This plugin only formats HTML syntax of your HTML/Vue/Svelte files.
You also need other dprint plugins to format the code in `<script>` and `<style>` tags.
You can use [dprint-plugin-typescript](https://github.com/dprint/dprint-plugin-typescript) to
format TypeScript/JavaScript code and [Malva](https://github.com/g-plane/malva) to format CSS/SCSS/Sass/Less code.

Run the commands below to add plugins:

```bash
dprint config add g-plane/markup_fmt
dprint config add g-plane/malva
dprint config add typescript
```

After adding the dprint plugins, update your `dprint.json` and add configuration:

```jsonc
{
  // ...
  "plugins": [
    // ... other plugins URL
    "https://plugins.dprint.dev/g-plane/markup_fmt-v0.0.0.wasm"
  ],
  "markup": { // <-- the key name here is "markup", not "markup_fmt"
    // config comes here
  }
}
```

You can also read [dprint CLI documentation](https://dprint.dev/cli/) for using dprint to format files.

### Use as a Rust crate

Please read the [documentation](https://docs.rs/markup_fmt).

## Configuration

Please refer to [Configuration](./docs/config.md).

## Credit

Tests come from:

- [Prettier](https://github.com/prettier/prettier/tree/main/tests/format)
- [prettier-plugin-svelte](https://github.com/sveltejs/prettier-plugin-svelte)

## License

MIT License

Copyright (c) 2023-present Pig Fang
