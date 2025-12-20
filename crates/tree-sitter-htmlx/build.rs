//! Build script for tree-sitter-htmlx
//!
//! Vendors HTML scanner and queries from tree-sitter-html crate and compiles the parser.

use std::fs;
use std::path::Path;
use std::process::Command;

const VENDOR_HEADER_C: &str = r#"/**
 * Vendored from tree-sitter-html
 *
 * This file is auto-generated during build. Do not edit manually.
 */

"#;

const VENDOR_HEADER_SCM: &str = r#"; Vendored from tree-sitter-html
;
; This file is auto-generated during build. Do not edit manually.

"#;

fn vendor_file(src: &Path, dst: &Path, header: &str) {
    if src.exists() {
        let content = fs::read_to_string(src)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", src.display(), e));
        fs::write(dst, format!("{}{}", header, content))
            .unwrap_or_else(|e| panic!("Failed to write {}: {}", dst.display(), e));
    }
}

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&manifest_dir);
    let src_dir = manifest_path.join("src");
    let html_vendor_dir = src_dir.join("html");
    let queries_dir = manifest_path.join("queries");
    let html_queries_dir = queries_dir.join("html");

    // Source from tree-sitter-html crate in workspace
    let html_crate = manifest_path.join("../tree-sitter-html");
    let html_crate_src = html_crate.join("src");
    let html_crate_queries = html_crate.join("queries");

    // Vendor files from tree-sitter-html crate
    if html_crate_src.exists() {
        // Vendor C source files
        fs::create_dir_all(&html_vendor_dir).expect("Failed to create html vendor directory");
        vendor_file(
            &html_crate_src.join("tag.h"),
            &html_vendor_dir.join("tag.h"),
            VENDOR_HEADER_C,
        );
        vendor_file(
            &html_crate_src.join("scanner.c"),
            &html_vendor_dir.join("scanner.c"),
            VENDOR_HEADER_C,
        );

        // Vendor query files
        if html_crate_queries.exists() {
            fs::create_dir_all(&html_queries_dir).expect("Failed to create html queries directory");
            vendor_file(
                &html_crate_queries.join("highlights.scm"),
                &html_queries_dir.join("highlights.scm"),
                VENDOR_HEADER_SCM,
            );
            if html_crate_queries.join("injections.scm").exists() {
                vendor_file(
                    &html_crate_queries.join("injections.scm"),
                    &html_queries_dir.join("injections.scm"),
                    VENDOR_HEADER_SCM,
                );
            }
        }

        println!("cargo:rerun-if-changed={}", html_crate.display());
    }

    // Verify vendored files exist
    if !html_vendor_dir.join("scanner.c").exists() {
        panic!(
            "Vendored HTML scanner not found at {:?}. \
             Ensure tree-sitter-html crate exists in the workspace.",
            html_vendor_dir
        );
    }

    // Rerun if grammar, scanner, or queries change
    println!("cargo:rerun-if-changed=grammar.js");
    println!("cargo:rerun-if-changed=src/scanner.c");
    println!("cargo:rerun-if-changed=src/html");
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
        .compile("tree_sitter_htmlx");
}
