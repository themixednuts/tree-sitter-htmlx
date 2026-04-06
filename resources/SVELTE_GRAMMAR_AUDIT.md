# Svelte Grammar Audit

Status after the CSS parser work:

## Fixed

- `{#each expr, index}` now yields a structured `index` field.
- `{#each expr (key)}` now yields a structured `key` field.
- `{#each expr, index (key)}` now yields both `index` and `key` without compiler-side raw-header fallback parsing.

## Cleanups applied outside the grammar

- `Document::script_attributes()` was added in `E:\Projects\svelte\crates\syntax\src\cst.rs` so script start-tag attributes are available through the CST wrapper.
- `Document::module_script_attributes()` and `Document::style_attributes()` were added alongside it for symmetry.

## Remaining gap worth addressing next

- `snippet_opening_header_source_error()` in `E:\Projects\svelte\crates\compiler\src\api\validation\snippet.rs` still performs source-text inspection for malformed `{#snippet ...}` headers.
- This is still needed because malformed snippet headers do not yet consistently surface a fully-localized CST error shape that distinguishes:
  - missing `)` before `}`
  - stray tokens between `)` and `}`
  - header-local error placement inside the snippet opener node

## Audit conclusion

- The main Svelte grammar gap that still blocks full removal of manual header parsing is snippet header recovery.
- Script/style attribute exposure is now a wrapper concern rather than a grammar gap.
