//! Tests for {#key} blocks

mod utils;
use utils::parse;

#[test]
fn test_key_simple() {
    assert_eq!(
        parse("{#key value}<Component />{/key}"),
        "(document (key_block expression: (expression content: (js)) (block_close) (element (self_closing_tag (tag_name))) (block_end (block_close))))"
    );
}

#[test]
fn test_key_with_element() {
    assert_eq!(
        parse("{#key x}<div>content</div>{/key}"),
        "(document (key_block expression: (expression content: (js)) (block_close) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end (block_close))))"
    );
}

#[test]
fn test_key_with_expression() {
    assert_eq!(
        parse("{#key item.id}{item.name}{/key}"),
        "(document (key_block expression: (expression content: (js)) (block_close) (expression content: (js)) (block_end (block_close))))"
    );
}

#[test]
fn test_key_with_property_access() {
    assert_eq!(
        parse("{#key items.length}<List {items} />{/key}"),
        "(document (key_block expression: (expression content: (js)) (block_close) (element (self_closing_tag (tag_name) (attribute (shorthand_attribute content: (js))))) (block_end (block_close))))"
    );
}

#[test]
fn test_key_nested_content() {
    assert_eq!(
        parse("{#key id}{#if show}<p>text</p>{/if}{/key}"),
        "(document (key_block expression: (expression content: (js)) (block_close) (if_block expression: (expression content: (js)) (block_close) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end (block_close))) (block_end (block_close))))"
    );
}

#[test]
fn test_key_multiple_children() {
    assert_eq!(
        parse("{#key id}<h1>Title</h1><p>Content</p>{/key}"),
        "(document (key_block expression: (expression content: (js)) (block_close) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end (block_close))))"
    );
}
