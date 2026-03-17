//! Tests for Svelte style elements

mod utils;
use utils::parse;

// =============================================================================
// Basic style tags
// =============================================================================

#[test]
fn test_style_empty() {
    assert_eq!(
        parse("<style></style>"),
        "(document (element (start_tag name: (tag_name)) (raw_text) (end_tag name: (tag_name))))"
    );
}

#[test]
fn test_style_with_content() {
    assert_eq!(
        parse("<style>p { color: red; }</style>"),
        "(document (element (start_tag name: (tag_name)) (raw_text) (end_tag name: (tag_name))))"
    );
}

#[test]
fn test_style_multiline() {
    assert_eq!(
        parse("<style>\n  p {\n    color: red;\n  }\n</style>"),
        "(document (element (start_tag name: (tag_name)) (raw_text) (end_tag name: (tag_name))))"
    );
}

// =============================================================================
// Style with lang attribute
// =============================================================================

#[test]
fn test_style_lang_scss() {
    assert_eq!(
        parse(r#"<style lang="scss">$color: red; p { color: $color; }</style>"#),
        r#"(document (element (start_tag name: (tag_name) (attribute name: (attribute_name) value: (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag name: (tag_name))))"#
    );
}

#[test]
fn test_style_lang_less() {
    assert_eq!(
        parse(r#"<style lang="less">@color: red; p { color: @color; }</style>"#),
        r#"(document (element (start_tag name: (tag_name) (attribute name: (attribute_name) value: (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag name: (tag_name))))"#
    );
}

// =============================================================================
// Global styles
// =============================================================================

#[test]
fn test_style_global_attribute() {
    assert_eq!(
        parse("<style global>p { color: red; }</style>"),
        "(document (element (start_tag name: (tag_name) (attribute name: (attribute_name))) (raw_text) (end_tag name: (tag_name))))"
    );
}

#[test]
fn test_style_global_with_lang() {
    assert_eq!(
        parse(r#"<style global lang="scss">p { color: red; }</style>"#),
        r#"(document (element (start_tag name: (tag_name) (attribute name: (attribute_name)) (attribute name: (attribute_name) value: (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag name: (tag_name))))"#
    );
}

// =============================================================================
// Style with special CSS content
// =============================================================================

#[test]
fn test_style_with_global_selector() {
    assert_eq!(
        parse("<style>:global(body) { margin: 0; }</style>"),
        "(document (element (start_tag name: (tag_name)) (raw_text) (end_tag name: (tag_name))))"
    );
}

#[test]
fn test_style_with_nested_braces() {
    assert_eq!(
        parse("<style>@media (min-width: 600px) { p { color: red; } }</style>"),
        "(document (element (start_tag name: (tag_name)) (raw_text) (end_tag name: (tag_name))))"
    );
}

#[test]
fn test_style_with_at_rules() {
    assert_eq!(
        parse("<style>@keyframes fade { from { opacity: 0; } to { opacity: 1; } }</style>"),
        "(document (element (start_tag name: (tag_name)) (raw_text) (end_tag name: (tag_name))))"
    );
}
