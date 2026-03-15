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

[
  "{#"
  "{:"
  "{@"
] @tag.delimiter

; Closing } of block headers and branch headers
(if_block "}" @tag.delimiter)
(else_if_clause "}" @tag.delimiter)
(else_clause "}" @tag.delimiter)
(each_block "}" @tag.delimiter)
(await_block "}" @tag.delimiter)
(await_branch "}" @tag.delimiter)
(orphan_branch "}" @tag.delimiter)
(key_block "}" @tag.delimiter)
(snippet_block "}" @tag.delimiter)
(const_tag "}" @tag.delimiter)
(render_tag "}" @tag.delimiter)
(html_tag "}" @tag.delimiter)
(debug_tag "}" @tag.delimiter)
(attach_tag "}" @tag.delimiter)

(block_end) @tag.delimiter
(block_end "}" @tag.delimiter)

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

; Comments inside tag attribute lists
(tag_comment kind: (line_comment) @comment)
(tag_comment kind: (block_comment) @comment)
