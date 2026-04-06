use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use tree_sitter_css_svelte::{
    language, reset_scanner_profile, scanner_profile_enabled, scanner_profile_stats,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os();
    let _bin = args.next();

    let path = args.next().map(PathBuf::from).ok_or(
        "usage: cargo run -p tree-sitter-css-svelte --example profile_parse -- <path> [repeat]",
    )?;
    let repeat = args
        .next()
        .map(|value| value.to_string_lossy().parse::<usize>())
        .transpose()?
        .unwrap_or(1);

    let source = fs::read_to_string(&path)?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language())?;

    reset_scanner_profile();

    let started = Instant::now();
    let mut last_tree = None;
    for _ in 0..repeat {
        last_tree = parser.parse(source.as_str(), None);
    }
    let elapsed = started.elapsed();

    let tree = last_tree.ok_or("parse returned no tree")?;
    let stats = scanner_profile_stats();

    println!("path: {}", path.display());
    println!("bytes: {}", source.len());
    println!("repeats: {repeat}");
    println!("profile_enabled: {}", scanner_profile_enabled());
    println!("has_error: {}", tree.root_node().has_error());
    println!("elapsed_ms: {:.2}", elapsed.as_secs_f64() * 1000.0);
    println!(
        "bytes_per_ms: {:.2}",
        source.len() as f64 * repeat as f64 / (elapsed.as_secs_f64() * 1000.0)
    );

    for (name, value) in [
        ("scan_calls", stats.scan_calls),
        (
            "scan_at_rule_prelude_calls",
            stats.scan_at_rule_prelude_calls,
        ),
        (
            "scan_at_rule_prelude_successes",
            stats.scan_at_rule_prelude_successes,
        ),
        (
            "scan_at_rule_prelude_bytes",
            stats.scan_at_rule_prelude_bytes,
        ),
        (
            "scan_descendant_operator_calls",
            stats.scan_descendant_operator_calls,
        ),
        (
            "scan_descendant_operator_successes",
            stats.scan_descendant_operator_successes,
        ),
        (
            "scan_descendant_operator_bytes",
            stats.scan_descendant_operator_bytes,
        ),
        (
            "scan_pseudo_class_colon_calls",
            stats.scan_pseudo_class_colon_calls,
        ),
        (
            "scan_pseudo_class_colon_successes",
            stats.scan_pseudo_class_colon_successes,
        ),
        (
            "scan_pseudo_class_colon_bytes",
            stats.scan_pseudo_class_colon_bytes,
        ),
        (
            "scan_forgiving_pseudo_element_calls",
            stats.scan_forgiving_pseudo_element_calls,
        ),
        (
            "scan_forgiving_pseudo_element_successes",
            stats.scan_forgiving_pseudo_element_successes,
        ),
        (
            "scan_forgiving_pseudo_element_bytes",
            stats.scan_forgiving_pseudo_element_bytes,
        ),
    ] {
        if value != 0 {
            println!("{name}: {value}");
        }
    }

    Ok(())
}
