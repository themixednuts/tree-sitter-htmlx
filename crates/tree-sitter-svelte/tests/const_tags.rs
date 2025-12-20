//! Tests for {@const} tags

mod utils;
use utils::parse;

#[test]
fn test_const_simple_assignment() {
    assert_eq!(
        parse("{#if true}{@const x = 5}{x}{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (tag kind: (tag_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_const_object_destructure() {
    assert_eq!(
        parse("{#each items as item}{@const { a, b } = item}{a}{/each}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (tag kind: (tag_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_const_array_destructure() {
    assert_eq!(
        parse("{#each items as item}{@const [first, second] = item}{first}{/each}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (tag kind: (tag_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_const_expression() {
    assert_eq!(
        parse("{#each boxes as box}{@const area = box.width * box.height}{area}{/each}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (tag kind: (tag_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_const_function_call() {
    assert_eq!(
        parse("{#each boxes as box}{@const result = calculate(box.w, box.h)}{result}{/each}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (tag kind: (tag_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_const_multiple() {
    assert_eq!(
        parse("{#each boxes as box}{@const area = box.w * box.h}{@const perimeter = 2 * (box.w + box.h)}{area} {perimeter}{/each}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (tag kind: (tag_kind) expression: (expression_value)) (tag kind: (tag_kind) expression: (expression_value)) (expression content: (js)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_const_in_await() {
    assert_eq!(
        parse("{#await promise then data}{@const items = data.items}{#each items as item}{item}{/each}{/await}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (tag kind: (tag_kind) expression: (expression_value)) (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (expression content: (js)) (block_end kind: (block_kind))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_const_in_component_slot() {
    assert_eq!(
        parse("<Component let:value>{@const doubled = value * 2}{doubled}</Component>"),
        "(document (element (start_tag (tag_name) (attribute (attribute_name (attribute_directive) (attribute_identifier)))) (tag kind: (tag_kind) expression: (expression_value)) (expression content: (js)) (end_tag (tag_name))))"
    );
}

#[test]
fn test_const_nested_destructure() {
    assert_eq!(
        parse("{#each items as item}{@const { user: { name, email } } = item}{name}{/each}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (tag kind: (tag_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_const_with_default() {
    assert_eq!(
        parse("{#each items as item}{@const { value = 0 } = item}{value}{/each}"),
        "(document (block (block_start kind: (block_kind) expression: (expression) binding: (pattern)) (tag kind: (tag_kind) expression: (expression_value)) (expression content: (js)) (block_end kind: (block_kind))))"
    );
}
