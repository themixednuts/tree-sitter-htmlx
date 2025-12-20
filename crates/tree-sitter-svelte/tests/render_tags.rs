//! Tests for {@render} tags

mod utils;
use utils::parse;

#[test]
fn test_render_no_args() {
    assert_eq!(
        parse("{@render greeting()}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_render_with_arg() {
    assert_eq!(
        parse("{@render button(text)}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_render_with_string_arg() {
    assert_eq!(
        parse(r#"{@render button("Click me")}"#),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_render_multiple_args() {
    assert_eq!(
        parse("{@render item(name, value, index)}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_render_expression_arg() {
    assert_eq!(
        parse("{@render row(items[i], i)}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_render_in_each() {
    assert_eq!(
        parse("{#each items as item}{@render row(item)}{/each}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (tag kind: (tag_kind) expression: (expression_value)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_render_in_if() {
    assert_eq!(
        parse("{#if show}{@render content()}{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (tag kind: (tag_kind) expression: (expression_value)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_render_with_object_arg() {
    assert_eq!(
        parse("{@render card({ title, body })}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_render_with_spread() {
    assert_eq!(
        parse("{@render component(...props)}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}
