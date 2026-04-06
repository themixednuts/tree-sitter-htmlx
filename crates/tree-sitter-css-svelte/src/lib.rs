//! Tree-sitter grammar for CSS with Svelte-oriented recovery.

use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_css() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for CSS.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_css) };

/// The tree-sitter language for CSS.
pub fn language() -> tree_sitter::Language {
    LANGUAGE.into()
}

/// Scanner profiling counters exposed by the opt-in `TREE_SITTER_CSS_PROFILE`
/// build flag. When profiling is disabled, the exported functions still exist
/// and return zeroed counters.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ScannerProfileStats {
    pub scan_calls: u64,
    pub scan_at_rule_prelude_calls: u64,
    pub scan_at_rule_prelude_successes: u64,
    pub scan_at_rule_prelude_bytes: u64,
    pub scan_descendant_operator_calls: u64,
    pub scan_descendant_operator_successes: u64,
    pub scan_descendant_operator_bytes: u64,
    pub scan_pseudo_class_colon_calls: u64,
    pub scan_pseudo_class_colon_successes: u64,
    pub scan_pseudo_class_colon_bytes: u64,
    pub scan_forgiving_pseudo_element_calls: u64,
    pub scan_forgiving_pseudo_element_successes: u64,
    pub scan_forgiving_pseudo_element_bytes: u64,
}

extern "C" {
    fn tree_sitter_css_profile_enabled() -> bool;
    fn tree_sitter_css_profile_reset();
    fn tree_sitter_css_profile_snapshot(out: *mut ScannerProfileStats);
}

pub fn scanner_profile_enabled() -> bool {
    unsafe { tree_sitter_css_profile_enabled() }
}

pub fn reset_scanner_profile() {
    unsafe { tree_sitter_css_profile_reset() }
}

pub fn scanner_profile_stats() -> ScannerProfileStats {
    let mut stats = ScannerProfileStats::default();
    unsafe {
        tree_sitter_css_profile_snapshot(&mut stats);
    }
    stats
}

/// The syntax highlighting query for CSS.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The content of the node-types.json file for CSS.
pub const NODE_TYPES: &str = include_str!("node-types.json");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&LANGUAGE.into())
            .expect("Error loading CSS parser");
    }
}
