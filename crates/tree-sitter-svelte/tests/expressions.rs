//! Tests for Svelte expressions (interpolation, reactive statements)

mod utils;
use utils::parse;

// =============================================================================
// Basic expressions
// =============================================================================

#[test]
fn test_expression_simple_variable() {
    assert_eq!(
        parse("{name}"),
        "(document (expression content: (js)))"
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
fn test_expression_multiple() {
    assert_eq!(
        parse("{a} and {b}"),
        "(document (expression content: (js)) (text) (expression content: (js)))"
    );
}

// =============================================================================
// Expression content types
// =============================================================================

#[test]
fn test_expression_number() {
    assert_eq!(
        parse("{42}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_string() {
    assert_eq!(
        parse(r#"{"hello"}"#),
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
fn test_expression_template_literal_nested() {
    assert_eq!(
        parse("{`${`nested ${deep}`}`}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_template_literal_with_braces() {
    assert_eq!(
        parse(r#"{`${ { key: "}" } }`}"#),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_template_literal_arrow_fn() {
    assert_eq!(
        parse("{`${ fn(() => { return x }) }`}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_arithmetic() {
    assert_eq!(
        parse("{a + b * c}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_ternary() {
    assert_eq!(
        parse("{isValid ? 'yes' : 'no'}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_function_call() {
    assert_eq!(
        parse("{formatDate(date)}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_method_chain() {
    assert_eq!(
        parse("{items.filter(x => x > 0).map(x => x * 2)}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_object_property() {
    assert_eq!(
        parse("{user.name}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_array_access() {
    assert_eq!(
        parse("{items[0]}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_optional_chaining() {
    assert_eq!(
        parse("{user?.address?.city}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_nullish_coalescing() {
    assert_eq!(
        parse("{value ?? 'default'}"),
        "(document (expression content: (js)))"
    );
}

// =============================================================================
// Expressions in element context
// =============================================================================

#[test]
fn test_expression_in_element() {
    assert_eq!(
        parse("<p>{message}</p>"),
        "(document (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_expression_mixed_with_text() {
    assert_eq!(
        parse("<p>Hello {name}, you have {count} messages</p>"),
        "(document (element (start_tag (tag_name)) (text) (expression content: (js)) (text) (expression content: (js)) (text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_expression_nested_elements() {
    assert_eq!(
        parse("<div><span>{a}</span><span>{b}</span></div>"),
        "(document (element (start_tag (tag_name)) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (end_tag (tag_name))))"
    );
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_expression_with_braces_in_string() {
    assert_eq!(
        parse(r#"{'{}'}"#),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_with_object_literal() {
    assert_eq!(
        parse("{{ key: 'value' }}"),
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

#[test]
fn test_expression_array_literal() {
    assert_eq!(
        parse("{[1, 2, 3]}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_logical_operators() {
    assert_eq!(
        parse("{a && b || c}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_comparison() {
    assert_eq!(
        parse("{a > b && c <= d}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_spread_array() {
    assert_eq!(
        parse("{[...items, newItem]}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_spread_object() {
    assert_eq!(
        parse("{{ ...obj, newKey: 'value' }}"),
        "(document (expression content: (js)))"
    );
}

// =============================================================================
// Empty and whitespace
// =============================================================================

#[test]
fn test_expression_empty_string() {
    assert_eq!(
        parse("{''}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_with_whitespace() {
    assert_eq!(
        parse("{ value }"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_expression_multiline() {
    assert_eq!(
        parse("{\n  value\n}"),
        "(document (expression content: (js)))"
    );
}

// =============================================================================
// Reactive statements ($:) - Svelte 4
// =============================================================================

#[test]
fn test_reactive_statement_in_script() {
    assert_eq!(
        parse("<script>$: doubled = count * 2;</script>"),
        "(document (script_element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_reactive_block_in_script() {
    assert_eq!(
        parse("<script>$: { console.log(count); updateUI(); }</script>"),
        "(document (script_element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

// =============================================================================
// Store subscriptions ($store)
// =============================================================================

#[test]
fn test_store_subscription() {
    assert_eq!(
        parse("{$count}"),
        "(document (expression content: (js)))"
    );
}

#[test]
fn test_store_subscription_in_element() {
    assert_eq!(
        parse("<p>Count: {$count}</p>"),
        "(document (element (start_tag (tag_name)) (text) (expression content: (js)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_store_property_access() {
    assert_eq!(
        parse("{$user.name}"),
        "(document (expression content: (js)))"
    );
}
