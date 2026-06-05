//! Parser coverage for the official Svelte VS Code grammar samples.
//!
//! The samples and TextMate snapshots are vendored unchanged from
//! `sveltejs/language-tools`. This test intentionally checks parser health only:
//! the snapshots remain available as a reference for editor-specific highlight
//! assertions without coupling this grammar crate to a specific editor's capture
//! names.

use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter_htmlx_svelte::LANGUAGE;

fn collect_svelte_inputs(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
    {
        let entry = entry.unwrap_or_else(|error| {
            panic!("failed to read entry in {}: {error}", directory.display())
        });
        let path = entry.path();

        if path.is_dir() {
            collect_svelte_inputs(&path, paths);
        } else if path.file_name().and_then(|name| name.to_str()) == Some("input.svelte") {
            paths.push(path);
        }
    }
}

fn find_parse_errors(node: tree_sitter::Node, source: &str, errors: &mut Vec<String>) {
    if node.is_error() || node.is_missing() {
        let start = node.start_position();
        let end = node.end_position();
        let text = &source[node.start_byte()..node.end_byte().min(source.len())];
        errors.push(format!(
            "  {} at {}:{}-{}:{}: {:?}",
            if node.is_error() { "ERROR" } else { "MISSING" },
            start.row + 1,
            start.column,
            end.row + 1,
            end.column,
            text
        ));
    }

    for child in node.children(&mut node.walk()) {
        find_parse_errors(child, source, errors);
    }
}

fn assert_parses_without_error(path: &Path) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&LANGUAGE.into())
        .expect("failed to load Svelte grammar");

    let tree = parser.parse(&source, None).expect("failed to parse source");
    let root = tree.root_node();

    if root.has_error() {
        let mut errors = Vec::new();
        find_parse_errors(root, &source, &mut errors);
        panic!(
            "parse errors in {}:\n{}\n\nSource:\n{}",
            path.display(),
            errors.join("\n"),
            source
        );
    }
}

#[test]
fn parses_official_svelte_vscode_grammar_samples() {
    let samples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/svelte-vscode-grammar/samples");
    let mut paths = Vec::new();
    collect_svelte_inputs(&samples_dir, &mut paths);
    paths.sort();

    assert!(
        !paths.is_empty(),
        "expected vendored Svelte VS Code grammar samples under {}",
        samples_dir.display()
    );

    for path in paths {
        assert_parses_without_error(&path);
    }
}
