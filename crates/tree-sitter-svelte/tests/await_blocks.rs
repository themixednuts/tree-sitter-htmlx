//! Tests for {#await} blocks

mod utils;
use utils::parse;

/// Parse and return the expression node's byte range from a block_start
fn get_block_expression_range(source: &str) -> Option<(usize, usize, String)> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_svelte::LANGUAGE.into())
        .expect("Failed to load Svelte grammar");

    let tree = parser.parse(source, None).expect("Failed to parse");
    let root = tree.root_node();
    
    // Find block -> block_start -> expression field
    let block = root.child(0)?;
    let block_start = block.child(0)?;
    let expr = block_start.child_by_field_name("expression")?;
    
    let start = expr.start_byte();
    let end = expr.end_byte();
    let text = source[start..end].to_string();
    
    Some((start, end, text))
}

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
fn test_await_then_shorthand_no_binding() {
    // {#await promise then} - shorthand then with no binding
    // Expression should be "promise" only, not "promise then"
    assert_eq!(
        parse("{#await somePromise then}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_await_then_shorthand_no_binding_span() {
    // Verify the expression span is correct: should be "somePromise" (8-19), not "somePromise then"
    let source = "{#await somePromise then}{/await}";
    let (start, end, text) = get_block_expression_range(source).unwrap();
    
    assert_eq!(start, 8, "Expression should start at byte 8");
    assert_eq!(end, 19, "Expression should end at byte 19");
    assert_eq!(text, "somePromise", "Expression should be 'somePromise' only, not include 'then'");
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
