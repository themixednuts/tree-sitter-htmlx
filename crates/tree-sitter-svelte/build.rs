//! Build script for tree-sitter-svelte
//!
//! Vendors HTMLX and HTML scanners/queries from workspace crates and compiles the parser.

use std::fs;
use std::path::Path;
use std::process::Command;

const VENDOR_HEADER_C: &str = r#"/**
 * Auto-vendored during build. Do not edit manually.
 */

"#;

const VENDOR_HEADER_SCM: &str = r#"; Auto-vendored during build. Do not edit manually.

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
    let queries_dir = manifest_path.join("queries");

    // Vendor directories
    let htmlx_vendor_dir = src_dir.join("htmlx");
    let html_vendor_dir = htmlx_vendor_dir.join("html");
    let htmlx_queries_dir = queries_dir.join("htmlx");
    let html_queries_dir = htmlx_queries_dir.join("html");

    // Source paths from workspace crates
    let htmlx_crate = manifest_path.join("../tree-sitter-htmlx");
    let htmlx_src_dir = htmlx_crate.join("src");
    let htmlx_queries = htmlx_crate.join("queries");
    let html_crate = manifest_path.join("../tree-sitter-html");
    let html_src_dir = html_crate.join("src");
    let html_crate_queries = html_crate.join("queries");

    // Vendor files if sources exist
    if htmlx_src_dir.exists() && html_src_dir.exists() {
        // Create vendor directories
        fs::create_dir_all(&html_vendor_dir).expect("Failed to create vendor directories");
        fs::create_dir_all(&html_queries_dir).expect("Failed to create query directories");

        // Vendor HTML scanner files into htmlx/html/
        vendor_file(
            &html_src_dir.join("tag.h"),
            &html_vendor_dir.join("tag.h"),
            VENDOR_HEADER_C,
        );
        vendor_file(
            &html_src_dir.join("scanner.c"),
            &html_vendor_dir.join("scanner.c"),
            VENDOR_HEADER_C,
        );

        // Vendor HTMLX scanner
        vendor_file(
            &htmlx_src_dir.join("scanner.c"),
            &htmlx_vendor_dir.join("scanner.c"),
            VENDOR_HEADER_C,
        );

        // Vendor HTML query files
        if html_crate_queries.exists() {
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

        // Vendor HTMLX query files
        if htmlx_queries.exists() {
            vendor_file(
                &htmlx_queries.join("highlights.scm"),
                &htmlx_queries_dir.join("highlights.scm"),
                VENDOR_HEADER_SCM,
            );
            vendor_file(
                &htmlx_queries.join("injections.scm"),
                &htmlx_queries_dir.join("injections.scm"),
                VENDOR_HEADER_SCM,
            );
        }

        println!("cargo:rerun-if-changed={}", htmlx_src_dir.display());
        println!("cargo:rerun-if-changed={}", htmlx_queries.display());
        println!("cargo:rerun-if-changed={}", html_crate.display());
    }

    // Verify vendored files exist
    if !htmlx_vendor_dir.join("scanner.c").exists() {
        panic!(
            "Vendored HTMLX scanner not found at {:?}. \
             Ensure workspace crates exist.",
            htmlx_vendor_dir
        );
    }

    // Rerun triggers
    println!("cargo:rerun-if-changed=grammar.js");
    println!("cargo:rerun-if-changed=src/scanner.c");
    println!("cargo:rerun-if-changed=src/htmlx");
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
        .compile("tree_sitter_svelte");
}
