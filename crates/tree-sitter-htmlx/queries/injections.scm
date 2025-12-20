; Script content (TypeScript)
((element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value (attribute_value) @_ts)))
  (raw_text) @injection.content)
  (#eq? @_tag "script")
  (#eq? @_lang "lang")
  (#eq? @_ts "ts")
  (#set! injection.language "typescript"))

; Script content (JavaScript - default)
((element
  (start_tag (tag_name) @_tag)
  (raw_text) @injection.content)
  (#eq? @_tag "script")
  (#set! injection.language "javascript"))

; Style content
((element
  (start_tag (tag_name) @_tag)
  (raw_text) @injection.content)
  (#eq? @_tag "style")
  (#set! injection.language "css"))

; Inline style attribute
((attribute
  (attribute_name) @_name
  (quoted_attribute_value) @injection.content)
  (#eq? @_name "style")
  (#set! injection.language "css")
  (#set! injection.include-children))

; Expressions
((expression content: (js) @injection.content)
  (#set! injection.language "javascript"))

((expression content: (ts) @injection.content)
  (#set! injection.language "typescript"))
