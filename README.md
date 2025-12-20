# tree-sitter-htmlx

Tree-sitter grammars for HTMLX and extended template languages.

## Prerequisites

Install the tree-sitter CLI:

```sh
cargo install tree-sitter-cli
```

## Build

```sh
cargo build
```

The build automatically runs `tree-sitter generate` and recompiles when you change:
- `grammar.js`
- `src/scanner.c`
- `queries/*.scm`

## Test

```sh
cargo test
```

## Crates

- `tree-sitter-htmlx` - HTML with embedded expressions
- `tree-sitter-svelte` - Svelte 5 components (extends HTMLX)

## License

MIT
