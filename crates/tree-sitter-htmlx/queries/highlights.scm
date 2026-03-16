; Component tags (PascalCase)
((tag_name) @type (#match? @type "^[A-Z]"))

; Namespaced tags
(tag_name
  namespace: (tag_namespace) @keyword
  ":" @punctuation.delimiter
  name: (tag_local_name) @tag)

; Directives (nested inside attribute_name)
(attribute_directive) @keyword
":" @punctuation.delimiter
(attribute_identifier) @property
(attribute_modifier) @attribute
(attribute_modifiers "|" @punctuation.delimiter)

; Expressions
(expression) @embedded

; Shorthand/spread attribute content — braces captured by generic "{" "}" rule
(shorthand_attribute content: (_) @variable)

; Punctuation
[
  "{"
  "}"
] @punctuation.bracket

"|" @punctuation.delimiter

; Comments inside tag attribute lists
(tag_comment kind: (line_comment) @comment)
(tag_comment kind: (block_comment) @comment)
