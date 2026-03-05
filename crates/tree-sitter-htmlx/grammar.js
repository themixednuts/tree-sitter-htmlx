/**
 * HTMLX grammar for tree-sitter
 *
 * Expression-enhanced HTML extending tree-sitter-html.
 * Adds expressions, shorthand attributes, spread, directives, and namespaced tags.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const HTML = require("../tree-sitter-html/grammar");

module.exports = grammar(HTML, {
  name: "htmlx",

  conflicts: ($) => [
    // Conflict between attribute ending with plain _attribute_value
    // vs continuing with unquoted_attribute_value (text{expr} pattern)
    // We prefer the longer unquoted_attribute_value match
    [$.attribute, $.unquoted_attribute_value],
  ],

  externals: ($, original) =>
    original.concat([
      $._tag_namespace,
      $._tag_local_name,
      $._ts_lang_marker,
      $._expression_js,
      $._expression_ts,
      $._attribute_expression_js,
      $._attribute_expression_ts,
      $._directive_marker,
      $._member_tag_object, // First part of dotted component (UI in UI.Button)
      $._member_tag_property, // Subsequent parts (.Button, .Card)
      $._attribute_value, // Unquoted attribute value that stops at { or whitespace
      $._pipe_attribute_name, // Attribute name starting with | (like |-wtf)
      $._line_tag_comment, // // comment in tag attribute list
      $._block_tag_comment, // /* comment */ in tag attribute list
      $._unterminated_tag_end, // newline-delimited malformed start tag terminator
    ]),

  rules: {
    _node: ($, original) => choice(original, prec(-1, $.expression)),

    element: ($) =>
      choice(
        // Unterminated start tags recover as standalone elements.
        seq(alias($._unterminated_start_tag, $.start_tag)),
        prec(
          -1,
          seq(alias($._broken_member_unterminated_start_tag, $.start_tag)),
        ),
        seq(alias($._namespaced_unterminated_start_tag, $.start_tag)),
        seq(alias($._member_unterminated_start_tag, $.start_tag)),
        seq(alias($._raw_text_unterminated_start_tag, $.start_tag)),
        // Normal elements - content is parsed as nodes
        seq(
          $.start_tag,
          repeat($._node),
          choice(
            prec(1, $.end_tag),
            prec(10, $._unterminated_tag_end),
            $._implicit_end_tag,
          ),
        ),
        // Namespaced elements (svelte:head)
        seq(
          alias($._namespaced_start_tag, $.start_tag),
          repeat($._node),
          alias($._namespaced_end_tag, $.end_tag),
        ),
        // Member/dotted component elements (UI.Button)
        seq(
          alias($._member_start_tag, $.start_tag),
          repeat($._node),
          alias($._member_end_tag, $.end_tag),
        ),
        // Raw text elements (script, style, textarea, title)
        $._raw_text_element,
        // Self-closing tags
        $.self_closing_tag,
        alias($._namespaced_self_closing_tag, $.self_closing_tag),
        alias($._member_self_closing_tag, $.self_closing_tag),
      ),

    start_tag: ($) =>
      seq(
        "<",
        alias($._start_tag_name, $.tag_name),
        repeat($._tag_attribute_item),
        ">",
      ),

    _unterminated_start_tag: ($) =>
      seq(
        "<",
        alias($._start_tag_name, $.tag_name),
        repeat($._tag_attribute_item),
        $._unterminated_tag_end,
      ),

    _broken_member_unterminated_start_tag: ($) =>
      seq(
        "<",
        alias($._start_tag_name, $.tag_name),
        ".",
        $._unterminated_tag_end,
      ),

    self_closing_tag: ($) =>
      seq(
        "<",
        alias($._start_tag_name, $.tag_name),
        repeat($._tag_attribute_item),
        "/>",
      ),

    // Override raw text element to use HTMLX-aware attribute handling
    _raw_text_element: ($) =>
      seq(
        alias($._raw_text_start_tag, $.start_tag),
        optional($.raw_text),
        $.end_tag,
      ),

    _raw_text_start_tag: ($) =>
      seq(
        "<",
        alias($._raw_text_start_tag_name, $.tag_name),
        repeat($._tag_attribute_item),
        ">",
      ),

    _raw_text_unterminated_start_tag: ($) =>
      seq(
        "<",
        alias($._raw_text_start_tag_name, $.tag_name),
        repeat($._tag_attribute_item),
        $._unterminated_tag_end,
      ),

    _namespaced_start_tag: ($) =>
      seq(
        "<",
        alias($._namespaced_tag_name, $.tag_name),
        repeat($._tag_attribute_item),
        ">",
      ),

    _namespaced_unterminated_start_tag: ($) =>
      seq(
        "<",
        alias($._namespaced_tag_name, $.tag_name),
        repeat($._tag_attribute_item),
        $._unterminated_tag_end,
      ),

    _namespaced_self_closing_tag: ($) =>
      seq(
        "<",
        alias($._namespaced_tag_name, $.tag_name),
        repeat($._tag_attribute_item),
        "/>",
      ),

    _namespaced_end_tag: ($) =>
      seq("</", alias($._namespaced_tag_name, $.tag_name), ">"),

    _namespaced_tag_name: ($) =>
      seq(
        field("namespace", alias($._tag_namespace, $.tag_namespace)),
        ":",
        field("name", alias($._tag_local_name, $.tag_local_name)),
      ),

    // Member/dotted component tags: UI.Button, Lib.UI.Card
    _member_start_tag: ($) =>
      seq(
        "<",
        alias($._member_tag_name, $.tag_name),
        repeat($._tag_attribute_item),
        ">",
      ),

    _member_unterminated_start_tag: ($) =>
      seq(
        "<",
        alias($._member_tag_name, $.tag_name),
        repeat($._tag_attribute_item),
        $._unterminated_tag_end,
      ),

    _member_self_closing_tag: ($) =>
      seq(
        "<",
        alias($._member_tag_name, $.tag_name),
        repeat($._tag_attribute_item),
        "/>",
      ),

    _tag_attribute_item: ($) => choice($.attribute, $.tag_comment),

    tag_comment: ($) =>
      choice(
        field("kind", alias($._line_tag_comment, $.line_comment)),
        field("kind", alias($._block_tag_comment, $.block_comment)),
      ),

    _member_end_tag: ($) =>
      seq("</", alias($._member_tag_name, $.tag_name), ">"),

    // Member tag name: Object.Property or Object.Nested.Property
    // Use prec.right to prefer continuing with more properties over matching as attributes
    _member_tag_name: ($) =>
      prec.right(
        seq(
          field("object", alias($._member_tag_object, $.tag_member)),
          repeat1(
            seq(
              ".",
              field("property", alias($._member_tag_property, $.tag_member)),
            ),
          ),
        ),
      ),

    attribute: ($) =>
      choice(
        seq($._ts_lang_marker, $.attribute_name, "=", $.quoted_attribute_value),
        prec(1, $.spread_attribute),
        $.shorthand_attribute,
        // Use dynamic precedence to prefer longer unquoted_attribute_value matches
        // over shorter attribute_value + shorthand_attribute sequences
        prec.dynamic(
          2,
          seq(
            $.attribute_name,
            optional(
              seq(
                "=",
                choice(
                  $.unquoted_attribute_value, // Match text{expr} patterns
                  $.quoted_attribute_value,
                  alias($.attribute_expression, $.expression),
                  alias($._attribute_value, $.attribute_value), // Plain text value via external scanner
                ),
              ),
            ),
          ),
        ),
      ),

    attribute_name: ($) =>
      choice(
        $.__attribute_directive,
        // Attribute names starting with | (like |-wtf) - handled by external scanner
        // to distinguish from directive modifiers (on:click|preventDefault)
        $._pipe_attribute_name,
        // Exclude '.', '|', '(' and ')' from the start to avoid conflicts with dotted
        // component properties, directive modifiers, and malformed expression tails
        /[^<>{}\"':\\/=\s|.()][^<>{}\"':\\/=\s|()]*/,
      ),

    expression: ($) =>
      seq(
        "{",
        optional(
          field(
            "content",
            choice(
              alias($._expression_js, $.js),
              alias($._expression_ts, $.ts),
            ),
          ),
        ),
        "}",
      ),

    attribute_expression: ($) =>
      seq(
        "{",
        optional(
          field(
            "content",
            choice(
              alias($._attribute_expression_js, $.js),
              alias($._attribute_expression_ts, $.ts),
            ),
          ),
        ),
        "}",
      ),
    // Spread attribute: {...expr}
    // Keep this expression-based so nested braces are handled correctly.
    spread_attribute: ($) =>
      seq(
        "{...",
        field(
          "content",
          choice(
            alias($._attribute_expression_js, $.js),
            alias($._attribute_expression_ts, $.ts),
          ),
        ),
        "}",
      ),
    // Shorthand attribute: {identifier} - an expression used as an attribute
    // Uses expression structure (not regex) to allow proper precedence resolution
    // with unquoted_attribute_value (text{expr} patterns like style:attr=string{mixed})
    shorthand_attribute: ($) =>
      seq(
        "{",
        optional(
          field(
            "content",
            choice(
              alias($._expression_js, $.js),
              alias($._expression_ts, $.ts),
            ),
          ),
        ),
        "}",
      ),

    // Directives: bind:value, on:click|preventDefault
    // The _directive_marker external scanner consumes the directive name (bind, on, let, etc.)
    // and checks for the following colon. It returns the directive name as the token.
    __attribute_directive: ($) =>
      seq(
        alias($._directive_marker, $.attribute_directive),
        ":",
        $.attribute_identifier,
        optional($.attribute_modifiers),
      ),
    // Source JS parser accepts directive names as raw tag token text up to
    // delimiters and then splits modifiers with `|`.
    // Keep identifier permissive so names like `--color` and `$store.action`
    // are parsed in CST and validated later in semantic phases.
    attribute_identifier: ($) => /[^<>{}"':\\\/=\s|]+/,
    attribute_modifiers: ($) => repeat1(seq("|", $.attribute_modifier)),
    attribute_modifier: ($) => /[a-zA-Z_$][a-zA-Z0-9_$]*/,

    // Unquoted attribute value with embedded expressions: text{expr} or text{expr}text
    // This allows patterns like style:color=red{expr} or class=item-{type}-active
    // Uses external scanner _attribute_value to properly handle lookahead for {
    unquoted_attribute_value: ($) =>
      prec.right(
        seq(
          alias($._attribute_value, $.attribute_value), // Required leading text via external scanner
          repeat1(
            seq(
              alias($.attribute_expression, $.expression),
              optional(alias($._attribute_value, $.attribute_value)), // Optional trailing text
            ),
          ),
        ),
      ),

    quoted_attribute_value: ($) =>
      choice(
        seq("'", repeat($._quoted_attribute_content_single), "'"),
        seq('"', repeat($._quoted_attribute_content_double), '"'),
      ),
    _quoted_attribute_content_single: ($) =>
      choice(
        alias($.attribute_expression, $.expression),
        alias(/[^'{]+/, $.attribute_value),
      ),
    _quoted_attribute_content_double: ($) =>
      choice(
        alias($.attribute_expression, $.expression),
        alias(/[^"{]+/, $.attribute_value),
      ),
  },
});
