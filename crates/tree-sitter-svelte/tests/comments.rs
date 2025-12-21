//! Tests for HTML and Svelte comments

mod utils;
use utils::parse;

// =============================================================================
// HTML comments
// =============================================================================

#[test]
fn test_html_comment_basic() {
    assert_eq!(
        parse("<!-- comment -->"),
        "(document (comment))"
    );
}

#[test]
fn test_html_comment_empty() {
    assert_eq!(
        parse("<!---->"),
        "(document (comment))"
    );
}

#[test]
fn test_html_comment_multiline() {
    assert_eq!(
        parse("<!--\n  multi\n  line\n  comment\n-->"),
        "(document (comment))"
    );
}

#[test]
fn test_html_comment_with_dashes() {
    assert_eq!(
        parse("<!-- comment -- with dashes -->"),
        "(document (comment))"
    );
}

#[test]
fn test_html_comment_in_markup() {
    assert_eq!(
        parse("<p>text</p><!-- comment --><p>more</p>"),
        "(document (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (comment) (element (start_tag (tag_name)) (text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_html_comment_between_elements() {
    assert_eq!(
        parse("<div>\n  <!-- comment -->\n  <p>text</p>\n</div>"),
        "(document (element (start_tag (tag_name)) (text) (comment) (text) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_html_comment_multiple() {
    assert_eq!(
        parse("<!-- first --><!-- second -->"),
        "(document (comment) (comment))"
    );
}

// =============================================================================
// Comments with special content
// =============================================================================

#[test]
fn test_html_comment_with_tags() {
    assert_eq!(
        parse("<!-- <p>commented out</p> -->"),
        "(document (comment))"
    );
}

#[test]
fn test_html_comment_with_svelte_block() {
    assert_eq!(
        parse("<!-- {#if condition}hidden{/if} -->"),
        "(document (comment))"
    );
}

#[test]
fn test_html_comment_with_expression() {
    assert_eq!(
        parse("<!-- {variable} -->"),
        "(document (comment))"
    );
}

// =============================================================================
// Comments in component structure
// =============================================================================

#[test]
fn test_comment_at_top() {
    assert_eq!(
        parse("<!-- Component description -->\n<script>let x = 1;</script>"),
        "(document (comment) (text) (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_comment_between_script_and_markup() {
    assert_eq!(
        parse("<script>let x = 1;</script>\n<!-- Markup below -->\n<p>{x}</p>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))) (text) (comment) (text) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_comment_before_style() {
    assert_eq!(
        parse("<p>text</p>\n<!-- Styles -->\n<style>p { color: red; }</style>"),
        "(document (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (text) (comment) (text) (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

// =============================================================================
// Comments inside blocks
// =============================================================================

#[test]
fn test_comment_in_if_block() {
    assert_eq!(
        parse("{#if condition}<!-- visible when true --><p>yes</p>{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (comment) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_comment_in_each_block() {
    assert_eq!(
        parse("{#each items as item}<!-- item comment --><li>{item}</li>{/each}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (comment) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_comment_with_newlines_only() {
    assert_eq!(
        parse("<!--\n\n\n-->"),
        "(document (comment))"
    );
}

#[test]
fn test_comment_with_html_entities() {
    assert_eq!(
        parse("<!-- &amp; &lt; &gt; -->"),
        "(document (comment))"
    );
}

#[test]
fn test_comment_adjacent_to_expression() {
    assert_eq!(
        parse("<!-- before -->{value}<!-- after -->"),
        "(document (comment) (expression content: (js)) (comment))"
    );
}

#[test]
fn test_comment_in_attribute_context() {
    // Comment cannot appear inside an element's attributes, so this is before/after
    assert_eq!(
        parse("<!-- comment --><div id=\"test\"></div>"),
        r#"(document (comment) (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (end_tag (tag_name))))"#
    );
}
