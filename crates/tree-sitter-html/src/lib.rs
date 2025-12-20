//! Tree-sitter grammar for HTML following the WHATWG HTML Living Standard
//!
//! This grammar provides spec-compliant HTML parsing including:
//!
//! - **Void elements** (§13.1.2): area, base, br, col, embed, hr, img, input, link, meta, source, track, wbr
//! - **Raw text elements** (§13.1.2.1): script, style
//! - **Escapable raw text elements** (§13.1.2.2): textarea, title
//! - **Optional end tags** (§13.1.2.4): Proper implicit closing
//! - **Character references** (§13.5): Named, decimal, and hex entities
//!
//! ## Example
//!
//! ```rust
//! use tree_sitter_html::LANGUAGE;
//!
//! let mut parser = tree_sitter::Parser::new();
//! parser.set_language(&LANGUAGE.into()).expect("Failed to load HTML grammar");
//!
//! let source = r#"<!DOCTYPE html>
//! <html>
//! <head>
//!   <title>Hello World</title>
//! </head>
//! <body>
//!   <p>Welcome to <strong>HTML</strong>!</p>
//!   <img src="logo.png" alt="Logo">
//! </body>
//! </html>"#;
//!
//! let tree = parser.parse(source, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```

use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_html() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for HTML.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_html) };

/// The tree-sitter language for HTML.
pub fn language() -> tree_sitter::Language {
    LANGUAGE.into()
}

/// The syntax highlighting query for HTML.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injection query for HTML (for script/style content).
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The content of the node-types.json file for HTML.
pub const NODE_TYPES: &str = include_str!("node-types.json");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&LANGUAGE.into())
            .expect("Failed to load HTML grammar");
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
    fn test_parse_void_elements() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<img src="test.png"><br><input type="text"><hr>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_self_closing_void() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<img src="test.png" /><br /><input type="text" />"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_script_element() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<script>const x = "<div></div>";</script>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_style_element() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<style>div { color: red; }</style>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_textarea_element() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<textarea>Some <b>text</b> here</textarea>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
        // textarea is an element with raw_text content
        let element = tree.root_node().child(0).unwrap();
        assert_eq!(element.kind(), "element");
    }

    #[test]
    fn test_parse_title_element() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<title>Page <Title> Test</title>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
        // title is an element with raw_text content
        let element = tree.root_node().child(0).unwrap();
        assert_eq!(element.kind(), "element");
    }

    #[test]
    fn test_parse_implicit_end_tags() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<ul><li>One<li>Two<li>Three</ul>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_p_implicit_close() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<p>Paragraph<div>Block</div>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_entities() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<p>&amp; &lt; &gt; &#60; &#x3C;</p>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_doctype() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<!DOCTYPE html><html></html>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_comments() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<!-- Comment --><div><!-- Another --></div>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_attributes() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();

        let source = r#"<div id="test" class='cls' data-value=123 disabled></div>"#;
        let tree = parser.parse(source, None).unwrap();

        assert!(!tree.root_node().has_error());
    }
}

