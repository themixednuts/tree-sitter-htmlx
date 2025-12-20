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

/// The content of the [`node-types.json`] file for HTMLX.
pub const NODE_TYPES: &str = include_str!("../src/node-types.json");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).expect("Failed to load HTMLX grammar");
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
}
