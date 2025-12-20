//! Fixture-based tests for tree-sitter-svelte
//! 
//! These tests parse Svelte files from the fixtures directory and verify
//! that they parse without errors. Tests are ported from the svelte2tsx
//! and Svelte compiler test suites.

use rstest::rstest;
use std::fs;
use std::path::PathBuf;
use tree_sitter_svelte::LANGUAGE;

fn parse_fixture(path: &str) -> (String, tree_sitter::Tree) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e));
    
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&LANGUAGE.into())
        .expect("Failed to load Svelte grammar");
    
    let tree = parser.parse(&source, None)
        .expect("Failed to parse");
    
    (source, tree)
}

fn assert_parses_without_error(path: &str) {
    let (source, tree) = parse_fixture(path);
    let root = tree.root_node();

    if root.has_error() {
        // Find and report the error location
        let mut errors = Vec::new();
        
        fn find_errors(node: tree_sitter::Node, errors: &mut Vec<String>, source: &str) {
            if node.is_error() || node.is_missing() {
                let start = node.start_position();
                let end = node.end_position();
                let text = &source[node.start_byte()..node.end_byte().min(source.len())];
                errors.push(format!(
                    "  {} at {}:{}-{}:{}: {:?}",
                    if node.is_error() { "ERROR" } else { "MISSING" },
                    start.row + 1, start.column,
                    end.row + 1, end.column,
                    text
                ));
            }
            for child in node.children(&mut node.walk()) {
                find_errors(child, errors, source);
            }
        }
        
        find_errors(root, &mut errors, &source);
        
        panic!(
            "Parse errors in {}:\n{}\n\nSource:\n{}",
            path,
            errors.join("\n"),
            source
        );
    }
}

// ==================== BLOCK TESTS ====================

#[rstest]
#[case::if_block("tests/fixtures/blocks/if-block.svelte")]
#[case::if_else("tests/fixtures/blocks/if-else.svelte")]
#[case::if_else_if("tests/fixtures/blocks/if-else-if.svelte")]
#[case::each_simple("tests/fixtures/blocks/each-simple.svelte")]
#[case::each_with_index_key("tests/fixtures/blocks/each-with-index-key.svelte")]
#[case::each_destructure("tests/fixtures/blocks/each-destructure.svelte")]
#[case::await_full("tests/fixtures/blocks/await-full.svelte")]
#[case::await_then_shorthand("tests/fixtures/blocks/await-then-shorthand.svelte")]
#[case::key_block("tests/fixtures/blocks/key-block.svelte")]
#[case::await_v5("tests/fixtures/blocks/await.v5.svelte")]
#[case::const_tag_each("tests/fixtures/blocks/const-tag-each.svelte")]
#[case::const_tag_await_then("tests/fixtures/blocks/const-tag-await-then.svelte")]
#[case::debug_block("tests/fixtures/blocks/debug-block.svelte")]
fn test_block_fixtures(#[case] path: &str) {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    assert_parses_without_error(fixture_path.to_str().unwrap());
}

// ==================== TAG TESTS ====================

#[rstest]
#[case::html_tag("tests/fixtures/tags/html-tag.svelte")]
#[case::const_tag("tests/fixtures/tags/const-tag.svelte")]
#[case::debug_tag("tests/fixtures/tags/debug-tag.svelte")]
#[case::attach_tag("tests/fixtures/tags/attach-tag.svelte")]
fn test_tag_fixtures(#[case] path: &str) {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    assert_parses_without_error(fixture_path.to_str().unwrap());
}

// ==================== RUNES/SNIPPET TESTS ====================

#[rstest]
#[case::runes_v5("tests/fixtures/runes/runes.v5.svelte")]
#[case::runes_bindable("tests/fixtures/runes/runes-bindable.v5.svelte")]
#[case::runes_full("tests/fixtures/runes/runes-full.svelte")]
#[case::snippet_instance("tests/fixtures/runes/snippet-instance-script.v5.svelte")]
#[case::snippet_generics("tests/fixtures/runes/snippet-generics.v5.svelte")]
#[case::snippet_render("tests/fixtures/runes/snippet-render.svelte")]
fn test_rune_fixtures(#[case] path: &str) {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    assert_parses_without_error(fixture_path.to_str().unwrap());
}

// ==================== COMPONENT TESTS ====================

#[rstest]
#[case::component_events("tests/fixtures/components/component-events-interface.svelte")]
#[case::component_slots("tests/fixtures/components/component-multiple-slots.svelte")]
#[case::component_slot_fallback("tests/fixtures/components/component-slot-fallback.svelte")]
fn test_component_fixtures(#[case] path: &str) {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    assert_parses_without_error(fixture_path.to_str().unwrap());
}

// ==================== DIRECTIVE TESTS ====================

#[rstest]
#[case::event_forwarded("tests/fixtures/directives/event-and-forwarded-event.svelte")]
#[case::event_bubble("tests/fixtures/directives/event-bubble-element.svelte")]
fn test_directive_fixtures(#[case] path: &str) {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    assert_parses_without_error(fixture_path.to_str().unwrap());
}

// ==================== AST VERIFICATION TESTS ====================

/// Test that parses the fixture and verifies specific AST node types exist
fn assert_contains_node_type(path: &str, node_type: &str) {
    let (_, tree) = parse_fixture(path);
    let root = tree.root_node();

    fn has_node_type(node: tree_sitter::Node, target: &str) -> bool {
        if node.kind() == target {
            return true;
        }
        for child in node.children(&mut node.walk()) {
            if has_node_type(child, target) {
                return true;
            }
        }
        false
    }

    assert!(
        has_node_type(root, node_type),
        "Expected to find node type '{}' in {}",
        node_type,
        path
    );
}

// Note: The grammar uses generic 'block' and 'tag' nodes with 'kind' field
// to identify if, each, await, snippet etc. These tests verify the
// generic node types exist.

#[test]
fn test_if_block_ast() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/blocks/if-block.svelte");
    assert_contains_node_type(fixture_path.to_str().unwrap(), "block");
}

#[test]
fn test_each_block_ast() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/blocks/each-simple.svelte");
    assert_contains_node_type(fixture_path.to_str().unwrap(), "block");
}

#[test]
fn test_await_block_ast() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/blocks/await-full.svelte");
    assert_contains_node_type(fixture_path.to_str().unwrap(), "block");
}

#[test]
fn test_snippet_block_ast() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/runes/snippet-render.svelte");
    assert_contains_node_type(fixture_path.to_str().unwrap(), "block");
}

#[test]
fn test_render_tag_ast() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/runes/snippet-render.svelte");
    assert_contains_node_type(fixture_path.to_str().unwrap(), "tag");
}

#[test]
fn test_html_tag_ast() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tags/html-tag.svelte");
    assert_contains_node_type(fixture_path.to_str().unwrap(), "tag");
}

#[test]
fn test_const_tag_ast() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tags/const-tag.svelte");
    assert_contains_node_type(fixture_path.to_str().unwrap(), "tag");
}
