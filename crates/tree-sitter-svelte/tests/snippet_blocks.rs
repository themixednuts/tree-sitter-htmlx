//! Tests for {#snippet} blocks

mod utils;
use utils::parse;

#[test]
fn test_snippet_no_params() {
    assert_eq!(
        parse("{#snippet greeting()}<h1>Hello</h1>{/snippet}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_snippet_with_param() {
    assert_eq!(
        parse("{#snippet button(text)}<button>{text}</button>{/snippet}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_snippet_multiple_params() {
    assert_eq!(
        parse("{#snippet item(name, value)}<li>{name}: {value}</li>{/snippet}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (expression content: (js)) (text) (expression content: (js)) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_snippet_typed_params() {
    assert_eq!(
        parse("{#snippet row(item: Item, index: number)}<tr>{item.name}</tr>{/snippet}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_snippet_destructured_param() {
    assert_eq!(
        parse("{#snippet row({ name, value })}<td>{name}</td><td>{value}</td>{/snippet}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_snippet_with_const() {
    assert_eq!(
        parse("{#snippet item(data)}{@const doubled = data * 2}{doubled}{/snippet}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (tag kind: (tag_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_snippet_nested_blocks() {
    assert_eq!(
        parse("{#snippet card(show)}{#if show}<div>content</div>{/if}{/snippet}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end kind: (block_kind))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_snippet_with_component() {
    assert_eq!(
        parse("{#snippet wrapper()}<Card><slot /></Card>{/snippet}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (element (self_closing_tag (tag_name))) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_snippet_default_value_with_parens() {
    assert_eq!(
        parse("{#snippet foo(bar = baz())}<p>x</p>{/snippet}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}
