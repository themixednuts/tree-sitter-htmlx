//! Tests for Svelte script elements

mod utils;
use utils::parse;

// =============================================================================
// Basic script tags
// =============================================================================

#[test]
fn test_script_empty() {
    assert_eq!(
        parse("<script></script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_script_with_content() {
    assert_eq!(
        parse("<script>let x = 1;</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_script_multiline() {
    assert_eq!(
        parse("<script>\n  let x = 1;\n  let y = 2;\n</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

// =============================================================================
// Script with lang attribute
// =============================================================================

#[test]
fn test_script_lang_ts() {
    // lang="ts" is parsed as a normal attribute; scanner sets TypeScript mode via zero-width marker
    assert_eq!(
        parse(r#"<script lang="ts">let x: number = 1;</script>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag (tag_name))))"#
    );
}

#[test]
fn test_script_lang_typescript() {
    assert_eq!(
        parse(r#"<script lang="typescript">let x: number = 1;</script>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag (tag_name))))"#
    );
}

// =============================================================================
// Module script (context="module")
// =============================================================================

#[test]
fn test_script_context_module() {
    assert_eq!(
        parse(r#"<script context="module">export const x = 1;</script>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag (tag_name))))"#
    );
}

#[test]
fn test_script_module_attribute() {
    // Svelte 5 style: just "module" attribute
    assert_eq!(
        parse("<script module>export const x = 1;</script>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name))) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_script_module_with_lang() {
    // lang="ts" is parsed as a normal attribute; scanner sets TypeScript mode via zero-width marker
    assert_eq!(
        parse(r#"<script module lang="ts">export const x: number = 1;</script>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name)) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag (tag_name))))"#
    );
}

// =============================================================================
// Script with generics (Svelte 5)
// =============================================================================

#[test]
fn test_script_generics() {
    // lang="ts" is parsed as a normal attribute; scanner sets TypeScript mode via zero-width marker
    assert_eq!(
        parse(r#"<script lang="ts" generics="T">let items: T[] = [];</script>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value))) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag (tag_name))))"#
    );
}

// =============================================================================
// Script in full component context
// =============================================================================

#[test]
fn test_script_with_markup() {
    assert_eq!(
        parse("<script>let name = 'world';</script>\n<p>Hello {name}!</p>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))) (element (start_tag (tag_name)) (text) (expression content: (js)) (text) (end_tag (tag_name))))"
    );
}
