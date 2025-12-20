//! Tests for HTMLX expressions

mod utils;
use utils::parse;

#[test]
fn test_expression_simple() {
    assert_eq!(
        parse("{name}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_in_element() {
    assert_eq!(
        parse("<p>{message}</p>"),
        "(document (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_expression_in_text() {
    assert_eq!(
        parse("Hello {name}!"),
        "(document (text) (expression content: (js)) (text))"
    );
}

#[test]
fn test_expression_object_literal() {
    assert_eq!(
        parse("{{ key: 'value' }}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_template_literal() {
    assert_eq!(
        parse("{`Hello ${name}`}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_arrow_function() {
    assert_eq!(
        parse("{() => 'hello'}"),
        "(document (expression content: (js)))"
    );
}
