# Modern CSS Spec Inputs

Downloaded into this directory for the local grammar audit/implementation:

- `css-syntax-3.html` - https://www.w3.org/TR/css-syntax-3/
- `selectors-4.html` - https://www.w3.org/TR/selectors-4/
- `css-nesting-1.html` - https://www.w3.org/TR/css-nesting-1/
- `css-contain-3.html` - https://www.w3.org/TR/css-contain-3/
- `mediaqueries-5.html` - https://www.w3.org/TR/mediaqueries-5/
- `css-values-4.html` - https://www.w3.org/TR/css-values-4/
- `css-cascade-6.html` - https://www.w3.org/TR/css-cascade-6/

## Parser-facing requirements pulled from these specs

- CSS Syntax 3
  - preserve stylesheet / rule-list / declaration-list structure
  - keep at-rules and qualified rules recoverable when preludes contain modern syntax
  - accept escaped code points and ident-like tokens in selectors and values
- Selectors 4
  - support class, id, attribute, pseudo-class, pseudo-element, namespace, and column combinator (`||`)
  - support selector arguments for `:is()`, `:where()`, `:has()`, `:not()`, `:dir()`, `:lang()`, etc.
  - support attribute selector flags (`i` / `s`)
- CSS Nesting 1
  - support nested style rules
  - support nesting selector `&`
  - allow nested conditional group rules like `@media`, `@supports`, and `@container`
- CSS Containment 3
  - support `@container` with optional container name plus a container condition
  - support size queries, range comparisons, boolean composition, and style queries like `style(--foo: bar)`
- Media Queries 5
  - support boolean operators (`not`, `and`, `or`)
  - support range context queries like `(400px <= width <= 1000px)` and comparisons using `calc()` / `clamp()`
- CSS Values 4
  - support modern identifiers, custom identifiers, functions, math functions, dimensions, percentages, and escape forms
- CSS Cascade 6
  - support `@layer`
  - support `@scope` prelude syntax and nested scoped rule blocks

## Current implementation notes

- `tree-sitter-css-svelte` now uses an external at-rule prelude token to keep modern `@media`, `@container`, `@supports`, `@scope`, and `@layer` blocks structurally intact.
- The grammar now models explicit `container_statement`, `layer_statement`, column combinators, attribute flags, and declaration value spans.
- Remaining work should focus on replacing compiler-side string heuristics with CST-backed reads from these nodes.
