; Script content
((script_element (raw_text) @injection.content)
  (#set! injection.language "javascript"))

((script_element
  (start_tag (attribute (ts_lang_attr)))
  (raw_text) @injection.content)
  (#set! injection.language "typescript"))

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
