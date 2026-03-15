//! Tests for {@debug} tags

mod utils;
use utils::parse;

#[test]
fn test_debug_single() {
    assert_eq!(
        parse("{@debug x}"),
        "(document (debug_tag expression: (expression_value content: (js)) (block_close)))"
    );
}

#[test]
fn test_debug_multiple() {
    assert_eq!(
        parse("{@debug a, b, c}"),
        "(document (debug_tag expression: (expression_value content: (js)) (block_close)))"
    );
}

#[test]
fn test_debug_with_store() {
    assert_eq!(
        parse("{@debug $store, localVar}"),
        "(document (debug_tag expression: (expression_value content: (js)) (block_close)))"
    );
}

#[test]
fn test_debug_property_access() {
    assert_eq!(
        parse("{@debug user.name, user.email}"),
        "(document (debug_tag expression: (expression_value content: (js)) (block_close)))"
    );
}

#[test]
fn test_debug_empty() {
    assert_eq!(parse("{@debug}"), "(document (debug_tag (block_close)))");
}

#[test]
fn test_debug_with_whitespace() {
    assert_eq!(
        parse("{@debug   myfile   ,   otherFile   }"),
        "(document (debug_tag expression: (expression_value content: (js)) (block_close)))"
    );
}
