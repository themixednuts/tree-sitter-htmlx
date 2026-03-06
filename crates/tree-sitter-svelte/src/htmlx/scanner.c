/**
 * Auto-vendored during build. Do not edit manually.
 */

#include "tree_sitter/parser.h"

#define scan html_scan
#define tree_sitter_html_external_scanner_create      html_create
#define tree_sitter_html_external_scanner_destroy     html_destroy
#define tree_sitter_html_external_scanner_scan        html_scanner_scan
#define tree_sitter_html_external_scanner_serialize   html_serialize
#define tree_sitter_html_external_scanner_deserialize html_deserialize

// Vendored by build.rs from tree-sitter-html crate
#include "html/scanner.c"

#undef scan
#undef tree_sitter_html_external_scanner_create
#undef tree_sitter_html_external_scanner_destroy
#undef tree_sitter_html_external_scanner_scan
#undef tree_sitter_html_external_scanner_serialize
#undef tree_sitter_html_external_scanner_deserialize

// HTMLX external token indices (after HTML's 9 tokens: 0-8)
// HTML tokens: START_TAG_NAME(0), RAW_TEXT_START_TAG_NAME(1), END_TAG_NAME(2),
//              ERRONEOUS_END_TAG_NAME(3), SELF_CLOSING_TAG_DELIMITER(4),
//              IMPLICIT_END_TAG(5), RAW_TEXT(6), COMMENT(7), TEXT(8)
enum {
    TAG_NAMESPACE = 9,
    TAG_LOCAL_NAME,
    TS_LANG_MARKER,
    EXPRESSION_JS,
    EXPRESSION_TS,
    ATTRIBUTE_EXPRESSION_JS,
    ATTRIBUTE_EXPRESSION_TS,
    DIRECTIVE_MARKER,
    MEMBER_TAG_OBJECT,    // First part of dotted component (UI in UI.Button)
    MEMBER_TAG_PROPERTY,  // Subsequent parts (Button in UI.Button)
    ATTRIBUTE_VALUE,      // Unquoted attribute value text segment
    PIPE_ATTRIBUTE_NAME,  // Attribute name starting with | (like |-wtf)
    LINE_TAG_COMMENT,     // // comment in tag attribute list
    BLOCK_TAG_COMMENT,    // /* comment */ in tag attribute list
    UNTERMINATED_TAG_END, // malformed start tag ended by newline boundary
    TEXTAREA_END_BOUNDARY,
};

typedef struct {
    Scanner *html;
    bool awaiting_local_name;
    bool is_typescript;
} State;

static inline bool is_alpha(int32_t c) {
    return (unsigned)(c | 0x20) - 'a' < 26;
}

static inline bool is_digit(int32_t c) {
    return (unsigned)(c - '0') < 10;
}

static inline bool is_alnum(int32_t c) {
    return is_alpha(c) || is_digit(c);
}

static inline bool is_name_char(int32_t c) {
    return is_alnum(c) || c == '-' || c == '_';
}

static inline bool is_ident_start(int32_t c) {
    return is_alpha(c) || c == '_' || c == '$';
}

static inline bool is_ident_char(int32_t c) {
    return is_alnum(c) || c == '_' || c == '$';
}

static inline bool is_space(int32_t c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r';
}

static inline int32_t to_upper(int32_t c) {
    return is_alpha(c) ? (c & ~0x20) : c;
}

static inline int32_t to_lower(int32_t c) {
    return is_alpha(c) ? (c | 0x20) : c;
}

/**
 * Scan text content for HTMLX
 *
 * Extends HTML text scanning (§13.1.3) to also stop at '{' for expressions.
 * Whitespace is significant and captured as part of the text node.
 */
static bool scan_htmlx_text(TSLexer *lexer) {
    bool has_content = false;

    while (lexer->lookahead != 0) {
        int32_t c = lexer->lookahead;

        // Stop at tag start, character reference, or expression start
        if (c == '<' || c == '&' || c == '{') {
            break;
        }

        advance(lexer);
        has_content = true;
    }

    if (has_content) {
        lexer->mark_end(lexer);
        lexer->result_symbol = TEXT;
        return true;
    }

    return false;
}

static bool scan_svelte_textarea(State *state, TSLexer *lexer, const bool *valid) {
    if (state->html->tags.size == 0) {
        return false;
    }

    Tag *tag = array_back(&state->html->tags);
    if (tag->type != TEXTAREA) {
        return false;
    }

    lexer->mark_end(lexer);

    unsigned match_index = 0;
    const char *delimiter = "</TEXTAREA";
    const unsigned delimiter_len = 10;
    bool has_content = false;

    while (lexer->lookahead != 0) {
        if (lexer->lookahead == '{') {
            if (match_index > 0) {
                lexer->mark_end(lexer);
                has_content = true;
            }
            break;
        }

        char upper = to_upper(lexer->lookahead);
        if (upper == delimiter[match_index]) {
            match_index++;
            if (match_index == delimiter_len) {
                if (!has_content && valid[TEXTAREA_END_BOUNDARY]) {
                    lexer->result_symbol = TEXTAREA_END_BOUNDARY;
                    return true;
                }
                break;
            }
            advance(lexer);
            continue;
        }

        match_index = 0;
        advance(lexer);
        has_content = true;
        lexer->mark_end(lexer);
    }

    if (match_index > 0 && lexer->lookahead == 0) {
        has_content = true;
        lexer->mark_end(lexer);
    }

    if (!has_content) {
        return false;
    }

    lexer->result_symbol = TEXT;
    return true;
}

static bool in_textarea(State *state) {
    if (state->html->tags.size == 0) {
        return false;
    }

    return array_back(&state->html->tags)->type == TEXTAREA;
}

static bool scan_start_tag(State *state, TSLexer *lexer, const bool *valid) {
    if (!is_alpha(lexer->lookahead)) return false;

    String name = array_new();
    bool preserve_mark_end = false;
    while (is_name_char(lexer->lookahead)) {
        array_push(&name, (char)to_upper(lexer->lookahead));
        advance(lexer);
    }

    // Check for namespaced tag (svelte:head)
    if (lexer->lookahead == ':' && valid[TAG_NAMESPACE]) {
        lexer->mark_end(lexer);
        lexer->result_symbol = TAG_NAMESPACE;
        state->awaiting_local_name = true;
        array_delete(&name);
        return true;
    }

    // Handle dot after a tag identifier:
    // - If followed by identifier and member tags are valid, emit MEMBER_TAG_OBJECT.
    // - Otherwise, consume the stray dot so newline-based unterminated recovery can fire.
    if (lexer->lookahead == '.') {
        lexer->mark_end(lexer);
        advance(lexer);
        if (is_alpha(lexer->lookahead) && valid[MEMBER_TAG_OBJECT]) {
            lexer->result_symbol = MEMBER_TAG_OBJECT;
            array_delete(&name);
            return true;
        }
        preserve_mark_end = true;
    }

    if (name.size > 0 && (valid[START_TAG_NAME] || valid[RAW_TEXT_START_TAG_NAME])) {
        if (!preserve_mark_end) {
            lexer->mark_end(lexer);
        }
        Tag tag = tag_for_name(name);
        array_push(&state->html->tags, tag);

        switch (tag.type) {
            case SCRIPT:
            case STYLE:
                lexer->result_symbol = RAW_TEXT_START_TAG_NAME;
                break;
            default:
                lexer->result_symbol = START_TAG_NAME;
                break;
        }
        return true;
    }

    array_delete(&name);
    return false;
}

static bool scan_local_name(State *state, TSLexer *lexer) {
    if (!is_alpha(lexer->lookahead)) return false;

    while (is_name_char(lexer->lookahead)) advance(lexer);

    lexer->mark_end(lexer);
    lexer->result_symbol = TAG_LOCAL_NAME;
    state->awaiting_local_name = false;
    return true;
}

static bool scan_end_tag(State *state, TSLexer *lexer, const bool *valid) {
    if (!is_alpha(lexer->lookahead)) return false;

    String name = array_new();
    bool preserve_mark_end = false;
    while (is_name_char(lexer->lookahead)) {
        array_push(&name, (char)to_upper(lexer->lookahead));
        advance(lexer);
    }

    // Check for namespaced tag (svelte:head)
    if (lexer->lookahead == ':' && valid[TAG_NAMESPACE]) {
        lexer->mark_end(lexer);
        lexer->result_symbol = TAG_NAMESPACE;
        state->awaiting_local_name = true;
        array_delete(&name);
        return true;
    }

    // Handle dot after an end-tag identifier (mirrors start-tag behavior).
    if (lexer->lookahead == '.') {
        lexer->mark_end(lexer);
        advance(lexer);
        if (is_alpha(lexer->lookahead) && valid[MEMBER_TAG_OBJECT]) {
            lexer->result_symbol = MEMBER_TAG_OBJECT;
            array_delete(&name);
            return true;
        }
        preserve_mark_end = true;
    }

    if (name.size == 0) {
        array_delete(&name);
        return false;
    }

    if (!preserve_mark_end) {
        lexer->mark_end(lexer);
    }

    if (valid[END_TAG_NAME]) {
        Tag tag = tag_for_name(name);
        if (state->html->tags.size > 0 && tag_eq(array_back(&state->html->tags), &tag)) {
            Tag popped = array_pop(&state->html->tags);
            tag_free(&popped);
            lexer->result_symbol = END_TAG_NAME;
        } else {
            lexer->result_symbol = ERRONEOUS_END_TAG_NAME;
        }
        tag_free(&tag);
        return true;
    }

    array_delete(&name);
    return false;
}

static bool scan_slash_prefixed(State *state, TSLexer *lexer, const bool *valid) {
    if (lexer->lookahead != '/') return false;

    advance(lexer);

    // Self-closing delimiter: />
    if (lexer->lookahead == '>' && valid[SELF_CLOSING_TAG_DELIMITER]) {
        advance(lexer);
        lexer->mark_end(lexer);

        if (state->html->tags.size > 0) {
            Tag popped = array_pop(&state->html->tags);
            tag_free(&popped);
        }

        lexer->result_symbol = SELF_CLOSING_TAG_DELIMITER;
        return true;
    }

    // Line comment in tag attributes: // ...
    if (lexer->lookahead == '/' && !valid[ATTRIBUTE_VALUE] && valid[LINE_TAG_COMMENT]) {
        advance(lexer);
        while (lexer->lookahead && lexer->lookahead != '\n' && lexer->lookahead != '\r' && lexer->lookahead != '>') {
            advance(lexer);
        }
        lexer->mark_end(lexer);
        lexer->result_symbol = LINE_TAG_COMMENT;
        return true;
    }

    // Block comment in tag attributes: /* ... */
    if (lexer->lookahead == '*' && !valid[ATTRIBUTE_VALUE] && valid[BLOCK_TAG_COMMENT]) {
        advance(lexer);
        while (lexer->lookahead) {
            if (lexer->lookahead != '*') {
                advance(lexer);
                continue;
            }

            advance(lexer);
            if (lexer->lookahead == '/') {
                advance(lexer);
                lexer->mark_end(lexer);
                lexer->result_symbol = BLOCK_TAG_COMMENT;
                return true;
            }
        }
    }

    return false;
}

static inline bool skip_string(TSLexer *lexer) {
    int32_t quote = lexer->lookahead;
    if (quote != '"' && quote != '\'' && quote != '`') return false;

    advance(lexer);
    while (lexer->lookahead && lexer->lookahead != quote) {
        int32_t c = lexer->lookahead;
        if (c == '\\') {
            advance(lexer);
            if (lexer->lookahead) advance(lexer);
        } else if (quote == '`' && c == '$') {
            advance(lexer);
            if (lexer->lookahead == '{') {
                advance(lexer);
                for (int depth = 1; lexer->lookahead && depth > 0;) {
                    c = lexer->lookahead;
                    if (c == '"' || c == '\'' || c == '`') {
                        skip_string(lexer);
                    } else {
                        if (c == '{') depth++;
                        else if (c == '}') depth--;
                        advance(lexer);
                    }
                }
            }
        } else {
            advance(lexer);
        }
    }
    if (lexer->lookahead == quote) advance(lexer);
    return true;
}

// Scan balanced expression, excluding trailing whitespace at depth 0.
// Marks end position before any trailing whitespace.
// If '<' appears at depth 0, peek one byte ahead:
// - '</', '<!', or '<' followed by alpha is treated as a tag boundary
// - otherwise '<' is treated as a normal expression character
// Returns false if we reach EOF without finding a valid terminator.
static bool scan_balanced_expr(TSLexer *lexer) {
    int depth = 0;
    bool has_content = false;
    bool needs_mark = false;
    bool found_terminator = false;

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        // Stop at } at depth 0
        if (depth == 0 && c == '}') {
            found_terminator = true;
            break;
        }

        // At depth 0, classify '<' as either tag boundary or expression operator
        if (depth == 0 && c == '<') {
            if (needs_mark) {
                lexer->mark_end(lexer);
                needs_mark = false;
            }

            advance(lexer);
            int32_t next = lexer->lookahead;
            if (next == '/' || next == '!') {
                found_terminator = true;
                break;
            }

            has_content = true;
            needs_mark = true;
            continue;
        }

        if (skip_string(lexer)) {
            has_content = true;
            needs_mark = true;
            continue;
        }

        if (c == '/') {
            advance(lexer);

            if (lexer->lookahead == '/') {
                advance(lexer);
                while (lexer->lookahead && lexer->lookahead != '\n' && lexer->lookahead != '\r') {
                    advance(lexer);
                }
                has_content = true;
                needs_mark = true;
                continue;
            }

            if (lexer->lookahead == '*') {
                advance(lexer);
                while (lexer->lookahead) {
                    if (lexer->lookahead != '*') {
                        advance(lexer);
                        continue;
                    }

                    advance(lexer);
                    if (lexer->lookahead == '/') {
                        advance(lexer);
                        break;
                    }
                }
                has_content = true;
                needs_mark = true;
                continue;
            }

            has_content = true;
            needs_mark = true;
            continue;
        }

        // At depth 0, mark before whitespace (handles trailing ws)
        if (depth == 0 && is_space(c)) {
            if (needs_mark) {
                lexer->mark_end(lexer);
                needs_mark = false;
            }
            do { advance(lexer); } while (is_space(lexer->lookahead));
            continue;
        }

        switch (c) {
            case '(': case '[': case '{': depth++; break;
            case ')': case ']': case '}': if (--depth < 0) goto done; break;
        }

        advance(lexer);
        has_content = true;
        needs_mark = true;
    }

done:
    if (needs_mark) {
        lexer->mark_end(lexer);
    }

    return has_content && found_terminator;
}

static bool check_ts_lang_attr(TSLexer *lexer) {
    // Skip horizontal whitespace only; do not consume newlines here.
    while (lexer->lookahead == ' ' || lexer->lookahead == '\t') skip(lexer);

    // Quick check: if first char isn't 'l', this isn't lang=
    // Return early WITHOUT consuming input.
    if (to_lower(lexer->lookahead) != 'l') return false;

    static const char lang[] = "lang";
    for (int i = 0; i < 4; i++) {
        if (to_lower(lexer->lookahead) != lang[i]) return false;
        advance(lexer);
    }

    while (is_space(lexer->lookahead)) advance(lexer);
    if (lexer->lookahead != '=') return false;
    advance(lexer);
    while (is_space(lexer->lookahead)) advance(lexer);

    int32_t quote = lexer->lookahead;
    if (quote != '"' && quote != '\'') return false;
    advance(lexer);

    if (to_lower(lexer->lookahead) != 't') return false;
    advance(lexer);
    if (to_lower(lexer->lookahead) != 's') return false;
    advance(lexer);

    if (lexer->lookahead == quote) return true;

    static const char cript[] = "cript";
    for (int i = 0; i < 5; i++) {
        if (to_lower(lexer->lookahead) != cript[i]) return false;
        advance(lexer);
    }

    return lexer->lookahead == quote;
}

static bool scan_ts_lang_marker(State *state, TSLexer *lexer) {
    lexer->mark_end(lexer);
    if (!check_ts_lang_attr(lexer)) return false;

    state->is_typescript = true;
    lexer->result_symbol = TS_LANG_MARKER;
    return true;
}

static bool scan_expression(State *state, TSLexer *lexer) {
    while (is_space(lexer->lookahead)) skip(lexer);

    int32_t c = lexer->lookahead;
    if (c == '#' || c == ':' || c == '@' || c == '/') return false;

    // scan_balanced_expr handles mark_end (excludes trailing whitespace)
    if (!scan_balanced_expr(lexer)) return false;

    lexer->result_symbol = state->is_typescript ? EXPRESSION_TS : EXPRESSION_JS;
    return true;
}

static bool scan_attribute_expression(State *state, TSLexer *lexer) {
    while (is_space(lexer->lookahead)) skip(lexer);

    int32_t c = lexer->lookahead;
    if (c == '#' || c == ':' || c == '@') return false;

    // Attribute expression context can legitimately start with comments.
    if (!scan_balanced_expr(lexer)) return false;

    lexer->result_symbol =
        state->is_typescript ? ATTRIBUTE_EXPRESSION_TS : ATTRIBUTE_EXPRESSION_JS;
    return true;
}

// Returns: 1 = matched, 0 = not at identifier, -1 = identifier without colon
// This function checks if we're at a directive (identifier followed by colon)
// It consumes the directive name and returns it as the token content.
static int check_directive_marker(TSLexer *lexer) {
    // If at whitespace, return 0 to let tree-sitter handle it via extras
    if (is_space(lexer->lookahead)) return 0;
    
    // If not at identifier start, not a directive
    if (!is_ident_start(lexer->lookahead)) return 0;
    
    // Consume the identifier
    while (is_ident_char(lexer->lookahead)) {
        advance(lexer);
    }
    
    // Check for colon - if not present, this isn't a directive
    if (lexer->lookahead != ':') return -1;

    // Mark the end after consuming the directive name
    lexer->mark_end(lexer);
    lexer->result_symbol = DIRECTIVE_MARKER;
    return 1;
}

// Scan member tag property (part after '.' in dotted component like UI.Button)
static bool scan_member_tag_property(TSLexer *lexer) {
    while (is_space(lexer->lookahead)) skip(lexer);

    if (!is_alpha(lexer->lookahead)) return false;

    while (is_ident_char(lexer->lookahead)) {
        advance(lexer);
    }

    lexer->mark_end(lexer);
    lexer->result_symbol = MEMBER_TAG_PROPERTY;
    return true;
}

// Scan unquoted attribute value segment
// Matches: [^<>{}\"'/=\s]+
// This is used for attribute values like: class=foo or style:color=red
static bool scan_attribute_value(TSLexer *lexer) {
    bool has_content = false;

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        // Stop at characters that end unquoted attribute values
        if (c == '<' || c == '>' || c == '{' || c == '}' ||
            c == '"' || c == '\'' || c == '/' || c == '=' ||
            is_space(c)) {
            break;
        }

        advance(lexer);
        has_content = true;
    }

    if (has_content) {
        lexer->mark_end(lexer);
        lexer->result_symbol = ATTRIBUTE_VALUE;
        return true;
    }

    return false;
}

static bool scan_unterminated_tag_end(State *state, TSLexer *lexer) {
    if (lexer->lookahead != '\n' && lexer->lookahead != '\r') return false;

    skip(lexer);
    if (lexer->lookahead == '\n') {
        skip(lexer);
    }

    while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
        skip(lexer);
    }

    // Boundary token spans newline + indentation only.
    lexer->mark_end(lexer);

    int32_t next = lexer->lookahead;

    if (next == '{') {
        // Peek marker to distinguish malformed block/tag starts from valid
        // multiline shorthand/spread attributes.
        advance(lexer);
        int32_t marker = lexer->lookahead;
        while (marker == ' ' || marker == '\t') {
            advance(lexer);
            marker = lexer->lookahead;
        }

        if (marker == '#' || marker == ':' || marker == '@' || marker == '/') {
            if (state->html->tags.size > 0) {
                Tag popped = array_pop(&state->html->tags);
                tag_free(&popped);
            }
            lexer->result_symbol = UNTERMINATED_TAG_END;
            return true;
        }

        return false;
    }

    // Continue parsing valid multiline tag attributes/comments/closers.
    if (next == '>' || next == '/' || next == '|' || next == '"' || next == '\'' || is_ident_start(next)) {
        return false;
    }

    if (state->html->tags.size > 0) {
        Tag popped = array_pop(&state->html->tags);
        tag_free(&popped);
    }

    lexer->result_symbol = UNTERMINATED_TAG_END;
    return true;
}

static bool scan_block_boundary(State *state, TSLexer *lexer) {
    if (state->html->tags.size == 0) return false;
    if (lexer->lookahead != '{') return false;

    // Zero-width element boundary before Svelte-style block close/branch marker.
    lexer->mark_end(lexer);

    advance(lexer);
    if (lexer->lookahead == '/') {
        advance(lexer);

        int len = 0;
        char kind[8];
        if (!is_alpha(lexer->lookahead) && lexer->lookahead != '_') return false;
        while (is_ident_char(lexer->lookahead)) {
            if (len < (int)sizeof(kind)) {
                kind[len++] = (char)to_lower(lexer->lookahead);
            }
            advance(lexer);
        }

        bool is_block_kind =
            (len == 2 && kind[0] == 'i' && kind[1] == 'f') ||
            (len == 3 && kind[0] == 'k' && kind[1] == 'e' && kind[2] == 'y') ||
            (len == 4 && kind[0] == 'e' && kind[1] == 'a' && kind[2] == 'c' && kind[3] == 'h') ||
            (len == 5 && kind[0] == 'a' && kind[1] == 'w' && kind[2] == 'a' && kind[3] == 'i' && kind[4] == 't') ||
            (len == 7 && kind[0] == 's' && kind[1] == 'n' && kind[2] == 'i' && kind[3] == 'p' && kind[4] == 'p' && kind[5] == 'e' && kind[6] == 't');
        if (!is_block_kind) return false;

        while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
            advance(lexer);
        }

        if (lexer->lookahead != '}') return false;
    } else if (lexer->lookahead == ':') {
        advance(lexer);

        int len = 0;
        char kind[8];
        if (!is_alpha(lexer->lookahead)) return false;
        while (is_ident_char(lexer->lookahead)) {
            if (len < (int)sizeof(kind)) {
                kind[len++] = (char)to_lower(lexer->lookahead);
            }
            advance(lexer);
        }

        bool is_branch_kind =
            (len == 4 && kind[0] == 'e' && kind[1] == 'l' && kind[2] == 's' && kind[3] == 'e') ||
            (len == 4 && kind[0] == 't' && kind[1] == 'h' && kind[2] == 'e' && kind[3] == 'n') ||
            (len == 5 && kind[0] == 'c' && kind[1] == 'a' && kind[2] == 't' && kind[3] == 'c' && kind[4] == 'h');
        if (!is_branch_kind) return false;

        while (lexer->lookahead && lexer->lookahead != '}') {
            advance(lexer);
        }

        if (lexer->lookahead != '}') return false;
    } else {
        return false;
    }

    Tag popped = array_pop(&state->html->tags);
    tag_free(&popped);

    lexer->result_symbol = UNTERMINATED_TAG_END;
    return true;
}

// Scan attribute name starting with | (like |-wtf)
// This is distinct from directive modifiers because modifiers start with an
// identifier character [a-zA-Z_$] after the |, while unusual attribute names
// start with other characters (like |-wtf starting with -).
// Matches: \|[^a-zA-Z_$<>{}\"':\\/=\s|.()][^<>{}\"':\\/=\s|()]*
// Caller should check that lexer->lookahead == '|'
static bool scan_pipe_attribute_name(TSLexer *lexer) {
    // Must start with |
    if (lexer->lookahead != '|') return false;
    
    advance(lexer);
    
    // If the first character after | is an identifier start, this is likely
    // a directive modifier (like |preventDefault), not a pipe attribute name.
    // Let the grammar handle it with the | literal token.
    if (is_ident_start(lexer->lookahead)) {
        return false;
    }
    
    bool has_content = false;
    
    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;
        
        // Stop at characters that end attribute names
        // Same as regex: [^<>{}\"':\\/=\s|.()]
        if (c == '<' || c == '>' || c == '{' || c == '}' ||
            c == '"' || c == '\'' || c == ':' || c == '\\' ||
            c == '/' || c == '=' || c == '|' || c == '.' ||
            c == '(' || c == ')' ||
            is_space(c)) {
            break;
        }
        
        advance(lexer);
        has_content = true;
    }
    
    // Must have at least one character after the |
    if (has_content) {
        lexer->mark_end(lexer);
        lexer->result_symbol = PIPE_ATTRIBUTE_NAME;
        return true;
    }
    
    return false;
}

static bool scan(State *state, TSLexer *lexer, const bool *valid) {
    if ((valid[TEXT] || valid[TEXTAREA_END_BOUNDARY]) && in_textarea(state)) {
        if (scan_svelte_textarea(state, lexer, valid)) {
            return true;
        }
        if (lexer->lookahead == '{') {
            return false;
        }
    }

    // Text content - handle before whitespace is skipped
    // HTMLX text stops at '{' in addition to '<' and '&'
    if (valid[TEXT]) {
        if (scan_htmlx_text(lexer)) {
            return true;
        }
        // At '{' means expression start - return false to let grammar handle it
        // HTML scanner doesn't know about '{' so don't fall through
        if (lexer->lookahead == '{') {
            if (valid[UNTERMINATED_TAG_END] && scan_block_boundary(state, lexer)) {
                return true;
            }
            return false;
        }
    }

    if (valid[UNTERMINATED_TAG_END] && scan_block_boundary(state, lexer)) {
        return true;
    }

    if (valid[UNTERMINATED_TAG_END] && scan_unterminated_tag_end(state, lexer)) {
        return true;
    }

    while (is_space(lexer->lookahead)) skip(lexer);

    if (valid[TS_LANG_MARKER] && scan_ts_lang_marker(state, lexer)) {
        return true;
    }

    if (valid[DIRECTIVE_MARKER]) {
        int result = check_directive_marker(lexer);
        if (result != 0) return result == 1;
    }

    if ((valid[EXPRESSION_JS] || valid[EXPRESSION_TS]) && scan_expression(state, lexer)) {
        return true;
    }

    if ((valid[ATTRIBUTE_EXPRESSION_JS] || valid[ATTRIBUTE_EXPRESSION_TS])
        && scan_attribute_expression(state, lexer)) {
        return true;
    }

    if (valid[RAW_TEXT] && !valid[START_TAG_NAME] && !valid[END_TAG_NAME]) {
        return html_scanner_scan(state->html, lexer, valid);
    }

    if (state->awaiting_local_name && valid[TAG_LOCAL_NAME]) {
        return scan_local_name(state, lexer);
    }

    int32_t c = lexer->lookahead;

    if (c == '/'
        && (valid[SELF_CLOSING_TAG_DELIMITER] || valid[LINE_TAG_COMMENT] || valid[BLOCK_TAG_COMMENT])) {
        if (scan_slash_prefixed(state, lexer, valid)) return true;
    }

    if (valid[MEMBER_TAG_PROPERTY] && scan_member_tag_property(lexer)) {
        return true;
    }

    // Pipe-starting attribute names (like |-wtf) - must check before ATTRIBUTE_VALUE
    // The grammar's precedence should handle the ambiguity between pipe attribute names
    // and directive modifiers - we just need to match when the token is valid
    if (valid[PIPE_ATTRIBUTE_NAME] && c == '|' && scan_pipe_attribute_name(lexer)) {
        return true;
    }

    if (valid[ATTRIBUTE_VALUE] && scan_attribute_value(lexer)) {
        return true;
    }

    if (is_alpha(c)) {
        if (valid[TAG_NAMESPACE] || valid[START_TAG_NAME] ||
            valid[RAW_TEXT_START_TAG_NAME] || valid[MEMBER_TAG_OBJECT]) {
            if (scan_start_tag(state, lexer, valid)) return true;
        }
        if (valid[TAG_NAMESPACE] || valid[END_TAG_NAME] || valid[MEMBER_TAG_OBJECT]) {
            if (scan_end_tag(state, lexer, valid)) return true;
        }
    }

    return html_scanner_scan(state->html, lexer, valid);
}

void *tree_sitter_htmlx_external_scanner_create(void) {
    State *state = ts_calloc(1, sizeof(State));
    state->html = html_create();
    return state;
}

void tree_sitter_htmlx_external_scanner_destroy(void *payload) {
    State *state = payload;
    html_destroy(state->html);
    ts_free(state);
}

unsigned tree_sitter_htmlx_external_scanner_serialize(void *payload, char *buffer) {
    State *state = payload;
    buffer[0] = (char)((state->awaiting_local_name ? 1 : 0) | (state->is_typescript ? 2 : 0));
    return 1 + html_serialize(state->html, buffer + 1);
}

void tree_sitter_htmlx_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
    State *state = payload;
    if (length > 0) {
        state->awaiting_local_name = buffer[0] & 1;
        state->is_typescript = buffer[0] & 2;
        html_deserialize(state->html, buffer + 1, length - 1);
    } else {
        state->awaiting_local_name = false;
        state->is_typescript = false;
        html_deserialize(state->html, NULL, 0);
    }
}

bool tree_sitter_htmlx_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid) {
    return scan(payload, lexer, valid);
}
