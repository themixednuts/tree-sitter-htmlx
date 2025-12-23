/**
 * Svelte 5 grammar for tree-sitter
 *
 * Extends HTMLX with blocks and tags.
 * Components and svelte:* elements are regular elements - use queries to distinguish.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const HTMLX = require('../tree-sitter-htmlx/grammar');

module.exports = grammar(HTMLX, {
  name: 'svelte',

  externals: ($, original) => original.concat([
    // Svelte-specific tokens for block syntax
    $._iterator_expression,
    $._binding_pattern,
    $._key_expression,
    $._tag_expression,
    // Note: _ts_lang_attr, _expression_js, _expression_ts are inherited from HTMLX
  ]),

  rules: {
    _node: ($, original) => choice(
      prec(2, $.block),
      prec(2, $.tag),
      original,
    ),

    // {#kind expression?}...{:kind expression?}...{/kind}
    block: $ => seq(
      $.block_start,
      repeat($._block_content),
      $.block_end,
    ),

    _block_content: $ => choice(
      $.block_branch,
      $._node,
    ),

    // {#kind expression? [as|then|catch binding [, index]] [(key)]}
    block_start: $ => seq(
      '{',
      token.immediate('#'),
      field('kind', $.block_kind),
      optional(seq(
        field('expression', alias($._iterator_expression, $.expression)),
        optional(choice(
          // {#each items as item, index (key)}
          seq(
            'as',
            field('binding', alias($._binding_pattern, $.pattern)),
            optional(seq(',', field('index', alias($._binding_pattern, $.pattern)))),
            optional(seq('(', field('key', alias($._key_expression, $.expression)), ')')),
          ),
          // {#await promise then value} or {#await promise catch error}
          seq(
            choice('then', 'catch'),
            field('binding', alias($._binding_pattern, $.pattern)),
          ),
        )),
      )),
      '}',
    ),

    // {:kind expression?}
    // Special case: {:else if expr} should have kind="else if", not kind="else" with expr="if ..."
    block_branch: $ => seq(
      '{',
      token.immediate(':'),
      field('kind', choice(
        alias(token(seq('else', /\s+/, 'if')), $.block_kind),
        $.block_kind,
      )),
      optional(field('expression', alias($._tag_expression, $.expression_value))),
      '}',
    ),

    // {/kind}
    block_end: $ => seq(
      '{',
      token.immediate('/'),
      field('kind', $.block_kind),
      '}',
    ),

    block_kind: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    // {@kind expression?}
    tag: $ => seq(
      '{',
      token.immediate('@'),
      field('kind', $.tag_kind),
      optional(field('expression', alias($._tag_expression, $.expression_value))),
      '}',
    ),

    tag_kind: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    // Generic expressions - excludes block/tag markers at start
    _expression_value: $ => /[^#:/@}\s][^}]*/,

    // Shorthand attributes - excludes block/tag markers
    shorthand_attribute: $ => /\{[^.#:/@}][^}]*\}|\{[.][^.#:/@}][^}]*\}|\{[.][.][^.#:/@}][^}]*\}|\{\}/,

    // Attributes - extend HTMLX to include tag for {@attach ...}
    attribute: ($, original) => choice(
      original,
      prec(2, $.tag),
    ),
  },
});
