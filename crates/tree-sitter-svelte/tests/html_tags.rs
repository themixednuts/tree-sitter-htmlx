//! Tests for {@html} tags

mod utils;
use utils::parse;

#[test]
fn test_html_variable() {
    assert_eq!(
        parse("{@html content}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_html_string_literal() {
    assert_eq!(
        parse(r#"{@html "<b>bold</b>"}"#),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_html_template_literal() {
    assert_eq!(
        parse("{@html `<p>${text}</p>`}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_html_function_call() {
    assert_eq!(
        parse("{@html sanitize(userInput)}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_html_marked() {
    assert_eq!(
        parse("{@html marked(markdown)}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_html_complex_expression() {
    assert_eq!(
        parse("{@html post.content || '<p>No content</p>'}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_html_json_script() {
    assert_eq!(
        parse(r#"{@html `<script type="application/ld+json">${JSON.stringify(schema)}</script>`}"#),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}
