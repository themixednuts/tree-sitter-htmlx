//! Tests for {#if} blocks

mod utils;
use utils::parse;

#[test]
fn test_if_simple() {
    assert_eq!(
        parse("{#if foo}bar{/if}"),
        "(document (if_block (block_open) expression: (expression content: (js)) (block_close) (text) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_if_with_element() {
    assert_eq!(
        parse("{#if visible}<p>Hello</p>{/if}"),
        "(document (if_block (block_open) expression: (expression content: (js)) (block_close) (element (start_tag name: (tag_name)) (text) (end_tag name: (tag_name))) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_if_inside_element_content() {
    assert_eq!(
        parse("<div>{#if visible}<span>ok</span>{/if}</div>"),
        "(document (element (start_tag name: (tag_name)) (if_block (block_open) expression: (expression content: (js)) (block_close) (element (start_tag name: (tag_name)) (text) (end_tag name: (tag_name))) (block_end (block_open) (block_keyword) (block_close))) (end_tag name: (tag_name))))"
    );
}

#[test]
fn test_if_with_expression() {
    assert_eq!(
        parse("{#if count > 0}<p>{count}</p>{/if}"),
        "(document (if_block (block_open) expression: (expression content: (js)) (block_close) (element (start_tag name: (tag_name)) (expression content: (js)) (end_tag name: (tag_name))) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_if_nested() {
    assert_eq!(
        parse("{#if a}{#if b}x{/if}{/if}"),
        "(document (if_block (block_open) expression: (expression content: (js)) (block_close) (if_block (block_open) expression: (expression content: (js)) (block_close) (text) (block_end (block_open) (block_keyword) (block_close))) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_if_else() {
    assert_eq!(
        parse("{#if a}yes{:else}no{/if}"),
        "(document (if_block (block_open) expression: (expression content: (js)) (block_close) (text) (else_clause (block_open) (block_close) (text)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_if_else_with_elements() {
    assert_eq!(
        parse("{#if show}<div>Visible</div>{:else}<span>Hidden</span>{/if}"),
        "(document (if_block (block_open) expression: (expression content: (js)) (block_close) (element (start_tag name: (tag_name)) (text) (end_tag name: (tag_name))) (else_clause (block_open) (block_close) (element (start_tag name: (tag_name)) (text) (end_tag name: (tag_name)))) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_if_else_if() {
    assert_eq!(
        parse("{#if a}1{:else if b}2{:else}3{/if}"),
        "(document (if_block (block_open) expression: (expression content: (js)) (block_close) (text) (else_if_clause (block_open) (branch_kind) expression: (expression_value content: (js)) (block_close) (text)) (else_clause (block_open) (block_close) (text)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_if_else_if_chain() {
    assert_eq!(
        parse("{#if a}1{:else if b}2{:else if c}3{:else}4{/if}"),
        "(document (if_block (block_open) expression: (expression content: (js)) (block_close) (text) (else_if_clause (block_open) (branch_kind) expression: (expression_value content: (js)) (block_close) (text)) (else_if_clause (block_open) (branch_kind) expression: (expression_value content: (js)) (block_close) (text)) (else_clause (block_open) (block_close) (text)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_if_else_if_no_else() {
    assert_eq!(
        parse("{#if a}1{:else if b}2{/if}"),
        "(document (if_block (block_open) expression: (expression content: (js)) (block_close) (text) (else_if_clause (block_open) (branch_kind) expression: (expression_value content: (js)) (block_close) (text)) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_if_with_component() {
    assert_eq!(
        parse("{#if show}<Component />{/if}"),
        "(document (if_block (block_open) expression: (expression content: (js)) (block_close) (element (self_closing_tag name: (tag_name))) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_if_complex_expression() {
    assert_eq!(
        parse("{#if user && user.isAdmin}admin{/if}"),
        "(document (if_block (block_open) expression: (expression content: (js)) (block_close) (text) (block_end (block_open) (block_keyword) (block_close))))"
    );
}

#[test]
fn test_else_if_block_kind_text() {
    // Verify that {:else if} clause can be found and has correct expression
    use tree_sitter_htmlx_svelte::LANGUAGE;

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();

    let source = "{#if a}1{:else if b}2{/if}";
    let tree = parser.parse(source, None).unwrap();

    // Find the if_block node
    let root = tree.root_node();
    let if_block = root.child(0).unwrap();
    assert_eq!(if_block.kind(), "if_block");

    // Find the else_if_clause (child after the text "1")
    let mut found_else_if = false;
    for i in 0..if_block.child_count() as u32 {
        let child = if_block.child(i).unwrap();
        if child.kind() == "else_if_clause" {
            found_else_if = true;
            // Get the expression field
            let expr_node = child.child_by_field_name("expression").unwrap();
            let expr_text = expr_node.utf8_text(source.as_bytes()).unwrap();
            assert_eq!(expr_text, "b", "expression should be 'b', not 'if b'");
            break;
        }
    }
    assert!(found_else_if, "Should find an else_if_clause node");
}
