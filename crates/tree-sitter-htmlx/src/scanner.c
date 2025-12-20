/**
 * HTMLX External Scanner
 *
 * Extends HTML with namespaced tags and expression support.
 */

#include "tree_sitter/parser.h"
#include <wctype.h>
#include <string.h>

#define TokenType                  HtmlTokenType
#define START_TAG_NAME             HTML_START_TAG_NAME
#define SCRIPT_START_TAG_NAME      HTML_SCRIPT_START_TAG_NAME
#define STYLE_START_TAG_NAME       HTML_STYLE_START_TAG_NAME
#define END_TAG_NAME               HTML_END_TAG_NAME
#define ERRONEOUS_END_TAG_NAME     HTML_ERRONEOUS_END_TAG_NAME
#define SELF_CLOSING_TAG_DELIMITER HTML_SELF_CLOSING_TAG_DELIMITER
#define IMPLICIT_END_TAG           HTML_IMPLICIT_END_TAG
#define RAW_TEXT                   HTML_RAW_TEXT
#define COMMENT                    HTML_COMMENT
#define scan                       html_scan

#define tree_sitter_html_external_scanner_create      html_create
#define tree_sitter_html_external_scanner_destroy     html_destroy
#define tree_sitter_html_external_scanner_scan        html_scanner_scan
#define tree_sitter_html_external_scanner_serialize   html_serialize
#define tree_sitter_html_external_scanner_deserialize html_deserialize

#include "../../../external/tree-sitter-html/src/scanner.c"

#undef TokenType
#undef START_TAG_NAME
#undef SCRIPT_START_TAG_NAME
#undef STYLE_START_TAG_NAME
#undef END_TAG_NAME
#undef ERRONEOUS_END_TAG_NAME
#undef SELF_CLOSING_TAG_DELIMITER
#undef IMPLICIT_END_TAG
#undef RAW_TEXT
#undef COMMENT
#undef scan
#undef tree_sitter_html_external_scanner_create
#undef tree_sitter_html_external_scanner_destroy
#undef tree_sitter_html_external_scanner_scan
#undef tree_sitter_html_external_scanner_serialize
#undef tree_sitter_html_external_scanner_deserialize

enum TokenType {
    START_TAG_NAME,
    SCRIPT_START_TAG_NAME,
    STYLE_START_TAG_NAME,
    END_TAG_NAME,
    ERRONEOUS_END_TAG_NAME,
    SELF_CLOSING_TAG_DELIMITER,
    IMPLICIT_END_TAG,
    RAW_TEXT,
    COMMENT,
    TAG_NAMESPACE,
    TAG_LOCAL_NAME,
    TS_LANG_MARKER,  // Zero-width marker that sets TypeScript mode
    EXPRESSION_JS,
    EXPRESSION_TS,
};

typedef struct {
    Scanner *html;
    bool awaiting_local_name;
    bool is_typescript;
} State;

static inline bool is_alpha(int32_t c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
}

static inline bool is_alnum(int32_t c) {
    return is_alpha(c) || (c >= '0' && c <= '9');
}

static inline bool is_name_char(int32_t c) {
    return is_alnum(c) || c == '-' || c == '_';
}

static inline bool is_space(int32_t c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r';
}

static inline int32_t to_upper(int32_t c) {
    return (c >= 'a' && c <= 'z') ? (c & ~0x20) : c;
}

static inline int32_t to_lower(int32_t c) {
    return (c >= 'A' && c <= 'Z') ? (c | 0x20) : c;
}

static bool scan_start_tag(State *state, TSLexer *lexer, const bool *valid) {
    if (!is_alpha(lexer->lookahead)) return false;

    String name = array_new();
    while (is_name_char(lexer->lookahead)) {
        array_push(&name, (char)to_upper(lexer->lookahead));
        lexer->advance(lexer, false);
    }

    if (lexer->lookahead == ':' && valid[TAG_NAMESPACE]) {
        lexer->mark_end(lexer);
        lexer->result_symbol = TAG_NAMESPACE;
        state->awaiting_local_name = true;
        array_delete(&name);
        return true;
    }

    if (name.size > 0 && (valid[START_TAG_NAME] ||
                          valid[SCRIPT_START_TAG_NAME] ||
                          valid[STYLE_START_TAG_NAME])) {
        lexer->mark_end(lexer);
        Tag tag = tag_for_name(name);
        array_push(&state->html->tags, tag);

        switch (tag.type) {
            case SCRIPT: lexer->result_symbol = SCRIPT_START_TAG_NAME; break;
            case STYLE:  lexer->result_symbol = STYLE_START_TAG_NAME; break;
            default:     lexer->result_symbol = START_TAG_NAME; break;
        }
        return true;
    }

    array_delete(&name);
    return false;
}

static bool scan_local_name(State *state, TSLexer *lexer) {
    if (!is_alpha(lexer->lookahead)) return false;

    while (is_name_char(lexer->lookahead)) {
        lexer->advance(lexer, false);
    }

    lexer->mark_end(lexer);
    lexer->result_symbol = TAG_LOCAL_NAME;
    state->awaiting_local_name = false;
    return true;
}

static bool scan_end_tag(State *state, TSLexer *lexer, const bool *valid) {
    if (!is_alpha(lexer->lookahead)) return false;

    String name = array_new();
    while (is_name_char(lexer->lookahead)) {
        array_push(&name, (char)to_upper(lexer->lookahead));
        lexer->advance(lexer, false);
    }

    if (lexer->lookahead == ':' && valid[TAG_NAMESPACE]) {
        lexer->mark_end(lexer);
        lexer->result_symbol = TAG_NAMESPACE;
        state->awaiting_local_name = true;
        array_delete(&name);
        return true;
    }

    if (name.size == 0) {
        array_delete(&name);
        return false;
    }

    lexer->mark_end(lexer);

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

static bool scan_self_closing(State *state, TSLexer *lexer) {
    lexer->advance(lexer, false);
    if (lexer->lookahead != '>') return false;

    lexer->advance(lexer, false);
    lexer->mark_end(lexer);

    if (state->html->tags.size > 0) {
        Tag popped = array_pop(&state->html->tags);
        tag_free(&popped);
    }

    lexer->result_symbol = SELF_CLOSING_TAG_DELIMITER;
    return true;
}

static inline bool skip_string(TSLexer *lexer) {
    int32_t quote = lexer->lookahead;
    if (quote != '"' && quote != '\'' && quote != '`') return false;

    lexer->advance(lexer, false);
    while (lexer->lookahead && lexer->lookahead != quote) {
        int32_t c = lexer->lookahead;
        if (c == '\\') {
            lexer->advance(lexer, false);
            if (lexer->lookahead) lexer->advance(lexer, false);
        } else if (quote == '`' && c == '$') {
            lexer->advance(lexer, false);
            if (lexer->lookahead == '{') {
                // Skip ${...} interpolation with balanced brace tracking
                lexer->advance(lexer, false);
                for (int depth = 1; lexer->lookahead && depth > 0;) {
                    c = lexer->lookahead;
                    if (c == '"' || c == '\'' || c == '`') {
                        skip_string(lexer);
                    } else {
                        if (c == '{') depth++;
                        else if (c == '}') depth--;
                        lexer->advance(lexer, false);
                    }
                }
            }
        } else {
            lexer->advance(lexer, false);
        }
    }
    if (lexer->lookahead == quote) lexer->advance(lexer, false);
    return true;
}

static bool scan_balanced_expr(TSLexer *lexer) {
    int depth = 0;
    bool has_content = false;

    while (lexer->lookahead) {
        int32_t c = lexer->lookahead;

        if (depth == 0 && c == '}') break;

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

// Lookahead to check if this is lang="ts" or lang="typescript" without consuming input
static bool check_ts_lang_attr(TSLexer *lexer) {
    // Skip whitespace
    while (is_space(lexer->lookahead)) lexer->advance(lexer, false);

    const char *lang = "lang";
    for (int i = 0; lang[i]; i++) {
        if (to_lower(lexer->lookahead) != lang[i]) return false;
        lexer->advance(lexer, false);
    }

    while (is_space(lexer->lookahead)) lexer->advance(lexer, false);
    if (lexer->lookahead != '=') return false;
    lexer->advance(lexer, false);
    while (is_space(lexer->lookahead)) lexer->advance(lexer, false);

    int32_t quote = lexer->lookahead;
    if (quote != '"' && quote != '\'') return false;
    lexer->advance(lexer, false);

    int32_t c = to_lower(lexer->lookahead);
    if (c != 't') return false;
    lexer->advance(lexer, false);

    c = to_lower(lexer->lookahead);
    if (c != 's') return false;
    lexer->advance(lexer, false);

    // Check for "ts" (short form)
    if (lexer->lookahead == quote) {
        return true;
    }

    // Check for "typescript" (long form)
    const char *cript = "cript";
    for (int i = 0; cript[i]; i++) {
        if (to_lower(lexer->lookahead) != cript[i]) return false;
        lexer->advance(lexer, false);
    }

    return lexer->lookahead == quote;
}

// Zero-width token that sets TypeScript mode without consuming input
static bool scan_ts_lang_marker(State *state, TSLexer *lexer) {
    // Mark end at current position (zero-width)
    lexer->mark_end(lexer);

    // Lookahead to check if this is a TypeScript lang attribute
    if (!check_ts_lang_attr(lexer)) return false;

    // Set TypeScript mode and return zero-width token
    state->is_typescript = true;
    lexer->result_symbol = TS_LANG_MARKER;
    return true;
}

static bool scan_expression(State *state, TSLexer *lexer) {
    while (is_space(lexer->lookahead)) lexer->advance(lexer, true);

    int32_t c = lexer->lookahead;
    if (c == '#' || c == ':' || c == '@' || c == '/') return false;

    if (!scan_balanced_expr(lexer)) return false;

    lexer->mark_end(lexer);
    lexer->result_symbol = state->is_typescript ? EXPRESSION_TS : EXPRESSION_JS;
    return true;
}

static bool scan(State *state, TSLexer *lexer, const bool *valid) {
    if (valid[TS_LANG_MARKER] && scan_ts_lang_marker(state, lexer)) {
        return true;
    }

    if ((valid[EXPRESSION_JS] || valid[EXPRESSION_TS]) && scan_expression(state, lexer)) {
        return true;
    }

    if (valid[RAW_TEXT] && !valid[START_TAG_NAME] && !valid[END_TAG_NAME]) {
        return html_scanner_scan(state->html, lexer, valid);
    }

    while (is_space(lexer->lookahead)) lexer->advance(lexer, true);

    if (state->awaiting_local_name && valid[TAG_LOCAL_NAME]) {
        return scan_local_name(state, lexer);
    }

    int32_t c = lexer->lookahead;

    if (c == '/' && valid[SELF_CLOSING_TAG_DELIMITER]) {
        lexer->mark_end(lexer);
        if (scan_self_closing(state, lexer)) return true;
    }

    if (is_alpha(c)) {
        if (valid[TAG_NAMESPACE] || valid[START_TAG_NAME] ||
            valid[SCRIPT_START_TAG_NAME] || valid[STYLE_START_TAG_NAME]) {
            if (scan_start_tag(state, lexer, valid)) return true;
        }
        if (valid[TAG_NAMESPACE] || valid[END_TAG_NAME]) {
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
