//! Tests for Svelte 5 runes ($props, $state, $derived, $effect, $bindable)

mod utils;
use utils::parse;

// =============================================================================
// $props rune
// =============================================================================

#[test]
fn test_props_basic() {
    assert_eq!(
        parse("<script>let { name } = $props();</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_props_multiple() {
    assert_eq!(
        parse("<script>let { a, b, c } = $props();</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_props_with_defaults() {
    assert_eq!(
        parse("<script>let { name = 'default' } = $props();</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_props_with_rest() {
    assert_eq!(
        parse("<script>let { a, ...rest } = $props();</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_props_typescript() {
    assert_eq!(
        parse(r#"<script lang="ts">let { name }: { name: string } = $props();</script>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag (tag_name))))"#
    );
}

#[test]
fn test_props_interface_typescript() {
    assert_eq!(
        parse(r#"<script lang="ts">interface Props { name: string; count?: number; } let { name, count = 0 }: Props = $props();</script>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag (tag_name))))"#
    );
}

// =============================================================================
// $state rune
// =============================================================================

#[test]
fn test_state_basic() {
    assert_eq!(
        parse("<script>let count = $state(0);</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_state_object() {
    assert_eq!(
        parse("<script>let user = $state({ name: '', age: 0 });</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_state_array() {
    assert_eq!(
        parse("<script>let items = $state([]);</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_state_raw() {
    assert_eq!(
        parse("<script>let data = $state.raw({ big: 'object' });</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_state_snapshot() {
    assert_eq!(
        parse("<script>let count = $state(0); const snapshot = $state.snapshot(count);</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

// =============================================================================
// $derived rune
// =============================================================================

#[test]
fn test_derived_basic() {
    assert_eq!(
        parse("<script>let count = $state(0); let doubled = $derived(count * 2);</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_derived_complex() {
    assert_eq!(
        parse("<script>let items = $state([]); let total = $derived(items.reduce((a, b) => a + b, 0));</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_derived_by() {
    assert_eq!(
        parse("<script>let count = $state(0); let result = $derived.by(() => { return heavyComputation(count); });</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

// =============================================================================
// $effect rune
// =============================================================================

#[test]
fn test_effect_basic() {
    assert_eq!(
        parse("<script>let count = $state(0); $effect(() => { console.log(count); });</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_effect_with_cleanup() {
    assert_eq!(
        parse("<script>$effect(() => { const id = setInterval(() => {}, 1000); return () => clearInterval(id); });</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_effect_pre() {
    assert_eq!(
        parse("<script>$effect.pre(() => { console.log('before DOM update'); });</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_effect_root() {
    assert_eq!(
        parse("<script>const cleanup = $effect.root(() => { $effect(() => {}); });</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

// =============================================================================
// $bindable rune
// =============================================================================

#[test]
fn test_bindable_basic() {
    assert_eq!(
        parse("<script>let { value = $bindable() } = $props();</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_bindable_with_default() {
    assert_eq!(
        parse("<script>let { value = $bindable('default') } = $props();</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_bindable_multiple() {
    assert_eq!(
        parse("<script>let { a = $bindable(), b = $bindable(0) } = $props();</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

// =============================================================================
// $inspect rune (debug)
// =============================================================================

#[test]
fn test_inspect_basic() {
    assert_eq!(
        parse("<script>let count = $state(0); $inspect(count);</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

#[test]
fn test_inspect_with_fn() {
    assert_eq!(
        parse("<script>$inspect(value).with(console.trace);</script>"),
        "(document (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}

// =============================================================================
// $host rune (custom elements)
// =============================================================================

#[test]
fn test_host_basic() {
    assert_eq!(
        parse(r#"<svelte:options customElement="my-element" /><script>$effect(() => { $host().dispatchEvent(new CustomEvent('change')); });</script>"#),
        r#"(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (quoted_attribute_value (attribute_value))))) (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"#
    );
}

// =============================================================================
// Combined runes usage
// =============================================================================

#[test]
fn test_runes_full_component() {
    assert_eq!(
        parse(r#"<script lang="ts">
    let { a, b = $bindable() } = $props();
    let count = $state(0);
    let doubled = $derived(count * 2);

    $effect(() => {
        console.log('count changed:', count);
    });
</script>

<button onclick={() => count++}>
    {count} * 2 = {doubled}
</button>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag (tag_name))) (element (start_tag (tag_name) (attribute (attribute_name) (expression content: (ts)))) (expression content: (ts)) (text) (expression content: (ts)) (end_tag (tag_name))))"#
    );
}

#[test]
fn test_runes_with_generics() {
    assert_eq!(
        parse(r#"<script lang="ts" generics="T">
    let { items }: { items: T[] } = $props();
    let selected = $state<T | null>(null);
</script>"#),
        r#"(document (element (start_tag (tag_name) (attribute (attribute_name) (quoted_attribute_value (attribute_value))) (attribute (attribute_name) (quoted_attribute_value (attribute_value)))) (raw_text) (end_tag (tag_name))))"#
    );
}

// =============================================================================
// Runes mode opt-in
// =============================================================================

#[test]
fn test_runes_mode_optin() {
    assert_eq!(
        parse("<svelte:options runes={true} />\n<script>let count = $state(0);</script>"),
        "(document (element (self_closing_tag (tag_name namespace: (tag_namespace) name: (tag_local_name)) (attribute (attribute_name) (expression content: (js))))) (element (start_tag (tag_name)) (raw_text) (end_tag (tag_name))))"
    );
}
