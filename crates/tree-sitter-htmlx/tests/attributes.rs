//! Tests for HTMLX attributes

mod utils;
use utils::parse;

// =============================================================================
// Standard attributes
// =============================================================================

#[test]
fn test_attribute_standard() {
    assert_eq!(
        parse(r#"<div class="foo"></div>"#),
        "(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_attribute_expression_value() {
    assert_eq!(
        parse("<input value={text} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name) (expression content: (js))))))"
    );
}

#[test]
fn test_tag_line_comment_between_attributes() {
    assert_eq!(
        parse("<div // comment\n class=\"x\" />"),
        "(document (element (self_closing_tag (tag_name) (tag_comment kind: (line_comment)) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))))"
    );
}

#[test]
fn test_tag_block_comment_between_attributes() {
    assert_eq!(
        parse("<span /* inline */ data-one=\"1\"></span>"),
        "(document (element (start_tag (tag_name) (tag_comment kind: (block_comment)) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_tag_comment_in_namespaced_tag() {
    assert_eq!(
        parse("<svelte:head // note\n data-x=\"1\"></svelte:head>"),
        "(document (element (start_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (tag_comment kind: (line_comment)) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (end_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)))))"
    );
}

#[test]
fn test_tag_comment_in_member_tag() {
    assert_eq!(
        parse("<UI.Button /* note */ data-x=\"1\" />"),
        "(document (element (self_closing_tag (tag_name object: (tag_member) property: (tag_member)) (tag_comment kind: (block_comment)) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))))"
    );
}

#[test]
fn test_tag_line_comment_stops_before_tag_close() {
    assert_eq!(
        parse("<div // note></div>"),
        "(document (element (start_tag (tag_name) (tag_comment kind: (line_comment))) (end_tag (tag_name))))"
    );
}

// =============================================================================
// Shorthand attributes
// =============================================================================

#[test]
fn test_shorthand_attribute() {
    // shorthand_attribute now uses expression structure with content field
    assert_eq!(
        parse("<div {hidden}></div>"),
        "(document (element (start_tag (tag_name) (attribute (shorthand_attribute content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_shorthand_attribute_multiple() {
    assert_eq!(
        parse("<div {id} {hidden}></div>"),
        "(document (element (start_tag (tag_name) (attribute (shorthand_attribute content: (js))) (attribute (shorthand_attribute content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_multiline_shorthand_attribute() {
    assert_eq!(
        parse("<div\n {hidden}\n></div>"),
        "(document (element (start_tag (tag_name) (attribute (shorthand_attribute content: (js)))) (end_tag (tag_name))))"
    );
}

// =============================================================================
// Spread attributes
// =============================================================================

#[test]
fn test_spread_attribute() {
    assert_eq!(
        parse("<Component {...props} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (spread_attribute content: (js))))))"
    );
}

#[test]
fn test_spread_attribute_with_others() {
    assert_eq!(
        parse(r#"<Component id="main" {...props} />"#),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value))) (attribute (spread_attribute content: (js))))))"
    );
}

#[test]
fn test_spread_attribute_nested_braces() {
    assert_eq!(
        parse("<input {...({})} onfocus={() => console.log('x')} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (spread_attribute content: (js))) (attribute (attribute_name) (expression content: (js))))))"
    );
}

// =============================================================================
// Directive attributes (nested inside attribute_name)
// =============================================================================

#[test]
fn test_directive_bind() {
    assert_eq!(
        parse("<input bind:value={name} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js))))))"
    );
}

#[test]
fn test_directive_on() {
    assert_eq!(
        parse("<button on:click={handleClick}></button>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_directive_with_modifiers() {
    assert_eq!(
        parse("<button on:click|preventDefault|stopPropagation={handler}></button>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier) (attribute_modifiers (attribute_modifier) (attribute_modifier))) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_directive_class() {
    assert_eq!(
        parse("<div class:active={isActive}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_directive_style() {
    assert_eq!(
        parse("<div style:color={textColor}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_directive_style_custom_property() {
    assert_eq!(
        parse("<div style:--color={textColor}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_directive_style_unquoted_mixed_value() {
    // Regression test: style:attr=string{mixed} should parse as a single attribute
    // with an unquoted_attribute_value containing text and expression,
    // NOT as two separate attributes (style:attr=string and {mixed} shorthand)
    assert_eq!(
        parse("<div style:attr=string{mixed}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (unquoted_attribute_value (attribute_value) (expression content: (js))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_directive_use() {
    assert_eq!(
        parse("<div use:tooltip></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_directive_use_store_member_action() {
    assert_eq!(
        parse("<div use:$store.action={text}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_directive_transition() {
    assert_eq!(
        parse("<div transition:fade></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)))) (end_tag (tag_name))))"
    );
}
