//! Tests for {#each} blocks

mod utils;
use utils::parse;

#[test]
fn test_each_simple() {
    assert_eq!(
        parse("{#each items as item}{item}{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (expression content: (js)) (block_end)))"
    );
}

#[test]
fn test_each_with_element() {
    assert_eq!(
        parse("{#each items as item}<li>{item}</li>{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_each_with_index() {
    assert_eq!(
        parse("{#each items as item, i}{i}: {item}{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) index: (pattern content: (js)) (expression content: (js)) (text) (expression content: (js)) (block_end)))"
    );
}

#[test]
fn test_each_with_key() {
    assert_eq!(
        parse("{#each items as item (item.id)}{item}{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) key: (expression content: (js)) (expression content: (js)) (block_end)))"
    );
}

#[test]
fn test_each_with_index_and_key() {
    assert_eq!(
        parse("{#each items as item, i (item.id)}{i}: {item}{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) index: (pattern content: (js)) key: (expression content: (js)) (expression content: (js)) (text) (expression content: (js)) (block_end)))"
    );
}

#[test]
fn test_each_destructure() {
    assert_eq!(
        parse("{#each items as { id, name }}{id}: {name}{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (expression content: (js)) (text) (expression content: (js)) (block_end)))"
    );
}

#[test]
fn test_each_destructure_nested() {
    assert_eq!(
        parse("{#each items as { user: { name } }}{name}{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (expression content: (js)) (block_end)))"
    );
}

#[test]
fn test_each_else() {
    assert_eq!(
        parse("{#each items as item}{item}{:else}No items{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (expression content: (js)) (else_clause (text)) (block_end)))"
    );
}

#[test]
fn test_each_else_with_element() {
    assert_eq!(
        parse("{#each items as item}<p>{item}</p>{:else}<p>Empty</p>{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (else_clause (element (start_tag (tag_name)) (text) (end_tag (tag_name)))) (block_end)))"
    );
}

#[test]
fn test_each_nested() {
    assert_eq!(
        parse("{#each outer as o}{#each o.inner as i}{i}{/each}{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (expression content: (js)) (block_end)) (block_end)))"
    );
}

#[test]
fn test_each_with_const() {
    assert_eq!(
        parse("{#each boxes as box}{@const area = box.w * box.h}{area}{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (const_tag expression: (expression_value content: (js))) (expression content: (js)) (block_end)))"
    );
}

#[test]
fn test_each_with_slot() {
    assert_eq!(
        parse("{#each items as item}<slot a={item}>Hello</slot>{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (element (start_tag (tag_name) (attribute (attribute_name) (expression content: (js)))) (text) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_each_array_pattern() {
    assert_eq!(
        parse("{#each items as [first, second]}{first}{/each}"),
        "(document (each_block expression: (expression content: (js)) binding: (pattern content: (js)) (expression content: (js)) (block_end)))"
    );
}
