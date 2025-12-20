//! Tests for {@debug} tags

mod utils;
use utils::parse;

#[test]
fn test_debug_single() {
    assert_eq!(
        parse("{@debug x}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_debug_multiple() {
    assert_eq!(
        parse("{@debug a, b, c}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_debug_with_store() {
    assert_eq!(
        parse("{@debug $store, localVar}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_debug_property_access() {
    assert_eq!(
        parse("{@debug user.name, user.email}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_debug_empty() {
    assert_eq!(
        parse("{@debug}"),
        "(document (tag kind: (tag_kind)))"
    );
}

#[test]
fn test_debug_with_whitespace() {
    assert_eq!(
        parse("{@debug   myfile   ,   otherFile   }"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}
