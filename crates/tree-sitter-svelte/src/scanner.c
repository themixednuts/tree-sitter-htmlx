#define tree_sitter_htmlx_external_scanner_create      htmlx_create
#define tree_sitter_htmlx_external_scanner_destroy     htmlx_destroy
#define tree_sitter_htmlx_external_scanner_scan        htmlx_scanner_scan
#define tree_sitter_htmlx_external_scanner_serialize   htmlx_serialize
#define tree_sitter_htmlx_external_scanner_deserialize htmlx_deserialize

// Vendored by build.rs from tree-sitter-htmlx (includes html scanner)
#include "htmlx/scanner.c"

// Svelte external token indices (after HTMLX's 15 tokens: 0-14)
// HTML tokens (0-8) + HTMLX tokens (9-14)
enum {
    ITERATOR_EXPRESSION = 15,
    BINDING_PATTERN,
    KEY_EXPRESSION,
    TAG_EXPRESSION,
};

static bool scan_balanced(TSLexer *lexer, int32_t stop_char, bool stop_comma) {
    int depth = 0;
    bool has_content = false;

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        if (depth == 0 && (c == stop_char || c == '}')) break;
        if (depth == 0 && stop_comma && c == ',') break;

        if (skip_string(lexer)) {
            has_content = true;
            continue;
        }

        switch (c) {
            case '(': case '[': case '{': depth++; break;
            case ')': case ']': case '}': if (--depth < 0) return has_content; break;
        }

        advance(lexer);
        has_content = true;
    }

    return has_content;
}

static inline bool match_keyword(TSLexer *lexer, const char *kw, int len) {
    for (int i = 0; i < len; i++) {
        if (lexer->lookahead != kw[i]) return false;
        advance(lexer);
    }
    return is_space(lexer->lookahead) || lexer->lookahead == '{';
}

static bool scan_iterator(TSLexer *lexer) {
    int depth = 0;
    bool has_content = false;

    while (is_space(lexer->lookahead)) skip(lexer);

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        if (depth == 0 && c == '}') break;

        if (depth == 0 && is_space(c)) {
            lexer->mark_end(lexer);
            while (is_space(lexer->lookahead)) advance(lexer);

            c = lexer->lookahead;
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

    if (has_content) {
        lexer->mark_end(lexer);
        lexer->result_symbol = ITERATOR_EXPRESSION;
    }
    return has_content;
}

static bool scan_binding(TSLexer *lexer) {
    while (is_space(lexer->lookahead)) advance(lexer);
    if (!scan_balanced(lexer, '(', true)) return false;

    lexer->mark_end(lexer);
    lexer->result_symbol = BINDING_PATTERN;
    return true;
}

static bool scan_key(TSLexer *lexer) {
    while (is_space(lexer->lookahead)) advance(lexer);
    if (!scan_balanced(lexer, ')', false)) return false;

    lexer->mark_end(lexer);
    lexer->result_symbol = KEY_EXPRESSION;
    return true;
}

static bool scan_tag_expression(TSLexer *lexer) {
    bool has_space = false;
    while (is_space(lexer->lookahead)) {
        skip(lexer);
        has_space = true;
    }

    if (!has_space || lexer->lookahead == '}') return false;
    if (!scan_balanced(lexer, '}', false)) return false;

    lexer->mark_end(lexer);
    lexer->result_symbol = TAG_EXPRESSION;
    return true;
}

static bool svelte_scan(State *state, TSLexer *lexer, const bool *valid) {
    if (valid[ITERATOR_EXPRESSION]) return scan_iterator(lexer);
    if (valid[BINDING_PATTERN]) return scan_binding(lexer);
    if (valid[KEY_EXPRESSION]) return scan_key(lexer);
    if (valid[TAG_EXPRESSION]) return scan_tag_expression(lexer);

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
