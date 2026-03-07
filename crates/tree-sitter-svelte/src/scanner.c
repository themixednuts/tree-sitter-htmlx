#define tree_sitter_htmlx_external_scanner_create      htmlx_create
#define tree_sitter_htmlx_external_scanner_destroy     htmlx_destroy
#define tree_sitter_htmlx_external_scanner_scan        htmlx_scanner_scan
#define tree_sitter_htmlx_external_scanner_serialize   htmlx_serialize
#define tree_sitter_htmlx_external_scanner_deserialize htmlx_deserialize

// Vendored HTMLX scanner (committed in this crate for portability).
#include "htmlx/scanner.c"

// Svelte external token indices (after HTMLX's 26 tokens: 0-25)
// HTML tokens (0-8) + HTMLX tokens (9-25, includes UNTERMINATED_TAG_END_OPEN at 25)
enum {
    ITERATOR_EXPRESSION = 26,
    BINDING_PATTERN,
    KEY_EXPRESSION,
    TAG_EXPRESSION,
    SNIPPET_PARAMETER,
    SNIPPET_TYPE_PARAMS,
    BLOCK_END_OPEN,
};

static bool is_svelte_tag_name_char(int32_t c) {
    return is_ident_char(c) || c == '-' || c == ':' || c == '.';
}

static bool scan_lt_as_tag_boundary(TSLexer *lexer) {
    advance(lexer);

    if (lexer->lookahead == '/' || lexer->lookahead == '!') {
        return true;
    }

    if (!is_alpha(lexer->lookahead)) {
        return false;
    }

    while (is_svelte_tag_name_char(lexer->lookahead)) {
        advance(lexer);
    }

    if (lexer->lookahead == '>') {
        advance(lexer);
        return lexer->lookahead != '(';
    }

    if (lexer->lookahead == '/') {
        advance(lexer);
        return lexer->lookahead == '>';
    }

    if (!is_space(lexer->lookahead)) {
        return false;
    }

    while (is_space(lexer->lookahead)) {
        advance(lexer);
    }

    return lexer->lookahead == '>'
        || lexer->lookahead == '/'
        || lexer->lookahead == '{'
        || lexer->lookahead == '"'
        || lexer->lookahead == '\''
        || lexer->lookahead == '|'
        || is_ident_start(lexer->lookahead);
}

// Scan balanced expression with trailing whitespace exclusion, comment handling,
// tag boundary detection, and Svelte block marker detection.
// Used for iterator expressions, binding patterns, key expressions, tag expressions,
// and snippet parameters.
static bool scan_balanced(TSLexer *lexer, int32_t stop_char, bool stop_comma) {
    int depth = 0;
    bool has_content = false;
    bool needs_mark = false;
    bool found_terminator = false;

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        if (depth == 0 && (c == stop_char || c == '}')) {
            found_terminator = true;
            break;
        }
        if (depth == 0 && stop_comma && c == ',') {
            found_terminator = true;
            break;
        }
        // At depth 0, classify '<' as either tag boundary or expression operator.
        if (depth == 0 && c == '<') {
            if (needs_mark) {
                lexer->mark_end(lexer);
                needs_mark = false;
            }

            if (scan_lt_as_tag_boundary(lexer)) {
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

        // JS comments: // line and /* block */
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

        // At depth 0, '{' followed by a Svelte block marker (#, :, /, @)
        // is a block boundary — terminate the balanced scan.
        if (depth == 0 && c == '{') {
            if (needs_mark) {
                lexer->mark_end(lexer);
                needs_mark = false;
            }
            advance(lexer);
            int32_t next = lexer->lookahead;
            if (next == '#' || next == ':' || next == '/' || next == '@') {
                found_terminator = true;
                break;
            }
            // Not a block marker — treat as nesting
            depth++;
            has_content = true;
            needs_mark = true;
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

static inline bool match_keyword(TSLexer *lexer, const char *kw, int len) {
    for (int i = 0; i < len; i++) {
        if (lexer->lookahead != kw[i]) return false;
        advance(lexer);
    }
    // Keyword must be followed by space, '{', or '}' (end of block)
    return is_space(lexer->lookahead) || lexer->lookahead == '{' || lexer->lookahead == '}';
}

static bool scan_iterator(TSLexer *lexer) {
    int depth = 0;
    bool has_content = false;
    bool found_terminator = false;

    while (is_space(lexer->lookahead)) skip(lexer);

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        if (depth == 0 && c == '}') {
            found_terminator = true;
            break;
        }

        // At depth 0, classify '<' as either tag boundary or expression operator.
        if (depth == 0 && c == '<') {
            advance(lexer);
            int32_t next = lexer->lookahead;
            if (next == '/' || next == '!') {
                found_terminator = true;
                break;
            }

            has_content = true;
            continue;
        }

        if (depth == 0 && is_space(c)) {
            lexer->mark_end(lexer);
            while (is_space(lexer->lookahead)) advance(lexer);

            c = lexer->lookahead;
            if (c == '<' && scan_lt_as_tag_boundary(lexer)) {
                lexer->result_symbol = ITERATOR_EXPRESSION;
                return has_content;
            }
            if (c == 'a') {
                advance(lexer);
                if (match_keyword(lexer, "s", 1)) {
                    lexer->result_symbol = ITERATOR_EXPRESSION;
                    return has_content;
                }
            } else if (c == 't' && match_keyword(lexer, "then", 4)) {
                lexer->result_symbol = ITERATOR_EXPRESSION;
                return has_content;
            } else if (c == 'c' && match_keyword(lexer, "catch", 5)) {
                lexer->result_symbol = ITERATOR_EXPRESSION;
                return has_content;
            }
            has_content = true;
            continue;
        }

        if (skip_string(lexer)) {
            has_content = true;
            continue;
        }

        switch (c) {
            case '(': case '[': case '{': depth++; break;
            case ')': case ']': case '}': depth--; break;
        }

        advance(lexer);
        has_content = true;
    }

    // Only return true if we found a valid terminator
    if (has_content && found_terminator) {
        lexer->mark_end(lexer);
        lexer->result_symbol = ITERATOR_EXPRESSION;
        return true;
    }
    return false;
}

static bool scan_binding(TSLexer *lexer) {
    while (is_space(lexer->lookahead)) advance(lexer);
    if (!scan_balanced(lexer, '(', true)) return false;

    lexer->result_symbol = BINDING_PATTERN;
    return true;
}

static bool scan_key(TSLexer *lexer) {
    while (is_space(lexer->lookahead)) advance(lexer);
    if (!scan_balanced(lexer, ')', false)) return false;

    lexer->result_symbol = KEY_EXPRESSION;
    return true;
}

static bool scan_snippet_parameter(TSLexer *lexer) {
    while (is_space(lexer->lookahead)) advance(lexer);
    if (!scan_balanced(lexer, ')', true)) return false;

    lexer->result_symbol = SNIPPET_PARAMETER;
    return true;
}

// Tag expression: content after {@html, {@debug, etc.
// Requires leading whitespace, then delegates to scan_balanced.
static bool scan_tag_expression(TSLexer *lexer) {
    bool has_space = false;
    while (is_space(lexer->lookahead)) {
        skip(lexer);
        has_space = true;
    }

    if (!has_space || lexer->lookahead == '}') return false;

    if (!scan_balanced(lexer, '}', false)) return false;

    lexer->result_symbol = TAG_EXPRESSION;
    return true;
}

static bool scan_snippet_type_params(TSLexer *lexer) {
    if (lexer->lookahead != '<') return false;

    int depth = 0;

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        if (skip_string(lexer)) {
            continue;
        }

        if (c == '<') {
            depth++;
            advance(lexer);
            continue;
        }

        if (c == '>') {
            depth--;
            advance(lexer);
            if (depth == 0) {
                lexer->mark_end(lexer);
                lexer->result_symbol = SNIPPET_TYPE_PARAMS;
                return true;
            }
            continue;
        }

        advance(lexer);
    }

    return false;
}

// Match {/ only when followed by identifier start (block end like {/if}).
// Returns false for {/* or {// (JS comments inside expressions).
static bool scan_block_end_open(TSLexer *lexer) {
    if (lexer->lookahead != '{') return false;
    advance(lexer);
    if (lexer->lookahead != '/') return false;
    advance(lexer);
    if (!is_ident_start(lexer->lookahead)) return false;
    lexer->mark_end(lexer);
    lexer->result_symbol = BLOCK_END_OPEN;
    return true;
}

// Main dispatch for Svelte external scanner.
//
// BLOCK_END_OPEN uses early-return (return result regardless of success/failure)
// because it advances the lexer on failure. When it returns false, tree-sitter
// falls back to its internal lexer at the original position, matching literal
// tokens like { and {:
//
// Other svelte tokens use the same early-return pattern because they're
// expected in exclusive grammar contexts where no other token competes.
static bool svelte_scan(State *state, TSLexer *lexer, const bool *valid) {
    // BLOCK_END_OPEN: disambiguates {/if} (block end) from {/* comment */} (expression).
    // Only attempt when lookahead is '{' — otherwise fall through to HTMLX scanner
    // so it can produce TEXT and other tokens that don't start with '{'.
    if (valid[BLOCK_END_OPEN] && lexer->lookahead == '{') return scan_block_end_open(lexer);

    // Svelte block expression tokens — each valid in exclusive grammar contexts.
    if (valid[SNIPPET_TYPE_PARAMS]) return scan_snippet_type_params(lexer);
    if (valid[SNIPPET_PARAMETER]) return scan_snippet_parameter(lexer);
    if (valid[ITERATOR_EXPRESSION]) return scan_iterator(lexer);
    if (valid[BINDING_PATTERN]) return scan_binding(lexer);
    if (valid[KEY_EXPRESSION]) return scan_key(lexer);
    if (valid[TAG_EXPRESSION]) return scan_tag_expression(lexer);

    // Fall through to HTMLX scanner for all other tokens.
    return scan(state, lexer, valid);
}

void *tree_sitter_svelte_external_scanner_create(void) {
    return htmlx_create();
}

void tree_sitter_svelte_external_scanner_destroy(void *payload) {
    htmlx_destroy(payload);
}

unsigned tree_sitter_svelte_external_scanner_serialize(void *payload, char *buffer) {
    return htmlx_serialize(payload, buffer);
}

void tree_sitter_svelte_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
    htmlx_deserialize(payload, buffer, length);
}

bool tree_sitter_svelte_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid) {
    return svelte_scan(payload, lexer, valid);
}
