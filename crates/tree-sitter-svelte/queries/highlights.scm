; Block keywords
(block_kind) @keyword.control

[
  "#"
  ":"
  "/"
  "@"
] @tag.delimiter

; Tag keywords
(tag_kind) @keyword.control

; Block expressions and patterns
(expression_value) @embedded
(block_start expression: (expression) @embedded)
(block_start key: (expression) @embedded)

[
  (block_start binding: (pattern))
  (block_start index: (pattern))
] @variable
