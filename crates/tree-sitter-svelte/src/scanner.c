/**
 * Svelte External Scanner
 *
 * Extends HTMLX with block expression parsing.
 */

#define tree_sitter_htmlx_external_scanner_create      htmlx_create
#define tree_sitter_htmlx_external_scanner_destroy     htmlx_destroy
#define tree_sitter_htmlx_external_scanner_scan        htmlx_scan
#define tree_sitter_htmlx_external_scanner_serialize   htmlx_serialize
#define tree_sitter_htmlx_external_scanner_deserialize htmlx_deserialize

#include "../../tree-sitter-htmlx/src/scanner.c"

#undef tree_sitter_htmlx_external_scanner_create
#undef tree_sitter_htmlx_external_scanner_destroy
#undef tree_sitter_htmlx_external_scanner_scan
#undef tree_sitter_htmlx_external_scanner_serialize
#undef tree_sitter_htmlx_external_scanner_deserialize

enum SvelteTokenType {
    SVELTE_ITERATOR_EXPRESSION = 14,
    SVELTE_BINDING_PATTERN = 15,
    SVELTE_KEY_EXPRESSION = 16,
    SVELTE_TAG_EXPRESSION = 17,
};

static bool scan_balanced(TSLexer *lexer, int32_t stop_char, bool stop_comma) {
    int depth = 0;
    bool has_content = false;

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        if (depth == 0) {
            if (c == stop_char || c == '}') break;
            if (stop_comma && c == ',') break;
        }

        if (skip_string(lexer)) {
            has_content = true;
            continue;
        }

        switch (c) {
            case '(': case '[': case '{': depth++; break;
            case ')': case ']': case '}': if (--depth < 0) return has_content; break;
        }

        lexer->advance(lexer, false);
        has_content = true;
    }

    return has_content;
}

static bool check_keyword(TSLexer *lexer, const char *keyword) {
    for (int i = 0; keyword[i]; i++) {
        if (lexer->lookahead != keyword[i]) return false;
        lexer->advance(lexer, false);
    }
    return is_space(lexer->lookahead) || lexer->lookahead == '{';
}

static bool scan_iterator(TSLexer *lexer) {
    int depth = 0;
    bool has_content = false;

    while (is_space(lexer->lookahead)) lexer->advance(lexer, true);

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        // Stop at closing brace at depth 0
        if (depth == 0 && c == '}') break;

        if (depth == 0 && is_space(c)) {
            lexer->mark_end(lexer);

            while (is_space(lexer->lookahead)) lexer->advance(lexer, false);

            // Check for keywords that end the iterator expression
            // 'as' for each blocks, 'then'/'catch' for await blocks
            if (lexer->lookahead == 'a') {
                lexer->advance(lexer, false);
                if (lexer->lookahead == 's' && check_keyword(lexer, "s")) {
                    lexer->result_symbol = SVELTE_ITERATOR_EXPRESSION;
                    return has_content;
                }
            } else if (lexer->lookahead == 't') {
                if (check_keyword(lexer, "then")) {
                    lexer->result_symbol = SVELTE_ITERATOR_EXPRESSION;
                    return has_content;
                }
            } else if (lexer->lookahead == 'c') {
                if (check_keyword(lexer, "catch")) {
                    lexer->result_symbol = SVELTE_ITERATOR_EXPRESSION;
                    return has_content;
                }
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

        lexer->advance(lexer, false);
        has_content = true;
    }

    if (has_content) {
        lexer->mark_end(lexer);
        lexer->result_symbol = SVELTE_ITERATOR_EXPRESSION;
    }
    return has_content;
}

static bool scan_binding(TSLexer *lexer) {
    while (is_space(lexer->lookahead)) lexer->advance(lexer, false);

    if (!scan_balanced(lexer, '(', true)) return false;

    lexer->mark_end(lexer);
    lexer->result_symbol = SVELTE_BINDING_PATTERN;
    return true;
}

static bool scan_key(TSLexer *lexer) {
    while (is_space(lexer->lookahead)) lexer->advance(lexer, false);

    if (!scan_balanced(lexer, ')', false)) return false;

    lexer->mark_end(lexer);
    lexer->result_symbol = SVELTE_KEY_EXPRESSION;
    return true;
}

static bool scan_tag_expression(TSLexer *lexer) {
    // Skip leading whitespace
    bool has_space = false;
    while (is_space(lexer->lookahead)) {
        lexer->advance(lexer, true);
        has_space = true;
    }

    // Must have content after the tag kind
    if (lexer->lookahead == '}') return false;
    if (!has_space) return false;

    // Scan balanced content up to closing }
    if (!scan_balanced(lexer, '}', false)) return false;

    lexer->mark_end(lexer);
    lexer->result_symbol = SVELTE_TAG_EXPRESSION;
    return true;
}

static bool svelte_scan(State *state, TSLexer *lexer, const bool *valid) {
    if (valid[SVELTE_ITERATOR_EXPRESSION]) return scan_iterator(lexer);
    if (valid[SVELTE_BINDING_PATTERN]) return scan_binding(lexer);
    if (valid[SVELTE_KEY_EXPRESSION]) return scan_key(lexer);
    if (valid[SVELTE_TAG_EXPRESSION]) return scan_tag_expression(lexer);

    return htmlx_scan(state, lexer, valid);
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
