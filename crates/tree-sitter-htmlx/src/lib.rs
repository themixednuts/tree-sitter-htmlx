//! Tree-sitter grammar for HTMLX (expression-enhanced HTML)
//!
//! HTMLX extends HTML with expression syntax commonly used in modern
//! frontend frameworks like Svelte, Vue, and others.
//!
//! ## Features
//!
//! - Expression interpolation: `{expression}`
//! - Shorthand attributes: `{name}` (equivalent to `name={name}`)
//! - Spread attributes: `{...props}`
//! - Directive attributes: `bind:value`, `on:click`, `class:active`, etc.
//!
//! ## Example
//!
//! ```rust
//! use tree_sitter_htmlx::LANGUAGE;
//!
//! let mut parser = tree_sitter::Parser::new();
//! parser.set_language(&LANGUAGE.into()).expect("Failed to load HTMLX grammar");
//!
//! let source = r#"<div class="container" {hidden}>
//!   <p>{greeting}, {name}!</p>
//!   <button onclick={handleClick}>Click me</button>
//! </div>"#;
//!
//! let tree = parser.parse(source, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```

use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_htmlx() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for HTMLX.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_htmlx) };

/// The tree-sitter language for HTMLX.
pub fn language() -> tree_sitter::Language {
    LANGUAGE.into()
}

/// The syntax highlighting query for HTMLX.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injection query for HTMLX.
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The folding query for HTMLX.
pub const FOLDS_QUERY: &str = include_str!("../queries/folds.scm");

/// The indentation query for HTMLX.
pub const INDENTS_QUERY: &str = include_str!("../queries/indents.scm");

/// The locals query for HTMLX.
pub const LOCALS_QUERY: &str = include_str!("../queries/locals.scm");

/// The content of the [`node-types.json`] file for HTMLX.
pub const NODE_TYPES: &str = include_str!("../src/node-types.json");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&LANGUAGE.into())
            .expect("Failed to load HTMLX grammar");
    }

    #[test]
    fn test_parse_simple_html() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = "<div>Hello</div>";
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
        assert_eq!(tree.root_node().kind(), "document");
    }

    #[test]
    fn test_parse_expression() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = "<div>{name}</div>";
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_expression_attribute() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<input value={text} />"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_shorthand_attribute() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<div {hidden} {id}></div>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_spread_attribute() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<Component {...props} />"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_directive_attribute() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<input bind:value={name} on:input={handleInput} />"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_nested_expressions() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<div>{items.map(item => item.name)}</div>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_highlights_query_includes_html_captures() {
        let language = language();
        let query = tree_sitter::Query::new(&language, HIGHLIGHTS_QUERY)
            .expect("HTMLX highlights query should compile");
        let captures = query.capture_names();

        for capture in ["tag", "attribute", "string", "punctuation.bracket"] {
            assert!(
                captures.iter().any(|name| *name == capture),
                "missing @{capture} from HTMLX highlights query"
            );
        }
    }

    #[test]
    fn test_editor_queries_compile() {
        let language = language();

        for (name, source, expected_capture) in [
            ("highlights", HIGHLIGHTS_QUERY, "tag"),
            ("injections", INJECTIONS_QUERY, "injection.content"),
            ("folds", FOLDS_QUERY, "fold"),
            (
                "html folds",
                include_str!("../queries/html/folds.scm"),
                "fold",
            ),
            ("indents", INDENTS_QUERY, "indent.begin"),
            (
                "html indents",
                include_str!("../queries/html/indents.scm"),
                "indent.begin",
            ),
            ("locals", LOCALS_QUERY, "local.scope"),
            (
                "html locals",
                include_str!("../queries/html/locals.scm"),
                "local.scope",
            ),
        ] {
            let query = tree_sitter::Query::new(&language, source)
                .unwrap_or_else(|error| panic!("{name} query should compile: {error}"));
            assert!(
                query
                    .capture_names()
                    .iter()
                    .any(|capture| *capture == expected_capture),
                "missing @{expected_capture} from HTMLX {name} query"
            );
        }
    }
}
