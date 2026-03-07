/**
 * Svelte 5 grammar for tree-sitter
 *
 * Extends HTMLX with typed blocks and tags.
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
      $._snippet_parameter,
      $._snippet_type_params,
      $._block_end_open, // {/ only when followed by identifier (not comment)
      // Note: _ts_lang_attr, _expression_js, _expression_ts are inherited from HTMLX
    ]),

  conflicts: ($, original) => original || [],

  rules: {
    _node: ($) =>
      choice(
        prec(2, $.if_block),
        prec(2, $.each_block),
        prec(2, $.await_block),
        prec(2, $.key_block),
        prec(2, $.snippet_block),
        prec(2, $.html_tag),
        prec(2, $.debug_tag),
        prec(2, $.const_tag),
        prec(2, $.render_tag),
        prec(2, $.attach_tag),
        $.doctype,
        $.entity,
        $.text,
        $.element,
        $.erroneous_end_tag,
        prec(-1, $.expression),
      ),

    element: ($, original) => original,

    // =========================================================================
    // Block end helper (shared by all typed blocks)
    // =========================================================================

    _block_end: ($) => seq($._block_end_open, $._block_end_keyword, "}"),

    _block_end_keyword: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    // =========================================================================
    // Recovery helper
    // =========================================================================

    _block_recovery_ws: ($) => /[ \t\r\n]+/,

    // =========================================================================
    // {#if expr}...({:else if expr}...)*({:else}...)?{/if}
    // =========================================================================

    if_block: ($) =>
      choice(
        seq(
          $._if_block_start,
          repeat($._node),
          repeat($.else_if_clause),
          optional($.else_clause),
          alias($._block_end, $.block_end),
        ),
        // Recovery: allow one trailing unclosed element start tag before block end
        prec(
          -1,
          seq(
            $._if_block_start,
            repeat($._node),
            $.start_tag,
            optional($._block_recovery_ws),
            alias($._block_end, $.block_end),
          ),
        ),
      ),

    _if_block_start: ($) =>
      seq(
        token("{#"),
        "if",
        optional(
          field("expression", alias($._iterator_expression, $.expression)),
        ),
        "}",
      ),

    else_if_clause: ($) =>
      prec.right(
        seq(
          token("{:"),
          token(seq("else", /\s+/, "if")),
          optional(
            field("expression", alias($._tag_expression, $.expression_value)),
          ),
          "}",
          repeat($._node),
        ),
      ),

    else_clause: ($) =>
      prec.right(
        seq(token("{:"), "else", "}", repeat($._node)),
      ),

    // =========================================================================
    // {#each expr as pat, idx (key)}...({:else}...)?{/each}
    // =========================================================================

    each_block: ($) =>
      choice(
        seq(
          $._each_block_start,
          repeat($._node),
          optional($.else_clause),
          alias($._block_end, $.block_end),
        ),
        // Recovery: allow one trailing unclosed element start tag before block end
        prec(
          -1,
          seq(
            $._each_block_start,
            repeat($._node),
            $.start_tag,
            optional($._block_recovery_ws),
            alias($._block_end, $.block_end),
          ),
        ),
      ),

    _each_block_start: ($) =>
      seq(
        token("{#"),
        "each",
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

    // =========================================================================
    // {#await expr}...({:then pat}...)?({:catch pat}...)?{/await}
    // + shorthand: {#await expr then pat}...{/await}
    // =========================================================================

    await_block: ($) =>
      choice(
        // Plain: {#await expr}pending{:then v}...{:catch e}...{/await}
        seq(
          $._await_block_start_plain,
          optional(field("pending", $.await_pending)),
          repeat($.await_branch),
          alias($._block_end, $.block_end),
        ),
        // Shorthand: {#await expr then v}...{:catch e}...{/await}
        seq(
          $._await_block_start_shorthand,
          optional(field("shorthand_children", $.await_branch_children)),
          repeat($.await_branch),
          alias($._block_end, $.block_end),
        ),
      ),

    _await_block_start_plain: ($) =>
      seq(
        token("{#"),
        "await",
        optional(
          field("expression", alias($._iterator_expression, $.expression)),
        ),
        "}",
      ),

    _await_block_start_shorthand: ($) =>
      seq(
        token("{#"),
        "await",
        field("expression", alias($._iterator_expression, $.expression)),
        field("shorthand", alias(choice("then", "catch"), $.shorthand_kind)),
        optional(field("binding", alias($._binding_pattern, $.pattern))),
        "}",
      ),

    await_pending: ($) => prec.left(repeat1($._node)),

    await_branch_children: ($) => prec.left(repeat1($._node)),

    await_branch: ($) =>
      prec.right(
        seq(
          field("branch", $._await_branch_header),
          optional(field("children", $.await_branch_children)),
        ),
      ),

    _await_branch_header: ($) =>
      seq(
        token("{:"),
        field("kind", alias(choice("then", "catch"), $.branch_kind)),
        optional(field("binding", alias($._binding_pattern, $.pattern))),
        "}",
      ),

    // =========================================================================
    // {#key expr}...{/key}
    // =========================================================================

    key_block: ($) =>
      choice(
        seq(
          $._key_block_start,
          repeat($._node),
          alias($._block_end, $.block_end),
        ),
        // Recovery: allow one trailing unclosed element start tag before block end
        prec(
          -1,
          seq(
            $._key_block_start,
            repeat($._node),
            $.start_tag,
            optional($._block_recovery_ws),
            alias($._block_end, $.block_end),
          ),
        ),
      ),

    _key_block_start: ($) =>
      seq(
        token("{#"),
        "key",
        optional(
          field("expression", alias($._iterator_expression, $.expression)),
        ),
        "}",
      ),

    // =========================================================================
    // {#snippet name(params)}...{/snippet}
    // =========================================================================

    snippet_block: ($) =>
      seq(
        $._snippet_block_start,
        repeat($._node),
        alias($._block_end, $.block_end),
      ),

    _snippet_block_start: ($) =>
      choice(
        seq(
          token("{#"),
          "snippet",
          field("name", $.snippet_name),
          optional(field("type_parameters", $.snippet_type_parameters)),
          optional(
            seq("(", optional(field("parameters", $.snippet_parameters)), ")"),
          ),
          "}",
        ),
        // Recovery: snippet with ( but missing ) — e.g. {#snippet foo(a, b}
        prec(
          -1,
          seq(
            token("{#"),
            "snippet",
            field("name", $.snippet_name),
            optional(field("type_parameters", $.snippet_type_parameters)),
            "(",
            optional(field("parameters", $.snippet_parameters)),
            "}",
          ),
        ),
        seq(
          token("{#"),
          "snippet",
          "}",
        ),
      ),

    snippet_name: ($) => /[A-Za-z_$][A-Za-z0-9_$]*/,
    snippet_type_parameters: ($) => $._snippet_type_params,
    snippet_parameters: ($) =>
      seq(
        field("parameter", alias($._snippet_parameter, $.pattern)),
        repeat(seq(",", field("parameter", alias($._snippet_parameter, $.pattern)))),
        optional(","),
      ),

    // =========================================================================
    // Typed tags
    // =========================================================================

    // {@html expr}
    html_tag: ($) =>
      seq(
        token("{@"),
        "html",
        optional(
          field("expression", alias($._tag_expression, $.expression_value)),
        ),
        "}",
      ),

    // {@debug expr?}
    debug_tag: ($) =>
      seq(
        token("{@"),
        "debug",
        optional(
          field("expression", alias($._tag_expression, $.expression_value)),
        ),
        "}",
      ),

    // {@const expr}
    const_tag: ($) =>
      seq(
        token("{@"),
        "const",
        optional(
          field("expression", alias($._tag_expression, $.expression_value)),
        ),
        "}",
      ),

    // {@render expr}
    render_tag: ($) =>
      seq(
        token("{@"),
        "render",
        optional(
          field("expression", alias($._tag_expression, $.expression_value)),
        ),
        "}",
      ),

    // {@attach expr}
    attach_tag: ($) =>
      seq(
        token("{@"),
        "attach",
        optional(
          field("expression", alias($._tag_expression, $.expression_value)),
        ),
        "}",
      ),

    // Generic expressions - excludes block/tag markers at start
    _expression_value: ($) => /[^#:/@}\s][^}]*/,

    // Note: shorthand_attribute is inherited from HTMLX as a structured rule (not regex)
    // to allow proper precedence resolution with unquoted_attribute_value.
    // We don't override it here - the expression content already excludes block/tag markers
    // via the external scanner.

    // Attributes - extend HTMLX to include attach_tag for {@attach ...}
    attribute: ($, original) => choice(original, prec(2, $.attach_tag)),
  },
});
