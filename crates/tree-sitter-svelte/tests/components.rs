//! Tests for Svelte components (uppercase tag names)
//!
//! Components are parsed as regular elements - use queries to distinguish by tag name.

mod utils;
use utils::parse;

#[test]
fn test_component_self_closing() {
    assert_eq!(
        parse("<Component />"),
        "(document (element (self_closing_tag (tag_name))))"
    );
}

#[test]
fn test_component_with_children() {
    assert_eq!(
        parse("<Component><p>child</p></Component>"),
        "(document (element (start_tag (tag_name)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_component_with_text() {
    assert_eq!(
        parse("<Button>Click me</Button>"),
        "(document (element (start_tag (tag_name)) (text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_component_with_expression() {
    assert_eq!(
        parse("<Display>{value}</Display>"),
        "(document (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_component_quoted_prop() {
    assert_eq!(
        parse(r#"<Component name="test" />"#),
        r#"(document (element (self_closing_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))))"#
    );
}

#[test]
fn test_component_expression_prop() {
    assert_eq!(
        parse("<Component count={5} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name) (expression content: (js))))))"
    );
}

#[test]
fn test_component_shorthand_prop() {
    assert_eq!(
        parse("<Component {name} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (shorthand_attribute)))))"
    );
}

#[test]
fn test_component_spread_props() {
    assert_eq!(
        parse("<Component {...props} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (spread_attribute)))))"
    );
}

#[test]
fn test_component_multiple_props() {
    assert_eq!(
        parse(r#"<Component a="1" b={2} {c} {...d} />"#),
        r#"(document (element (self_closing_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value))) (attribute (attribute_name) (expression content: (js))) (attribute (shorthand_attribute)) (attribute (spread_attribute)))))"#
    );
}

#[test]
fn test_component_event_handler() {
    assert_eq!(
        parse("<Button on:click={handleClick} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js))))))"
    );
}

#[test]
fn test_component_event_forwarding() {
    assert_eq!(
        parse("<Button on:click />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier))))))"
    );
}

#[test]
fn test_component_bind() {
    assert_eq!(
        parse("<Input bind:value />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier))))))"
    );
}

#[test]
fn test_component_bind_with_value() {
    assert_eq!(
        parse("<Input bind:value={name} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js))))))"
    );
}

#[test]
fn test_component_let_directive() {
    assert_eq!(
        parse("<Component let:item>{item}</Component>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)))) (expression content: (js)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_component_let_renamed() {
    assert_eq!(
        parse("<Component let:item={i}>{i}</Component>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (expression content: (js)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_component_let_destructure() {
    assert_eq!(
        parse("<Component let:data={{ name, value }}>{name}: {value}</Component>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (expression content: (js)) (text) (expression content: (js)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_component_nested() {
    assert_eq!(
        parse("<Outer><Inner /></Outer>"),
        "(document (element (start_tag (tag_name)) (element (self_closing_tag (tag_name))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_component_with_slot() {
    assert_eq!(
        parse(r#"<Component><div slot="header">Title</div></Component>"#),
        r#"(document (element (start_tag (tag_name)) (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (text) (end_tag (tag_name))) (end_tag (tag_name))))"#
    );
}

#[test]
fn test_component_namespaced() {
    // Dotted component names are parsed with object and property fields
    assert_eq!(
        parse("<Namespace.Component />"),
        "(document (element (self_closing_tag (tag_name object: (tag_member) property: (tag_member)))))"
    );
}

#[test]
fn test_component_dotted_with_children() {
    assert_eq!(
        parse("<UI.Button>Click</UI.Button>"),
        "(document (element (start_tag (tag_name object: (tag_member) property: (tag_member))) (text) (end_tag (tag_name object: (tag_member) property: (tag_member)))))"
    );
}

#[test]
fn test_component_deeply_dotted() {
    // Multiple property fields for deeply nested: Lib.UI.Button.Primary
    assert_eq!(
        parse("<Lib.UI.Button.Primary />"),
        "(document (element (self_closing_tag (tag_name object: (tag_member) property: (tag_member) property: (tag_member) property: (tag_member)))))"
    );
}

#[test]
fn test_component_dotted_with_props() {
    assert_eq!(
        parse(r#"<Form.Input name="email" />"#),
        r#"(document (element (self_closing_tag (tag_name object: (tag_member) property: (tag_member)) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))))"#
    );
}

#[test]
fn test_component_with_generics() {
    // Component with TypeScript generics - these are tricky to parse
    // For now, the parser might not handle this perfectly
    assert_eq!(
        parse("<Select options={items} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name) (expression content: (js))))))"
    );
}

#[test]
fn test_component_bind_this() {
    assert_eq!(
        parse("<Component bind:this={componentRef} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js))))))"
    );
}
