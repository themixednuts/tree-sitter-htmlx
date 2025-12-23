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

#[test]
fn test_else_if_block_kind_text() {
    // Verify that {:else if} has block_kind containing "else if", not just "else"
    use tree_sitter_svelte::LANGUAGE;
    
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    
    let source = "{#if a}1{:else if b}2{/if}";
    let tree = parser.parse(source, None).unwrap();
    
    // Find the block_branch node
    let root = tree.root_node();
    let block = root.child(0).unwrap();
    
    // block_branch is the 3rd child (after block_start and text "1")
    let block_branch = block.child(2).unwrap();
    assert_eq!(block_branch.kind(), "block_branch");
    
    // Get the kind field
    let kind_node = block_branch.child_by_field_name("kind").unwrap();
    let kind_text = kind_node.utf8_text(source.as_bytes()).unwrap();
    assert_eq!(kind_text, "else if", "block_kind should be 'else if', not just 'else'");
    
    // Get the expression field
    let expr_node = block_branch.child_by_field_name("expression").unwrap();
    let expr_text = expr_node.utf8_text(source.as_bytes()).unwrap();
    assert_eq!(expr_text, "b", "expression should be 'b', not 'if b'");
}
