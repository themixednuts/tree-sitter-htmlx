//! Tests for Svelte special elements (svelte:self, svelte:component, svelte:window, etc.)
//!
//! Svelte special elements are parsed as regular elements with namespaced tag names.
//! Use queries to distinguish them from regular HTML elements.

mod utils;
use utils::parse;

// =============================================================================
// svelte:self
// =============================================================================

#[test]
fn test_svelte_self_basic() {
    assert_eq!(
        parse("<svelte:self />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

#[test]
fn test_svelte_self_with_props() {
    assert_eq!(
        parse("<svelte:self count={n - 1} />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (expression content: (js))))))"
    );
}

// =============================================================================
// svelte:component
// =============================================================================

#[test]
fn test_svelte_component_this() {
    assert_eq!(
        parse("<svelte:component this={Component} />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (expression content: (js))))))"
    );
}

#[test]
fn test_svelte_component_with_props() {
    assert_eq!(
        parse(r#"<svelte:component this={comp} name="test" />"#),
        r#"(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (expression content: (js))) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))))"#
    );
}

#[test]
fn test_svelte_component_with_children() {
    assert_eq!(
        parse("<svelte:component this={Layout}><p>content</p></svelte:component>"),
        "(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (expression content: (js)))) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

// =============================================================================
// svelte:element
// =============================================================================

#[test]
fn test_svelte_element_this() {
    assert_eq!(
        parse("<svelte:element this={tag} />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (expression content: (js))))))"
    );
}

#[test]
fn test_svelte_element_with_content() {
    assert_eq!(
        parse(r#"<svelte:element this="div">content</svelte:element>"#),
        r#"(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (text) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"#
    );
}

// =============================================================================
// svelte:window
// =============================================================================

#[test]
fn test_svelte_window_basic() {
    assert_eq!(
        parse("<svelte:window />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

#[test]
fn test_svelte_window_on_event() {
    assert_eq!(
        parse("<svelte:window on:keydown={handleKey} />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js)))))))"
    );
}

#[test]
fn test_svelte_window_bind_scrolly() {
    assert_eq!(
        parse("<svelte:window bind:scrollY={y} />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js)))))))"
    );
}

#[test]
fn test_svelte_window_bind_innerwidth() {
    assert_eq!(
        parse("<svelte:window bind:innerWidth={width} bind:innerHeight={height} />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js)))) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js)))))))"
    );
}

// =============================================================================
// svelte:document
// =============================================================================

#[test]
fn test_svelte_document_basic() {
    assert_eq!(
        parse("<svelte:document />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

#[test]
fn test_svelte_document_on_event() {
    assert_eq!(
        parse("<svelte:document on:visibilitychange={handleVisibility} />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js)))))))"
    );
}

// =============================================================================
// svelte:body
// =============================================================================

#[test]
fn test_svelte_body_basic() {
    assert_eq!(
        parse("<svelte:body />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

#[test]
fn test_svelte_body_on_event() {
    assert_eq!(
        parse("<svelte:body on:mouseenter={handleEnter} on:mouseleave={handleLeave} />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js)))) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js)))))))"
    );
}

// =============================================================================
// svelte:head
// =============================================================================

#[test]
fn test_svelte_head_empty() {
    assert_eq!(
        parse("<svelte:head></svelte:head>"),
        "(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

#[test]
fn test_svelte_head_with_title() {
    assert_eq!(
        parse("<svelte:head><title>Page Title</title></svelte:head>"),
        "(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name))) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

#[test]
fn test_svelte_head_with_meta() {
    assert_eq!(
        parse(r#"<svelte:head><meta name="description" content="My app" /></svelte:head>"#),
        r#"(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name))) (element (self_closing_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value))) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"#
    );
}

#[test]
fn test_svelte_head_dynamic_title() {
    assert_eq!(
        parse("<svelte:head><title>{pageTitle}</title></svelte:head>"),
        "(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name))) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

// =============================================================================
// svelte:options
// =============================================================================

#[test]
fn test_svelte_options_immutable() {
    assert_eq!(
        parse("<svelte:options immutable />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name)))))"
    );
}

#[test]
fn test_svelte_options_immutable_true() {
    assert_eq!(
        parse("<svelte:options immutable={true} />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (expression content: (js))))))"
    );
}

#[test]
fn test_svelte_options_accessors() {
    assert_eq!(
        parse("<svelte:options accessors={true} />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (expression content: (js))))))"
    );
}

#[test]
fn test_svelte_options_namespace() {
    assert_eq!(
        parse(r#"<svelte:options namespace="svg" />"#),
        r#"(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))))"#
    );
}

#[test]
#[allow(non_snake_case)]
fn test_svelte_options_customElement() {
    assert_eq!(
        parse(r#"<svelte:options customElement="my-element" />"#),
        r#"(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))))"#
    );
}

#[test]
fn test_svelte_options_runes() {
    assert_eq!(
        parse("<svelte:options runes={true} />"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (expression content: (js))))))"
    );
}

// =============================================================================
// svelte:fragment
// =============================================================================

#[test]
fn test_svelte_fragment_basic() {
    assert_eq!(
        parse("<svelte:fragment></svelte:fragment>"),
        "(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

#[test]
fn test_svelte_fragment_with_slot() {
    assert_eq!(
        parse(r#"<svelte:fragment slot="header"><h1>Title</h1></svelte:fragment>"#),
        r#"(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"#
    );
}

#[test]
fn test_svelte_fragment_let_directive() {
    assert_eq!(
        parse("<svelte:fragment let:item>{item}</svelte:fragment>"),
        "(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value))))) (expression content: (js)) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

// =============================================================================
// svelte:boundary (Svelte 5)
// =============================================================================

#[test]
fn test_svelte_boundary_basic() {
    assert_eq!(
        parse("<svelte:boundary><Child /></svelte:boundary>"),
        "(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name))) (element (self_closing_tag (tag_name))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

#[test]
fn test_svelte_boundary_onerror() {
    assert_eq!(
        parse("<svelte:boundary onerror={handleError}><Child /></svelte:boundary>"),
        "(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (expression content: (js)))) (element (self_closing_tag (tag_name))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

#[test]
fn test_svelte_boundary_failed_snippet() {
    assert_eq!(
        parse("{#snippet failed(error)}<p>Error: {error}</p>{/snippet}<svelte:boundary {failed}><Child /></svelte:boundary>"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (text) (expression content: (js)) (end_tag (tag_name))) (block_end kind: (block_kind))) (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (shorthand_attribute))) (element (self_closing_tag (tag_name))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}
