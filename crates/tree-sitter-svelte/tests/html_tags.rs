//! Tests for {@html} tags

mod utils;
use utils::parse;

/// Parse and return the expression node's byte range
fn get_expression_range(source: &str) -> (usize, usize, String) {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_svelte::LANGUAGE.into())
        .expect("Failed to load Svelte grammar");

    let tree = parser.parse(source, None).expect("Failed to parse");
    let root = tree.root_node();
    
    // Find the expression_value node
    let tag = root.child(0).expect("No tag node");
    let expr = tag.child_by_field_name("expression").expect("No expression field");
    
    let start = expr.start_byte();
    let end = expr.end_byte();
    let text = source[start..end].to_string();
    
    (start, end, text)
}

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
fn test_html_template_literal_nested() {
    assert_eq!(
        parse("{@html `${`nested ${x}`}`}"),
        "(document (tag kind: (tag_kind) expression: (expression_value)))"
    );
}

#[test]
fn test_html_template_literal_with_object() {
    assert_eq!(
        parse(r#"{@html `${ { class: "}" } }`}"#),
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

// =============================================================================
// Expression span tests - verify trailing whitespace is NOT included
// =============================================================================

#[test]
fn test_html_expression_span_no_trailing_space() {
    // Input: {@html myfile + someOtherFile }
    // Expected: expression should be "myfile + someOtherFile" (positions 7-29)
    // NOT "myfile + someOtherFile " (positions 7-30)
    let source = "{@html myfile + someOtherFile }";
    let (start, end, text) = get_expression_range(source);
    
    assert_eq!(start, 7, "Expression should start at byte 7");
    assert_eq!(end, 29, "Expression should end at byte 29 (before trailing space)");
    assert_eq!(text, "myfile + someOtherFile", "Expression should not include trailing whitespace");
}

#[test]
fn test_html_expression_span_no_space() {
    // No trailing space - should work the same
    let source = "{@html myfile + someOtherFile}";
    let (start, end, text) = get_expression_range(source);
    
    assert_eq!(start, 7);
    assert_eq!(end, 29);
    assert_eq!(text, "myfile + someOtherFile");
}

#[test]
fn test_html_expression_span_multiple_trailing_spaces() {
    // Multiple trailing spaces
    let source = "{@html content   }";
    let (start, end, text) = get_expression_range(source);
    
    assert_eq!(start, 7);
    assert_eq!(end, 14, "Expression should end at 'content', not include trailing spaces");
    assert_eq!(text, "content");
}
