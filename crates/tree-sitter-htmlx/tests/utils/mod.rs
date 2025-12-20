//! Shared test utilities for tree-sitter-htmlx tests

use tree_sitter_htmlx::LANGUAGE;

/// Parse an HTMLX source string and return the S-expression representation
pub fn parse(source: &str) -> String {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&LANGUAGE.into())
        .expect("Failed to load HTMLX grammar");

    let tree = parser.parse(source, None).expect("Failed to parse");
    tree.root_node().to_sexp()
}
