/**
 * HTMLX grammar for tree-sitter
 *
 * Expression-enhanced HTML extending tree-sitter-html.
 * Adds expressions, shorthand attributes, spread, directives, and namespaced tags.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const HTML = require('../../external/tree-sitter-html/grammar');

module.exports = grammar(HTML, {
  name: 'htmlx',

  externals: ($, original) => original.concat([
    $._tag_namespace,
    $._tag_local_name,
    $._ts_lang_marker,    // Zero-width marker that sets TypeScript mode
    $._expression_js,     // Expression content in JavaScript mode
    $._expression_ts,     // Expression content in TypeScript mode
    $._attribute_directive, // Identifier followed by : (lookahead, doesn't consume :)
  ]),

  rules: {
    _node: ($, original) => choice(
      original,
      $.expression,
    ),

    // Override element to support namespaced tags
    element: $ => choice(
      seq($.start_tag, repeat($._node), choice($.end_tag, $._implicit_end_tag)),
      seq(alias($._namespaced_start_tag, $.start_tag), repeat($._node), alias($._namespaced_end_tag, $.end_tag)),
      $.self_closing_tag,
      alias($._namespaced_self_closing_tag, $.self_closing_tag),
    ),

    // Namespaced tag variants (aliased to standard names)
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

    // Namespaced tag name with namespace and local name fields
    _namespaced_tag_name: $ => seq(
      field('namespace', alias($._tag_namespace, $.tag_namespace)),
      ':',
      field('name', alias($._tag_local_name, $.tag_local_name)),
    ),

    // Attributes
    attribute: $ => choice(
      // TypeScript lang attribute - zero-width marker sets mode, then parse normally
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

    // Attribute name can be a simple identifier or a directive
    attribute_name: $ => choice(
      prec(2, $.__attribute_directive),
      /[^<>{}"':\\/=\s|][^<>{}"':\\/=\s|]*/,
    ),

    // Expressions - scanner determines JS or TS based on script lang
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

    // Directives (e.g., bind:value, on:click|preventDefault)
    // Nested inside attribute_name for HTML spec compatibility
    __attribute_directive: $ => seq(
      alias($._attribute_directive, $.attribute_directive),
      ':',
      $.attribute_identifier,
      optional($.attribute_modifiers),
    ),
    attribute_identifier: $ => /[a-zA-Z_$][a-zA-Z0-9_$-]*/,
    attribute_modifiers: $ => repeat1(seq('|', $.attribute_modifier)),
    attribute_modifier: $ => /[a-zA-Z_$][a-zA-Z0-9_$]*/,

    // Overrides
    text: $ => /[^<>&{}\s]([^<>&{}]*[^<>&{}\s])?/,
    attribute_value: $ => /[^<>{}"'/=\s]+/,
  },
});
