; Vendored from tree-sitter-html
;
; This file is auto-generated during build. Do not edit manually.

; HTML syntax highlighting queries
; Following the WHATWG HTML Living Standard

; Tag names
(tag_name) @tag

; Erroneous/mismatched end tags
(erroneous_end_tag_name) @tag.error

; DOCTYPE declaration
(doctype) @constant

; Attribute names
(attribute_name) @attribute

; Attribute values
(attribute_value) @string

; Comments
(comment) @comment

; Character entities
(entity) @constant.character.escape

; Text content
(text) @text

; Raw text in script/style/textarea/title
(raw_text) @text.literal

; Punctuation
[
  "<"
  ">"
  "</"
  "/>"
  "="
] @punctuation.bracket

