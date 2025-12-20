//! Tree-sitter grammar for Svelte 5
//!
//! This grammar extends HTMLX to support Svelte 5 syntax including:
//!
//! ## Control Flow Blocks
//! - `{#if}...{:else if}...{:else}...{/if}`
//! - `{#each items as item}...{:else}...{/each}`
//! - `{#await promise}...{:then}...{:catch}...{/await}`
//! - `{#key expression}...{/key}`
//! - `{#snippet name(params)}...{/snippet}`
//!
//! ## Special Tags
//! - `{@render snippet()}`
//! - `{@attach handler}`
//! - `{@html expression}`
//! - `{@const assignment}`
//! - `{@debug vars}`
//!
//! ## Components
//! - Uppercase tag names are parsed as components: `<Button>`, `<MyComponent>`
//!
//! ## Example
//!
//! ```rust
//! use tree_sitter_svelte::LANGUAGE;
//!
//! let mut parser = tree_sitter::Parser::new();
//! parser.set_language(&LANGUAGE.into()).expect("Failed to load Svelte grammar");
//!
//! let source = r#"{#if visible}<p>Hello</p>{/if}"#;
//!
//! let tree = parser.parse(source, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```

use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_svelte() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for Svelte.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_svelte) };

/// The tree-sitter language for Svelte.
pub fn language() -> tree_sitter::Language {
    LANGUAGE.into()
}

/// The syntax highlighting query for Svelte.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../queries/highlights.scm");

/// The injection query for Svelte (TypeScript/CSS).
pub const INJECTIONS_QUERY: &str = include_str!("../queries/injections.scm");

/// The content of the [`node-types.json`] file for Svelte.
pub const NODE_TYPES: &str = include_str!("../src/node-types.json");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).expect("Failed to load Svelte grammar");
    }

    #[test]
    fn test_parse_simple_html() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = "<div>Hello</div>";
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_if_block() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = r#"{#if visible}<p>Hello</p>{/if}"#;
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_if_else_block() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = r#"{#if count > 0}<p>{count}</p>{:else}<p>Zero</p>{/if}"#;
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_each_block() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = r#"{#each items as item}<li>{item}</li>{/each}"#;
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_await_block() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = r#"{#await promise}{:then value}<p>{value}</p>{:catch error}<p>{error}</p>{/await}"#;
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_snippet_block() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = r#"{#snippet button(text)}<button>{text}</button>{/snippet}"#;
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_render_tag() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = r#"{@render button("Click me")}"#;
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_html_tag() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = r#"{@html content}"#;
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_component() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = r#"<Button onclick={handleClick}>Click me</Button>"#;
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_self_closing_component() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = r#"<Icon name="check" />"#;
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_script_element() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = r#"<script>
  let count = $state(0);
  let doubled = $derived(count * 2);
</script>"#;
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_key_block() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&LANGUAGE.into()).unwrap();
        
        let source = r#"{#key item.id}<Component />{/key}"#;
        let tree = parser.parse(source, None).unwrap();
        
        assert!(!tree.root_node().has_error());
    }
}
