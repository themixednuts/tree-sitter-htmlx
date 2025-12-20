; Auto-vendored during build. Do not edit manually.

; Script content (TypeScript)
((script_element
  (start_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value (attribute_value) @_ts)))
  (raw_text) @injection.content)
  (#eq? @_lang "lang")
  (#eq? @_ts "ts")
  (#set! injection.language "typescript"))

; Script content (JavaScript - default)
((script_element (raw_text) @injection.content)
  (#set! injection.language "javascript"))

; Style content
((style_element (raw_text) @injection.content)
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
