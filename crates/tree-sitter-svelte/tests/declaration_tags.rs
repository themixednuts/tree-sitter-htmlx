//! Tests for Svelte 5.56 declaration tags: {let ...} and {const ...}

mod utils;
use utils::parse;

#[test]
fn test_const_declaration_tag() {
    let tree = parse("{const area = box.width * box.height}{area}");

    assert!(
        tree.contains("(declaration_tag (block_open) kind: (declaration_kind) declaration: (expression_value content: (js)) (block_close))"),
        "{tree}"
    );
    assert!(tree.contains("(expression content: (js))"), "{tree}");
}

#[test]
fn test_let_declaration_tag() {
    let tree = parse("{#if editing}{let name = $state(user.name)}{name}{/if}");

    assert!(
        tree.contains("(declaration_tag (block_open) kind: (declaration_kind) declaration: (expression_value content: (js)) (block_close))"),
        "{tree}"
    );
    assert!(tree.contains("(if_block"), "{tree}");
}

#[test]
fn test_declaration_tag_allows_whitespace_after_open_brace() {
    let tree = parse("{ const label = `${width} x ${height}` }");

    assert!(
        tree.contains("(declaration_tag (block_open) kind: (declaration_kind) declaration: (expression_value content: (js)) (block_close))"),
        "{tree}"
    );
}

#[test]
fn test_declaration_tag_destructuring_and_multiple_declarators() {
    let tree = parse("{#each items as item}{const { a, b } = item, c = a + b}{c}{/each}");

    assert!(tree.contains("(declaration_tag"), "{tree}");
    assert!(tree.contains("binding: (pattern content: (js))"), "{tree}");
}

#[test]
fn test_declaration_tag_uses_typescript_content_in_ts_components() {
    let tree = parse("<script lang=\"ts\"></script>{const value: number = 1}");

    assert!(
        tree.contains("declaration: (expression_value content: (ts))"),
        "{tree}"
    );
}

#[test]
fn test_declaration_tag_incomplete_recovery_stays_typed() {
    for source in ["{let }", "{const}"] {
        let tree = parse(source);
        assert!(tree.contains("(declaration_tag"), "{tree}");
        assert!(!tree.contains("(ERROR"), "{tree}");
    }
}

#[test]
fn test_declaration_tag_requires_keyword_boundary() {
    let tree = parse("{constfoo = 'bar'}");

    assert!(!tree.contains("(declaration_tag"), "{tree}");
    assert!(tree.contains("(expression content: (js))"), "{tree}");
}

#[test]
fn test_declaration_tag_is_not_rich_attribute_content() {
    let tree = parse("<div title=\"{let x = 1}\"></div>");

    assert!(!tree.contains("(declaration_tag"), "{tree}");
    assert!(tree.contains("(expression content: (js))"), "{tree}");
}

#[test]
fn test_declaration_tag_reports_top_level_semicolon_as_error() {
    let tree = parse("{let x = 1;}");

    assert!(tree.contains("(declaration_tag"), "{tree}");
    assert!(tree.contains("(ERROR (UNEXPECTED ';'))"), "{tree}");
}
