//! Recovery-focused tests for malformed Svelte block syntax.

mod utils;
use utils::parse;

#[test]
fn test_trimmed_file_does_not_let_unclosed_if_recovery_swallow_siblings() {
    use tree_sitter_htmlx_svelte::LANGUAGE;

    let source = r#"<script>
	import { SvelteMap } from 'svelte/reactivity';

	let outside_basic = $state(false);
	let outside_basic_map = new SvelteMap();
	const throw_basic = $derived.by(() => {
		outside_basic_map.set(1, 1);
		return outside_basic_map;
	});

	let inside_basic = $state(false);
	const works_basic = $derived.by(() => {
		let inside = new SvelteMap();
		inside.set(1, 1);
		return inside;
	});

	let outside_has = $state(false);
	let outside_has_map = new SvelteMap([[1, 1]]);
	const throw_has = $derived.by(() => {
		outside_has_map.has(1);
		outside_has_map.set(1, 2);
		return outside_has_map;
	});

	let inside_has = $state(false);
	const works_has = $derived.by(() => {
		let inside = new SvelteMap([[1, 1]]);
		inside.has(1);
		inside.set(1, 1);
		return inside;
	});

	let outside_get = $state(false);
	let outside_get_map = new SvelteMap([[1, 1]]);
	const throw_get = $derived.by(() => {
		outside_get_map.get(1);
		outside_get_map.set(1, 2);
		return outside_get_map;
	});

	let inside_get = $state(false);
	const works_get = $derived.by(() => {
		let inside = new SvelteMap([[1, 1]]);
		inside.get(1);
		inside.set(1, 1);
		return inside;
	});

	let outside_values = $state(false);
	let outside_values_map = new SvelteMap([[1, 1]]);
	const throw_values = $derived.by(() => {
		outside_values_map.values(1);
		outside_values_map.set(1, 2);
		return outside_values_map;
	});

	let inside_values = $state(false);
	const works_values = $derived.by(() => {
		let inside = new SvelteMap([[1, 1]]);
		inside.values();
		inside.set(1, 1);
		return inside;
	});
</script>

<button onclick={() => (outside_basic = true)}>external</button>
{#if outside_basic}
	{throw_basic}
{/if}
<button onclick={() => (inside_basic = true)}>internal</button>
{#if inside_basic}
	{works_basic}
{/if}

<button onclick={() => (outside_has = true)}>external</button>
{#if outside_has}
	{throw_has}
{/if}
<button onclick={() => (inside_has = true)}>internal</button>
{#if inside_has}
	{works_has}
{/if}

<button onclick={() => (outside_get = true)}>external</button>
{#if outside_get}
	{throw_get}
{/if}
<button onclick={() => (inside_get = true)}>internal</button>
{#if inside_get}
	{works_get}
{/if}

<button onclick={() => (outside_values = true)}>external</button>
{#if outside_values}
	{throw_values}
{/if}
<button onclick={() => (inside_values = true)}>internal</button>
{#if inside_values}
	{works_values}
{/if}"#;

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();

    let tree = parser.parse(source, None).expect("parse");
    let root = tree.root_node();
    let sexp = root.to_sexp();

    let mut cursor = root.walk();
    let children: Vec<_> = root.named_children(&mut cursor).collect();
    let first_if = children
        .iter()
        .find(|node| node.kind() == "if_block")
        .copied()
        .unwrap_or_else(|| panic!("first top-level if_block missing: {sexp}"));

    let next = first_if
        .next_named_sibling()
        .unwrap_or_else(|| panic!("first if_block should not swallow following siblings: {sexp}"));
    assert_eq!(next.kind(), "text");

    let second = next
        .next_named_sibling()
        .expect("expected second top-level button after first if_block");
    assert_eq!(second.kind(), "element");
}

#[test]
fn test_top_level_continuation_exposes_orphan_branch() {
    let tree = parse("{:then foo}");
    assert!(
        tree.contains(
            "(orphan_branch kind: (branch_kind) binding: (pattern content: (js)) (block_close))"
        ),
        "Expected typed orphan_branch node: {tree}"
    );
}

#[test]
fn test_valid_if_else_does_not_expose_orphan_branch() {
    let tree = parse("{#if ok}<p/>{:else}<p/>{/if}");
    assert!(
        !tree.contains("(orphan_branch"),
        "Valid branches should stay inside block nodes: {tree}"
    );
}

#[test]
fn test_unclosed_if_block_has_local_recovery_shape() {
    let tree = parse("{#if ready}<a></a><p></p>");
    // Grammar now handles unclosed blocks as typed nodes (no ERROR)
    assert!(
        tree.contains("(if_block"),
        "Expected if_block node in unclosed block parse: {tree}"
    );
    assert!(tree.matches("(element").count() >= 2);
}

#[test]
fn test_unclosed_each_block_has_local_recovery_shape() {
    let tree = parse("{#each items as item}<li>{item}</li><li>tail</li>");
    // Grammar now handles unclosed blocks as typed nodes (no ERROR)
    assert!(
        tree.contains("(each_block"),
        "Expected each_block node in unclosed block parse: {tree}"
    );
    assert!(tree.matches("(element").count() >= 2);
}

#[test]
fn test_unclosed_await_block_has_local_recovery_shape() {
    let tree = parse("{#await promise}<p>Loading</p>{:then value}<p>{value}</p>");
    assert!(
        tree.contains("(await_block"),
        "Expected await_block node in malformed parse: {tree}"
    );
    assert!(tree.contains("(await_branch"));
}

#[test]
fn test_eof_truncated_if_block_start_stays_typed() {
    let tree = parse("{#if ready");
    assert!(
        tree.contains("(if_block"),
        "Expected typed if_block node at EOF: {tree}"
    );
    assert!(
        !tree.contains("(ERROR"),
        "EOF-truncated if block should not collapse to ERROR: {tree}"
    );
}

#[test]
fn test_eof_truncated_each_block_start_stays_typed() {
    let tree = parse("{#each items as item");
    assert!(
        tree.contains("(each_block"),
        "Expected typed each_block node at EOF: {tree}"
    );
    assert!(
        !tree.contains("(ERROR"),
        "EOF-truncated each block should not collapse to ERROR: {tree}"
    );
}

#[test]
fn test_eof_truncated_await_block_start_stays_typed() {
    let tree = parse("{#await promise");
    assert!(
        tree.contains("(await_block"),
        "Expected typed await_block node at EOF: {tree}"
    );
    assert!(
        !tree.contains("(ERROR"),
        "EOF-truncated await block should not collapse to ERROR: {tree}"
    );
}

#[test]
fn test_eof_truncated_key_block_start_stays_typed() {
    let tree = parse("{#key value");
    assert!(
        tree.contains("(key_block"),
        "Expected typed key_block node at EOF: {tree}"
    );
    assert!(
        !tree.contains("(ERROR"),
        "EOF-truncated key block should not collapse to ERROR: {tree}"
    );
}

#[test]
fn test_eof_truncated_snippet_block_start_stays_typed() {
    let tree = parse("{#snippet children(name)");
    assert!(
        tree.contains("(snippet_block"),
        "Expected typed snippet_block node at EOF: {tree}"
    );
    assert!(
        !tree.contains("(ERROR"),
        "EOF-truncated snippet block should not collapse to ERROR: {tree}"
    );
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
        "(document (if_block expression: (expression content: (js)) (MISSING block_close) (element (start_tag (tag_name)) (text) (end_tag (tag_name))) (block_end (block_close))))"
    );
}

#[test]
fn test_snippet_block_missing_right_brace_recovers_typed_block_start() {
    assert_eq!(
        parse("{#snippet children(name)<p>{name}</p>{/snippet}"),
        "(document (snippet_block name: (snippet_name) parameters: (snippet_parameters parameter: (pattern content: (js))) (MISSING block_close) (element (start_tag (tag_name)) (expression content: (js)) (end_tag (tag_name))) (block_end (block_close))))"
    );
}

#[test]
fn test_snippet_block_missing_right_paren_preserves_parameters() {
    // {#snippet foo(a, b} — missing ) before }
    // Parameters should be preserved (not wrapped in ERROR)
    let tree = parse("{#snippet foo(a, b}<p>x</p>{/snippet}");
    assert!(
        tree.contains("parameters: (snippet_parameters parameter: (pattern content: (js)) parameter: (pattern content: (js)))"),
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
