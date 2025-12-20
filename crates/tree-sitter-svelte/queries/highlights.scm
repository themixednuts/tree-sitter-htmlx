; inherits: htmlx

; Blocks
(block_kind) @keyword.control

(block_start
  "{" @punctuation.bracket
  "#" @keyword.control.import
  "}" @punctuation.bracket)

(block_branch
  "{" @punctuation.bracket
  ":" @keyword.control
  "}" @punctuation.bracket)

(block_end
  "{" @punctuation.bracket
  "/" @keyword.control
  "}" @punctuation.bracket)

; Tags
(tag_kind) @keyword.control

(tag
  "{" @punctuation.bracket
  "@" @keyword.control
  "}" @punctuation.bracket)

; Block expressions and patterns
(expression_value) @embedded
(block_start expression: (expression) @embedded)
(block_start binding: (pattern) @variable)
(block_start index: (pattern) @variable)
(block_start key: (expression) @embedded)

; Expression braces
(expression "{" @punctuation.bracket "}" @punctuation.bracket)
