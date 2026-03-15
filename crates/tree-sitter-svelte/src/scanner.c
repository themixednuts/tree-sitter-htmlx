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
    ITERATOR_EXPRESSION_JS = 26,
    ITERATOR_EXPRESSION_TS,
    BINDING_PATTERN_JS,
    BINDING_PATTERN_TS,
    KEY_EXPRESSION_JS,
    KEY_EXPRESSION_TS,
    TAG_EXPRESSION_JS,
    TAG_EXPRESSION_TS,
    SNIPPET_PARAMETER_JS,
    SNIPPET_PARAMETER_TS,
    SNIPPET_TYPE_PARAMS,
    BLOCK_END_OPEN,
    SNIPPET_NAME,
    BLOCK_START_EOF,
    BLOCK_EOF,
};

typedef struct {
    uint64_t svelte_scan_calls;
    uint64_t htmlx_fallback_calls;
    uint64_t scan_lt_as_tag_boundary_calls;
    uint64_t scan_lt_as_tag_boundary_successes;
    uint64_t scan_lt_as_tag_boundary_bytes;
    uint64_t scan_balanced_calls;
    uint64_t scan_balanced_successes;
    uint64_t scan_balanced_bytes;
    uint64_t scan_iterator_calls;
    uint64_t scan_iterator_successes;
    uint64_t scan_iterator_bytes;
    uint64_t scan_binding_calls;
    uint64_t scan_binding_successes;
    uint64_t scan_key_calls;
    uint64_t scan_key_successes;
    uint64_t scan_tag_expression_calls;
    uint64_t scan_tag_expression_successes;
    uint64_t scan_tag_expression_bytes;
    uint64_t scan_snippet_parameter_calls;
    uint64_t scan_snippet_parameter_successes;
    uint64_t scan_snippet_type_params_calls;
    uint64_t scan_snippet_type_params_successes;
    uint64_t scan_snippet_type_params_bytes;
    uint64_t scan_snippet_name_calls;
    uint64_t scan_snippet_name_successes;
    uint64_t scan_snippet_name_bytes;
    uint64_t scan_block_end_open_calls;
    uint64_t scan_block_end_open_successes;
    uint64_t scan_block_end_open_bytes;
} SvelteScannerProfileStats;

static SvelteScannerProfileStats s_profile_stats;

#ifdef TREE_SITTER_SVELTE_PROFILE
#define PROFILE_COUNT(field) (++s_profile_stats.field)
#define PROFILE_ADVANCE(field, lexer) do { ++s_profile_stats.field; advance(lexer); } while (0)
#define PROFILE_SKIP(field, lexer) do { ++s_profile_stats.field; skip(lexer); } while (0)
#else
#define PROFILE_COUNT(field) ((void)0)
#define PROFILE_ADVANCE(field, lexer) advance(lexer)
#define PROFILE_SKIP(field, lexer) skip(lexer)
#endif

bool tree_sitter_svelte_profile_enabled(void) {
#ifdef TREE_SITTER_SVELTE_PROFILE
    return true;
#else
    return false;
#endif
}

void tree_sitter_svelte_profile_reset(void) {
    memset(&s_profile_stats, 0, sizeof(s_profile_stats));
}

void tree_sitter_svelte_profile_snapshot(SvelteScannerProfileStats *out) {
    if (out != NULL) {
        *out = s_profile_stats;
    }
}

static bool is_svelte_tag_name_char(int32_t c) {
    return is_ident_char(c) || c == '-' || c == ':' || c == '.';
}

static bool scan_lt_as_tag_boundary(TSLexer *lexer) {
    PROFILE_COUNT(scan_lt_as_tag_boundary_calls);
    PROFILE_ADVANCE(scan_lt_as_tag_boundary_bytes, lexer);

    if (lexer->lookahead == '/' || lexer->lookahead == '!') {
        PROFILE_COUNT(scan_lt_as_tag_boundary_successes);
        return true;
    }

    if (!is_alpha(lexer->lookahead)) {
        return false;
    }

    while (is_svelte_tag_name_char(lexer->lookahead)) {
        PROFILE_ADVANCE(scan_lt_as_tag_boundary_bytes, lexer);
    }

    if (lexer->lookahead == '>') {
        PROFILE_ADVANCE(scan_lt_as_tag_boundary_bytes, lexer);
        if (lexer->lookahead != '(') {
            PROFILE_COUNT(scan_lt_as_tag_boundary_successes);
            return true;
        }
        return false;
    }

    if (lexer->lookahead == '/') {
        PROFILE_ADVANCE(scan_lt_as_tag_boundary_bytes, lexer);
        if (lexer->lookahead == '>') {
            PROFILE_COUNT(scan_lt_as_tag_boundary_successes);
            return true;
        }
        return false;
    }

    if (!is_space(lexer->lookahead)) {
        return false;
    }

    while (is_space(lexer->lookahead)) {
        PROFILE_ADVANCE(scan_lt_as_tag_boundary_bytes, lexer);
    }

    bool is_boundary = lexer->lookahead == '>'
        || lexer->lookahead == '/'
        || lexer->lookahead == '{'
        || lexer->lookahead == '"'
        || lexer->lookahead == '\''
        || lexer->lookahead == '|'
        || is_ident_start(lexer->lookahead);
    if (is_boundary) {
        PROFILE_COUNT(scan_lt_as_tag_boundary_successes);
    }
    return is_boundary;
}

// Scan balanced expression with trailing whitespace exclusion, comment handling,
// tag boundary detection, and Svelte block marker detection.
// Used for iterator expressions, binding patterns, key expressions, tag expressions,
// and snippet parameters.
static bool scan_balanced(TSLexer *lexer, int32_t stop_char, bool stop_comma, bool allow_eof) {
    PROFILE_COUNT(scan_balanced_calls);
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
                PROFILE_ADVANCE(scan_balanced_bytes, lexer);
            }
                has_content = true;
                needs_mark = true;
                continue;
            }

            if (lexer->lookahead == '*') {
                PROFILE_ADVANCE(scan_balanced_bytes, lexer);
                while (lexer->lookahead) {
                    if (lexer->lookahead != '*') {
                        PROFILE_ADVANCE(scan_balanced_bytes, lexer);
                        continue;
                    }

                    PROFILE_ADVANCE(scan_balanced_bytes, lexer);
                    if (lexer->lookahead == '/') {
                        PROFILE_ADVANCE(scan_balanced_bytes, lexer);
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
            do { PROFILE_ADVANCE(scan_balanced_bytes, lexer); } while (is_space(lexer->lookahead));
            continue;
        }

        // At depth 0, '{' followed by a Svelte block marker (#, :, /, @)
        // is a block boundary — terminate the balanced scan.
        if (depth == 0 && c == '{') {
            if (needs_mark) {
                lexer->mark_end(lexer);
                needs_mark = false;
            }
            PROFILE_ADVANCE(scan_balanced_bytes, lexer);
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

        PROFILE_ADVANCE(scan_balanced_bytes, lexer);
        has_content = true;
        needs_mark = true;
    }

done:
    if (needs_mark) {
        lexer->mark_end(lexer);
    }

    if (allow_eof && has_content && lexer->lookahead == 0) {
        PROFILE_COUNT(scan_balanced_successes);
        return true;
    }

    bool success = has_content && found_terminator;
    if (success) {
        PROFILE_COUNT(scan_balanced_successes);
    }
    return success;
}

static inline bool match_keyword(TSLexer *lexer, const char *kw, int len) {
    for (int i = 0; i < len; i++) {
        if (lexer->lookahead != kw[i]) return false;
        PROFILE_ADVANCE(scan_iterator_bytes, lexer);
    }
    // Keyword must be followed by space, '{', or '}' (end of block)
    return is_space(lexer->lookahead) || lexer->lookahead == '{' || lexer->lookahead == '}';
}

static bool scan_iterator(State *state, TSLexer *lexer) {
    PROFILE_COUNT(scan_iterator_calls);
    int depth = 0;
    bool has_content = false;
    bool found_terminator = false;

    while (is_space(lexer->lookahead)) PROFILE_SKIP(scan_iterator_bytes, lexer);

    // Empty expression: produce zero-width token at the terminator position.
    // This lets the compiler know WHERE the expression would be, even when absent.
    if (lexer->lookahead == '}') {
        lexer->mark_end(lexer);
        lexer->result_symbol = state->is_typescript ? ITERATOR_EXPRESSION_TS : ITERATOR_EXPRESSION_JS;
        PROFILE_COUNT(scan_iterator_successes);
        return true;
    }

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        if (depth == 0 && c == '}') {
            found_terminator = true;
            break;
        }

        // At depth 0, classify '<' as either tag boundary or expression operator.
        if (depth == 0 && c == '<') {
            PROFILE_ADVANCE(scan_iterator_bytes, lexer);
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
            while (is_space(lexer->lookahead)) PROFILE_ADVANCE(scan_iterator_bytes, lexer);

            c = lexer->lookahead;
            if (c == '<' && scan_lt_as_tag_boundary(lexer)) {
                lexer->result_symbol = state->is_typescript ? ITERATOR_EXPRESSION_TS : ITERATOR_EXPRESSION_JS;
                PROFILE_COUNT(scan_iterator_successes);
                return has_content;
            }
            if (c == 'a') {
                PROFILE_ADVANCE(scan_iterator_bytes, lexer);
                if (match_keyword(lexer, "s", 1)) {
                    lexer->result_symbol = state->is_typescript ? ITERATOR_EXPRESSION_TS : ITERATOR_EXPRESSION_JS;
                    PROFILE_COUNT(scan_iterator_successes);
                    return has_content;
                }
            } else if (c == 't' && match_keyword(lexer, "then", 4)) {
                lexer->result_symbol = state->is_typescript ? ITERATOR_EXPRESSION_TS : ITERATOR_EXPRESSION_JS;
                PROFILE_COUNT(scan_iterator_successes);
                return has_content;
            } else if (c == 'c' && match_keyword(lexer, "catch", 5)) {
                lexer->result_symbol = state->is_typescript ? ITERATOR_EXPRESSION_TS : ITERATOR_EXPRESSION_JS;
                PROFILE_COUNT(scan_iterator_successes);
                return has_content;
            } else if (c == 0) {
                lexer->result_symbol = state->is_typescript ? ITERATOR_EXPRESSION_TS : ITERATOR_EXPRESSION_JS;
                PROFILE_COUNT(scan_iterator_successes);
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

        PROFILE_ADVANCE(scan_iterator_bytes, lexer);
        has_content = true;
    }

    // Only return true if we found a valid terminator
    if (has_content && (found_terminator || lexer->lookahead == 0)) {
        lexer->mark_end(lexer);
        lexer->result_symbol = state->is_typescript ? ITERATOR_EXPRESSION_TS : ITERATOR_EXPRESSION_JS;
        PROFILE_COUNT(scan_iterator_successes);
        return true;
    }
    return false;
}

static bool scan_binding(State *state, TSLexer *lexer) {
    PROFILE_COUNT(scan_binding_calls);
    while (is_space(lexer->lookahead)) PROFILE_ADVANCE(scan_balanced_bytes, lexer);
    if (!scan_balanced(lexer, '(', true, true)) return false;

    lexer->result_symbol = state->is_typescript ? BINDING_PATTERN_TS : BINDING_PATTERN_JS;
    PROFILE_COUNT(scan_binding_successes);
    return true;
}

static bool scan_key(State *state, TSLexer *lexer) {
    PROFILE_COUNT(scan_key_calls);
    while (is_space(lexer->lookahead)) PROFILE_ADVANCE(scan_balanced_bytes, lexer);
    if (!scan_balanced(lexer, ')', false, true)) return false;

    lexer->result_symbol = state->is_typescript ? KEY_EXPRESSION_TS : KEY_EXPRESSION_JS;
    PROFILE_COUNT(scan_key_successes);
    return true;
}

static bool scan_snippet_parameter(State *state, TSLexer *lexer) {
    PROFILE_COUNT(scan_snippet_parameter_calls);
    while (is_space(lexer->lookahead)) PROFILE_ADVANCE(scan_balanced_bytes, lexer);
    if (!scan_balanced(lexer, ')', true, true)) return false;

    lexer->result_symbol = state->is_typescript ? SNIPPET_PARAMETER_TS : SNIPPET_PARAMETER_JS;
    PROFILE_COUNT(scan_snippet_parameter_successes);
    return true;
}

static bool scan_block_start_eof(TSLexer *lexer) {
    if (lexer->lookahead != 0) return false;

    lexer->mark_end(lexer);
    lexer->result_symbol = BLOCK_START_EOF;
    return true;
}

static bool scan_block_eof(TSLexer *lexer) {
    if (lexer->lookahead != 0) return false;

    lexer->mark_end(lexer);
    lexer->result_symbol = BLOCK_EOF;
    return true;
}

// Tag expression: content after {@html, {@debug, etc.
// Requires leading whitespace, then delegates to scan_balanced.
static bool scan_tag_expression(State *state, TSLexer *lexer) {
    PROFILE_COUNT(scan_tag_expression_calls);
    bool has_space = false;
    while (is_space(lexer->lookahead)) {
        PROFILE_SKIP(scan_tag_expression_bytes, lexer);
        has_space = true;
    }

    if (!has_space) return false;

    // Empty expression: produce zero-width token at the terminator position.
    if (lexer->lookahead == '}') {
        lexer->mark_end(lexer);
        lexer->result_symbol = state->is_typescript ? TAG_EXPRESSION_TS : TAG_EXPRESSION_JS;
        PROFILE_COUNT(scan_tag_expression_successes);
        return true;
    }

    if (!scan_balanced(lexer, '}', false, false)) return false;

    lexer->result_symbol = state->is_typescript ? TAG_EXPRESSION_TS : TAG_EXPRESSION_JS;
    PROFILE_COUNT(scan_tag_expression_successes);
    return true;
}

static bool scan_snippet_type_params(TSLexer *lexer) {
    PROFILE_COUNT(scan_snippet_type_params_calls);
    if (lexer->lookahead != '<') return false;

    int depth = 0;

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        if (skip_string(lexer)) {
            continue;
        }

        if (c == '<') {
            depth++;
            PROFILE_ADVANCE(scan_snippet_type_params_bytes, lexer);
            continue;
        }

        if (c == '>') {
            depth--;
            PROFILE_ADVANCE(scan_snippet_type_params_bytes, lexer);
            if (depth == 0) {
                lexer->mark_end(lexer);
                lexer->result_symbol = SNIPPET_TYPE_PARAMS;
                PROFILE_COUNT(scan_snippet_type_params_successes);
                return true;
            }
            continue;
        }

        PROFILE_ADVANCE(scan_snippet_type_params_bytes, lexer);
    }

    return false;
}

// Snippet name: identifier or zero-width token when name is absent.
static bool scan_snippet_name(TSLexer *lexer) {
    PROFILE_COUNT(scan_snippet_name_calls);
    while (is_space(lexer->lookahead)) PROFILE_SKIP(scan_snippet_name_bytes, lexer);

    // Empty name: produce zero-width token at the terminator position.
    if (lexer->lookahead == '}' || lexer->lookahead == '(') {
        lexer->mark_end(lexer);
        lexer->result_symbol = SNIPPET_NAME;
        PROFILE_COUNT(scan_snippet_name_successes);
        return true;
    }

    // Must start with identifier start char or $
    if (!is_ident_start(lexer->lookahead) && lexer->lookahead != '$') return false;

    PROFILE_ADVANCE(scan_snippet_name_bytes, lexer);
    while (is_ident_char(lexer->lookahead) || lexer->lookahead == '$') {
        PROFILE_ADVANCE(scan_snippet_name_bytes, lexer);
    }

    lexer->mark_end(lexer);
    while (is_space(lexer->lookahead)) PROFILE_SKIP(scan_snippet_name_bytes, lexer);
    if (
        lexer->lookahead != '}'
        && lexer->lookahead != '('
        && lexer->lookahead != '<'
    ) {
        return false;
    }
    lexer->result_symbol = SNIPPET_NAME;
    PROFILE_COUNT(scan_snippet_name_successes);
    return true;
}

// Match {/ only when followed by identifier start (block end like {/if}).
// Returns false for {/* or {// (JS comments inside expressions).
static bool scan_block_end_open(TSLexer *lexer) {
    PROFILE_COUNT(scan_block_end_open_calls);
    if (lexer->lookahead != '{') return false;
    PROFILE_ADVANCE(scan_block_end_open_bytes, lexer);
    if (lexer->lookahead != '/') return false;
    PROFILE_ADVANCE(scan_block_end_open_bytes, lexer);
    if (!is_ident_start(lexer->lookahead)) return false;
    lexer->mark_end(lexer);
    lexer->result_symbol = BLOCK_END_OPEN;
    PROFILE_COUNT(scan_block_end_open_successes);
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
    PROFILE_COUNT(svelte_scan_calls);
    // BLOCK_END_OPEN: disambiguates {/if} (block end) from {/* comment */} (expression).
    // Only attempt when lookahead is '{' — otherwise fall through to HTMLX scanner
    // so it can produce TEXT and other tokens that don't start with '{'.
    if (valid[BLOCK_END_OPEN] && lexer->lookahead == '{') return scan_block_end_open(lexer);
    if (valid[BLOCK_START_EOF] && lexer->lookahead == 0) return scan_block_start_eof(lexer);
    if (valid[BLOCK_EOF] && lexer->lookahead == 0) return scan_block_eof(lexer);

    // Svelte block expression tokens — each valid in exclusive grammar contexts.
    // SNIPPET_NAME must be checked before SNIPPET_TYPE_PARAMS and SNIPPET_PARAMETER
    // because all three can be valid simultaneously (all are optional in the grammar).
    if (valid[SNIPPET_NAME]) return scan_snippet_name(lexer);
    if (valid[SNIPPET_TYPE_PARAMS]) return scan_snippet_type_params(lexer);
    if (valid[SNIPPET_PARAMETER_JS] || valid[SNIPPET_PARAMETER_TS]) return scan_snippet_parameter(state, lexer);
    if (valid[ITERATOR_EXPRESSION_JS] || valid[ITERATOR_EXPRESSION_TS]) return scan_iterator(state, lexer);
    if (valid[BINDING_PATTERN_JS] || valid[BINDING_PATTERN_TS]) return scan_binding(state, lexer);
    if (valid[KEY_EXPRESSION_JS] || valid[KEY_EXPRESSION_TS]) return scan_key(state, lexer);
    if (valid[TAG_EXPRESSION_JS] || valid[TAG_EXPRESSION_TS]) return scan_tag_expression(state, lexer);

    // Fall through to HTMLX scanner for all other tokens.
    PROFILE_COUNT(htmlx_fallback_calls);
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
