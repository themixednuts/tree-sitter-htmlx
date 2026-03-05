/**
 * Svelte 5 grammar for tree-sitter
 *
 * Extends HTMLX with blocks and tags.
 * Components and svelte:* elements are regular elements - use queries to distinguish.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const HTMLX = require("../tree-sitter-htmlx/grammar");

module.exports = grammar(HTMLX, {
  name: "svelte",

  externals: ($, original) =>
    original.concat([
      // Svelte-specific tokens for block syntax
      $._iterator_expression,
      $._binding_pattern,
      $._key_expression,
      $._tag_expression,
      $._snippet_type_params,
      // Note: _ts_lang_attr, _expression_js, _expression_ts are inherited from HTMLX
    ]),

  conflicts: ($, original) => original || [],

  rules: {
    _node: ($) =>
      choice(
        prec(2, $.block),
        prec(2, $.tag),
        $.doctype,
        $.entity,
        $.text,
        $.element,
        $.erroneous_end_tag,
        prec(-1, $.expression),
      ),

    element: ($, original) => original,

    // {#kind expression?}...{:kind expression?}...{/kind}
    block: ($) =>
      choice(
        prec(2, $._await_block),
        seq($.block_start, repeat($._block_content), $.block_end),
        // Recovery: allow one trailing unclosed element start tag before block end.
        prec(
          -1,
          seq(
            $.block_start,
            repeat($._block_content),
            $.start_tag,
            optional($._block_recovery_ws),
            $.block_end,
          ),
        ),
      ),

    _block_recovery_ws: ($) => /[ \t\r\n]+/,

    _block_content: ($) => choice($.block_branch, $._node),

    _await_content_node: ($) =>
      choice(
        prec(2, $.block),
        prec(2, $.tag),
        $.doctype,
        $.entity,
        $.text,
        $.element,
        $.erroneous_end_tag,
        prec(-1, $.expression),
      ),

    _await_block: ($) =>
      choice(
        seq(
          alias($._await_block_start_plain, $.block_start),
          optional(field("pending", $.await_pending)),
          repeat($.await_branch),
          $.block_end,
        ),
        seq(
          alias($._await_block_start_shorthand, $.block_start),
          optional(field("shorthand_children", $.await_branch_children)),
          repeat($.await_branch),
          $.block_end,
        ),
      ),

    await_pending: ($) => prec.left(repeat1($._await_content_node)),

    await_branch_children: ($) => prec.left(repeat1($._await_content_node)),

    await_branch: ($) =>
      prec.right(
        seq(
          field("branch", alias($._await_block_branch, $.block_branch)),
          optional(field("children", $.await_branch_children)),
        ),
      ),

    _await_block_branch: ($) =>
      seq(
        token("{:"),
        field("kind", alias(choice("then", "catch"), $.block_kind)),
        optional(field("binding", alias($._binding_pattern, $.pattern))),
        "}",
      ),

    // {#kind expression? [as|then|catch binding [, index]] [(key)]}
    block_start: ($) =>
      choice(
        $._if_or_key_block_start,
        $._each_block_start,
        $._snippet_block_start,
      ),

    _if_or_key_block_start: ($) =>
      seq(
        token("{#"),
        field("kind", alias(choice("if", "key"), $.block_kind)),
        optional(
          field("expression", alias($._iterator_expression, $.expression)),
        ),
        "}",
      ),

    _each_block_start: ($) =>
      seq(
        token("{#"),
        field("kind", alias("each", $.block_kind)),
        optional(
          seq(
            field("expression", alias($._iterator_expression, $.expression)),
            optional(
              seq(
                "as",
                field("binding", alias($._binding_pattern, $.pattern)),
                optional(
                  seq(
                    ",",
                    field("index", alias($._binding_pattern, $.pattern)),
                  ),
                ),
                optional(
                  seq(
                    "(",
                    field("key", alias($._key_expression, $.expression)),
                    ")",
                  ),
                ),
              ),
            ),
          ),
        ),
        "}",
      ),

    _await_block_start_plain: ($) =>
      seq(
        token("{#"),
        field("kind", alias("await", $.block_kind)),
        optional(
          seq(
            field("expression", alias($._iterator_expression, $.expression)),
          ),
        ),
        "}",
      ),

    _await_block_start_shorthand: ($) =>
      seq(
        token("{#"),
        field("kind", alias("await", $.block_kind)),
        field("expression", alias($._iterator_expression, $.expression)),
        field("shorthand", alias(choice("then", "catch"), $.block_kind)),
        optional(field("binding", alias($._binding_pattern, $.pattern))),
        "}",
      ),

    _snippet_block_start: ($) =>
      choice(
        seq(
          token("{#"),
          field("kind", alias("snippet", $.block_kind)),
          field("name", $.snippet_name),
          optional(field("type_parameters", $.snippet_type_parameters)),
          optional(
            seq("(", optional(field("parameters", $.snippet_parameters)), ")"),
          ),
          "}",
        ),
        seq(
          token("{#"),
          field("kind", alias("snippet", $.block_kind)),
          "}",
        ),
      ),

    snippet_name: ($) => /[A-Za-z_$][A-Za-z0-9_$]*/,
    snippet_type_parameters: ($) => $._snippet_type_params,
    snippet_parameters: ($) =>
      repeat1(
        choice(
          /[^\n(){}]+/,
          seq("(", optional($.snippet_parameters), ")"),
          seq("{", optional($.snippet_destructured_parameter_content), "}"),
        ),
      ),
    snippet_destructured_parameter_content: ($) => /[^}\n]*/,

    // {:kind expression?}
    // {:kind expression?}
    // Special case: {:else if expr} should have kind="else if", not kind="else" with expr="if ..."
    block_branch: ($) =>
      choice(
        // {:else if condition}
        seq(
          token("{:"),
          field("kind", alias(token(seq("else", /\s+/, "if")), $.block_kind)),
          optional(
            field("expression", alias($._tag_expression, $.expression_value)),
          ),
          "}",
        ),
        // {:then value} / {:catch error}
        seq(
          token("{:"),
          field("kind", alias(choice("then", "catch"), $.block_kind)),
          optional(field("binding", alias($._binding_pattern, $.pattern))),
          "}",
        ),
        // {:else}
        seq(
          token("{:"),
          field("kind", alias("else", $.block_kind)),
          "}",
        ),
      ),

    // {/kind}
    block_end: ($) =>
      seq(token("{/"), field("kind", $.block_kind), "}"),

    block_kind: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    // {@kind expression?}
    tag: ($) =>
      seq(
        token("{@"),
        field("kind", $.tag_kind),
        optional(
          field("expression", alias($._tag_expression, $.expression_value)),
        ),
        "}",
      ),

    tag_kind: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    // Generic expressions - excludes block/tag markers at start
    _expression_value: ($) => /[^#:/@}\s][^}]*/,

    // Note: shorthand_attribute is inherited from HTMLX as a structured rule (not regex)
    // to allow proper precedence resolution with unquoted_attribute_value.
    // We don't override it here - the expression content already excludes block/tag markers
    // via the external scanner.

    // Attributes - extend HTMLX to include tag for {@attach ...}
    attribute: ($, original) => choice(original, prec(2, $.tag)),
  },
});
