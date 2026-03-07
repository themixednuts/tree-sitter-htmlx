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
    assert_eq!(parse("<br>"), "(document (element (start_tag (tag_name))))");
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
fn test_textarea_plain_text_stays_text_like() {
    assert_eq!(
        parse("<textarea>plain <b>text</b></textarea>"),
        "(document (element (start_tag (tag_name)) (text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_textarea_expression_exposes_svelte_expression() {
    assert_eq!(
        parse("<textarea>{value}</textarea>"),
        "(document (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_textarea_html_tag_exposes_svelte_tag() {
    assert_eq!(
        parse("<textarea>{@html value}</textarea>"),
        "(document (element (start_tag (tag_name)) (html_tag expression: (expression_value)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_textarea_if_block_exposes_svelte_block() {
    assert_eq!(
        parse("<textarea>{#if ok}{/if}</textarea>"),
        "(document (element (start_tag (tag_name)) (if_block expression: (expression) (block_end)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_textarea_ignores_malformed_close_until_real_end_tag() {
    assert_eq!(
        parse("<textarea>\n\t<p>not actu </textar ally an element. {foo}</p>\n</textare\n\n\n> </textaread >asdf</textarea\n\n\n</textarea\n\n>\n"),
        "(document (element (start_tag (tag_name)) (text) (expression content: (js)) (text) (end_tag (tag_name))) (text))"
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
        "(document (element (start_tag (tag_name) (attribute (shorthand_attribute content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_multiline_shorthand_attribute() {
    assert_eq!(
        parse("<div\n {id}\n></div>"),
        "(document (element (start_tag (tag_name) (attribute (shorthand_attribute content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_spread_attribute() {
    assert_eq!(
        parse("<div {...props}></div>"),
        "(document (element (start_tag (tag_name) (attribute (spread_attribute content: (js)))) (end_tag (tag_name))))"
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
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value))) (attribute (attribute_name) (expression content: (js))) (attribute (shorthand_attribute content: (js))) (attribute (spread_attribute content: (js)))) (end_tag (tag_name))))"#
    );
}

#[test]
fn test_element_event_handler() {
    assert_eq!(
        parse("<button on:click={handleClick}></button>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_multiline_event_handler() {
    assert_eq!(
        parse("<button\n\ton:click={handleClick}\n></button>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_event_handler_comment_prefix_expression() {
    assert_eq!(
        parse("<button\n\ton:click={// comment\n\t\t() => {\n\t\t\tfn();\n\t\t}}\n></button>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_event_forwarding() {
    assert_eq!(
        parse("<button on:click></button>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_event_modifiers() {
    assert_eq!(
        parse("<button on:click|once|preventDefault={handler}></button>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier) (attribute_modifiers (attribute_modifier) (attribute_modifier))) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_bind_value() {
    assert_eq!(
        parse("<input bind:value={name} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js))))))"
    );
}

#[test]
fn test_element_bind_shorthand() {
    assert_eq!(
        parse("<input bind:value />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier))))))"
    );
}

#[test]
fn test_element_bind_this() {
    assert_eq!(
        parse("<canvas bind:this={canvas} />"),
        "(document (element (self_closing_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js))))))"
    );
}

#[test]
fn test_element_bind_group() {
    assert_eq!(
        parse(r#"<input type="radio" bind:group={selected} value="a" />"#),
        r#"(document (element (self_closing_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value))) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js))) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))))"#
    );
}

#[test]
fn test_element_class_directive() {
    assert_eq!(
        parse("<div class:active={isActive}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_class_directive_shorthand() {
    assert_eq!(
        parse("<div class:active></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_style_directive() {
    assert_eq!(
        parse("<div style:color={textColor}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_style_directive_custom_property() {
    assert_eq!(
        parse("<div style:--color={textColor}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_style_directive_important() {
    assert_eq!(
        parse("<div style:color|important={color}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier) (attribute_modifiers (attribute_modifier))) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_use_directive() {
    assert_eq!(
        parse("<div use:action></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_use_directive_with_params() {
    assert_eq!(
        parse("<div use:action={params}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_use_directive_store_member_action() {
    assert_eq!(
        parse("<div use:$store.action={text}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_transition() {
    assert_eq!(
        parse("<div transition:fade></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_transition_with_params() {
    assert_eq!(
        parse("<div transition:fade={{ duration: 300 }}></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)) (expression content: (js)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_in_out() {
    assert_eq!(
        parse("<div in:fly out:fade></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier))) (attribute (attribute_name (attribute_directive) (attribute_identifier)))) (end_tag (tag_name))))"
    );
}

#[test]
fn test_element_animate() {
    assert_eq!(
        parse("<div animate:flip></div>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)))) (end_tag (tag_name))))"
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
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value) (expression content: (js))))) (end_tag (tag_name))))"#
    );
}
