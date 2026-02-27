//! Build script for tree-sitter-svelte.
//!
//! Keep this crate self-contained for crates.io/git/path consumers: compile
//! committed C sources only and do not read sibling workspace crates.

fn main() {
    println!("cargo:rerun-if-changed=src/parser.c");
    println!("cargo:rerun-if-changed=src/scanner.c");
    println!("cargo:rerun-if-changed=src/htmlx/scanner.c");
    println!("cargo:rerun-if-changed=src/htmlx/html/scanner.c");
    println!("cargo:rerun-if-changed=src/htmlx/html/tag.h");

    cc::Build::new()
        .include("src")
        .file("src/parser.c")
        .file("src/scanner.c")
        .warnings(false)
        .compile("tree_sitter_svelte");
}
