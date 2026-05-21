; HTMLX indentation queries

((element
  (start_tag
    name: (tag_name) @_tag))
  (#not-any-of? @_tag
    "area" "base" "br" "col" "embed" "hr" "img" "input"
    "link" "meta" "source" "track" "wbr")) @indent.begin

(element
  (self_closing_tag
    "/>" @indent.end)) @indent.begin

((start_tag
  name: (tag_name) @_tag)
  (#any-of? @_tag
    "area" "base" "br" "col" "embed" "hr" "img" "input"
    "link" "meta" "source" "track" "wbr")) @indent.begin

(element
  (end_tag
    ">" @indent.end))

(element
  (end_tag) @indent.branch)

[
  ">"
  "/>"
] @indent.branch

(comment) @indent.ignore
