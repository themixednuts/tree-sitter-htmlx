//! Runs tree-sitter highlight tests from test/highlight/

use std::process::Command;

#[test]
fn test_highlights() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let output = Command::new("tree-sitter")
        .args(["test", "--include", "highlight"])
        .current_dir(manifest_dir)
        .output()
        .expect("Failed to run tree-sitter test. Is tree-sitter-cli installed?");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!(
            "Highlight tests failed!\n\nstdout:\n{}\n\nstderr:\n{}",
            stdout, stderr
        );
    }

    // Print output for visibility
    if !stdout.is_empty() {
        println!("{}", stdout);
    }
}
