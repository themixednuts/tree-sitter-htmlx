; inherits: htmlx

; Style preprocessors
((style_element
  (start_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value (attribute_value) @_scss)))
  (raw_text) @injection.content)
  (#eq? @_lang "lang")
  (#eq? @_scss "scss")
  (#set! injection.language "scss"))

((style_element
  (start_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value (attribute_value) @_sass)))
  (raw_text) @injection.content)
  (#eq? @_lang "lang")
  (#eq? @_sass "sass")
  (#set! injection.language "sass"))

((style_element
  (start_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value (attribute_value) @_less)))
  (raw_text) @injection.content)
  (#eq? @_lang "lang")
  (#eq? @_less "less")
  (#set! injection.language "less"))

; Block and tag expressions
((expression_value) @injection.content
  (#set! injection.language "javascript"))

((block_start expression: (expression) @injection.content)
  (#set! injection.language "javascript"))

((block_start key: (expression) @injection.content)
  (#set! injection.language "javascript"))

((block_start binding: (pattern) @injection.content)
  (#set! injection.language "javascript"))

((block_start index: (pattern) @injection.content)
  (#set! injection.language "javascript"))
