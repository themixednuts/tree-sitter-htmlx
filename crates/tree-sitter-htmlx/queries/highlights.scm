; Tags
(tag_name) @tag
((tag_name) @type (#match? @type "^[A-Z]"))

(tag_name
  namespace: (tag_namespace) @keyword
  ":" @punctuation.delimiter
  name: (tag_local_name) @tag)

; Attributes
(attribute_name) @attribute
(attribute_value) @string
(quoted_attribute_value) @string

; Directives
(directive_prefix) @keyword
(directive_value) @property
(directive_modifier) @attribute
(directive_modifiers "|" @punctuation.delimiter)

; Expressions
(expression content: (js) @embedded)
(expression content: (ts) @embedded)
(spread_attribute) @punctuation.special
(shorthand_attribute) @variable

; Other
(erroneous_end_tag_name) @error
(comment) @comment
(text) @none
(doctype) @keyword

; Punctuation
"<" @punctuation.bracket
">" @punctuation.bracket
"</" @punctuation.bracket
"/>" @punctuation.bracket
"=" @punctuation.delimiter
"{" @punctuation.bracket
"}" @punctuation.bracket
