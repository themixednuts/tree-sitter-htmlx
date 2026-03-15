; Block keywords
[
  "if"
  "each"
  "await"
  "key"
  "snippet"
  "else"
  "html"
  "debug"
  "const"
  "render"
  "attach"
] @keyword.control

; Block delimiters
(block_open) @tag.delimiter
(block_close) @tag.delimiter

; Shorthand kind (then/catch in await shorthand)
(shorthand_kind) @keyword.control
; Branch kind (then/catch in await branches)
(branch_kind) @keyword.control

; Block expressions and patterns
(expression_value) @embedded

; If block
(if_block expression: (expression) @embedded)
(else_if_clause expression: (expression_value) @embedded)

; Each block
(each_block expression: (expression) @embedded)
(each_block binding: (pattern) @variable)
(each_block index: (pattern) @variable)
(each_block key: (expression) @embedded)

; Await block
(await_block expression: (expression) @embedded)
(await_branch (pattern) @variable)
(await_block (pattern) @variable)
(orphan_branch (pattern) @variable)

; Key block
(key_block expression: (expression) @embedded)

; Snippet block
(snippet_block name: (snippet_name) @function)
(snippet_parameters parameter: (pattern) @variable)

; Malformed blocks (e.g. { #if ...} with space before sigil)
(block_sigil) @keyword.control

; Comments inside tag attribute lists
(tag_comment kind: (line_comment) @comment)
(tag_comment kind: (block_comment) @comment)
