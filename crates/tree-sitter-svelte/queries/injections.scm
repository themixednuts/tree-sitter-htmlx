; Style preprocessors
((element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value (attribute_value) @_scss)))
  (raw_text) @injection.content)
  (#eq? @_tag "style")
  (#eq? @_lang "lang")
  (#eq? @_scss "scss")
  (#set! injection.language "scss"))

((element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value (attribute_value) @_sass)))
  (raw_text) @injection.content)
  (#eq? @_tag "style")
  (#eq? @_lang "lang")
  (#eq? @_sass "sass")
  (#set! injection.language "sass"))

((element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value (attribute_value) @_less)))
  (raw_text) @injection.content)
  (#eq? @_tag "style")
  (#eq? @_lang "lang")
  (#eq? @_less "less")
  (#set! injection.language "less"))

; Block and tag expressions
((expression_value) @injection.content
  (#set! injection.language "javascript"))

; If block expressions
((if_block expression: (expression) @injection.content)
  (#set! injection.language "javascript"))

; Each block expressions and bindings
((each_block expression: (expression) @injection.content)
  (#set! injection.language "javascript"))
((each_block binding: (pattern) @injection.content)
  (#set! injection.language "javascript"))
((each_block index: (pattern) @injection.content)
  (#set! injection.language "javascript"))
((each_block key: (expression) @injection.content)
  (#set! injection.language "javascript"))

; Await block expressions and bindings
((await_block expression: (expression) @injection.content)
  (#set! injection.language "javascript"))
((await_block binding: (pattern) @injection.content)
  (#set! injection.language "javascript"))
((_await_branch_header binding: (pattern) @injection.content)
  (#set! injection.language "javascript"))

; Key block expressions
((key_block expression: (expression) @injection.content)
  (#set! injection.language "javascript"))

; Snippet parameters
((snippet_parameters parameter: (pattern) @injection.content)
  (#set! injection.language "javascript"))
