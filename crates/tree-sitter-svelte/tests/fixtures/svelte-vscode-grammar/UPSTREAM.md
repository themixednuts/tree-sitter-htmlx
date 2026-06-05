# Svelte VS Code Grammar Fixtures

These fixtures are copied from the official Svelte language-tools repository:

- Repository: `https://github.com/sveltejs/language-tools`
- Path: `packages/svelte-vscode/test/grammar/samples`
- Commit: `79799c80e2fe9710b23b60394a63cbf143ee0964`

Keep the copied `samples` directory unchanged so it can be refreshed from
upstream and used as a stable parser regression corpus.

To refresh the fixtures:

```sh
crates/tree-sitter-svelte/scripts/sync-svelte-vscode-grammar-fixtures.sh
```
