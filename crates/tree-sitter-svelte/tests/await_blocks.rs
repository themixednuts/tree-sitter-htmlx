//! Tests for {#await} blocks

mod utils;
use utils::parse;

#[test]
fn test_await_pending_only() {
    assert_eq!(
        parse("{#await promise}Loading...{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (text) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_then_only() {
    assert_eq!(
        parse("{#await promise}{:then value}{value}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (block_branch kind: (block_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_pending_then() {
    assert_eq!(
        parse("{#await promise}Loading{:then value}{value}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (text) (block_branch kind: (block_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_pending_then_catch() {
    assert_eq!(
        parse("{#await promise}Loading{:then value}{value}{:catch error}{error}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (text) (block_branch kind: (block_kind) expression: (expression_value)) (expression content: (js)) (block_branch kind: (block_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_then_shorthand() {
    assert_eq!(
        parse("{#await promise then value}{value}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_then_shorthand_with_catch() {
    assert_eq!(
        parse("{#await promise then value}{value}{:catch error}{error}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (expression content: (js)) (block_branch kind: (block_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_catch_shorthand() {
    assert_eq!(
        parse("{#await promise catch error}{error}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_with_elements() {
    assert_eq!(
        parse("{#await promise}<p>Loading</p>{:then data}<p>{data}</p>{:catch err}<p>{err}</p>{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_branch kind: (block_kind) expression: (expression_value)) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_branch kind: (block_kind) expression: (expression_value)) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_with_destructure() {
    assert_eq!(
        parse("{#await promise then { data, error }}{data}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_catch_destructure() {
    assert_eq!(
        parse("{#await promise catch { message }}{message}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_with_const() {
    assert_eq!(
        parse("{#await promise then data}{@const items = data.items}{#each items as item}{item}{/each}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (tag kind: (tag_kind) expression: (expression_value)) (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (expression content: (js)) (block_end kind: (block_kind))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_with_store() {
    assert_eq!(
        parse("{#await $store}{:then data}{data}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (block_branch kind: (block_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_with_slot() {
    assert_eq!(
        parse("{#await promise then value}<slot a={value}>Hello</slot>{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (element (start_tag (tag_name) (attribute (attribute_name) (expression content: (js)))) (text) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}
