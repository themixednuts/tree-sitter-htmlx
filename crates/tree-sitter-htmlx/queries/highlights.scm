; HTML syntax

(tag_name) @tag

(erroneous_end_tag_name) @tag.error

(doctype) @constant

(attribute_name) @attribute

(attribute_value) @string

(comment) @comment

(entity) @constant.character.escape

(text) @text

(raw_text) @text.literal

[
  "<"
  ">"
  "</"
  "/>"
  "="
] @punctuation.bracket

; HTMLX syntax

((tag_name) @type (#match? @type "^[A-Z]"))

(tag_name
  namespace: (tag_namespace) @keyword
  ":" @punctuation.delimiter
  name: (tag_local_name) @tag)

(attribute_directive) @keyword
":" @punctuation.delimiter
(attribute_identifier) @property
(attribute_modifier) @attribute
(attribute_modifiers "|" @punctuation.delimiter)

(expression) @embedded

(shorthand_attribute content: (_) @variable)

[
  "{"
  "}"
] @punctuation.bracket

"|" @punctuation.delimiter

(tag_comment kind: (line_comment) @comment)
(tag_comment kind: (block_comment) @comment)
