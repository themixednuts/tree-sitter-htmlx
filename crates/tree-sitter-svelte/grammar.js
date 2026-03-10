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
      $._snippet_name, // snippet identifier or zero-width when absent
      // Note: _ts_lang_attr, _expression_js, _expression_ts are inherited from HTMLX
    ]),

  conflicts: ($, original) => (original || []).concat([
    [$._node, $._node_in_unclosed_block],
    [$.await_pending],
    [$.await_branch_children],
    [$.await_block],
    [$._await_recovery_continuation, $.await_block],
  ]),

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
        $.malformed_block,
        prec(-2, $.orphan_branch),
        prec(-1, $.expression),
      ),

    _node_without_orphan_branch: ($) =>
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
        $.malformed_block,
        prec(-1, $.expression),
      ),

    _await_recovery_continuation: ($) => choice($.await_branch, $.orphan_branch),

    // { #if ...} or { @html ...} — space between { and sigil.
    // Produces a typed node so the compiler can detect the pattern without string matching.
    malformed_block: ($) =>
      seq(
        "{",
        /\s+/,
        field(
          "kind",
          alias(token(seq(/[#:@]/, /[a-z]+/)), $.block_sigil),
        ),
        /[^}]*/,
        "}",
      ),

    // A branch continuation outside the block grammar is still preserved as a typed
    // CST node so the compiler can diagnose placement without raw ERROR text scans.
    orphan_branch: ($) =>
      prec.left(
        -2,
        choice(
          seq(
            token("{:"),
            field("kind", alias(token(seq("else", /\s+/, "if")), $.branch_kind)),
            optional(
              field("expression", alias($._tag_expression, $.expression_value)),
            ),
            "}",
          ),
          seq(
            token("{:"),
            field("kind", alias("else", $.branch_kind)),
            "}",
          ),
          seq(
            token("{:"),
            field("kind", alias(choice("then", "catch"), $.branch_kind)),
            optional(field("binding", alias($._binding_pattern, $.pattern))),
            "}",
          ),
        ),
      ),

    element: ($, original) => original,

    // Node choice excluding erroneous_end_tag — used in unclosed block recovery
    // to prevent the recovery body from consuming end tags that close parent elements.
    _node_in_unclosed_block: ($) =>
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
        // erroneous_end_tag intentionally excluded
        prec(-1, $.expression),
      ),

    // =========================================================================
    // Block end helper (shared by all typed blocks)
    // =========================================================================

    _block_end: ($) => seq($._block_end_open, $._block_end_keyword, "}"),

    _block_end_keyword: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    // Typed block ends — each block type uses its own keyword for proper matching.
    // This prevents {/if} from closing a key_block, etc.
    _if_block_end: ($) => seq($._block_end_open, "if", "}"),
    _each_block_end: ($) => seq($._block_end_open, "each", "}"),
    _await_block_end: ($) => seq($._block_end_open, "await", "}"),
    _key_block_end: ($) => seq($._block_end_open, "key", "}"),
    _snippet_block_end: ($) => seq($._block_end_open, "snippet", "}"),

    // =========================================================================
    // Recovery helper
    // =========================================================================

    _block_recovery_ws: ($) => /[ \t\r\n]+/,

    // =========================================================================
    // {#if expr}...({:else if expr}...)*({:else}...)?{/if}
    // =========================================================================

    if_block: ($) =>
      choice(
        prec.dynamic(
          1,
          seq(
            $._if_block_start,
            repeat($._node),
            repeat($.else_if_clause),
            optional($.else_clause),
            alias($._if_block_end, $.block_end),
          ),
        ),
        // Recovery: allow one trailing unclosed element start tag before block end
        prec.dynamic(
          1,
          seq(
            $._if_block_start,
            repeat($._node),
            $.start_tag,
            optional($._block_recovery_ws),
            alias($._if_block_end, $.block_end),
          ),
        ),
        // Recovery: unclosed block (no block_end) — strongly disfavored at runtime
        prec.dynamic(-10, prec(-2, seq($._if_block_start, repeat($._node_in_unclosed_block)))),
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
          repeat($._node_without_orphan_branch),
        ),
      ),

    else_clause: ($) =>
      prec.right(
        seq(token("{:"), "else", "}", repeat($._node_without_orphan_branch)),
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
          alias($._each_block_end, $.block_end),
        ),
        // Recovery: allow one trailing unclosed element start tag before block end
        prec(
          -1,
          seq(
            $._each_block_start,
            repeat($._node),
            $.start_tag,
            optional($._block_recovery_ws),
            alias($._each_block_end, $.block_end),
          ),
        ),
        // Recovery: unclosed block (no block_end)
        prec.dynamic(-10, prec(-2, seq($._each_block_start, repeat($._node_in_unclosed_block)))),
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
          alias($._await_block_end, $.block_end),
        ),
        // Shorthand: {#await expr then v}...{:catch e}...{/await}
        seq(
          $._await_block_start_shorthand,
          optional(field("shorthand_children", $.await_branch_children)),
          repeat($.await_branch),
          alias($._await_block_end, $.block_end),
        ),
        // Recovery: unclosed plain await
        prec.dynamic(
          -10,
          prec(
            -2,
            seq(
              $._await_block_start_plain,
              optional(field("pending", $.await_pending)),
              repeat($._await_recovery_continuation),
            ),
          ),
        ),
        // Recovery: unclosed shorthand await
        prec.dynamic(
          -10,
          prec(
            -2,
            seq(
              $._await_block_start_shorthand,
              optional(field("shorthand_children", $.await_branch_children)),
              repeat($._await_recovery_continuation),
            ),
          ),
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

    await_pending: ($) => repeat1($._node_without_orphan_branch),

    await_branch_children: ($) => repeat1($._node_without_orphan_branch),

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
          alias($._key_block_end, $.block_end),
        ),
        // Recovery: allow one trailing unclosed element start tag before block end
        prec(
          -1,
          seq(
            $._key_block_start,
            repeat($._node),
            $.start_tag,
            optional($._block_recovery_ws),
            alias($._key_block_end, $.block_end),
          ),
        ),
        // Recovery: unclosed block (no block_end)
        prec.dynamic(-10, prec(-2, seq($._key_block_start, repeat($._node_in_unclosed_block)))),
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
      choice(
        seq(
          $._snippet_block_start,
          repeat($._node),
          alias($._snippet_block_end, $.block_end),
        ),
        // Recovery: unclosed block (no block_end)
        prec.dynamic(-10, prec(-2, seq($._snippet_block_start, repeat($._node_in_unclosed_block)))),
      ),

    _snippet_block_start: ($) =>
      choice(
        seq(
          token("{#"),
          "snippet",
          optional(
            field("name", alias($._snippet_name, $.snippet_name)),
          ),
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
            field("name", alias($._snippet_name, $.snippet_name)),
            optional(field("type_parameters", $.snippet_type_parameters)),
            "(",
            optional(field("parameters", $.snippet_parameters)),
            "}",
          ),
        ),
      ),
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
    // We don't override it here - invalid Svelte-specific tag/block placement is
    // represented by allowing existing typed nodes in quoted attribute content.

    // Attributes - extend HTMLX to include attach_tag for {@attach ...}
    attribute: ($, original) => choice(original, prec(2, $.attach_tag)),

    _quoted_attribute_content_single: ($) =>
      choice(
        $.html_tag,
        $.if_block,
        $.each_block,
        $.await_block,
        $.key_block,
        $.snippet_block,
        alias($.attribute_expression, $.expression),
        alias(/[^'{]+/, $.attribute_value),
      ),

    _quoted_attribute_content_double: ($) =>
      choice(
        $.html_tag,
        $.if_block,
        $.each_block,
        $.await_block,
        $.key_block,
        $.snippet_block,
        alias($.attribute_expression, $.expression),
        alias(/[^"{]+/, $.attribute_value),
      ),
  },
});
