# Grammar Architecture Plan (HTML -> HTMLX -> Svelte)

This document defines the target CST architecture for a clean, high-performance
pipeline:

1. `tree-sitter-html` handles HTML structure
2. `tree-sitter-htmlx` adds expression-aware HTML attributes/content
3. `tree-sitter-svelte` adds block/tag syntax on top of HTMLX

It also covers Svelte's "comments in tags" behavior from
https://github.com/sveltejs/svelte/pull/17671.

Status: normative specification for this repository.

Any grammar/scanner change MUST either:

1. satisfy this document as written, or
2. update this document in the same PR.

## Goals

- Keep each grammar layer small and composable.
- Prefer typed CST nodes and fields over string reparsing in AST conversion.
- Make malformed input recoverable with stable, local error nodes.
- Preserve parser throughput: linear scanning, low backtracking, bounded lookahead.

## Non-Goals

- Parsing full JavaScript semantics inside this grammar stack.
- Replacing a JS parser for expression AST (that can stay delegated).

## Normative Lexical Grammar

The following token contracts are exact for parser-facing behavior.

### Shared lexical classes

```ebnf
alpha := "A".."Z" | "a".."z"
digit := "0".."9"
alnum := alpha | digit
ws := " " | "\t" | "\n" | "\r"

name_char := alnum | "-" | "_"
ident_start := alpha | "_" | "$"
ident_char := alnum | "_" | "$"
```

### HTMLX token contracts

```ebnf
tag_namespace := alpha name_char*
tag_local_name := alpha name_char*

member_tag_object := alpha ident_char*
member_tag_property := alpha ident_char*

attribute_value_segment := (any_char - one_of("<", ">", "{", "}", "\"", "'", "/", "=", ws))+

pipe_attribute_name := "|" (any_char - one_of(alpha, "_", "$", "<", ">", "{", "}", "\"", "'", ":", "\\", "/", "=", ws, "|", "."))
                      (any_char - one_of("<", ">", "{", "}", "\"", "'", ":", "\\", "/", "=", ws, "|", "."))*
```

### Svelte token contracts

```ebnf
block_kind := (alpha | "_") (alnum | "_")*
tag_kind := (alpha | "_") (alnum | "_")*
```

Balanced expression scanners MUST:

- track nested `()[]{}`,
- skip over string/template literals,
- trim trailing top-level whitespace from captured token extent,
- stop at `<` in depth 0 recovery context,
- fail if no valid terminator is reached.

## Formal Grammar View (EBNF-ish)

This section is the authoritative syntax model used for implementation.

Notation:

- `A := B C` means sequence.
- `A := B | C` means choice.
- `A*` zero or more.
- `A+` one or more.
- `A?` optional.
- terminals are in quotes.

### Layer 1 (HTML core)

```ebnf
document := node*

node := element
      | text
      | entity
      | comment
      | doctype

element := start_tag node* end_tag
         | self_closing_tag
         | raw_text_element

start_tag := "<" tag_name attribute* ">"
end_tag := "</" tag_name ">"
self_closing_tag := "<" tag_name attribute* "/>"

raw_text_element := raw_text_start_tag raw_text? end_tag
raw_text_start_tag := "<" raw_text_tag_name attribute* ">"

attribute := attribute_name ("=" attribute_value)?
attribute_value := quoted_attribute_value | unquoted_attribute_value
```

### Layer 2 (HTMLX extensions)

```ebnf
document := node*

node := html_node | expression

html_node := element
          | text
          | entity
          | comment
          | doctype

expression := "{" expression_content? "}"
expression_content := js | ts

attribute := typed_lang_attribute
          | spread_attribute
          | shorthand_attribute
          | directive_attribute
          | plain_attribute

typed_lang_attribute := ts_lang_marker attribute_name "=" quoted_attribute_value

plain_attribute := attribute_name ("=" (quoted_attribute_value
                                      | unquoted_mixed_value
                                      | expression
                                      | attribute_value))?

spread_attribute := "{" "..." expression_content "}"
shorthand_attribute := "{" expression_content? "}"

directive_attribute := attribute_directive ":" attribute_identifier attribute_modifiers?
attribute_modifiers := ("|" attribute_modifier)+

unquoted_mixed_value := attribute_value (expression attribute_value?)+

tag_name := normal_tag_name | namespaced_tag_name | member_tag_name
namespaced_tag_name := tag_namespace ":" tag_local_name
member_tag_name := tag_member ("." tag_member)+
```

### Layer 3 (Svelte extensions)

The required end-state CST is typed block nodes.

```ebnf
document := node*

node := htmlx_node
      | block
      | svelte_tag

block := if_block | each_block | await_block | key_block

if_block := if_start block_body else_if_branch* else_branch? if_end
if_start := "{#if" expression "}"
else_if_branch := "{:else if" expression "}" block_body
else_branch := "{:else}" block_body
if_end := "{/if}"

each_block := each_start block_body else_branch? each_end
each_start := "{#each" expression "as" pattern ("," pattern)? ("(" expression ")")? "}"
each_end := "{/each}"

await_block := await_start pending_body? then_branch? catch_branch? await_end
await_start := "{#await" expression ("then" pattern?)? ("catch" pattern?)? "}"
then_branch := "{:then" pattern? "}" block_body
catch_branch := "{:catch" pattern? "}" block_body
await_end := "{/await}"

key_block := key_start block_body key_end
key_start := "{#key" expression "}"
key_end := "{/key}"

svelte_tag := "{@" tag_kind expression? "}"

block_body := node*
```

### Comments in tags (PR #17671)

```ebnf
tag_attribute_item := attribute | tag_comment

tag_comment := line_tag_comment | block_tag_comment
line_tag_comment := "//" line_comment_text
block_tag_comment := "/*" block_comment_text "*/"

start_tag := "<" tag_name tag_attribute_item* ">"
self_closing_tag := "<" tag_name tag_attribute_item* "/>"
```

Semantic rule: `tag_comment` is not an attribute; it is trivia/comment metadata.

## Layer 1: HTML (base state machine)

Source of truth: `tree-sitter-html` behavior.

### State model

- `Data`: text/entity/comment/doctypes/elements.
- `TagOpen`: `<` consumed, decide start/end/erroneous.
- `StartTag`: tag name + attributes + close.
- `EndTag`: tag name + close.
- `RawText`: script/style/textarea/title content until matching end tag.

### CST contract we rely on

- `document`, `element`, `start_tag`, `end_tag`, `self_closing_tag`,
  `raw_text`, `text`, `comment`, `doctype`, `erroneous_end_tag`.
- Stable byte offsets for every node.

## Layer 2: HTMLX (expression-aware HTML)

HTMLX extends HTML by adding expression islands and directive-like attributes.

### Target states

- Inherit all HTML states.
- Add `BeforeAttribute` and `AttributeValueSegment` handling for mixed
  unquoted/quoted/expression values.
- Add `ExpressionIsland` for `{...}` payload scanning (balanced delimiters,
  strings/templates).

### Target CST nodes/fields

- `expression` with `content: js|ts` child (already present).
- `attribute` variants kept structured:
  - normal attribute (`attribute_name` + value payload)
  - `spread_attribute`
  - `shorthand_attribute`
  - directive shape (`attribute_directive`, `attribute_identifier`, modifiers)
- Keep namespaced and dotted tag names as structured children.

### Required cleanup for cleaner CST->AST

1. Replace regex-like leaf capture where structure matters:
   - make `spread_attribute` fully structural (not opaque regex capture).
2. Preserve explicit boundaries for mixed unquoted values (`text{expr}text`) so
   AST mapping does not rescan source slices.
3. Emit dedicated recovery nodes for truncated/unclosed tag attribute values.

## Layer 3: Svelte (blocks/tags + HTMLX)

Svelte extends HTMLX with block/tag syntax.

### Target states

- Inherit HTMLX states.
- Add `SvelteBlockHeader` and `SvelteBranchHeader` states:
  - `{#if ...}` / `{#each ...}` / `{#await ...}` / `{#key ...}`
  - `{:else}` / `{:else if ...}` / `{:then ...}` / `{:catch ...}`
  - `{/if}` etc.
- Add `SvelteTagHeader` state for `{@html ...}` / `{@render ...}` / etc.

### Required CST shape (normative)

Move from generic `block + block_start/block_branch/block_end` to typed nodes:

- `if_block` with fields: `condition`, `consequent`, optional `alternate`.
- `each_block` with fields: `iterable`, `context`, optional `index`, optional
  `key`, `body`, optional `fallback`.
- `await_block` with fields: `expression`, optional `pending`, optional `then`,
  optional `catch`.
- `key_block` with fields: `expression`, `body`.
- `svelte_tag` with fields: `kind`, optional `expression`.

Transitional generic `block_start/block_branch/block_end` is allowed only while
migrating. During transition, every reparsed substring MUST also be exposed via
typed fields so AST conversion can stop source slicing incrementally.

### Required cleanup for cleaner CST->AST

1. Avoid source-string parsing for block headers by exposing fielded children
   for each header part.
2. Emit bounded recovery nodes for unclosed blocks and malformed branches,
   instead of broad `ERROR` regions swallowing unrelated content.
3. Preserve implied HTML end-tag boundaries (especially list-item-like cases)
   via local recovery markers.

## Comments In Tags (PR #17671)

Behavior to support:

```svelte
<MyComponent
  // this comment
  class="myclass"
/>
```

and inline block comments between attributes:

```svelte
<span /* inline */ data-one="1" />
```

### CST design

Add a dedicated node in tag attribute lists:

- `tag_comment`
  - fields:
    - `kind`: `line` | `block`
    - `value`: comment body without delimiters

Allow `tag_comment` anywhere attributes are allowed in:

- `start_tag`
- `self_closing_tag`
- namespaced/member tag variants

Do not coerce `tag_comment` into an attribute node.

### Scanner behavior

- In tag attribute context, recognize `//...` and `/*...*/` as comment tokens.
- Keep comment parsing disabled in normal text/raw-text states.
- For line comments, terminate at newline (`\n` or `\r\n`) or EOF.
- For block comments, terminate at first `*/` with linear scanning.
- Line comments in tag-attribute context MUST NOT consume the `>` of a closing
  tag unless EOF is reached before newline.

### AST mapping contract

- `tag_comment` nodes do not become element attributes.
- They are collected into root/program comments metadata (legacy `_comments`,
  modern `comments`) with offsets preserved.

## Performance Constraints

- No unbounded nested rescans over the same region.
- Scanner transitions must be single-pass over bytes.
- Keep expression scanners balanced-delimiter based with early exits.
- Prefer fixed token classes and small conflict sets.

## Implementation Phases

1. Introduce `tag_comment` tokens/nodes in HTMLX, inherit in Svelte.
2. Add typed field children for block header parts that are currently reparsed.
3. Add narrow recovery nodes for malformed/unclosed block/tag forms.
4. Migrate generic Svelte blocks to typed block nodes.
5. Remove equivalent source-string recovery in AST conversion as each CST
   capability lands.

## Phase Gates (Exact Exit Criteria)

Current implementation status:

- Gate A: in progress (grammar/scanner/tests implemented; AST-integration wiring in downstream compiler is pending)
- Gate B: in progress (block start/branch grammar is now kind-specialized; downstream CST->AST simplification pending)
- Gate C: not started
- Gate D: not started

### Gate A: In-tag comments

- Grammar emits `tag_comment` nodes in `start_tag` and `self_closing_tag`
  attribute lists.
- `tag_comment` does not appear in attribute arrays in AST output.
- Modern AST `comments` includes these comments with exact offsets.
- Legacy AST `_comments` includes these comments with exact offsets.

### Gate B: Block header structure

- `if/each/await/key` headers expose structured fields so no string split
  helpers are needed for header semantics.
- Parser recovery fixtures for malformed block headers remain green.

### Gate C: Recovery shape

- Unclosed block/tag scenarios produce local recovery nodes and preserve
  surrounding sibling structure.
- No broad top-level `ERROR` region may swallow unrelated later siblings.

### Gate D: De-manualization

- AST conversion does not require ad-hoc reparsing for:
  - block header internals,
  - implicit list item closure repair,
  - textarea mustache splitting,
  - in-tag comment extraction.

## No-Drift Rule

- "Equivalent but different" grammar changes are not accepted by default.
- If syntax behavior changes, this file must include the new exact productions,
  token contracts, and updated phase-gate expectations.

## Done Criteria

- AST conversion no longer needs manual reparsing of:
  - each/await/if/key header internals,
  - textarea child mustache boundaries,
  - ad-hoc implicit list-item recovery,
  - in-tag comments.
- Error fixtures remain stable or improve.
- Parser throughput regression remains within noise threshold.
