/**
 * HTMLX grammar for tree-sitter
 *
 * Expression-enhanced HTML extending tree-sitter-html.
 * Adds expressions, shorthand attributes, spread, directives, and namespaced tags.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const HTML = require('../tree-sitter-html/grammar');

module.exports = grammar(HTML, {
  name: 'htmlx',

  externals: ($, original) => original.concat([
    $._tag_namespace,
    $._tag_local_name,
    $._ts_lang_marker,
    $._expression_js,
    $._expression_ts,
    $._directive_marker,
  ]),

  rules: {
    _node: ($, original) => choice(
      original,
      $.expression,
    ),

    element: $ => choice(
      // Normal elements - content is parsed as nodes
      seq($.start_tag, repeat($._node), choice($.end_tag, $._implicit_end_tag)),
      // Namespaced elements
      seq(alias($._namespaced_start_tag, $.start_tag), repeat($._node), alias($._namespaced_end_tag, $.end_tag)),
      // Raw text elements (script, style, textarea, title)
      $._raw_text_element,
      // Self-closing tags
      $.self_closing_tag,
      alias($._namespaced_self_closing_tag, $.self_closing_tag),
    ),

    // Override raw text element to use HTMLX-aware attribute handling
    _raw_text_element: $ => seq(
      alias($._raw_text_start_tag, $.start_tag),
      optional($.raw_text),
      $.end_tag,
    ),

    _raw_text_start_tag: $ => seq(
      '<',
      alias($._raw_text_start_tag_name, $.tag_name),
      repeat($.attribute),
      '>',
    ),

    _namespaced_start_tag: $ => seq(
      '<',
      alias($._namespaced_tag_name, $.tag_name),
      repeat($.attribute),
      '>',
    ),

    _namespaced_self_closing_tag: $ => seq(
      '<',
      alias($._namespaced_tag_name, $.tag_name),
      repeat($.attribute),
      '/>',
    ),

    _namespaced_end_tag: $ => seq(
      '</',
      alias($._namespaced_tag_name, $.tag_name),
      '>',
    ),

    _namespaced_tag_name: $ => seq(
      field('namespace', alias($._tag_namespace, $.tag_namespace)),
      ':',
      field('name', alias($._tag_local_name, $.tag_local_name)),
    ),

    attribute: $ => choice(
      seq($._ts_lang_marker, $.attribute_name, '=', $.quoted_attribute_value),
      prec(1, $.spread_attribute),
      $.shorthand_attribute,
      seq(
        $.attribute_name,
        optional(seq('=', choice(
          $.attribute_value,
          $.quoted_attribute_value,
          $.expression,
        ))),
      ),
    ),

    attribute_name: $ => choice(
      $.__attribute_directive,
      /[^<>{}\"':\\/=\s|][^<>{}\"':\\/=\s|]*/,
    ),

    expression: $ => seq(
      '{',
      optional(field('content', choice(
        alias($._expression_js, $.js),
        alias($._expression_ts, $.ts),
      ))),
      '}',
    ),
    spread_attribute: $ => /\{\.\.\.([^}]*)\}/,
    shorthand_attribute: $ => /\{[^.}][^}]*\}|\{[.][^.}][^}]*\}|\{[.][.][^.}][^}]*\}|\{\}/,

    // Directives: bind:value, on:click|preventDefault
    __attribute_directive: $ => seq(
      $._directive_marker,
      $.attribute_directive,
      ':',
      $.attribute_identifier,
      optional($.attribute_modifiers),
    ),
    attribute_directive: $ => /[a-zA-Z_$][a-zA-Z0-9_$]*/,
    attribute_identifier: $ => /[a-zA-Z_$][a-zA-Z0-9_$-]*/,
    attribute_modifiers: $ => repeat1(seq('|', $.attribute_modifier)),
    attribute_modifier: $ => /[a-zA-Z_$][a-zA-Z0-9_$]*/,

    // Text content handled by external scanner (stops at '{' for expressions)
    attribute_value: $ => /[^<>{}\"'/=\s]+/,

    quoted_attribute_value: $ => choice(
      seq("'", repeat($._quoted_attribute_content_single), "'"),
      seq('"', repeat($._quoted_attribute_content_double), '"'),
    ),
    _quoted_attribute_content_single: $ => choice(
      $.expression,
      alias(/[^'{]+/, $.attribute_value),
    ),
    _quoted_attribute_content_double: $ => choice(
      $.expression,
      alias(/[^"{]+/, $.attribute_value),
    ),
  },
});
