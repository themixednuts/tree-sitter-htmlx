//! Tests for {@render} tags

mod utils;
use utils::parse;

#[test]
fn test_render_no_args() {
    assert_eq!(
        parse("{@render greeting()}"),
        "(document (render_tag expression: (expression_value content: (js))))"
    );
}

#[test]
fn test_render_with_arg() {
    assert_eq!(
        parse("{@render button(text)}"),
        "(document (render_tag expression: (expression_value content: (js))))"
    );
}

#[test]
fn test_render_with_string_arg() {
    assert_eq!(
        parse(r#"{@render button("Click me")}"#),
        "(document (render_tag expression: (expression_value content: (js))))"
    );
}

#[test]
fn test_render_multiple_args() {
    assert_eq!(
        parse("{@render item(name, value, index)}"),
        "(document (render_tag expression: (expression_value content: (js))))"
    );
}

#[test]
fn test_render_expression_arg() {
    assert_eq!(
        parse("{@render row(items[i], i)}"),
        "(document (render_tag expression: (expression_value content: (js))))"
    );
}

#[test]
fn test_render_in_each() {
    assert_eq!(
        parse("{#each items as item}{@render row(item)}{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (render_tag expression: (expression_value content: (js))) (block_end)))"
    );
}

#[test]
fn test_render_in_if() {
    assert_eq!(
        parse("{#if show}{@render content()}{/if}"),
        "(document (if_block expression: (expression content: (js)) (render_tag expression: (expression_value content: (js))) (block_end)))"
    );
}

#[test]
fn test_render_with_object_arg() {
    assert_eq!(
        parse("{@render card({ title, body })}"),
        "(document (render_tag expression: (expression_value content: (js))))"
    );
}

#[test]
fn test_render_with_spread() {
    assert_eq!(
        parse("{@render component(...props)}"),
        "(document (render_tag expression: (expression_value content: (js))))"
    );
}
