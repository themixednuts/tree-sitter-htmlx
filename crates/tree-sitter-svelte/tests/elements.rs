//! Tests for HTML elements

mod utils;
use utils::parse;

#[test]
fn test_element_empty() {
    assert_eq!(
        parse("<div></div>"),
        "(document (element (start_tag (tag_name)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_self_closing() {
    assert_eq!(
        parse("<input />"),
        "(document (element (self_closing_tag (tag_name))))"
    );
}

#[test]
fn test_element_void() {
    // Void elements like <br> are parsed as start_tag (HTML behavior)
    assert_eq!(
        parse("<br>"),
        "(document (element (start_tag (tag_name))))"
    );
}

#[test]
fn test_element_with_text() {
    assert_eq!(
        parse("<p>Hello world</p>"),
        "(document (element (start_tag (tag_name)) (text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_with_expression() {
    assert_eq!(
        parse("<p>{message}</p>"),
        "(document (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_mixed_content() {
    assert_eq!(
        parse("<p>Hello {name}!</p>"),
        "(document (element (start_tag (tag_name)) (text) (expression content: (js)) (text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_nested() {
    assert_eq!(
        parse("<div><span>text</span></div>"),
        "(document (element (start_tag (tag_name)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_deeply_nested() {
    assert_eq!(
        parse("<div><section><article><p>text</p></article></section></div>"),
        "(document (element (start_tag (tag_name)) (element (start_tag (tag_name)) (element (start_tag (tag_name)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (end_tag (tag_name))) (end_tag (tag_name))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_quoted_attribute() {
    assert_eq!(
        parse(r#"<div class="container"></div>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (end_tag (tag_name))))"#
    );
}

#[test]
fn test_element_single_quoted_attribute() {
    assert_eq!(
        parse(r#"<div class='container'></div>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (end_tag (tag_name))))"#
    );
}

#[test]
fn test_element_unquoted_attribute() {
    assert_eq!(
        parse("<div class=container></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name) (attribute_value))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_expression_attribute() {
    assert_eq!(
        parse("<div class={styles}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_shorthand_attribute() {
    assert_eq!(
        parse("<div {id}></div>"),
        "(document (element (start_tag (tag_name) (attribute (shorthand_attribute))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_spread_attribute() {
    assert_eq!(
        parse("<div {...props}></div>"),
        "(document (element (start_tag (tag_name) (attribute (spread_attribute))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_boolean_attribute() {
    assert_eq!(
        parse("<input disabled />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name)))))"
    );
}

#[test]
fn test_element_multiple_attributes() {
    assert_eq!(
        parse(r#"<div class="a" id={b} {c} {...d}></div>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value))) (attribute (attribute_name) (expression content: (js))) (attribute (shorthand_attribute)) (attribute (spread_attribute))) (end_tag (tag_name))))"#
    );
}

#[test]
fn test_element_event_handler() {
    assert_eq!(
        parse("<button on:click={handleClick}></button>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_event_forwarding() {
    assert_eq!(
        parse("<button on:click></button>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_event_modifiers() {
    assert_eq!(
        parse("<button on:click|once|preventDefault={handler}></button>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value) modifiers: (directive_modifiers (directive_modifier) (directive_modifier))) (expression content: (js))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_bind_value() {
    assert_eq!(
        parse("<input bind:value={name} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js)))))))"
    );
}

#[test]
fn test_element_bind_shorthand() {
    assert_eq!(
        parse("<input bind:value />"),
        "(document (element (self_closing_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)))))))"
    );
}

#[test]
fn test_element_bind_this() {
    assert_eq!(
        parse("<canvas bind:this={canvas} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js)))))))"
    );
}

#[test]
fn test_element_bind_group() {
    assert_eq!(
        parse(r#"<input type="radio" bind:group={selected} value="a" />"#),
        r#"(document (element (self_closing_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value))) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js)))) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))))"#
    );
}

#[test]
fn test_element_class_directive() {
    assert_eq!(
        parse("<div class:active={isActive}></div>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_class_directive_shorthand() {
    assert_eq!(
        parse("<div class:active></div>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_style_directive() {
    assert_eq!(
        parse("<div style:color={textColor}></div>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_style_directive_important() {
    assert_eq!(
        parse("<div style:color|important={color}></div>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value) modifiers: (directive_modifiers (directive_modifier))) (expression content: (js))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_use_directive() {
    assert_eq!(
        parse("<div use:action></div>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_use_directive_with_params() {
    assert_eq!(
        parse("<div use:action={params}></div>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_transition() {
    assert_eq!(
        parse("<div transition:fade></div>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_transition_with_params() {
    assert_eq!(
        parse("<div transition:fade={{ duration: 300 }}></div>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)) (expression content: (js))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_in_out() {
    assert_eq!(
        parse("<div in:fly out:fade></div>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value)))) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_animate() {
    assert_eq!(
        parse("<div animate:flip></div>"),
        "(document (element (start_tag (tag_name) (attribute (directive_attribute (directive_name (directive_prefix) name: (directive_value))))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_data_attribute() {
    assert_eq!(
        parse(r#"<div data-testid="my-div"></div>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (end_tag (tag_name))))"#
    );
}

#[test]
fn test_element_interpolated_attribute() {
    assert_eq!(
        parse(r#"<div class="item-{type}"></div>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (end_tag (tag_name))))"#
    );
}
