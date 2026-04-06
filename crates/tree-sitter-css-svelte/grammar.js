/**
 * CSS grammar for tree-sitter with Svelte-oriented recovery.
 *
 * Based on tree-sitter-css 0.25.0 with looser at-rule prelude parsing so
 * modern media/container queries keep their nested rule blocks intact.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "css",

  extras: ($) => [
    /\s/,
    $.comment,
    $.js_comment,
    $.html_comment_delimiter,
  ],

  externals: ($) => [
    $._descendant_operator,
    $._pseudo_class_selector_colon,
    $.__error_recovery,
    $._at_rule_prelude,
    $._general_enclosed_value,
    $._unicode_range_value,
    $._bad_url_value,
    $._forgiving_pseudo_element_recovery,
  ],

  inline: ($) => [
    $._top_level_item,
    $._block_item,
  ],

  rules: {
    stylesheet: ($) => repeat($._top_level_item),

    _top_level_item: ($) => choice(
      $.declaration,
      $.rule_set,
      $.import_statement,
      $.custom_media_statement,
      $.container_statement,
      $.media_statement,
      $.charset_statement,
      $.namespace_statement,
      $.keyframes_statement,
      $.supports_statement,
      alias($.top_level_scope_statement, $.scope_statement),
      $.layer_statement,
      $.at_rule,
    ),

    import_statement: ($) => seq(
      "@import",
      field("source", choice($.string_value, $.call_expression)),
      optional(field("prelude", $.import_conditions)),
      ";",
    ),

    custom_media_statement: ($) => seq(
      "@custom-media",
      field("name", alias($.custom_property_name, $.custom_media_name)),
      field("value", choice($.media_query_list, $.custom_media_boolean)),
      ";",
    ),

    media_statement: ($) => seq(
      "@media",
      optional(field("query", $.media_query_list)),
      field("block", $.block),
    ),

    container_statement: ($) => seq(
      "@container",
      optional(field("query", $.container_query)),
      field("block", $.block),
    ),

    charset_statement: ($) => seq(
      "@charset",
      $._value,
      ";",
    ),

    namespace_statement: ($) => seq(
      "@namespace",
      optional(alias($.identifier, $.namespace_name)),
      choice($.string_value, $.call_expression),
      ";",
    ),

    keyframes_statement: ($) => seq(
      choice(
        "@keyframes",
        alias(/@[-a-z]+keyframes/, $.at_keyword),
      ),
      field("name", alias($.identifier, $.keyframes_name)),
      field("block", $.keyframe_block_list),
    ),

    keyframe_block_list: ($) => seq(
      "{",
      repeat($.keyframe_block),
      "}",
    ),

    keyframe_block: ($) => seq(
      sep1(",", choice($.from, $.to, $.integer_value, $.float_value)),
      $.block,
    ),

    from: (_) => "from",
    to: (_) => "to",

    supports_statement: ($) => seq(
      "@supports",
      optional(field("query", $.supports_condition)),
      field("block", $.block),
    ),

    top_level_scope_statement: ($) => seq(
      "@scope",
      optional(field("query", $.top_level_scope_query)),
      field("block", $.block),
    ),

    nested_scope_statement: ($) => seq(
      "@scope",
      optional(field("query", $.scope_query)),
      field("block", $.block),
    ),

    top_level_scope_query: ($) => choice(
      seq(field("start", $.top_level_scope_start), optional(field("end", $.top_level_scope_end))),
      field("end", $.top_level_scope_end),
    ),

    scope_query: ($) => choice(
      seq(field("start", $.scope_start), optional(field("end", $.scope_end))),
      field("end", $.scope_end),
    ),

    top_level_scope_start: ($) => seq("(", $._selectors_no_pseudo_element_no_nesting, ")"),

    top_level_scope_end: ($) => seq("to", "(", $._selectors_no_pseudo_element_no_nesting, ")"),

    scope_start: ($) => seq("(", $._selectors_no_pseudo_element, ")"),

    scope_end: ($) => seq("to", "(", $._selectors_no_pseudo_element, ")"),

    layer_statement: ($) => seq(
      choice(
        seq(
          "@layer",
          optional(field("name", $.layer_names)),
          ";",
        ),
        seq(
          "@layer",
          optional(field("name", $.layer_name)),
          field("block", $.block),
        ),
      ),
    ),

    layer_names: ($) => sep1(",", $.layer_name),

    layer_name: ($) => sep1(".", alias($.identifier, $.layer_name_part)),

    import_conditions: ($) => choice(
      seq(
        field("layer", $.import_layer),
        optional(field("supports", $.import_supports)),
        optional(field("media", $.media_query_list)),
      ),
      seq(
        field("supports", $.import_supports),
        optional(field("media", $.media_query_list)),
      ),
      field("media", $.media_query_list),
    ),

    import_layer: ($) => choice(
      "layer",
      seq(
        "layer",
        token.immediate("("),
        field("name", $.layer_name),
        ")",
      ),
    ),

    import_supports: ($) => seq(
      "supports",
      token.immediate("("),
      field("condition", $.supports_function_condition),
      ")",
    ),

    postcss_statement: ($) => prec(-1, seq(
      $.at_keyword,
      repeat(choice($._value, $.important_value)),
      ";",
    )),

    at_rule: ($) => seq(
      $.at_keyword,
      optional(field("prelude", alias($._at_rule_prelude, $.query_prelude))),
      choice(";", field("block", $.block)),
    ),

    rule_set: ($) => seq(
      field("prelude", $.selectors),
      field("block", $.block),
    ),

    selectors: ($) => sep1(",", $._selector),

    block: ($) => seq(
      "{",
      repeat($._block_item),
      optional(alias($.last_declaration, $.declaration)),
      "}",
    ),

    _block_item: ($) => choice(
      $.declaration,
      $.rule_set,
      $.import_statement,
      $.custom_media_statement,
      $.container_statement,
      $.media_statement,
      $.charset_statement,
      $.namespace_statement,
      $.keyframes_statement,
      $.supports_statement,
      alias($.nested_scope_statement, $.scope_statement),
      $.layer_statement,
      $.postcss_statement,
      $.at_rule,
    ),

    _selector: ($) => choice(
      $.universal_selector,
      prec(1, seq(optional($._selector), alias($.identifier, $.tag_name))),
      $.class_selector,
      $.nesting_selector,
      $.pseudo_class_selector,
      $.pseudo_element_selector,
      $.id_selector,
      $.attribute_selector,
      $.string_value,
      $.child_selector,
      $.descendant_selector,
      $.sibling_selector,
      $.adjacent_sibling_selector,
      $.column_selector,
      $.namespace_selector,
    ),

    _selector_no_pseudo_element: ($) => selectorVariant($, $._selector_no_pseudo_element, {
      allowPseudoElements: false,
      allowHas: true,
      selectorOnlyArguments: $.pseudo_class_selector_arguments,
      forgivingSelectorArguments: $.pseudo_class_forgiving_selector_arguments,
      selectorOrValueArguments: $.pseudo_class_arguments_no_pseudo_element,
      nthArguments: $.pseudo_class_nth_child_arguments_no_pseudo_element,
    }),

    _selectors_no_pseudo_element: ($) => sep1(",", $._selector_no_pseudo_element),

    _selector_no_pseudo_element_no_nesting: ($) => selectorVariant($, $._selector_no_pseudo_element_no_nesting, {
      allowPseudoElements: false,
      allowHas: true,
      allowNesting: false,
      selectorOnlyArguments: $.pseudo_class_selector_arguments,
      forgivingSelectorArguments: $.pseudo_class_forgiving_selector_arguments,
      selectorOrValueArguments: $.pseudo_class_arguments_no_pseudo_element,
      nthArguments: $.pseudo_class_nth_child_arguments_no_pseudo_element,
    }),

    _selectors_no_pseudo_element_no_nesting: ($) => sep1(",", $._selector_no_pseudo_element_no_nesting),

    _selector_in_has: ($) => selectorVariant($, $._selector_in_has, {
      allowPseudoElements: false,
      allowHas: false,
      selectorOnlyArguments: $.pseudo_class_selector_arguments_in_has,
      forgivingSelectorArguments: $.pseudo_class_forgiving_selector_arguments_in_has,
      selectorOrValueArguments: $.pseudo_class_value_arguments,
      nthArguments: $.pseudo_class_nth_child_arguments_in_has,
    }),

    _selectors_in_has: ($) => sep1(",", $._selector_in_has),

    nesting_selector: ($) => prec(1, seq(optional($._selector), "&")),

    universal_selector: (_) => "*",

    class_selector: ($) => prec(1, seq(
      optional($._selector),
      ".",
      field("name", $.class_name),
    )),

    pseudo_class_selector: ($) => seq(
      optional($._selector),
      alias($._pseudo_class_selector_colon, ":"),
        choice(
          seq(
          alias(choice("is", "where", "matches"), $.class_name),
          alias($.pseudo_class_forgiving_selector_arguments, $.arguments),
        ),
        seq(
          alias(choice("not", "host", "host-context"), $.class_name),
          alias($.pseudo_class_selector_arguments, $.arguments),
        ),
        seq(alias("has", $.class_name), alias($.pseudo_class_has_arguments, $.arguments)),
        $._nth_pseudo_class_selector,
        seq(
          alias(choice("dir", "lang"), $.class_name),
          alias($.pseudo_class_value_arguments, $.arguments),
        ),
        prec(1, seq($.class_name, alias($.pseudo_class_arguments, $.arguments))),
        $.class_name,
        alias("host", $.class_name),
      ),
    ),

    _nth_pseudo_class_selector: ($) => choice(
      $._nth_child_pseudo_class_selector,
      $._nth_type_pseudo_class_selector,
    ),

    _nth_child_pseudo_class_selector: ($) => seq(
      alias(
        choice("nth-child", "nth-last-child"),
        $.class_name,
      ),
      alias($.pseudo_class_nth_child_arguments, $.arguments),
    ),

    _nth_type_pseudo_class_selector: ($) => seq(
      alias(
        choice("nth-of-type", "nth-last-of-type", "nth-col", "nth-last-col"),
        $.class_name,
      ),
      alias($.pseudo_class_nth_arguments, $.arguments),
    ),

    pseudo_element_selector: ($) => choice(
      seq(
        optional($._selector),
        "::",
        alias($.identifier, $.tag_name),
        optional(alias($.pseudo_element_arguments, $.arguments)),
      ),
      seq(
        optional($._selector),
        alias($._pseudo_class_selector_colon, ":"),
        alias(choice("before", "after", "first-line", "first-letter"), $.tag_name),
        optional(alias($.pseudo_element_arguments, $.arguments)),
      ),
    ),

    id_selector: ($) => seq(
      optional($._selector),
      "#",
      field("name", alias($.class_name, $.id_name)),
    ),

    attribute_selector: ($) => seq(
      optional($._selector),
      token(prec(1, "[")),
      field("name", alias(choice($.identifier, $.namespace_selector), $.attribute_name)),
      optional(seq(
        field("operator", choice("=", "~=", "^=", "|=", "*=", "$=")),
        field("value", $._value),
        optional(field("flags", $.attribute_flags)),
      )),
      "]",
    ),

    child_selector: ($) => prec.left(seq(optional($._selector), ">", $._selector)),

    descendant_selector: ($) => prec.left(seq($._selector, $._descendant_operator, $._selector)),

    sibling_selector: ($) => prec.left(seq(optional($._selector), "~", $._selector)),

    adjacent_sibling_selector: ($) => prec.left(seq(optional($._selector), "+", $._selector)),

    column_selector: ($) => prec.left(1, seq(optional($._selector), "||", $._selector)),

    namespace_selector: ($) => prec.left(seq(optional($._selector), "|", $._selector)),

    pseudo_class_arguments: ($) => seq(
      token.immediate("("),
      sep(",", choice($._selector, repeat1($._value))),
      ")",
    ),

    pseudo_class_value_arguments: ($) => seq(
      token.immediate("("),
      sep(",", repeat1($._value)),
      ")",
    ),

    pseudo_class_arguments_no_pseudo_element: ($) => seq(
      token.immediate("("),
      sep(",", choice($._selector_no_pseudo_element, repeat1($._value))),
      ")",
    ),

    pseudo_class_selector_arguments: ($) => seq(
      token.immediate("("),
      sep1(",", $._selector_no_pseudo_element),
      ")",
    ),

    pseudo_class_selector_arguments_in_has: ($) => seq(
      token.immediate("("),
      sep1(",", $._selector_in_has),
      ")",
    ),

    pseudo_class_forgiving_selector_arguments: ($) => seq(
      token.immediate("("),
      sep(",", choice($._selector_no_pseudo_element, $.forgiving_pseudo_element_recovery)),
      ")",
    ),

    pseudo_class_forgiving_selector_arguments_in_has: ($) => seq(
      token.immediate("("),
      sep(",", choice($._selector_in_has, $.forgiving_pseudo_element_recovery)),
      ")",
    ),

    pseudo_class_has_arguments: ($) => seq(
      token.immediate("("),
      sep1(",", $._selector_in_has),
      ")",
    ),

    pseudo_class_nth_child_arguments: ($) => prec(-1, seq(
      token.immediate("("),
      choice(
        alias("even", $.plain_value),
        alias("odd", $.plain_value),
        $.integer_value,
        alias($._nth_functional_notation, $.plain_value),
      ),
      optional(seq("of", $._selectors_no_pseudo_element)),
      ")",
    )),

    pseudo_class_nth_child_arguments_no_pseudo_element: ($) => prec(-1, seq(
      token.immediate("("),
      choice(
        alias("even", $.plain_value),
        alias("odd", $.plain_value),
        $.integer_value,
        alias($._nth_functional_notation, $.plain_value),
      ),
      optional(seq("of", $._selectors_no_pseudo_element)),
      ")",
    )),

    pseudo_class_nth_child_arguments_in_has: ($) => prec(-1, seq(
      token.immediate("("),
      choice(
        alias("even", $.plain_value),
        alias("odd", $.plain_value),
        $.integer_value,
        alias($._nth_functional_notation, $.plain_value),
      ),
      optional(seq("of", $._selectors_in_has)),
      ")",
    )),

    pseudo_class_nth_arguments: ($) => prec(-1, seq(
      token.immediate("("),
      choice(
        alias("even", $.plain_value),
        alias("odd", $.plain_value),
        $.integer_value,
        alias($._nth_functional_notation, $.plain_value),
      ),
      ")",
    )),

    _nth_functional_notation: (_) => /[+-]?(\d*)[nN](\s*[+-]\s*\d+)?/,

    pseudo_element_arguments: ($) => seq(
      token.immediate("("),
      sep(",", choice($._selector, repeat1($._value))),
      ")",
    ),

    declaration_value: ($) => prec.left(seq(
      $._value,
      repeat(seq(optional(","), $._value)),
    )),

    declaration: ($) => seq(
      choice(
        seq(
          field("property", alias($.custom_property_name, $.property_name)),
          ":",
          optional(field("value", $.declaration_value)),
          optional($.important),
          ";",
        ),
        seq(
          field("property", alias("unicode-range", $.property_name)),
          ":",
          field("value", $.unicode_range_list),
          optional($.important),
          ";",
        ),
        seq(
          field("property", alias($.identifier, $.property_name)),
          ":",
          field("value", $.declaration_value),
          optional($.important),
          ";",
        ),
      ),
    ),

    last_declaration: ($) => prec(1, seq(
      choice(
        seq(
          field("property", alias($.custom_property_name, $.property_name)),
          ":",
          optional(field("value", $.declaration_value)),
          optional($.important),
        ),
        seq(
          field("property", alias("unicode-range", $.property_name)),
          ":",
          field("value", $.unicode_range_list),
          optional($.important),
        ),
        seq(
          field("property", alias($.identifier, $.property_name)),
          ":",
          field("value", $.declaration_value),
          optional($.important),
        ),
      ),
    )),

    important: (_) => "!important",

    media_query_list: ($) => sep1(",", $.media_query),

    media_query: ($) => choice(
      $.media_condition,
      seq(
        optional(choice("not", "only")),
        field("type", alias($.identifier, $.media_type)),
        optional(seq("and", field("condition", $.media_condition_without_or))),
      ),
    ),

    media_condition: ($) => choice(
      $.media_not,
      $.media_and_condition,
      $.media_or_condition,
      $.media_condition_term,
    ),

    media_condition_without_or: ($) => choice(
      $.media_not,
      prec(1, $.media_and_condition),
      $.media_condition_term,
    ),

    media_not: ($) => seq("not", $.media_condition_term),

    media_and_condition: ($) => prec.left(2, seq(
      $.media_condition_term,
      repeat1(seq("and", $.media_condition_term)),
    )),

    media_or_condition: ($) => prec.left(1, seq(
      $.media_condition_without_or,
      repeat1(seq("or", $.media_condition_without_or)),
    )),

    media_condition_term: ($) => choice(
      $.general_enclosed_parens,
      $.media_in_parens,
      $.general_enclosed_function,
    ),

    media_in_parens: ($) => choice(
      prec.dynamic(2, $.media_feature),
      prec.dynamic(1, seq("(", $.media_condition, ")")),
    ),

    media_feature: ($) => seq(
      "(",
      choice($.query_feature_plain, $.query_feature_boolean, $.query_feature_range),
      ")",
    ),

    container_query: ($) => choice(
      prec(1, seq(
        field("name", alias($.identifier, $.container_name)),
        field("condition", $.container_condition),
      )),
      field("condition", $.container_condition),
    ),

    container_condition: ($) => choice(
      $.container_not,
      $.container_and_condition,
      $.container_or_condition,
      $.container_condition_term,
    ),

    container_condition_without_or: ($) => choice(
      $.container_not,
      prec(1, $.container_and_condition),
      $.container_condition_term,
    ),

    container_not: ($) => seq("not", $.container_condition_term),

    container_and_condition: ($) => prec.left(2, seq(
      $.container_condition_term,
      repeat1(seq("and", $.container_condition_term)),
    )),

    container_or_condition: ($) => prec.left(1, seq(
      $.container_condition_without_or,
      repeat1(seq("or", $.container_condition_without_or)),
    )),

    container_condition_term: ($) => choice(
      $.general_enclosed_parens,
      $.container_in_parens,
      $.general_enclosed_function,
    ),

    container_in_parens: ($) => choice(
      prec.dynamic(3, $.container_feature),
      prec.dynamic(2, $.container_style_query),
      prec.dynamic(1, seq("(", $.container_condition, ")")),
    ),

    container_feature: ($) => seq(
      "(",
      choice($.query_feature_plain, $.query_feature_boolean, $.query_feature_range),
      ")",
    ),

    container_style_query: ($) => seq(
      "style",
      token.immediate("("),
      $.style_query,
      ")",
    ),

    style_query: ($) => choice($.style_condition, $.style_feature_body),

    style_condition: ($) => choice(
      $.style_not,
      $.style_and_condition,
      $.style_or_condition,
      $.style_condition_term,
    ),

    style_condition_without_or: ($) => choice(
      $.style_not,
      prec(1, $.style_and_condition),
      $.style_condition_term,
    ),

    style_not: ($) => seq("not", $.style_condition_term),

    style_and_condition: ($) => prec.left(2, seq(
      $.style_condition_term,
      repeat1(seq("and", $.style_condition_term)),
    )),

    style_or_condition: ($) => prec.left(1, seq(
      $.style_condition_without_or,
      repeat1(seq("or", $.style_condition_without_or)),
    )),

    style_condition_term: ($) => choice(
      $.general_enclosed_parens,
      $.style_in_parens,
      $.general_enclosed_function,
    ),

    style_in_parens: ($) => choice(
      prec.dynamic(2, $.style_feature),
      prec.dynamic(1, seq("(", $.style_condition, ")")),
    ),

    style_feature: ($) => seq("(", $.style_feature_body, ")"),

    style_feature_body: ($) => seq(
      field("name", alias(choice($.identifier, $.custom_property_name), $.feature_name)),
      ":",
      optional(field("value", $.declaration_value)),
    ),

    supports_condition: ($) => choice(
      $.supports_not,
      $.supports_and_condition,
      $.supports_or_condition,
      $.supports_condition_term,
    ),

    supports_condition_without_or: ($) => choice(
      $.supports_not,
      prec(1, $.supports_and_condition),
      $.supports_condition_term,
    ),

    supports_function_condition: ($) => choice(
      $.supports_condition,
      $.supports_feature_body,
    ),

    supports_not: ($) => seq("not", $.supports_condition_term),

    supports_and_condition: ($) => prec.left(2, seq(
      $.supports_condition_term,
      repeat1(seq("and", $.supports_condition_term)),
    )),

    supports_or_condition: ($) => prec.left(1, seq(
      $.supports_condition_without_or,
      repeat1(seq("or", $.supports_condition_without_or)),
    )),

    supports_condition_term: ($) => choice(
      $.general_enclosed_parens,
      $.supports_in_parens,
      $.general_enclosed_function,
    ),

    supports_in_parens: ($) => choice(
      prec.dynamic(3, $.supports_feature),
      prec.dynamic(2, $.selector_query),
      prec.dynamic(1, seq("(", $.supports_condition, ")")),
    ),

    general_enclosed_function: ($) => prec(-1, seq(
      alias($.identifier, $.function_name),
      $.arguments,
    )),

    general_enclosed_parens: ($) => seq("(", optional($.general_enclosed_value), ")"),

    general_enclosed_value: ($) => $._general_enclosed_value,

    supports_feature: ($) => seq("(", $.supports_feature_body, ")"),

    supports_feature_body: ($) => seq(
      field("name", alias(choice($.identifier, $.custom_property_name), $.feature_name)),
      ":",
      optional(field("value", $.declaration_value)),
    ),

    selector_query: ($) => seq("selector", "(", $.selectors, ")"),

    query_feature_boolean: ($) => field("name", alias($.identifier, $.feature_name)),

    query_feature_plain: ($) => seq(
      field("name", alias($.identifier, $.feature_name)),
      ":",
      field("value", $.declaration_value),
    ),

    query_feature_range: ($) => choice(
      prec.dynamic(2, seq(
        field("left", $._query_value),
        field("operator", $.comparison_operator),
        field("name", alias($.identifier, $.feature_name)),
      )),
      prec.dynamic(1, seq(
        field("name", alias($.identifier, $.feature_name)),
        field("operator", $.comparison_operator),
        field("right", $._query_value),
      )),
      prec.dynamic(3, seq(
        field("left", $._query_value),
        field("left_operator", $.comparison_operator),
        field("name", alias($.identifier, $.feature_name)),
        field("right_operator", $.comparison_operator),
        field("right", $._query_value),
      )),
    ),

    _query_value: ($) => prec(1, choice(
      prec.dynamic(2, $.call_expression),
      prec.dynamic(1, $.binary_expression),
      $.parenthesized_value,
      $.color_value,
      $.integer_value,
      $.float_value,
      $.string_value,
      $.grid_value,
      $.plain_value,
    )),

    comparison_operator: (_) => choice("<=", ">=", "<", ">", "="),

    _value: ($) => prec(-1, choice(
      alias($.identifier, $.plain_value),
      $.plain_value,
      $.color_value,
      $.integer_value,
      $.float_value,
      $.string_value,
      $.grid_value,
      $.binary_expression,
      $.parenthesized_value,
      $.call_expression,
      $.important,
    )),

    parenthesized_value: ($) => seq("(", $._value, ")"),

    color_value: (_) => seq("#", token.immediate(/[0-9a-fA-F]{3,8}/)),

    string_value: ($) => choice(
      seq(
        "'",
        repeat(choice(
          alias(/[^\\'\n]+/, $.string_content),
          $.escape_sequence,
        )),
        "'",
      ),
      seq(
        '"',
        repeat(choice(
          alias(/[^\\"\n]+/, $.string_content),
          $.escape_sequence,
        )),
        '"',
      ),
    ),

    escape_sequence: (_) => token(seq(
      "\\",
      choice(
        /[0-9a-fA-F]{1,6}\s?/,
        /[^0-9a-fA-F\n\r]/,
      ),
    )),

    integer_value: ($) => seq(
      token(seq(optional(choice("+", "-")), /\d+/)),
      optional($.unit),
    ),

    float_value: ($) => seq(
      token(seq(
        optional(choice("+", "-")),
        /\d*/,
        choice(
          seq(".", /\d+/),
          seq(/[eE]/, optional(choice("+", "-")), /\d+/),
          seq(".", /\d+/, /[eE]/, optional(choice("+", "-")), /\d+/),
        ),
      )),
      optional($.unit),
    ),

    unit: (_) => token.immediate(/[a-zA-Z%]+/),

    grid_value: ($) => seq("[", sep1(",", $._value), "]"),

    unicode_range_list: ($) => sep1(",", $.unicode_range_value),

    unicode_range_value: ($) => $._unicode_range_value,

    call_expression: ($) => choice(
      seq(alias("url", $.function_name), $.url_arguments),
      seq(alias($.identifier, $.function_name), $.arguments),
    ),

    binary_expression: ($) => prec.left(seq($._value, choice("+", "-", "*", "/"), $._value)),

    url_arguments: ($) => seq(
      token.immediate("("),
      optional(choice(
        $.url_with_modifiers,
        $.string_value,
        $.url_unquoted_value,
        $.bad_url_value,
      )),
      ")",
    ),

    url_with_modifiers: ($) => seq(
      field("value", $.string_value),
      repeat1(field("modifier", $.url_modifier)),
    ),

    url_modifier: ($) => choice(
      alias($.identifier, $.url_modifier_name),
      $.url_modifier_function,
    ),

    url_modifier_function: ($) => seq(
      alias($.identifier, $.function_name),
      $.arguments,
    ),

    arguments: ($) => seq(
      token.immediate("("),
      optional(seq(
        optional(repeat1($._value)),
        repeat(seq(choice(",", ";"), optional(repeat1($._value)))),
      )),
      ")",
    ),

    class_name: ($) => seq(
      choice($.identifier, $.escape_sequence),
      repeat(choice(
        alias(/([a-zA-Z0-9_-]|[^\x00-\x7F])+/, $.identifier),
        $.escape_sequence,
      )),
    ),

    url_unquoted_value: ($) => seq(
      choice(alias(/[^\s()'"\\]+/, $.url_unquoted_chunk), $.escape_sequence),
      repeat(choice(alias(/[^\s()'"\\]+/, $.url_unquoted_chunk), $.escape_sequence)),
    ),

    bad_url_value: ($) => $._bad_url_value,

    attribute_flags: (_) => token(/[iIsS]/),

    custom_media_boolean: (_) => choice("true", "false"),

    forgiving_pseudo_element_recovery: ($) => $._forgiving_pseudo_element_recovery,

    custom_property_name: (_) => /--([a-zA-Z0-9_-]|[^\x00-\x7F]|\\[0-9a-fA-F]{1,6}\s?|\\[^\n\r0-9a-fA-F])+/,

    identifier: (_) => /(--|-?([a-zA-Z_]|[^\x00-\x7F]|\\[0-9a-fA-F]{1,6}\s?|\\[^\n\r0-9a-fA-F]))(([a-zA-Z0-9_-]|[^\x00-\x7F]|\\[0-9a-fA-F]{1,6}\s?|\\[^\n\r0-9a-fA-F]))*/,

    at_keyword: (_) => /@[a-zA-Z-_]+/,

    js_comment: (_) => token(prec(-1, seq("//", /.*/))),

    comment: (_) => token(seq(
      "/*",
      /[^*]*\*+([^/*][^*]*\*+)*/,
      "/",
    )),

    html_comment_delimiter: (_) => token(choice("<!--", "-->")),

    plain_value: (_) => token(prec(-1, seq(
      repeat(choice(
        /[-_]/,
        /\/[^*\s,;!{}()\[\]]/,
      )),
      /[a-zA-Z]/,
      repeat(choice(
        /[^/\s,;!{}()\[\]:]/,
        /\/[^*\s,;!{}()\[\]]/,
      )),
    ))),

    important_value: (_) => token(seq(
      "!",
      /[a-zA-Z]/,
      repeat(/[a-zA-Z0-9-_]/),
    )),
  },
});

function sep(separator, rule) {
  return optional(sep1(separator, rule));
}

function sep1(separator, rule) {
  return seq(rule, repeat(seq(separator, rule)));
}

function selectorVariant($, self, options) {
  const {
    allowPseudoElements,
    allowHas,
    allowNesting = true,
    selectorOnlyArguments,
    forgivingSelectorArguments,
    selectorOrValueArguments,
    nthArguments,
  } = options;

  const variants = [
    $.universal_selector,
    prec(1, seq(optional(self), alias($.identifier, $.tag_name))),
    alias(prec(1, seq(optional(self), ".", field("name", $.class_name))), $.class_selector),
    alias(seq(
      optional(self),
      alias($._pseudo_class_selector_colon, ":"),
        choice(
          seq(
          alias(choice("is", "where", "matches"), $.class_name),
          alias(forgivingSelectorArguments, $.arguments),
        ),
        seq(
          alias(choice("not", "host", "host-context"), $.class_name),
          alias(selectorOnlyArguments, $.arguments),
        ),
        ...(allowHas ? [seq(alias("has", $.class_name), alias($.pseudo_class_has_arguments, $.arguments))] : []),
        seq(
          alias(
            choice("nth-of-type", "nth-last-of-type", "nth-col", "nth-last-col"),
            $.class_name,
          ),
          alias($.pseudo_class_nth_arguments, $.arguments),
        ),
        seq(
          alias(choice("nth-child", "nth-last-child"), $.class_name),
          alias(nthArguments, $.arguments),
        ),
        seq(
          alias(choice("dir", "lang"), $.class_name),
          alias($.pseudo_class_value_arguments, $.arguments),
        ),
        prec(1, seq($.class_name, alias(selectorOrValueArguments, $.arguments))),
        $.class_name,
        alias("host", $.class_name),
      ),
    ), $.pseudo_class_selector),
    alias(seq(optional(self), "#", field("name", alias($.class_name, $.id_name))), $.id_selector),
    alias(seq(
      optional(self),
      token(prec(1, "[")),
      field("name", alias(choice($.identifier, $.namespace_selector), $.attribute_name)),
      optional(seq(
        field("operator", choice("=", "~=", "^=", "|=", "*=", "$=")),
        field("value", $._value),
        optional(field("flags", $.attribute_flags)),
      )),
      "]",
    ), $.attribute_selector),
    $.string_value,
    alias(prec.left(seq(optional(self), ">", self)), $.child_selector),
    alias(prec.left(seq(self, $._descendant_operator, self)), $.descendant_selector),
    alias(prec.left(seq(optional(self), "~", self)), $.sibling_selector),
    alias(prec.left(seq(optional(self), "+", self)), $.adjacent_sibling_selector),
    alias(prec.left(seq(optional(self), "||", self)), $.column_selector),
    alias(prec.left(seq(optional(self), "|", self)), $.namespace_selector),
  ];

  if (allowNesting) {
    variants.splice(3, 0, alias(prec(1, seq(optional(self), "&")), $.nesting_selector));
  }

  if (allowPseudoElements) {
    variants.splice(allowNesting ? 5 : 4, 0,
      alias(seq(
        optional(self),
        "::",
        alias($.identifier, $.tag_name),
        optional(alias($.pseudo_element_arguments, $.arguments)),
      ), $.pseudo_element_selector),
      alias(seq(
        optional(self),
        alias($._pseudo_class_selector_colon, ":"),
        alias(choice("before", "after", "first-line", "first-letter"), $.tag_name),
        optional(alias($.pseudo_element_arguments, $.arguments)),
      ), $.pseudo_element_selector),
    );
  }

  return choice(...variants);
}
