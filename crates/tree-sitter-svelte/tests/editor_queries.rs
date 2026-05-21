//! Coverage tests for editor-facing Svelte queries.

use std::collections::BTreeSet;
use tree_sitter::StreamingIterator;
use tree_sitter_htmlx_svelte::{FOLDS_QUERY, INDENTS_QUERY, LANGUAGE, LOCALS_QUERY};

fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&LANGUAGE.into())
        .expect("failed to load Svelte grammar");

    let tree = parser.parse(source, None).expect("failed to parse source");
    assert!(!tree.root_node().has_error(), "{source}");
    tree
}

fn capture_texts(query_source: &str, capture_name: &str, source: &str) -> BTreeSet<String> {
    let language = LANGUAGE.into();
    let query = tree_sitter::Query::new(&language, query_source)
        .expect("editor query should compile before capture assertions");
    let tree = parse(source);

    let capture_names = query.capture_names();
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut captures = cursor.captures(&query, tree.root_node(), source.as_bytes());
    let mut texts = BTreeSet::new();

    loop {
        captures.advance();
        let Some((query_match, capture_index)) = captures.get() else {
            break;
        };
        let capture = query_match.captures[*capture_index];
        if capture_names[capture.index as usize] == capture_name {
            texts.insert(
                capture
                    .node
                    .utf8_text(source.as_bytes())
                    .expect("capture text should be valid UTF-8")
                    .to_string(),
            );
        }
    }

    texts
}

fn assert_captures_include(
    query_source: &str,
    capture_name: &str,
    source: &str,
    expected: &[&str],
) {
    let texts = capture_texts(query_source, capture_name, source);

    for text in expected {
        assert!(
            texts.contains(*text),
            "missing @{capture_name} capture {text:?}\nactual captures:\n{texts:#?}"
        );
    }
}

fn assert_captures_exclude(
    query_source: &str,
    capture_name: &str,
    source: &str,
    unexpected: &[&str],
) {
    let texts = capture_texts(query_source, capture_name, source);

    for text in unexpected {
        assert!(
            !texts.contains(*text),
            "unexpected @{capture_name} capture {text:?}\nactual captures:\n{texts:#?}"
        );
    }
}

fn assert_trimmed_captures_include(
    query_source: &str,
    capture_name: &str,
    source: &str,
    expected: &[&str],
) {
    let texts: BTreeSet<_> = capture_texts(query_source, capture_name, source)
        .into_iter()
        .map(|text| text.trim().to_string())
        .collect();

    for text in expected {
        assert!(
            texts.contains(*text),
            "missing trimmed @{capture_name} capture {text:?}\nactual captures:\n{texts:#?}"
        );
    }
}

#[test]
fn folds_cover_structural_svelte_features() {
    let source = r#"
<!-- comment -->
<script>let count = 0;</script>
<style>p { color: red; }</style>
<svelte:head><title>{title}</title></svelte:head>
<Component>
  {#if ready}
    <p>{count}</p>
  {:else if pending}
    <p>pending</p>
  {:else}
    <p>empty</p>
  {/if}

  {#each items as item, i (item.id)}
    <p>{i}: {item.name}</p>
  {:else}
    <p>none</p>
  {/each}

  {#await promise}
    <p>loading</p>
  {:then data}
    <p>{data.name}</p>
  {:catch error}
    <p>{error.message}</p>
  {/await}

  {#key count}
    <p>{count}</p>
  {/key}

  {#snippet row(value)}
    <p>{value}</p>
  {/snippet}
</Component>
"#;

    assert_trimmed_captures_include(
        FOLDS_QUERY,
        "fold",
        source,
        &[
            "<!-- comment -->",
            "<script>let count = 0;</script>",
            "<style>p { color: red; }</style>",
            "<svelte:head><title>{title}</title></svelte:head>",
            "<Component>\n  {#if ready}\n    <p>{count}</p>\n  {:else if pending}\n    <p>pending</p>\n  {:else}\n    <p>empty</p>\n  {/if}\n\n  {#each items as item, i (item.id)}\n    <p>{i}: {item.name}</p>\n  {:else}\n    <p>none</p>\n  {/each}\n\n  {#await promise}\n    <p>loading</p>\n  {:then data}\n    <p>{data.name}</p>\n  {:catch error}\n    <p>{error.message}</p>\n  {/await}\n\n  {#key count}\n    <p>{count}</p>\n  {/key}\n\n  {#snippet row(value)}\n    <p>{value}</p>\n  {/snippet}\n</Component>",
            "{#if ready}\n    <p>{count}</p>\n  {:else if pending}\n    <p>pending</p>\n  {:else}\n    <p>empty</p>\n  {/if}",
            "{:else if pending}\n    <p>pending</p>",
            "{:else}\n    <p>empty</p>",
            "{#each items as item, i (item.id)}\n    <p>{i}: {item.name}</p>\n  {:else}\n    <p>none</p>\n  {/each}",
            "{#await promise}\n    <p>loading</p>\n  {:then data}\n    <p>{data.name}</p>\n  {:catch error}\n    <p>{error.message}</p>\n  {/await}",
            "<p>loading</p>",
            "{:then data}\n    <p>{data.name}</p>",
            "<p>{data.name}</p>",
            "{#key count}\n    <p>{count}</p>\n  {/key}",
            "{#snippet row(value)}\n    <p>{value}</p>\n  {/snippet}",
        ],
    );
}

#[test]
fn folds_cover_recovery_branches() {
    assert_trimmed_captures_include(
        FOLDS_QUERY,
        "fold",
        "{:else if orphaned}",
        &["{:else if orphaned}"],
    );
}

#[test]
fn indents_cover_structural_svelte_features() {
    let source = r#"
<div>
  {#if ready}
    <p>{count}</p>
  {:else if pending}
    <p>pending</p>
  {:else}
    <p>empty</p>
  {/if}

  {#each items as item}
    <p>{item}</p>
  {/each}

  {#await promise}
    <p>loading</p>
  {:then data}
    <p>{data}</p>
  {/await}

  {#key count}
    <p>{count}</p>
  {/key}

  {#snippet row(value)}
    <p>{value}</p>
  {/snippet}
</div>
"#;

    assert_trimmed_captures_include(
        INDENTS_QUERY,
        "indent.begin",
        source,
        &[
            "<div>\n  {#if ready}\n    <p>{count}</p>\n  {:else if pending}\n    <p>pending</p>\n  {:else}\n    <p>empty</p>\n  {/if}\n\n  {#each items as item}\n    <p>{item}</p>\n  {/each}\n\n  {#await promise}\n    <p>loading</p>\n  {:then data}\n    <p>{data}</p>\n  {/await}\n\n  {#key count}\n    <p>{count}</p>\n  {/key}\n\n  {#snippet row(value)}\n    <p>{value}</p>\n  {/snippet}\n</div>",
            "{#if ready}\n    <p>{count}</p>\n  {:else if pending}\n    <p>pending</p>\n  {:else}\n    <p>empty</p>\n  {/if}",
            "{#each items as item}\n    <p>{item}</p>\n  {/each}",
            "{#await promise}\n    <p>loading</p>\n  {:then data}\n    <p>{data}</p>\n  {/await}",
            "{#key count}\n    <p>{count}</p>\n  {/key}",
            "{#snippet row(value)}\n    <p>{value}</p>\n  {/snippet}",
        ],
    );

    assert_trimmed_captures_include(
        INDENTS_QUERY,
        "indent.branch",
        source,
        &[
            "{:else if pending}\n    <p>pending</p>",
            "{:else}\n    <p>empty</p>",
            "{:then data}\n    <p>{data}</p>",
            "{/if}",
            "{/each}",
            "{/await}",
            "{/key}",
            "{/snippet}",
        ],
    );
}

#[test]
fn locals_cover_svelte_bindings_and_references() {
    let source = r#"
{#snippet row(value)}
  {@const label = value.name}
  <Component let:slotValue {disabled} on:click={handler}>
    {#if ready}
      {@html rawHtml}
    {:else if pending}
      {@debug label, slotValue}
    {:else}
      {@attach action}
      {@render row(label, slotValue)}
    {/if}
  </Component>
  <div
    use:tooltip={params}
    transition:fade={transitionParams}
    in:fly
    out:slide
    animate:flip
    bind:value
    class:active
    style:color
    style:--theme={themeColor}
    on:click
  />
{/snippet}

{#each items as item, i (item.id)}
  {#await item.promise then data}
    <p>{data.name}</p>
  {:catch error}
    <p>{error.message}</p>
  {/await}
{/each}
"#;

    assert_captures_include(LOCALS_QUERY, "local.definition.function", source, &["row"]);
    assert_captures_include(
        LOCALS_QUERY,
        "local.definition",
        source,
        &[
            "value",
            "label = value.name",
            "slotValue",
            "item",
            "i",
            "data",
            "error",
        ],
    );
    assert_captures_include(
        LOCALS_QUERY,
        "local.reference",
        source,
        &[
            "disabled",
            "handler",
            "tooltip",
            "params",
            "fade",
            "transitionParams",
            "fly",
            "slide",
            "flip",
            "value",
            "active",
            "color",
            "themeColor",
            "ready",
            "rawHtml",
            "pending",
            "label, slotValue",
            "action",
            "row(label, slotValue)",
            "items",
            "item.id",
            "item.promise",
            "data.name",
            "error.message",
        ],
    );
    assert_captures_exclude(
        LOCALS_QUERY,
        "local.reference",
        source,
        &["click", "--theme"],
    );
}

#[test]
fn locals_trim_leading_whitespace_after_svelte_markers() {
    let source = "{@attach\n\t\taction}
{@html\n\t\trawHtml}
{@debug\n\t\tlabel, slotValue}
{@render\n\t\trow(label, slotValue)}
{@const\n\t\tlabel = value.name}
{#if\n\t\tready}
{:else if\n\t\tpending}
{/if}
{#each\n\t\titems as item, i (item.id)}
{/each}
{#await\n\t\titem.promise then data}
{:catch\n\t\terror}
{/await}
{#key\n\t\titem.id}
{/key}
{#snippet row(\n\t\tvalue)}
{/snippet}";

    assert_captures_include(
        LOCALS_QUERY,
        "local.definition",
        source,
        &["label = value.name", "item", "i", "data", "error", "value"],
    );
    assert_captures_include(
        LOCALS_QUERY,
        "local.reference",
        source,
        &[
            "action",
            "rawHtml",
            "label, slotValue",
            "row(label, slotValue)",
            "ready",
            "pending",
            "items",
            "item.id",
            "item.promise",
        ],
    );

    for capture_name in ["local.definition", "local.reference"] {
        for text in capture_texts(LOCALS_QUERY, capture_name, source) {
            assert_eq!(
                text.trim_start(),
                text,
                "@{capture_name} capture should not include leading whitespace: {text:?}"
            );
        }
    }
}

#[test]
fn locals_cover_recovery_branch_bindings_and_references() {
    assert_captures_include(
        LOCALS_QUERY,
        "local.definition",
        "{:then orphaned}",
        &["orphaned"],
    );
    assert_captures_include(
        LOCALS_QUERY,
        "local.reference",
        "{:else if orphaned}",
        &["orphaned"],
    );
}
