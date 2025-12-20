//! Tests for HTMLX elements

mod utils;
use utils::parse;

// =============================================================================
// Basic elements
// =============================================================================

#[test]
fn test_element_simple() {
    assert_eq!(
        parse("<div></div>"),
        "(document (element (start_tag (tag_name)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_self_closing() {
    assert_eq!(
        parse("<br />"),
        "(document (element (self_closing_tag (tag_name))))"
    );
}

#[test]
fn test_element_with_text() {
    assert_eq!(
        parse("<p>Hello</p>"),
        "(document (element (start_tag (tag_name)) (text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_nested() {
    assert_eq!(
        parse("<div><span>text</span></div>"),
        "(document (element (start_tag (tag_name)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (end_tag (tag_name))))"
    );
}

// =============================================================================
// Component elements (PascalCase)
// =============================================================================

#[test]
fn test_component_element() {
    assert_eq!(
        parse("<MyComponent />"),
        "(document (element (self_closing_tag (tag_name))))"
    );
}

#[test]
fn test_component_with_children() {
    assert_eq!(
        parse("<Layout><Content /></Layout>"),
        "(document (element (start_tag (tag_name)) (element (self_closing_tag (tag_name))) (end_tag (tag_name))))"
    );
}

// =============================================================================
// Namespaced elements
// =============================================================================

#[test]
fn test_namespaced_element() {
    assert_eq!(
        parse("<svelte:head></svelte:head>"),
        "(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

#[test]
fn test_namespaced_self_closing() {
    assert_eq!(
        parse("<svelte:self />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

// =============================================================================
// Script and style elements
// =============================================================================

#[test]
fn test_script_element() {
    assert_eq!(
        parse("<script>let x = 1;</script>"),
        "(document (script_element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_style_element() {
    assert_eq!(
        parse("<style>div { color: red; }</style>"),
        "(document (style_element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}
