//! Tests for {#if} blocks

mod utils;
use utils::parse;

#[test]
fn test_if_simple() {
    assert_eq!(
        parse("{#if foo}bar{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (text) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_if_with_element() {
    assert_eq!(
        parse("{#if visible}<p>Hello</p>{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_if_with_expression() {
    assert_eq!(
        parse("{#if count > 0}<p>{count}</p>{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_if_nested() {
    assert_eq!(
        parse("{#if a}{#if b}x{/if}{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (block (block_start kind: (block_kind) expression: (expression)) (text) (block_end kind: (block_kind))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_if_else() {
    assert_eq!(
        parse("{#if a}yes{:else}no{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (text) (block_branch kind: (block_kind)) (text) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_if_else_with_elements() {
    assert_eq!(
        parse("{#if show}<div>Visible</div>{:else}<span>Hidden</span>{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_branch kind: (block_kind)) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_if_else_if() {
    assert_eq!(
        parse("{#if a}1{:else if b}2{:else}3{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (text) (block_branch kind: (block_kind) expression: (expression_value)) (text) (block_branch kind: (block_kind)) (text) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_if_else_if_chain() {
    assert_eq!(
        parse("{#if a}1{:else if b}2{:else if c}3{:else}4{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (text) (block_branch kind: (block_kind) expression: (expression_value)) (text) (block_branch kind: (block_kind) expression: (expression_value)) (text) (block_branch kind: (block_kind)) (text) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_if_else_if_no_else() {
    assert_eq!(
        parse("{#if a}1{:else if b}2{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (text) (block_branch kind: (block_kind) expression: (expression_value)) (text) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_if_with_component() {
    assert_eq!(
        parse("{#if show}<Component />{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (element (self_closing_tag (tag_name))) (block_end kind: (block_kind))))"
    );
}

#[test]
fn test_if_complex_expression() {
    assert_eq!(
        parse("{#if user && user.isAdmin}admin{/if}"),
        "(document (block (block_start kind: (block_kind) expression: (expression)) (text) (block_end kind: (block_kind))))"
    );
}
