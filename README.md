<div align="center"><img src="./media/markup_fmt.svg" width="160"></div>
<h1 align="center">markup_fmt</h1>

<p align="center">
markup_fmt is a configurable HTML, Vue, Svelte, Astro, Angular, Jinja, Twig, Nunjucks and Vento formatter.
</p>

## Notes for Vue and Svelte Users

This formatter provides some options such as `vBindStyle`, `vOnStyle` and more for Vue and
`svelteAttrShorthand` and `svelteDirectiveShorthand` for Svelte.

It's recommended to enable these options in this formatter and disable the corresponding
rules in [eslint-plugin-vue](https://eslint.vuejs.org) and [eslint-plugin-svelte](https://sveltejs.github.io/eslint-plugin-svelte) if you used.
This will make ESLint faster because less rules will be executed.

## Getting Started

### dprint

We've provided [dprint](https://dprint.dev/) integration.

This plugin only formats HTML syntax of your HTML, Vue, Svelte, Astro, Angular, Jinja, Twig, Nunjucks and Vento files.
You also need other dprint plugins to format the code in `<script>` and `<style>` tags.
You can use [dprint-plugin-typescript](https://github.com/dprint/dprint-plugin-typescript) to
format TypeScript/JavaScript code and [Malva](https://github.com/g-plane/malva) to format CSS/SCSS/Sass/Less code.

Run the commands below to add plugins:

```bash
dprint config add g-plane/markup_fmt
dprint config add g-plane/malva
dprint config add typescript
```

If you also want to format JSON in `<script>` tag whose `"type"` is `"importmap"`, `"application/json"`, or `"application/ld+json"`,
you can add dprint-plugin-json:

```bash
dprint config add json
```

Or Biome:

```diff
- dprint config add typescript
- dprint config add json
+ dprint config add biome
```

After adding the dprint plugins, update your `dprint.json` and add configuration:

```jsonc
{
  // ...
  "plugins": [
    // ... other plugins URL
    "https://plugins.dprint.dev/g-plane/markup_fmt-v0.19.1.wasm"
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

Please refer to [Configuration](https://markup-fmt.netlify.app/).

## Credit

Tests come from:

- [Prettier](https://github.com/prettier/prettier/tree/main/tests/format)
- [prettier-plugin-svelte](https://github.com/sveltejs/prettier-plugin-svelte)
- [prettier-plugin-astro](https://github.com/withastro/prettier-plugin-astro)

## License

MIT License

Copyright (c) 2023-present Pig Fang
