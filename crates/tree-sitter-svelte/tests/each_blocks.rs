//! Tests for {#each} blocks

mod utils;
use utils::parse;

#[test]
fn test_each_simple() {
    assert_eq!(
        parse("{#each items as item}{item}{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) (block_close) (expression content: (js)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_with_element() {
    assert_eq!(
        parse("{#each items as item}<li>{item}</li>{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) (block_close) (element (start_tag name: (tag_name)) (expression content: (js)) (end_tag name: (tag_name))) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_with_index() {
    assert_eq!(
        parse("{#each items as item, i}{i}: {item}{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) index: (pattern content: (js)) (block_close) (expression content: (js)) (text) (expression content: (js)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_with_key() {
    assert_eq!(
        parse("{#each items as item (item.id)}{item}{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) key: (expression content: (js)) (block_close) (expression content: (js)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_with_index_and_key() {
    assert_eq!(
        parse("{#each items as item, i (item.id)}{i}: {item}{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) index: (pattern content: (js)) key: (expression content: (js)) (block_close) (expression content: (js)) (text) (expression content: (js)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_destructure() {
    assert_eq!(
        parse("{#each items as { id, name }}{id}: {name}{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) (block_close) (expression content: (js)) (text) (expression content: (js)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_destructure_nested() {
    assert_eq!(
        parse("{#each items as { user: { name } }}{name}{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) (block_close) (expression content: (js)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_else() {
    assert_eq!(
        parse("{#each items as item}{item}{:else}No items{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) (block_close) (expression content: (js)) (else_clause (block_open) (block_close) (text)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_else_with_element() {
    assert_eq!(
        parse("{#each items as item}<p>{item}</p>{:else}<p>Empty</p>{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) (block_close) (element (start_tag name: (tag_name)) (expression content: (js)) (end_tag name: (tag_name))) (else_clause (block_open) (block_close) (element (start_tag name: (tag_name)) (text) (end_tag name: (tag_name)))) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_nested() {
    assert_eq!(
        parse("{#each outer as o}{#each o.inner as i}{i}{/each}{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) (block_close) (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) (block_close) (expression content: (js)) (block_end (block_open) (block_keyword) (block_close))) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_with_const() {
    assert_eq!(
        parse("{#each boxes as box}{@const area = box.w * box.h}{area}{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) (block_close) (const_tag (block_open) expression: (expression_value content: (js)) (block_close)) (expression content: (js)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_with_slot() {
    assert_eq!(
        parse("{#each items as item}<slot a={item}>Hello</slot>{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) (block_close) (element (start_tag name: (tag_name) (attribute name: (attribute_name) value: (expression content: (js)))) (text) (end_tag name: (tag_name))) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_each_array_pattern() {
    assert_eq!(
        parse("{#each items as [first, second]}{first}{/each}"),
        "(document (each_block (block_open) expression: (expression content: (js)) binding: (pattern content: (js)) (block_close) (expression content: (js)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}
