//! Recovery-focused tests for malformed Svelte block syntax.

mod utils;
use utils::parse;

#[test]
fn test_unclosed_if_block_has_local_recovery_shape() {
    let tree = parse("{#if ready}<a></a><p></p>");
    assert!(
        tree.contains("(ERROR"),
        "Expected recovery ERROR node in malformed parse: {tree}"
    );
    assert!(tree.matches("(element").count() >= 2);
}

#[test]
fn test_unclosed_each_block_has_local_recovery_shape() {
    let tree = parse("{#each items as item}<li>{item}</li><li>tail</li>");
    assert!(
        tree.contains("(ERROR"),
        "Expected recovery ERROR node in malformed parse: {tree}"
    );
    assert!(tree.matches("(element").count() >= 2);
}

#[test]
fn test_unclosed_await_block_has_local_recovery_shape() {
    let tree = parse("{#await promise}<p>Loading</p>{:then value}<p>{value}</p>");
    assert!(
        tree.contains("(ERROR"),
        "Expected recovery ERROR node in malformed parse: {tree}"
    );
    assert!(tree.contains("(await_branch"));
}

#[test]
fn test_loose_unclosed_open_tag_does_not_swallow_following_blocks() {
    let source = "<div>\n\t<Comp foo={bar}\n</div>\n\n<div>\n\t<span foo={bar}\n</div>\n\n{#if foo}\n\t<Comp foo={bar}\n{/if}\n\n{#if foo}\n\t<Comp foo={bar}\n\t{#if bar}\n\t\t{bar}\n\t{/if}\n{/if}\n\n<div foo={bar}";
    let tree = parse(source);

    assert!(
        !tree.contains("(ERROR (if_block"),
        "Malformed open tags should not collapse later blocks into one error: {tree}"
    );
    assert!(tree.matches("(if_block").count() >= 2);
}

#[test]
fn test_loose_unclosed_tag_prefix_recovers_without_root_error() {
    let source = "<div>\n\t<Comp>\n</div>\n\n<div>\n\t<Comp foo={bar}\n</div>\n\n<div>\n\t<span\n</div>\n\n<div>\n\t<Comp.\n</div>\n\n<div>\n\t<comp.\n</div>\n";
    let tree = parse(source);

    assert!(
        !tree.contains("(ERROR"),
        "Malformed prefix should recover without root ERROR: {tree}"
    );
}

#[test]
fn test_loose_unclosed_tag_recovers_without_root_error() {
    let source = "<div>\n\t<Comp>\n</div>\n\n<div>\n\t<Comp foo={bar}\n</div>\n\n<div>\n\t<span\n</div>\n\n<div>\n\t<Comp.\n</div>\n\n<div>\n\t<comp.\n</div>\n\n{#if foo}\n\t<div>\n{/if}\n\n{#if foo}\n\t<Comp foo={bar}\n{/if}\n\n<div>\n<p>hi</p>\n\n<open-ended\n";
    let tree = parse(source);

    assert!(
        !tree.contains("(ERROR"),
        "Full loose-unclosed-tag sample should avoid root ERROR: {tree}"
    );
}

#[test]
fn test_if_block_with_unclosed_element_recovers_locally() {
    let source = "{#if foo}\n\t<div>\n{/if}";
    let tree = parse(source);

    assert!(
        !tree.contains("(ERROR"),
        "Unclosed element inside if block should recover without root ERROR: {tree}"
    );
}

#[test]
fn test_if_block_with_nested_unclosed_elements_recovers_locally() {
    let source = "{#if foo}\n\t<div>\n\t\t<span>\n{/if}";
    let tree = parse(source);

    assert!(
        !tree.contains("(ERROR"),
        "Nested unclosed elements before block close should recover without root ERROR: {tree}"
    );
}

#[test]
fn test_unterminated_tag_breaks_before_block_branches() {
    let source = "{#if ok}\n\t<input\n{:else}\n\t<p>fallback</p>\n{/if}\n\n{#await promise}\n\t<input\n{:then value}\n\t<p>{value}</p>\n{:catch error}\n\t<p>{error}</p>\n{/await}";
    let tree = parse(source);

    assert!(
        !tree.contains("(ERROR"),
        "Unterminated tags before block branches should recover without root ERROR: {tree}"
    );
    // Check for else_clause, else_if_clause, or await_branch nodes
    let branch_count = tree.matches("(else_clause").count()
        + tree.matches("(else_if_clause").count()
        + tree.matches("(await_branch").count();
    assert!(
        branch_count >= 3,
        "Expected else/then/catch branches to survive recovery: {tree}"
    );
}

#[test]
fn test_if_block_missing_right_brace_recovers_typed_block_start() {
    assert_eq!(
        parse("{#if visible <p>ok</p>{/if}"),
        "(document (if_block expression: (expression) (MISSING \"}\") (text) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_snippet_block_missing_right_brace_recovers_typed_block_start() {
    assert_eq!(
        parse("{#snippet children(name)<p>{name}</p>{/snippet}"),
        "(document (snippet_block name: (snippet_name) parameters: (snippet_parameters parameter: (pattern)) (MISSING \"}\") (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end)))"
    );
}

#[test]
fn test_snippet_block_missing_right_paren_preserves_parameters() {
    // {#snippet foo(a, b} — missing ) before }
    // Parameters should be preserved (not wrapped in ERROR)
    let tree = parse("{#snippet foo(a, b}<p>x</p>{/snippet}");
    assert!(
        tree.contains("parameters: (snippet_parameters parameter: (pattern) parameter: (pattern))"),
        "Parameters should be preserved in recovery: {tree}"
    );
    assert!(
        !tree.contains("(ERROR"),
        "Should not produce ERROR node: {tree}"
    );
}

#[test]
fn test_snippet_block_missing_right_paren_before_block_end() {
    // {#snippet children(hi{/snippet} — missing ) and } is inside {/snippet}
    // The scanner should treat {/ as a block boundary, allowing parameter recovery.
    let tree = parse("{#snippet children(hi{/snippet}");
    assert!(
        tree.contains("(snippet_name"),
        "Should preserve snippet_name: {tree}"
    );
    assert!(
        tree.contains("(snippet_parameters"),
        "Should preserve snippet_parameters: {tree}"
    );
}
