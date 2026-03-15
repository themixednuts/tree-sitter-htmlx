//! Tests for {#snippet} blocks

mod utils;
use utils::parse;

#[test]
fn test_snippet_no_params() {
    assert_eq!(
        parse("{#snippet greeting()}<h1>Hello</h1>{/snippet}"),
        "(document (snippet_block name: (snippet_name) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_snippet_with_param() {
    assert_eq!(
        parse("{#snippet button(text)}<button>{text}</button>{/snippet}"),
        "(document (snippet_block name: (snippet_name) parameters: (snippet_parameters parameter: (pattern content: (js))) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_snippet_multiple_params() {
    assert_eq!(
        parse("{#snippet item(name, value)}<li>{name}: {value}</li>{/snippet}"),
        "(document (snippet_block name: (snippet_name) parameters: (snippet_parameters parameter: (pattern content: (js)) parameter: (pattern content: (js))) (element (start_tag (tag_name)) (expression content: (js)) (text) (expression content: (js)) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_snippet_typed_params() {
    assert_eq!(
        parse("{#snippet row(item: Item, index: number)}<tr>{item.name}</tr>{/snippet}"),
        "(document (snippet_block name: (snippet_name) parameters: (snippet_parameters parameter: (pattern content: (js)) parameter: (pattern content: (js))) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_snippet_with_type_parameters() {
    assert_eq!(
        parse("{#snippet row<T>(item)}<tr>{item}</tr>{/snippet}"),
        "(document (snippet_block name: (snippet_name) type_parameters: (snippet_type_parameters) parameters: (snippet_parameters parameter: (pattern content: (js))) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_snippet_destructured_param() {
    assert_eq!(
        parse("{#snippet row({ name, value })}<td>{name}</td><td>{value}</td>{/snippet}"),
        "(document (snippet_block name: (snippet_name) parameters: (snippet_parameters parameter: (pattern content: (js))) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_snippet_with_const() {
    assert_eq!(
        parse("{#snippet item(data)}{@const doubled = data * 2}{doubled}{/snippet}"),
        "(document (snippet_block name: (snippet_name) parameters: (snippet_parameters parameter: (pattern content: (js))) (const_tag expression: (expression_value content: (js))) (expression content: (js)) (block_end)))"
    );
}

#[test]
fn test_snippet_nested_blocks() {
    assert_eq!(
        parse("{#snippet card(show)}{#if show}<div>content</div>{/if}{/snippet}"),
        "(document (snippet_block name: (snippet_name) parameters: (snippet_parameters parameter: (pattern content: (js))) (if_block expression: (expression content: (js)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end)) (block_end)))"
    );
}

#[test]
fn test_snippet_with_component() {
    assert_eq!(
        parse("{#snippet wrapper()}<Card><slot /></Card>{/snippet}"),
        "(document (snippet_block name: (snippet_name) (element (start_tag (tag_name)) (element (self_closing_tag (tag_name))) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_snippet_default_value_with_parens() {
    assert_eq!(
        parse("{#snippet foo(bar = baz())}<p>x</p>{/snippet}"),
        "(document (snippet_block name: (snippet_name) parameters: (snippet_parameters parameter: (pattern content: (js))) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end)))"
    );
}
