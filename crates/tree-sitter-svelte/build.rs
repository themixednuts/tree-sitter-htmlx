//! Build script for tree-sitter-svelte
//! 
//! Runs tree-sitter generate and compiles the parser.

use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&manifest_dir);
    let src_dir = manifest_path.join("src");
    
    // Rerun if grammar, scanner, or queries change
    println!("cargo:rerun-if-changed=grammar.js");
    println!("cargo:rerun-if-changed=src/scanner.c");
    println!("cargo:rerun-if-changed=queries");
    // Also rerun if HTMLX changes (since we extend it)
    println!("cargo:rerun-if-changed=../tree-sitter-htmlx/grammar.js");
    println!("cargo:rerun-if-changed=../tree-sitter-htmlx/src/scanner.c");
    println!("cargo:rerun-if-changed=../tree-sitter-htmlx/queries");

    // Run tree-sitter generate
    let status = Command::new("tree-sitter")
        .arg("generate")
        .current_dir(&manifest_dir)
        .status()
        .expect(
            "Failed to run 'tree-sitter generate'. \
             Please install tree-sitter-cli: cargo install tree-sitter-cli"
        );

    if !status.success() {
        panic!("tree-sitter generate failed with status: {}", status);
    }

    // Compile the generated parser and our scanner
    // Include paths for HTML scanner's tag.h and HTMLX scanner
    let html_src_dir = manifest_path.join("../../external/tree-sitter-html/src");
    let htmlx_src_dir = manifest_path.join("../tree-sitter-htmlx/src");

    cc::Build::new()
        .include(&src_dir)
        .include(&html_src_dir)
        .include(&htmlx_src_dir)
        .file(src_dir.join("parser.c"))
        .file(src_dir.join("scanner.c"))
        .warnings(false)  // Suppress warnings from generated code
        .compile("tree_sitter_svelte");
}
