//! Tests for HTML and Svelte comments

mod utils;
use utils::parse;

// =============================================================================
// HTML comments
// =============================================================================

#[test]
fn test_html_comment_basic() {
    assert_eq!(parse("<!-- comment -->"), "(document (comment))");
}

#[test]
fn test_html_comment_empty() {
    assert_eq!(parse("<!---->"), "(document (comment))");
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
    assert_eq!(parse("<!-- {variable} -->"), "(document (comment))");
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
        "(document (if_block expression: (expression) (comment) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_comment_in_each_block() {
    assert_eq!(
        parse("{#each items as item}<!-- item comment --><li>{item}</li>{/each}"),
        "(document (each_block expression: (expression) binding: (pattern) (comment) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end)))"
    );
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_comment_with_newlines_only() {
    assert_eq!(parse("<!--\n\n\n-->"), "(document (comment))");
}

#[test]
fn test_comment_with_html_entities() {
    assert_eq!(parse("<!-- &amp; &lt; &gt; -->"), "(document (comment))");
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
    assert_eq!(
        parse("<MyComponent\n// this comment\nclass=\"myclass\" />"),
        r#"(document (element (self_closing_tag (tag_name) (tag_comment kind: (line_comment)) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))))"#
    );
}

#[test]
fn test_line_comment_between_multiline_attributes() {
    assert_eq!(
        parse("<div\n\tdata-one=\"1\"\n\t// comment\n\tdata-two=\"2\"\n></div>"),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value))) (tag_comment kind: (line_comment)) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (end_tag (tag_name))))"#
    );
}

#[test]
fn test_inline_block_comments_in_tag() {
    assert_eq!(
        parse("<span /* inline */ /* another inline */ data-one=\"1\"></span>"),
        r#"(document (element (start_tag (tag_name) (tag_comment kind: (block_comment)) (tag_comment kind: (block_comment)) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (end_tag (tag_name))))"#
    );
}
