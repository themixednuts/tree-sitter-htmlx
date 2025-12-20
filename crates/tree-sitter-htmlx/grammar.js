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
      prec(2, $.directive_attribute),
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

    // Directives
    directive_attribute: $ => seq(
      $.directive_name,
      optional(seq('=', choice(
        $.attribute_value,
        $.quoted_attribute_value,
        $.expression,
      ))),
    ),

    directive_name: $ => seq(
      $.directive_prefix,
      field('name', $.directive_value),
      optional(field('modifiers', $.directive_modifiers)),
    ),

    directive_prefix: $ => token(seq(
      choice('bind', 'on', 'class', 'style', 'use', 'transition', 'in', 'out', 'animate', 'let'),
      ':'
    )),

    directive_value: $ => /[a-zA-Z_$][a-zA-Z0-9_$]*/,
    directive_modifiers: $ => repeat1(seq('|', $.directive_modifier)),
    directive_modifier: $ => /[a-zA-Z_$][a-zA-Z0-9_$]*/,

    // Overrides
    text: $ => /[^<>&{}\s]([^<>&{}]*[^<>&{}\s])?/,
    attribute_name: $ => /[^<>{}"':\\/=\s|][^<>{}"':\\/=\s|]*/,
    attribute_value: $ => /[^<>{}"'/=\s]+/,
  },
});
