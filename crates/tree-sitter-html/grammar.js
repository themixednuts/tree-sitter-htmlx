/**
 * HTML grammar for tree-sitter
 *
 * Follows the WHATWG HTML Living Standard:
 * https://html.spec.whatwg.org/multipage/syntax.html
 *
 * Per §13.1.2, there are six kinds of elements:
 * - Void elements: area, base, br, col, embed, hr, img, input, link, meta, source, track, wbr
 * - The template element
 * - Raw text elements: script, style
 * - Escapable raw text elements: textarea, title
 * - Foreign elements: MathML/SVG
 * - Normal elements: all others
 *
 * All are represented as `element` nodes - the tag_name differentiates them.
 * Content model differs: raw text elements contain `raw_text`, normal elements contain nodes.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'html',

  extras: $ => [
    $.comment,
    /\s+/,
  ],

  externals: $ => [
    $._start_tag_name,
    $._script_start_tag_name,
    $._style_start_tag_name,
    $._textarea_start_tag_name,
    $._title_start_tag_name,
    $._end_tag_name,
    $.erroneous_end_tag_name,
    '/>',
    $._implicit_end_tag,
    $.raw_text,
    $.comment,
  ],

  rules: {
    document: $ => repeat($._node),

    // §13.1.1 - DOCTYPE
    doctype: $ => seq(
      '<!',
      alias($._doctype, 'doctype'),
      /[^>]+/,
      '>',
    ),

    _doctype: _ => /[Dd][Oo][Cc][Tt][Yy][Pp][Ee]/,

    _node: $ => choice(
      $.doctype,
      $.entity,
      $.text,
      $.element,
      $.erroneous_end_tag,
    ),

    // §13.1.2 - Elements (all kinds unified under `element`)
    //
    // The element kind is determined by the tag_name:
    // - Normal elements: contain child nodes
    // - Raw text elements (script, style): contain raw_text
    // - Escapable raw text elements (textarea, title): contain raw_text
    // - Void elements: no content (via self_closing_tag or implicit end)
    element: $ => choice(
      // Normal elements - content is parsed as nodes
      seq(
        $.start_tag,
        repeat($._node),
        choice($.end_tag, $._implicit_end_tag),
      ),
      // Raw text elements (§13.1.2) - content is unparsed raw_text
      $._raw_text_element,
      // Void elements / self-closing
      $.self_closing_tag,
    ),

    // Raw text elements: script, style, textarea, title
    // Content is raw_text (not parsed as nodes)
    _raw_text_element: $ => choice(
      seq(alias($._script_start_tag, $.start_tag), optional($.raw_text), $.end_tag),
      seq(alias($._style_start_tag, $.start_tag), optional($.raw_text), $.end_tag),
      seq(alias($._textarea_start_tag, $.start_tag), optional($.raw_text), $.end_tag),
      seq(alias($._title_start_tag, $.start_tag), optional($.raw_text), $.end_tag),
    ),

    // Start tags for different element kinds
    start_tag: $ => seq(
      '<',
      alias($._start_tag_name, $.tag_name),
      repeat($.attribute),
      '>',
    ),

    _script_start_tag: $ => seq(
      '<',
      alias($._script_start_tag_name, $.tag_name),
      repeat($.attribute),
      '>',
    ),

    _style_start_tag: $ => seq(
      '<',
      alias($._style_start_tag_name, $.tag_name),
      repeat($.attribute),
      '>',
    ),

    _textarea_start_tag: $ => seq(
      '<',
      alias($._textarea_start_tag_name, $.tag_name),
      repeat($.attribute),
      '>',
    ),

    _title_start_tag: $ => seq(
      '<',
      alias($._title_start_tag_name, $.tag_name),
      repeat($.attribute),
      '>',
    ),

    // §13.1.2 - Void elements can use self-closing syntax
    self_closing_tag: $ => seq(
      '<',
      alias($._start_tag_name, $.tag_name),
      repeat($.attribute),
      '/>',
    ),

    end_tag: $ => seq(
      '</',
      alias($._end_tag_name, $.tag_name),
      '>',
    ),

    erroneous_end_tag: $ => seq(
      '</',
      $.erroneous_end_tag_name,
      '>',
    ),

    // §13.1.2.3 - Attributes
    attribute: $ => seq(
      $.attribute_name,
      optional(seq(
        '=',
        choice(
          $.attribute_value,
          $.quoted_attribute_value,
        ),
      )),
    ),

    // Attribute names: any character except whitespace, quotes, =, <, >, /
    attribute_name: _ => /[^<>"'/=\s]+/,

    // Unquoted attribute values: any character except whitespace, quotes, =, <, >, `
    attribute_value: _ => /[^<>"'=\s`]+/,

    // Quoted attribute values
    quoted_attribute_value: $ => choice(
      seq('\'', optional(alias(/[^']+/, $.attribute_value)), '\''),
      seq('"', optional(alias(/[^"]+/, $.attribute_value)), '"'),
    ),

    // §13.1.4 - Character references (named, decimal, hex)
    entity: _ => /&(#([xX][0-9a-fA-F]{1,6}|[0-9]{1,7})|[A-Za-z][A-Za-z0-9]*);?/,

    // Text content
    text: _ => /[^<>&\s]([^<>&]*[^<>&\s])?/,
  },
});
