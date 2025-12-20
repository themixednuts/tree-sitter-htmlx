; Injection queries for HTML
; Injects JavaScript into script elements and CSS into style elements
;
; Note: With the unified element structure, we match on tag_name content
; to identify script/style elements.

; Script elements - inject JavaScript
((element
  (start_tag
    (tag_name) @_tag)
  (raw_text) @injection.content)
 (#match? @_tag "^[Ss][Cc][Rr][Ii][Pp][Tt]$")
 (#set! injection.language "javascript"))

; Style elements - inject CSS
((element
  (start_tag
    (tag_name) @_tag)
  (raw_text) @injection.content)
 (#match? @_tag "^[Ss][Tt][Yy][Ll][Ee]$")
 (#set! injection.language "css"))

; Script elements with type="module" - still JavaScript
((element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_type
      (quoted_attribute_value
        (attribute_value) @_module)))
  (raw_text) @injection.content)
 (#match? @_tag "^[Ss][Cc][Rr][Ii][Pp][Tt]$")
 (#eq? @_type "type")
 (#eq? @_module "module")
 (#set! injection.language "javascript"))

; Script elements with type="text/javascript" - JavaScript
((element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_type
      (quoted_attribute_value
        (attribute_value) @_js)))
  (raw_text) @injection.content)
 (#match? @_tag "^[Ss][Cc][Rr][Ii][Pp][Tt]$")
 (#eq? @_type "type")
 (#match? @_js "text/javascript")
 (#set! injection.language "javascript"))

; Script elements with lang="ts" or type="text/typescript" - TypeScript
((element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_attr
      (quoted_attribute_value
        (attribute_value) @_value)))
  (raw_text) @injection.content)
 (#match? @_tag "^[Ss][Cc][Rr][Ii][Pp][Tt]$")
 (#any-of? @_attr "lang" "type")
 (#any-of? @_value "ts" "typescript" "text/typescript")
 (#set! injection.language "typescript"))

; Style elements with lang="scss" - SCSS
((element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value
        (attribute_value) @_scss)))
  (raw_text) @injection.content)
 (#match? @_tag "^[Ss][Tt][Yy][Ll][Ee]$")
 (#eq? @_lang "lang")
 (#eq? @_scss "scss")
 (#set! injection.language "scss"))

; Style elements with lang="less" - Less
((element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_lang
      (quoted_attribute_value
        (attribute_value) @_less)))
  (raw_text) @injection.content)
 (#match? @_tag "^[Ss][Tt][Yy][Ll][Ee]$")
 (#eq? @_lang "lang")
 (#eq? @_less "less")
 (#set! injection.language "less"))

; On-event attributes - inject JavaScript
((attribute
  (attribute_name) @_name
  (quoted_attribute_value
    (attribute_value) @injection.content))
 (#match? @_name "^on[a-z]+$")
 (#set! injection.language "javascript"))

; Style attribute - inject CSS
((attribute
  (attribute_name) @_name
  (quoted_attribute_value
    (attribute_value) @injection.content))
 (#eq? @_name "style")
 (#set! injection.language "css"))
