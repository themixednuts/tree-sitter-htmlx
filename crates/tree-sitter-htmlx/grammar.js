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

  conflicts: $ => [
    // Conflict between attribute ending with plain _attribute_value
    // vs continuing with unquoted_attribute_value (text{expr} pattern)
    // We prefer the longer unquoted_attribute_value match
    [$.attribute, $.unquoted_attribute_value],
  ],

  externals: ($, original) => original.concat([
    $._tag_namespace,
    $._tag_local_name,
    $._ts_lang_marker,
    $._expression_js,
    $._expression_ts,
    $._directive_marker,
    $._member_tag_object,    // First part of dotted component (UI in UI.Button)
    $._member_tag_property,  // Subsequent parts (.Button, .Card)
    $._attribute_value,      // Unquoted attribute value that stops at { or whitespace
  ]),

  rules: {
    _node: ($, original) => choice(
      original,
      $.expression,
    ),

    element: $ => choice(
      // Normal elements - content is parsed as nodes
      seq($.start_tag, repeat($._node), choice($.end_tag, $._implicit_end_tag)),
      // Namespaced elements (svelte:head)
      seq(alias($._namespaced_start_tag, $.start_tag), repeat($._node), alias($._namespaced_end_tag, $.end_tag)),
      // Member/dotted component elements (UI.Button)
      seq(alias($._member_start_tag, $.start_tag), repeat($._node), alias($._member_end_tag, $.end_tag)),
      // Raw text elements (script, style, textarea, title)
      $._raw_text_element,
      // Self-closing tags
      $.self_closing_tag,
      alias($._namespaced_self_closing_tag, $.self_closing_tag),
      alias($._member_self_closing_tag, $.self_closing_tag),
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

    // Member/dotted component tags: UI.Button, Lib.UI.Card
    _member_start_tag: $ => seq(
      '<',
      alias($._member_tag_name, $.tag_name),
      repeat($.attribute),
      '>',
    ),

    _member_self_closing_tag: $ => seq(
      '<',
      alias($._member_tag_name, $.tag_name),
      repeat($.attribute),
      '/>',
    ),

    _member_end_tag: $ => seq(
      '</',
      alias($._member_tag_name, $.tag_name),
      '>',
    ),

    // Member tag name: Object.Property or Object.Nested.Property
    // Use prec.right to prefer continuing with more properties over matching as attributes
    _member_tag_name: $ => prec.right(seq(
      field('object', alias($._member_tag_object, $.tag_member)),
      repeat1(seq('.', field('property', alias($._member_tag_property, $.tag_member)))),
    )),

    attribute: $ => choice(
      seq($._ts_lang_marker, $.attribute_name, '=', $.quoted_attribute_value),
      prec(1, $.spread_attribute),
      $.shorthand_attribute,
      // Use dynamic precedence to prefer longer unquoted_attribute_value matches
      // over shorter attribute_value + shorthand_attribute sequences
      prec.dynamic(2, seq(
        $.attribute_name,
        optional(seq('=', choice(
          $.unquoted_attribute_value,  // Match text{expr} patterns
          $.quoted_attribute_value,
          $.expression,
          alias($._attribute_value, $.attribute_value),  // Plain text value via external scanner
        ))),
      )),
    ),

    attribute_name: $ => choice(
      $.__attribute_directive,
      // Exclude '.' from the start to avoid conflicts with dotted component properties
      /[^<>{}\"':\\/=\s|.][^<>{}\"':\\/=\s|]*/,
    ),

    expression: $ => seq(
      '{',
      optional(field('content', choice(
        alias($._expression_js, $.js),
        alias($._expression_ts, $.ts),
      ))),
      '}',
    ),
    // Spread attribute: {...props}
    // Keep as regex since it has a distinctive pattern with ...
    spread_attribute: $ => /\{\.\.\.([^}]*)\}/,
    // Shorthand attribute: {identifier} - an expression used as an attribute
    // Uses expression structure (not regex) to allow proper precedence resolution
    // with unquoted_attribute_value (text{expr} patterns like style:attr=string{mixed})
    shorthand_attribute: $ => seq(
      '{',
      optional(field('content', choice(
        alias($._expression_js, $.js),
        alias($._expression_ts, $.ts),
      ))),
      '}',
    ),

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

    // Unquoted attribute value with embedded expressions: text{expr} or text{expr}text
    // This allows patterns like style:color=red{expr} or class=item-{type}-active
    // Uses external scanner _attribute_value to properly handle lookahead for {
    unquoted_attribute_value: $ => prec.right(seq(
      alias($._attribute_value, $.attribute_value),  // Required leading text via external scanner
      repeat1(seq(
        $.expression,
        optional(alias($._attribute_value, $.attribute_value)),  // Optional trailing text
      )),
    )),

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
