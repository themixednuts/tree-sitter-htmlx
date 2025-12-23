//! Tests for HTMLX expressions

mod utils;
use utils::parse;

/// Parse and return the expression content node's byte range
fn get_expression_content_range(source: &str) -> Option<(usize, usize, String)> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_htmlx::LANGUAGE.into())
        .expect("Failed to load HTMLX grammar");

    let tree = parser.parse(source, None).expect("Failed to parse");
    let root = tree.root_node();
    
    // Find the expression node, then get content field
    let expr = root.child(0)?;
    let content = expr.child_by_field_name("content")?;
    
    let start = content.start_byte();
    let end = content.end_byte();
    let text = source[start..end].to_string();
    
    Some((start, end, text))
}

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

// =============================================================================
// Expression span tests - verify trailing whitespace is NOT included
// =============================================================================

#[test]
fn test_expression_span_no_trailing_space() {
    // {value } should have expression content "value", not "value "
    let source = "{value }";
    let (start, end, text) = get_expression_content_range(source).unwrap();
    
    assert_eq!(start, 1, "Expression content should start at byte 1");
    assert_eq!(end, 6, "Expression content should end at byte 6");
    assert_eq!(text, "value", "Expression should not include trailing whitespace");
}

#[test]
fn test_expression_span_multiple_trailing_spaces() {
    let source = "{foo + bar   }";
    let (start, end, text) = get_expression_content_range(source).unwrap();
    
    assert_eq!(start, 1);
    assert_eq!(end, 10);
    assert_eq!(text, "foo + bar");
}
