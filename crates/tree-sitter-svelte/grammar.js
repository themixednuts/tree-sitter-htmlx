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
      // Svelte-specific tokens for block syntax (JS/TS typed pairs)
      $._iterator_expression_js,
      $._iterator_expression_ts,
      $._binding_pattern_js,
      $._binding_pattern_ts,
      $._key_expression_js,
      $._key_expression_ts,
      $._tag_expression_js,
      $._tag_expression_ts,
      $._snippet_parameter_js,
      $._snippet_parameter_ts,
      $._snippet_type_params,
      $._block_end_open, // {/ only when followed by identifier (not comment)
      $._snippet_name, // snippet identifier or zero-width when absent
      $._block_start_eof, // zero-width EOF marker for unterminated {#... block starts
      $._block_eof, // zero-width EOF marker for blocks missing their {/end}
      // Note: _ts_lang_attr, _expression_js, _expression_ts are inherited from HTMLX
    ]),

  conflicts: ($, original) => (original || []).concat([
    [$._node, $._node_in_unclosed_block],
    [$._await_recovery_continuation, $.await_block],
  ]),

  rules: {
    // Named block delimiters — two node types cover all Svelte block syntax:
    //   block_open  = {#  {:  {@  {/
    //   block_close = }
    // Queries only need: (block_open) @x  (block_close) @x
    _block_close: ($) => alias("}", $.block_close),

    // Typed content helpers — produce nodes with content: js | ts
    _iterator_expression: ($) =>
      field("content", choice(
        alias($._iterator_expression_js, $.js),
        alias($._iterator_expression_ts, $.ts),
      )),

    _key_expression: ($) =>
      field("content", choice(
        alias($._key_expression_js, $.js),
        alias($._key_expression_ts, $.ts),
      )),

    _tag_expression: ($) =>
      field("content", choice(
        alias($._tag_expression_js, $.js),
        alias($._tag_expression_ts, $.ts),
      )),

    _binding_pattern: ($) =>
      field("content", choice(
        alias($._binding_pattern_js, $.js),
        alias($._binding_pattern_ts, $.ts),
      )),

    _snippet_parameter: ($) =>
      field("content", choice(
        alias($._snippet_parameter_js, $.js),
        alias($._snippet_parameter_ts, $.ts),
      )),

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
        $._block_close,
      ),

    // A branch continuation outside the block grammar is still preserved as a typed
    // CST node so the compiler can diagnose placement without raw ERROR text scans.
    orphan_branch: ($) =>
      prec.left(
        -2,
        choice(
          seq(
            alias(token("{:"), $.block_open),
            field("kind", alias(token(seq("else", /\s+/, "if")), $.branch_kind)),
            optional(
              field("expression", alias($._tag_expression, $.expression_value)),
            ),
            $._block_close,
          ),
          seq(
            alias(token("{:"), $.block_open),
            field("kind", alias("else", $.branch_kind)),
            $._block_close,
          ),
          seq(
            alias(token("{:"), $.block_open),
            field("kind", alias(choice("then", "catch"), $.branch_kind)),
            optional(field("binding", alias($._binding_pattern, $.pattern))),
            $._block_close,
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

    _block_end: ($) => seq(alias($._block_end_open, $.block_open), $._block_end_keyword, $._block_close),

    _block_end_keyword: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    // Typed block ends — each block type uses its own keyword for proper matching.
    // This prevents {/if} from closing a key_block, etc.
    _if_block_end: ($) => seq(alias($._block_end_open, $.block_open), "if", $._block_close),
    _each_block_end: ($) => seq(alias($._block_end_open, $.block_open), "each", $._block_close),
    _await_block_end: ($) => seq(alias($._block_end_open, $.block_open), "await", $._block_close),
    _key_block_end: ($) => seq(alias($._block_end_open, $.block_open), "key", $._block_close),
    _snippet_block_end: ($) => seq(alias($._block_end_open, $.block_open), "snippet", $._block_close),

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
        prec.dynamic(
          -10,
          prec(-2, seq($._if_block_start, repeat($._node_in_unclosed_block), $._block_eof)),
        ),
        // Recovery: block start reaches EOF before closing }
        prec.dynamic(-11, prec(-3, $._if_block_start_unclosed)),
      ),

    _if_block_start: ($) =>
      seq(
        alias(token("{#"), $.block_open),
        "if",
        optional(
          field("expression", alias($._iterator_expression, $.expression)),
        ),
        $._block_close,
      ),

    _if_block_start_unclosed: ($) =>
      seq(
        alias(token("{#"), $.block_open),
        "if",
        field("expression", alias($._iterator_expression, $.expression)),
        $._block_start_eof,
      ),

    else_if_clause: ($) =>
      prec.right(
        seq(
          alias(token("{:"), $.block_open),
          alias(token(seq("else", /\s+/, "if")), $.branch_kind),
          optional(
            field("expression", alias($._tag_expression, $.expression_value)),
          ),
          $._block_close,
          repeat($._node_without_orphan_branch),
        ),
      ),

    else_clause: ($) =>
      prec.right(
        seq(alias(token("{:"), $.block_open), "else", $._block_close, repeat($._node_without_orphan_branch)),
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
        prec.dynamic(
          -10,
          prec(-2, seq($._each_block_start, repeat($._node_in_unclosed_block), $._block_eof)),
        ),
        // Recovery: block start reaches EOF before closing }
        prec.dynamic(-11, prec(-3, $._each_block_start_unclosed)),
      ),

    _each_block_start: ($) =>
      seq(
        alias(token("{#"), $.block_open),
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
        $._block_close,
      ),

    _each_block_start_unclosed: ($) =>
      seq(
        alias(token("{#"), $.block_open),
        "each",
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
        $._block_start_eof,
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
              $._block_eof,
            ),
          ),
        ),
        // Recovery: plain await start reaches EOF before closing }
        prec.dynamic(-11, prec(-3, $._await_block_start_plain_unclosed)),
        // Recovery: unclosed shorthand await
        prec.dynamic(
          -10,
          prec(
            -2,
            seq(
              $._await_block_start_shorthand,
              optional(field("shorthand_children", $.await_branch_children)),
              repeat($._await_recovery_continuation),
              $._block_eof,
            ),
          ),
        ),
      ),

    _await_block_start_plain: ($) =>
      seq(
        alias(token("{#"), $.block_open),
        "await",
        optional(
          field("expression", alias($._iterator_expression, $.expression)),
        ),
        $._block_close,
      ),

    _await_block_start_plain_unclosed: ($) =>
      seq(
        alias(token("{#"), $.block_open),
        "await",
        field("expression", alias($._iterator_expression, $.expression)),
        $._block_start_eof,
      ),

    _await_block_start_shorthand: ($) =>
      seq(
        alias(token("{#"), $.block_open),
        "await",
        field("expression", alias($._iterator_expression, $.expression)),
        field("shorthand", alias(choice("then", "catch"), $.shorthand_kind)),
        optional(field("binding", alias($._binding_pattern, $.pattern))),
        $._block_close,
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
        alias(token("{:"), $.block_open),
        field("kind", alias(choice("then", "catch"), $.branch_kind)),
        optional(field("binding", alias($._binding_pattern, $.pattern))),
        $._block_close,
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
        prec.dynamic(
          -10,
          prec(-2, seq($._key_block_start, repeat($._node_in_unclosed_block), $._block_eof)),
        ),
        // Recovery: block start reaches EOF before closing }
        prec.dynamic(-11, prec(-3, $._key_block_start_unclosed)),
      ),

    _key_block_start: ($) =>
      seq(
        alias(token("{#"), $.block_open),
        "key",
        optional(
          field("expression", alias($._iterator_expression, $.expression)),
        ),
        $._block_close,
      ),

    _key_block_start_unclosed: ($) =>
      seq(
        alias(token("{#"), $.block_open),
        "key",
        field("expression", alias($._iterator_expression, $.expression)),
        $._block_start_eof,
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
        prec.dynamic(
          -10,
          prec(-2, seq($._snippet_block_start, repeat($._node_in_unclosed_block), $._block_eof)),
        ),
        // Recovery: block start reaches EOF before closing }
        prec.dynamic(-11, prec(-3, $._snippet_block_start_unclosed)),
      ),

    _snippet_block_start: ($) =>
      choice(
        seq(
          alias(token("{#"), $.block_open),
          "snippet",
          field("name", alias($._snippet_name, $.snippet_name)),
          choice(
            prec(
              2,
              seq(
                field("type_parameters", $.snippet_type_parameters),
                "(",
                optional(field("parameters", $.snippet_parameters)),
                ")",
                $._block_close,
              ),
            ),
            prec(
              1,
              seq(
                "(",
                optional(field("parameters", $.snippet_parameters)),
                ")",
                $._block_close,
              ),
            ),
          ),
        ),
        // Recovery: snippet with ( but missing ) — e.g. {#snippet foo(a, b}
        prec(
          -1,
          seq(
            alias(token("{#"), $.block_open),
            "snippet",
            field("name", alias($._snippet_name, $.snippet_name)),
            optional(field("type_parameters", $.snippet_type_parameters)),
            "(",
            optional(field("parameters", $.snippet_parameters)),
            $._block_close,
          ),
        ),
      ),

    _snippet_block_start_unclosed: ($) =>
      seq(
        alias(token("{#"), $.block_open),
        "snippet",
        field("name", alias($._snippet_name, $.snippet_name)),
        choice(
          prec(
            2,
            seq(
              field("type_parameters", $.snippet_type_parameters),
              "(",
              optional(field("parameters", $.snippet_parameters)),
              ")",
              $._block_start_eof,
            ),
          ),
          prec(
            1,
            seq(
              "(",
              optional(field("parameters", $.snippet_parameters)),
              ")",
              $._block_start_eof,
            ),
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
        alias(token("{@"), $.block_open),
        "html",
        optional(
          field("expression", alias($._tag_expression, $.expression_value)),
        ),
        $._block_close,
      ),

    // {@debug expr?}
    debug_tag: ($) =>
      seq(
        alias(token("{@"), $.block_open),
        "debug",
        optional(
          field("expression", alias($._tag_expression, $.expression_value)),
        ),
        $._block_close,
      ),

    // {@const expr}
    const_tag: ($) =>
      seq(
        alias(token("{@"), $.block_open),
        "const",
        optional(
          field("expression", alias($._tag_expression, $.expression_value)),
        ),
        $._block_close,
      ),

    // {@render expr}
    render_tag: ($) =>
      seq(
        alias(token("{@"), $.block_open),
        "render",
        optional(
          field("expression", alias($._tag_expression, $.expression_value)),
        ),
        $._block_close,
      ),

    // {@attach expr}
    attach_tag: ($) =>
      seq(
        alias(token("{@"), $.block_open),
        "attach",
        optional(
          field("expression", alias($._tag_expression, $.expression_value)),
        ),
        $._block_close,
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
