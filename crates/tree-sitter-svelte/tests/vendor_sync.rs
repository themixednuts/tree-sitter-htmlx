use std::fs;
use std::path::PathBuf;

const VENDORED_HEADER: &str = "/**\n * Auto-vendored during build. Do not edit manually.\n */\n\n";

#[test]
fn test_vendored_htmlx_scanner_matches_canonical_source() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let canonical = crate_root.join("../tree-sitter-htmlx/src/scanner.c");

    // Standalone clones may not include sibling workspace crates.
    if !canonical.exists() {
        return;
    }

    let canonical_content = fs::read_to_string(&canonical)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", canonical.display(), e));
    let vendored = crate_root.join("src/htmlx/scanner.c");
    let vendored_content = fs::read_to_string(&vendored)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", vendored.display(), e));

    let expected = format!("{}{}", VENDORED_HEADER, canonical_content);
    assert_eq!(
        vendored_content,
        expected,
        "Vendored HTMLX scanner is out of sync with canonical source.\n\
         Update {} from {}.",
        vendored.display(),
        canonical.display()
    );
}
