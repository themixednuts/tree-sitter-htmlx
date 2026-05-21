; Svelte locals queries

[
  (document)
  (element)
  (if_block)
  (else_if_clause)
  (else_clause)
  (each_block)
  (await_block)
  (await_pending)
  (await_branch)
  (await_branch_children)
  (key_block)
  (snippet_block)
] @local.scope

(each_block
  binding: (pattern) @local.definition)

(each_block
  index: (pattern) @local.definition)

(await_block
  binding: (pattern) @local.definition)

(await_branch
  (pattern) @local.definition)

(orphan_branch
  (pattern) @local.definition)

(snippet_block
  name: (snippet_name) @local.definition.function)

(snippet_parameters
  parameter: (pattern) @local.definition)

(const_tag
  expression: (expression_value
    content: (_) @local.definition))

((attribute
  name: (attribute_name
    (attribute_directive) @_directive
    (attribute_identifier) @local.definition))
  (#eq? @_directive "let"))

(expression
  content: (_) @local.reference)

(attach_tag
  expression: (expression_value
    content: (_) @local.reference))

(debug_tag
  expression: (expression_value
    content: (_) @local.reference))

(html_tag
  expression: (expression_value
    content: (_) @local.reference))

(render_tag
  expression: (expression_value
    content: (_) @local.reference))

(shorthand_attribute
  content: (_) @local.reference)
