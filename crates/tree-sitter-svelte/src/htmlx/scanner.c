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

// Vendored by build.rs from external/tree-sitter-html
#include "html/scanner.c"

#undef scan
#undef tree_sitter_html_external_scanner_create
#undef tree_sitter_html_external_scanner_destroy
#undef tree_sitter_html_external_scanner_scan
#undef tree_sitter_html_external_scanner_serialize
#undef tree_sitter_html_external_scanner_deserialize

enum {
    TAG_NAMESPACE = 9,
    TAG_LOCAL_NAME,
    TS_LANG_MARKER,
    EXPRESSION_JS,
    EXPRESSION_TS,
    DIRECTIVE_MARKER,
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

static bool scan_start_tag(State *state, TSLexer *lexer, const bool *valid) {
    if (!is_alpha(lexer->lookahead)) return false;

    String name = array_new();
    while (is_name_char(lexer->lookahead)) {
        array_push(&name, (char)to_upper(lexer->lookahead));
        advance(lexer);
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

    while (is_name_char(lexer->lookahead)) advance(lexer);

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
        advance(lexer);
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
    advance(lexer);
    if (lexer->lookahead != '>') return false;

    advance(lexer);
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

        advance(lexer);
        has_content = true;
    }

    return has_content;
}

static bool check_ts_lang_attr(TSLexer *lexer) {
    while (is_space(lexer->lookahead)) advance(lexer);

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

    if (!scan_balanced_expr(lexer)) return false;

    lexer->mark_end(lexer);
    lexer->result_symbol = state->is_typescript ? EXPRESSION_TS : EXPRESSION_JS;
    return true;
}

// Returns: 1 = matched, 0 = not at identifier, -1 = identifier without colon
static int check_directive_marker(TSLexer *lexer) {
    while (is_space(lexer->lookahead)) skip(lexer);
    lexer->mark_end(lexer);

    if (!is_ident_start(lexer->lookahead)) return 0;
    while (is_ident_char(lexer->lookahead)) advance(lexer);
    if (lexer->lookahead != ':') return -1;

    lexer->result_symbol = DIRECTIVE_MARKER;
    return 1;
}

static bool scan(State *state, TSLexer *lexer, const bool *valid) {
    if (valid[TS_LANG_MARKER] && scan_ts_lang_marker(state, lexer)) {
        return true;
    }

    if (valid[DIRECTIVE_MARKER]) {
        int result = check_directive_marker(lexer);
        if (result != 0) return result == 1;
    }

    while (is_space(lexer->lookahead)) skip(lexer);

    if ((valid[EXPRESSION_JS] || valid[EXPRESSION_TS]) && scan_expression(state, lexer)) {
        return true;
    }

    if (valid[RAW_TEXT] && !valid[START_TAG_NAME] && !valid[END_TAG_NAME]) {
        return html_scanner_scan(state->html, lexer, valid);
    }

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
