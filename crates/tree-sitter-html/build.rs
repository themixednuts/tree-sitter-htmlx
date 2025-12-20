//! Build script for tree-sitter-html
//!
//! Compiles the HTML parser following the WHATWG HTML Living Standard.

use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&manifest_dir);
    let src_dir = manifest_path.join("src");

    // Rerun if grammar, scanner, or queries change
    println!("cargo:rerun-if-changed=grammar.js");
    println!("cargo:rerun-if-changed=src/scanner.c");
    println!("cargo:rerun-if-changed=src/tag.h");
    println!("cargo:rerun-if-changed=queries");

    // Run tree-sitter generate
    let status = Command::new("tree-sitter")
        .arg("generate")
        .current_dir(&manifest_dir)
        .status()
        .expect(
            "Failed to run 'tree-sitter generate'. \
             Please install tree-sitter-cli: cargo install tree-sitter-cli",
        );

    if !status.success() {
        panic!("tree-sitter generate failed with status: {}", status);
    }

    // Compile the parser and scanner
    cc::Build::new()
        .include(&src_dir)
        .file(src_dir.join("parser.c"))
        .file(src_dir.join("scanner.c"))
        .warnings(false)
        .compile("tree_sitter_html");
}
