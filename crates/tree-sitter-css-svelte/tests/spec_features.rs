use tree_sitter_css_svelte::LANGUAGE;

fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&LANGUAGE.into())
        .expect("failed to load css grammar");
    parser.parse(source, None).expect("parse")
}

#[test]
fn parses_container_query_with_style_condition() {
    let tree = parse("@container card (inline-size > 30em) and style(--responsive: true) { .item { color: red; } }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("container_statement"), "{sexp}");
    assert!(sexp.contains("container_query"), "{sexp}");
    assert!(sexp.contains("container_style_query"), "{sexp}");
}

#[test]
fn parses_plain_container_feature_queries() {
    let tree = parse("@container (min-width: 400px) { .item { color: red; } }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("container_feature"), "{sexp}");
    assert!(sexp.contains("query_feature_plain"), "{sexp}");
}

#[test]
fn parses_media_range_queries() {
    let tree = parse("@media (400px <= width <= calc(1000px + 1px)) { .item { display: block; } }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("media_statement"), "{sexp}");
    assert!(sexp.contains("media_query_list"), "{sexp}");
    assert!(sexp.contains("query_feature_range"), "{sexp}");
}

#[test]
fn parses_plain_media_feature_queries() {
    let tree = parse("@media (min-width: 400px) { .item { display: block; } }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("media_feature"), "{sexp}");
    assert!(sexp.contains("query_feature_plain"), "{sexp}");
}

#[test]
fn parses_boolean_media_queries_with_media_types() {
    let tree = parse(
        "@media screen and (width >= 40rem), not print and (hover) { .item { display: block; } }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("media_type"), "{sexp}");
    assert!(sexp.contains("media_condition_without_or"), "{sexp}");
}

#[test]
fn parses_mixed_boolean_media_queries() {
    let tree = parse(
        "@media (width > 400px) and (width > 800px) or (orientation: portrait) { .item { display: block; } }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("media_or_condition"), "{sexp}");
    assert!(sexp.contains("media_and_condition"), "{sexp}");
}

#[test]
fn parses_general_enclosed_media_queries() {
    for css in [
        "@media unknown(foo(bar[baz])) { .item { display: block; } }",
        "@media (foo(bar[baz])) { .item { display: block; } }",
        "@media (foo bar) { .item { display: block; } }",
    ] {
        let tree = parse(css);
        let sexp = tree.root_node().to_sexp();
        assert!(!tree.root_node().has_error(), "{sexp}");
        assert!(
            sexp.contains("general_enclosed_function") || sexp.contains("general_enclosed_parens"),
            "{sexp}"
        );
    }
}

#[test]
fn parses_supports_selector_conditions() {
    let tree =
        parse("@supports (display: grid) and selector(:has(> img)) { .item { display: grid; } }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("supports_condition"), "{sexp}");
    assert!(sexp.contains("supports_feature"), "{sexp}");
    assert!(sexp.contains("selector_query"), "{sexp}");
}

#[test]
fn parses_plain_supports_feature_queries() {
    let tree = parse("@supports (display: grid) { .item { display: grid; } }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("supports_feature"), "{sexp}");
    assert!(sexp.contains("supports_feature_body"), "{sexp}");
}

#[test]
fn parses_mixed_boolean_supports_queries() {
    let tree = parse(
        "@supports (display: grid) and (color: red) or selector(.item) { .item { display: grid; } }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("supports_or_condition"), "{sexp}");
    assert!(sexp.contains("supports_and_condition"), "{sexp}");
}

#[test]
fn parses_general_enclosed_supports_queries() {
    for css in [
        "@supports font-tech(color-COLRv1) { .item { display: grid; } }",
        "@supports (foo(bar[baz])) { .item { display: grid; } }",
        "@supports (foo bar) { .item { display: grid; } }",
        "@import url(\"theme.css\") supports(font-tech(color-COLRv1));",
    ] {
        let tree = parse(css);
        let sexp = tree.root_node().to_sexp();
        assert!(!tree.root_node().has_error(), "{sexp}");
        assert!(
            sexp.contains("general_enclosed_function") || sexp.contains("general_enclosed_parens"),
            "{sexp}"
        );
    }
}

#[test]
fn parses_nested_style_queries_inside_container() {
    let tree = parse(
        "@container card (inline-size > 30em) and style((--responsive: true) and (color: green)) { .item { color: red; } }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("style_condition"), "{sexp}");
    assert!(sexp.contains("style_feature"), "{sexp}");
}

#[test]
fn parses_general_enclosed_container_queries() {
    for css in [
        "@container (foo(bar[baz])) { .item { color: red; } }",
        "@container style(foo(bar[baz])) { .item { color: red; } }",
        "@container (foo bar) { .item { color: red; } }",
        "@container style((foo bar)) { .item { color: red; } }",
    ] {
        let tree = parse(css);
        let sexp = tree.root_node().to_sexp();
        assert!(!tree.root_node().has_error(), "{sexp}");
        assert!(
            sexp.contains("general_enclosed_function") || sexp.contains("general_enclosed_parens"),
            "{sexp}"
        );
    }
}

#[test]
fn parses_mixed_boolean_container_queries() {
    let tree = parse(
        "@container (width > 400px) and (width > 800px) or (orientation: portrait) { .item { color: red; } }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("container_or_condition"), "{sexp}");
    assert!(sexp.contains("container_and_condition"), "{sexp}");
}

#[test]
fn parses_calc_bounded_container_ranges() {
    let tree = parse(
        "@container test-container (calc(400px + 1px) <= width < calc(500px + 1px)) { .item { color: purple; } }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("query_feature_range"), "{sexp}");
    assert!(sexp.contains("call_expression"), "{sexp}");
}

#[test]
fn parses_scope_rule_with_relative_selector() {
    let tree = parse("@scope (.card) to (.card > *) { & .title { color: red; } }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("scope_statement"), "{sexp}");
}

#[test]
fn parses_scope_rule_with_to_only_limit() {
    let tree = parse("@scope to (.card > *) { .title { color: red; } }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("scope_query"), "{sexp}");
    assert!(sexp.contains("scope_end"), "{sexp}");
}

#[test]
fn rejects_pseudo_elements_in_scope_prelude() {
    for css in [
        "@scope (::before) { .x { color: red; } }",
        "@scope (.card) to (::before) { .x { color: red; } }",
    ] {
        let tree = parse(css);
        assert!(
            tree.root_node().has_error(),
            "{}",
            tree.root_node().to_sexp()
        );
    }
}

#[test]
fn rejects_top_level_nesting_selector_in_scope_prelude() {
    let tree = parse("@scope (& > .scope) { .x { color: red; } }");
    assert!(
        tree.root_node().has_error(),
        "{}",
        tree.root_node().to_sexp()
    );
}

#[test]
fn parses_nested_scope_prelude_with_nesting_selector() {
    let tree = parse(".parent { @scope (& > .scope) { .x { color: red; } } }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("scope_statement"), "{sexp}");
    assert!(sexp.contains("nesting_selector"), "{sexp}");
}

#[test]
fn parses_layer_and_keyframes_metadata() {
    let tree = parse(
        "@layer theme.components { @keyframes fade { from { opacity: 0; } to { opacity: 1; } } }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("layer_statement"), "{sexp}");
    assert!(sexp.contains("keyframes_statement"), "{sexp}");
}

#[test]
fn rejects_comma_separated_layer_names_in_block_form() {
    let tree = parse("@layer theme, components { .x { color: red; } }");
    assert!(
        tree.root_node().has_error(),
        "{}",
        tree.root_node().to_sexp()
    );
}

#[test]
fn parses_column_combinator_and_attribute_flags() {
    let tree = parse(".table || .cell[data-kind='cta' i][data-mode='compact' s] { color: red; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("column_selector"), "{sexp}");
    assert!(sexp.contains("attribute_flags"), "{sexp}");
}

#[test]
fn rejects_invalid_attribute_flags() {
    let tree = parse(".cell[data-kind='cta' banana] { color: red; }");
    assert!(
        tree.root_node().has_error(),
        "{}",
        tree.root_node().to_sexp()
    );
}

#[test]
fn rejects_invalid_attribute_flags_in_restricted_selector_contexts() {
    for css in [
        "a:has([data-kind='cta' banana]) { color: red; }",
        "@scope ([data-mode='x' banana]) { .x { color: red; } }",
    ] {
        let tree = parse(css);
        assert!(
            tree.root_node().has_error(),
            "{}",
            tree.root_node().to_sexp()
        );
    }
}

#[test]
fn parses_unicode_and_escaped_selectors() {
    let tree = parse(".fö, #\\31 23, .a\\1f642 b { color: green; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("class_selector"), "{sexp}");
    assert!(sexp.contains("id_selector"), "{sexp}");
}

#[test]
fn parses_non_latin_selectors_and_custom_properties() {
    let tree = parse(".你好 { --变量: 1px; color: red; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("class_selector"), "{sexp}");
    assert!(sexp.contains("property_name"), "{sexp}");
}

#[test]
fn parses_nth_child_of_selector_list() {
    let tree = parse("li:nth-child(2n of .a, .b) { color: red; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("pseudo_class_selector"), "{sexp}");
    assert!(sexp.contains("selectors"), "{sexp}");
}

#[test]
fn parses_import_layer_supports_and_media_chain() {
    let tree = parse(
        "@import url(\"theme.css\") layer(theme) supports(display: grid) screen and (width >= 40rem);",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("import_statement"), "{sexp}");
    assert!(sexp.contains("import_conditions"), "{sexp}");
    assert!(sexp.contains("import_layer"), "{sexp}");
    assert!(sexp.contains("import_supports"), "{sexp}");
    assert!(sexp.contains("media_query_list"), "{sexp}");
}

#[test]
fn parses_custom_media_rules() {
    for css in [
        "@custom-media --narrow-window (max-width: 30em); @media (--narrow-window) { .x { color: red; } }",
        "@custom-media --supports-hover true; @media (--supports-hover) { .x { color: red; } }",
        "@custom-media --supports-hover false; @media (--supports-hover) { .x { color: red; } }",
    ] {
        let tree = parse(css);
        let sexp = tree.root_node().to_sexp();
        assert!(!tree.root_node().has_error(), "{sexp}");
        assert!(sexp.contains("custom_media_statement"), "{sexp}");
    }
}

#[test]
fn parses_unicode_range_values() {
    let tree = parse("a { unicode-range: U+416, U+00??, U+4E00-9FFF, u+??????; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("unicode_range_value"), "{sexp}");
}

#[test]
fn parses_url_modifiers() {
    let tree = parse(
        ".icon { background-image: url(\"sprite.svg\" type(\"image/svg+xml\") cors); mask-image: url(\"mask.svg\" format(svg)); }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("url_with_modifiers"), "{sexp}");
    assert!(sexp.contains("url_modifier"), "{sexp}");
}

#[test]
fn recovers_bad_url_values() {
    let tree = parse(
        ".icon { background-image: url(foo\"bar); color: red; mask-image: url(\"mask.svg\" type(\"image/svg+xml\") \"oops\"); display: block; }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(sexp.contains("bad_url_value"), "{sexp}");
    assert!(
        sexp.matches("(declaration property:").count() >= 4,
        "{sexp}"
    );
}

#[test]
fn parses_webkit_autofill_alias() {
    let tree = parse("input:-webkit-autofill { color: red; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("pseudo_class_selector"), "{sexp}");
}

#[test]
fn parses_nth_last_child_of_selector_list() {
    let tree = parse("li:nth-last-child(odd of .featured, .fallback) { color: red; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("pseudo_class_selector"), "{sexp}");
    assert!(sexp.contains("selectors"), "{sexp}");
}

#[test]
fn parses_typed_nth_pseudo_classes() {
    let tree = parse(
        "li:nth-of-type(2n+1) { color: red; } td:nth-last-of-type(odd) { color: blue; } col:nth-col(3n) { color: green; } col:nth-last-col(+n) { color: black; }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("pseudo_class_selector"), "{sexp}");
    assert!(sexp.contains("plain_value"), "{sexp}");
}

#[test]
fn rejects_of_clauses_in_typed_nth_pseudo_classes() {
    for css in [
        "li:nth-of-type(2n of .item) { color: red; }",
        "li:nth-last-of-type(odd of .item) { color: red; }",
        "col:nth-col(2n of .item) { color: red; }",
        "col:nth-last-col(2n of .item) { color: red; }",
    ] {
        let tree = parse(css);
        assert!(
            tree.root_node().has_error(),
            "{}",
            tree.root_node().to_sexp()
        );
    }
}

#[test]
fn parses_extended_nth_formulas() {
    let tree = parse(
        "li:nth-child(2n-1 of .item) { color: red; } li:nth-child(+n) { color: blue; } li:nth-last-child(-n+3 of .item, .other) { color: green; } li:nth-last-child(3N-2 of .hero) { color: black; }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("pseudo_class_selector"), "{sexp}");
}

#[test]
fn rejects_pseudo_elements_in_nth_of_clauses() {
    for css in [
        "li:nth-child(2n of ::before, .item) { color: red; }",
        "li:nth-last-child(odd of ::after, .item) { color: red; }",
    ] {
        let tree = parse(css);
        assert!(
            tree.root_node().has_error(),
            "{}",
            tree.root_node().to_sexp()
        );
    }
}

#[test]
fn parses_var_empty_fallback_and_nested_empty_fallback() {
    let tree = parse(".x { color: var(--brand,); background: var(--a, var(--b,)); }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("call_expression"), "{sexp}");
}

#[test]
fn parses_empty_custom_property_values() {
    let tree = parse(".x { --foo: ; --bar: !important; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("property_name"), "{sexp}");
}

#[test]
fn parses_decimal_keyframe_offsets() {
    let tree = parse("@keyframes fade { 12.5% { opacity: 0; } 87.5% { opacity: 1; } }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("keyframe_block"), "{sexp}");
}

#[test]
fn parses_nesting_selector_in_compound_positions() {
    let tree = parse(".x { &div { color: green; } div& { color: blue; } &:hover { color: red; } }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("nesting_selector"), "{sexp}");
    assert!(sexp.contains("tag_name"), "{sexp}");
}

#[test]
fn parses_strings_with_braces_brackets_and_semicolons() {
    let tree = parse("[foo='{;}'] { content: \"{};[]\"; color: red; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("attribute_selector"), "{sexp}");
    assert!(sexp.contains("declaration"), "{sexp}");
}

#[test]
fn parses_unquoted_url_values_with_schemes_and_trailing_slashes() {
    let tree = parse(
        "@import url(chrome://communicator/skin/); .x { background: url(https://example.com/a/b/); }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("call_expression"), "{sexp}");
}

#[test]
fn parses_unquoted_url_values_with_escapes() {
    let tree = parse(r#".x { background: url(foo\)bar); mask-image: url(icon\ 2.svg); }"#);
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("url_unquoted_value"), "{sexp}");
    assert!(sexp.contains("escape_sequence"), "{sexp}");
}

#[test]
fn parses_scientific_notation_values_and_keyframe_offsets() {
    let tree = parse(
        ".x { width: 1e+3px; opacity: .5e+2; } @keyframes fade { 1e+1% { opacity: 0; } 2.5e+1% { opacity: 1; } }",
    );
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("float_value"), "{sexp}");
    assert!(sexp.contains("keyframe_block"), "{sexp}");
}

#[test]
fn parses_has_relative_selector_lists() {
    let tree =
        parse("a:has(> img, + dt, .foo, :not(.bar), :nth-child(2n of .item)) { color: red; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("pseudo_class_selector"), "{sexp}");
    assert!(sexp.contains("child_selector"), "{sexp}");
    assert!(sexp.contains("adjacent_sibling_selector"), "{sexp}");
}

#[test]
fn rejects_pseudo_elements_inside_has() {
    let tree = parse("a:has(::before) { color: red; }");
    assert!(
        tree.root_node().has_error(),
        "{}",
        tree.root_node().to_sexp()
    );
}

#[test]
fn rejects_nested_has_inside_has() {
    let tree = parse("a:has(:has(.x)) { color: red; }");
    assert!(
        tree.root_node().has_error(),
        "{}",
        tree.root_node().to_sexp()
    );
}

#[test]
fn forgives_pseudo_elements_inside_is_and_where() {
    for css in [
        ".x:is(::before, .a) { color: red; }",
        ".x:where(::before, .a) { color: red; }",
        ".x:matches(::before, .a) { color: red; }",
        ".x:is(::slotted(.foo), .a) { color: red; }",
    ] {
        let tree = parse(css);
        let sexp = tree.root_node().to_sexp();
        assert!(!tree.root_node().has_error(), "{sexp}");
        assert!(sexp.contains("forgiving_pseudo_element_recovery"), "{sexp}");
    }
}

#[test]
fn parses_legacy_single_colon_pseudo_elements() {
    let tree = parse("a:before, p:first-line, span:first-letter, div:after { color: red; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("pseudo_element_selector"), "{sexp}");
}

#[test]
fn rejects_pseudo_elements_inside_not() {
    let tree = parse(".x:not(::before) { color: red; }");
    assert!(
        tree.root_node().has_error(),
        "{}",
        tree.root_node().to_sexp()
    );
}

#[test]
fn parses_grapheme_cluster_selector_names() {
    let tree = parse(".Cafe\u{301}, .👩‍🚀 { color: blue; }");
    let sexp = tree.root_node().to_sexp();
    assert!(!tree.root_node().has_error(), "{sexp}");
    assert!(sexp.contains("class_selector"), "{sexp}");
}
