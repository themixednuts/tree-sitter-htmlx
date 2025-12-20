; Auto-vendored during build. Do not edit manually.

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
[
  (expression)
  (spread_attribute)
] @embedded

(shorthand_attribute) @variable

; Punctuation
[
  "{"
  "}"
] @punctuation.bracket

"|" @punctuation.delimiter
