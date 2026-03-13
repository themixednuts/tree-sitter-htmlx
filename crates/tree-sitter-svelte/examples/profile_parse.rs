use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use tree_sitter_htmlx_svelte::{
    language, reset_scanner_profile, scanner_profile_enabled, scanner_profile_stats,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os();
    let _bin = args.next();

    let path = args
        .next()
        .map(PathBuf::from)
        .ok_or("usage: cargo run -p tree-sitter-htmlx-svelte --example profile_parse -- <path> [repeat]")?;
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
        ("svelte_scan_calls", stats.svelte_scan_calls),
        ("htmlx_fallback_calls", stats.htmlx_fallback_calls),
        ("scan_lt_as_tag_boundary_calls", stats.scan_lt_as_tag_boundary_calls),
        (
            "scan_lt_as_tag_boundary_successes",
            stats.scan_lt_as_tag_boundary_successes,
        ),
        ("scan_lt_as_tag_boundary_bytes", stats.scan_lt_as_tag_boundary_bytes),
        ("scan_balanced_calls", stats.scan_balanced_calls),
        ("scan_balanced_successes", stats.scan_balanced_successes),
        ("scan_balanced_bytes", stats.scan_balanced_bytes),
        ("scan_iterator_calls", stats.scan_iterator_calls),
        ("scan_iterator_successes", stats.scan_iterator_successes),
        ("scan_iterator_bytes", stats.scan_iterator_bytes),
        ("scan_binding_calls", stats.scan_binding_calls),
        ("scan_binding_successes", stats.scan_binding_successes),
        ("scan_key_calls", stats.scan_key_calls),
        ("scan_key_successes", stats.scan_key_successes),
        ("scan_tag_expression_calls", stats.scan_tag_expression_calls),
        ("scan_tag_expression_successes", stats.scan_tag_expression_successes),
        ("scan_tag_expression_bytes", stats.scan_tag_expression_bytes),
        ("scan_snippet_parameter_calls", stats.scan_snippet_parameter_calls),
        (
            "scan_snippet_parameter_successes",
            stats.scan_snippet_parameter_successes,
        ),
        (
            "scan_snippet_type_params_calls",
            stats.scan_snippet_type_params_calls,
        ),
        (
            "scan_snippet_type_params_successes",
            stats.scan_snippet_type_params_successes,
        ),
        ("scan_snippet_type_params_bytes", stats.scan_snippet_type_params_bytes),
        ("scan_snippet_name_calls", stats.scan_snippet_name_calls),
        ("scan_snippet_name_successes", stats.scan_snippet_name_successes),
        ("scan_snippet_name_bytes", stats.scan_snippet_name_bytes),
        ("scan_block_end_open_calls", stats.scan_block_end_open_calls),
        ("scan_block_end_open_successes", stats.scan_block_end_open_successes),
        ("scan_block_end_open_bytes", stats.scan_block_end_open_bytes),
    ] {
        if value != 0 {
            println!("{name}: {value}");
        }
    }

    Ok(())
}
