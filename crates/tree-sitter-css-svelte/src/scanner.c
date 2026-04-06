#include "tree_sitter/parser.h"

#include <stdbool.h>
#include <stdint.h>
#include <string.h>

enum TokenType {
    DESCENDANT_OP,
    PSEUDO_CLASS_SELECTOR_COLON,
    ERROR_RECOVERY,
    AT_RULE_PRELUDE,
    GENERAL_ENCLOSED_VALUE,
    UNICODE_RANGE_VALUE,
    BAD_URL_VALUE,
    FORGIVING_PSEUDO_ELEMENT_RECOVERY,
};

typedef struct {
    uint64_t scan_calls;
    uint64_t scan_at_rule_prelude_calls;
    uint64_t scan_at_rule_prelude_successes;
    uint64_t scan_at_rule_prelude_bytes;
    uint64_t scan_descendant_operator_calls;
    uint64_t scan_descendant_operator_successes;
    uint64_t scan_descendant_operator_bytes;
    uint64_t scan_pseudo_class_colon_calls;
    uint64_t scan_pseudo_class_colon_successes;
    uint64_t scan_pseudo_class_colon_bytes;
    uint64_t scan_forgiving_pseudo_element_calls;
    uint64_t scan_forgiving_pseudo_element_successes;
    uint64_t scan_forgiving_pseudo_element_bytes;
} CssScannerProfileStats;

static CssScannerProfileStats s_profile_stats;

#ifdef TREE_SITTER_CSS_PROFILE
#define PROFILE_COUNT(field) (++s_profile_stats.field)
#define PROFILE_ADVANCE(field, lexer) do { ++s_profile_stats.field; advance(lexer); } while (0)
#define PROFILE_SKIP(field, lexer) do { ++s_profile_stats.field; skip(lexer); } while (0)
#else
#define PROFILE_COUNT(field) ((void)0)
#define PROFILE_ADVANCE(field, lexer) advance(lexer)
#define PROFILE_SKIP(field, lexer) skip(lexer)
#endif

static inline void advance(TSLexer *lexer) { lexer->advance(lexer, false); }

static inline void skip(TSLexer *lexer) { lexer->advance(lexer, true); }

static inline bool is_css_space(int32_t c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f';
}

static inline bool is_ascii_alpha(int32_t c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
}

static inline bool is_ascii_digit(int32_t c) {
    return c >= '0' && c <= '9';
}

static inline bool is_ascii_hex_digit(int32_t c) {
    return is_ascii_digit(c) || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F');
}

static inline bool is_css_ident_start(int32_t c) {
    return is_ascii_alpha(c) || c == '_' || c >= 0x80;
}

static inline bool is_css_ident_continue(int32_t c) {
    return is_css_ident_start(c) || is_ascii_digit(c) || c == '-';
}

static inline bool can_start_selector(int32_t c) {
    return c == '#' || c == '.' || c == '[' || c == '-' || c == '*' || c == '&' || c == '\\'
        || c == '|' || c == ':' || is_css_ident_start(c);
}

static inline bool can_start_pseudo_name(int32_t c) {
    return c == '-' || c == '\\' || is_css_ident_start(c);
}

static bool scan_css_comment(TSLexer *lexer) {
    if (lexer->lookahead != '/') {
        return false;
    }
    advance(lexer);
    if (lexer->lookahead != '*') {
        return false;
    }
    advance(lexer);
    while (!lexer->eof(lexer)) {
        int32_t c = lexer->lookahead;
        advance(lexer);
        if (c == '*' && lexer->lookahead == '/') {
            advance(lexer);
            return true;
        }
    }
    return true;
}

static void skip_css_space_and_comments(TSLexer *lexer) {
    for (;;) {
        if (is_css_space(lexer->lookahead)) {
            skip(lexer);
            continue;
        }
        if (lexer->lookahead == '/') {
            TSLexer copy = *lexer;
            if (scan_css_comment(&copy)) {
                advance(lexer);
                advance(lexer);
                while (!lexer->eof(lexer)) {
                    int32_t c = lexer->lookahead;
                    advance(lexer);
                    if (c == '*' && lexer->lookahead == '/') {
                        advance(lexer);
                        break;
                    }
                }
                continue;
            }
        }
        break;
    }
}

static bool scan_string_like(TSLexer *lexer, int32_t quote) {
    if (lexer->lookahead != quote) {
        return false;
    }
    advance(lexer);
    while (!lexer->eof(lexer)) {
        int32_t c = lexer->lookahead;
        if (c == '\\') {
            advance(lexer);
            if (!lexer->eof(lexer)) {
                advance(lexer);
            }
            continue;
        }
        if (c == quote) {
            advance(lexer);
            return true;
        }
        if (c == '\n' || c == '\r' || c == '\f') {
            return false;
        }
        advance(lexer);
    }
    return false;
}

static bool is_unquoted_url_char(int32_t c) {
    return !is_css_space(c) && c != '(' && c != ')' && c != '\'' && c != '"' && c != '\\';
}

static void consume_bad_url_remnants(TSLexer *lexer) {
    unsigned depth_paren = 0;
    while (!lexer->eof(lexer)) {
        int32_t c = lexer->lookahead;
        if (c == '\\') {
            advance(lexer);
            if (!lexer->eof(lexer)) {
                advance(lexer);
            }
            continue;
        }
        if (c == '(') {
            depth_paren++;
            advance(lexer);
            continue;
        }
        if (c == ')') {
            if (depth_paren == 0) {
                return;
            }
            depth_paren--;
            advance(lexer);
            continue;
        }
        advance(lexer);
    }
}

static bool scan_bad_url_value(TSLexer *lexer) {
    if (lexer->lookahead == ')') {
        return false;
    }

    bool valid = true;

    if (lexer->lookahead == '\'' || lexer->lookahead == '"') {
        int32_t quote = lexer->lookahead;
        if (!scan_string_like(lexer, quote)) {
            valid = false;
            consume_bad_url_remnants(lexer);
            lexer->mark_end(lexer);
            lexer->result_symbol = BAD_URL_VALUE;
            return true;
        }

        skip_css_space_and_comments(lexer);
        while (can_start_pseudo_name(lexer->lookahead)) {
            while (is_css_ident_continue(lexer->lookahead) || lexer->lookahead == '\\') {
                if (lexer->lookahead == '\\') {
                    advance(lexer);
                    if (!lexer->eof(lexer)) {
                        advance(lexer);
                    }
                } else {
                    advance(lexer);
                }
            }
            if (lexer->lookahead == '(') {
                unsigned depth = 1;
                advance(lexer);
                while (!lexer->eof(lexer) && depth > 0) {
                    int32_t c = lexer->lookahead;
                    if (c == '\\') {
                        advance(lexer);
                        if (!lexer->eof(lexer)) {
                            advance(lexer);
                        }
                        continue;
                    }
                    if (c == '\'' || c == '"') {
                        if (!scan_string_like(lexer, c)) {
                            valid = false;
                            break;
                        }
                        continue;
                    }
                    if (c == '(') depth++;
                    if (c == ')') depth--;
                    advance(lexer);
                }
                if (!valid) break;
            }
            skip_css_space_and_comments(lexer);
        }

        if (valid && lexer->lookahead == ')') {
            return false;
        }
    } else if (is_unquoted_url_char(lexer->lookahead) || lexer->lookahead == '\\') {
        while (!lexer->eof(lexer)) {
            if (is_unquoted_url_char(lexer->lookahead)) {
                advance(lexer);
                continue;
            }
            if (lexer->lookahead == '\\') {
                advance(lexer);
                if (lexer->eof(lexer) || lexer->lookahead == '\n' || lexer->lookahead == '\r' || lexer->lookahead == '\f') {
                    break;
                }
                advance(lexer);
                continue;
            }
            break;
        }
        if (lexer->lookahead == ')') {
            return false;
        }
        skip_css_space_and_comments(lexer);
        if (lexer->lookahead == ')') {
            return false;
        }
    }

    consume_bad_url_remnants(lexer);
    lexer->mark_end(lexer);
    lexer->result_symbol = BAD_URL_VALUE;
    return true;
}

bool tree_sitter_css_profile_enabled(void) {
#ifdef TREE_SITTER_CSS_PROFILE
    return true;
#else
    return false;
#endif
}

void tree_sitter_css_profile_reset(void) {
    memset(&s_profile_stats, 0, sizeof(s_profile_stats));
}

void tree_sitter_css_profile_snapshot(CssScannerProfileStats *out) {
    if (out != NULL) {
        *out = s_profile_stats;
    }
}

void *tree_sitter_css_external_scanner_create() { return NULL; }

void tree_sitter_css_external_scanner_destroy(void *payload) {}

unsigned tree_sitter_css_external_scanner_serialize(void *payload, char *buffer) { return 0; }

void tree_sitter_css_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {}

static bool scan_at_rule_prelude(TSLexer *lexer) {
    PROFILE_COUNT(scan_at_rule_prelude_calls);

    while (is_css_space(lexer->lookahead)) {
        PROFILE_SKIP(scan_at_rule_prelude_bytes, lexer);
    }

    bool has_content = false;
    bool needs_mark = false;
    unsigned depth_paren = 0;
    unsigned depth_bracket = 0;
    bool in_single = false;
    bool in_double = false;

    while (!lexer->eof(lexer)) {
        int32_t c = lexer->lookahead;

        if (!in_single && !in_double && depth_paren == 0 && depth_bracket == 0 && (c == '{' || c == ';')) {
            break;
        }

        if (!in_single && !in_double && c == '/' ) {
            PROFILE_ADVANCE(scan_at_rule_prelude_bytes, lexer);
            if (lexer->lookahead == '*') {
                PROFILE_ADVANCE(scan_at_rule_prelude_bytes, lexer);
                has_content = true;
                needs_mark = true;
                while (!lexer->eof(lexer)) {
                    if (lexer->lookahead == '*') {
                        PROFILE_ADVANCE(scan_at_rule_prelude_bytes, lexer);
                        if (lexer->lookahead == '/') {
                            PROFILE_ADVANCE(scan_at_rule_prelude_bytes, lexer);
                            break;
                        }
                    } else {
                        PROFILE_ADVANCE(scan_at_rule_prelude_bytes, lexer);
                    }
                }
                continue;
            }

            has_content = true;
            needs_mark = true;
            continue;
        }

        if (!in_single && !in_double && depth_paren == 0 && depth_bracket == 0 && is_css_space(c)) {
            if (needs_mark) {
                lexer->mark_end(lexer);
                needs_mark = false;
            }
            do {
                PROFILE_ADVANCE(scan_at_rule_prelude_bytes, lexer);
            } while (is_css_space(lexer->lookahead));
            continue;
        }

        if (c == '\\') {
            PROFILE_ADVANCE(scan_at_rule_prelude_bytes, lexer);
            if (!lexer->eof(lexer)) {
                PROFILE_ADVANCE(scan_at_rule_prelude_bytes, lexer);
            }
            has_content = true;
            needs_mark = true;
            continue;
        }

        if (in_single) {
            PROFILE_ADVANCE(scan_at_rule_prelude_bytes, lexer);
            if (c == '\'') {
                in_single = false;
            }
            has_content = true;
            needs_mark = true;
            continue;
        }

        if (in_double) {
            PROFILE_ADVANCE(scan_at_rule_prelude_bytes, lexer);
            if (c == '"') {
                in_double = false;
            }
            has_content = true;
            needs_mark = true;
            continue;
        }

        switch (c) {
            case '\'':
                in_single = true;
                break;
            case '"':
                in_double = true;
                break;
            case '(':
                depth_paren++;
                break;
            case ')':
                if (depth_paren > 0) depth_paren--;
                break;
            case '[':
                depth_bracket++;
                break;
            case ']':
                if (depth_bracket > 0) depth_bracket--;
                break;
        }

        PROFILE_ADVANCE(scan_at_rule_prelude_bytes, lexer);
        has_content = true;
        needs_mark = true;
    }

    if (needs_mark) {
        lexer->mark_end(lexer);
    }

    if (!has_content) {
        return false;
    }

    if (lexer->lookahead == '{' || lexer->lookahead == ';') {
        lexer->result_symbol = AT_RULE_PRELUDE;
        PROFILE_COUNT(scan_at_rule_prelude_successes);
        return true;
    }

    return false;
}

static bool scan_general_enclosed_value(TSLexer *lexer) {
    if (lexer->lookahead == ')') {
        return false;
    }

    unsigned depth_paren = 0;
    unsigned depth_bracket = 0;
    unsigned depth_brace = 0;
    bool in_single = false;
    bool in_double = false;
    bool in_comment = false;
    bool saw_top_level_non_nested = false;
    bool saw_top_level_colon_or_comparison = false;
    bool saw_top_level_boolean_keyword = false;

    char ident[8];
    unsigned ident_len = 0;
    bool ident_active = false;

    while (!lexer->eof(lexer)) {
        int32_t c = lexer->lookahead;

        if (c == '\\') {
            if (depth_paren == 1 && depth_bracket == 0 && depth_brace == 0) {
                saw_top_level_non_nested = true;
            }
            advance(lexer);
            if (!lexer->eof(lexer)) {
                advance(lexer);
            }
            ident_active = false;
            ident_len = 0;
            continue;
        }

        if (in_single) {
            advance(lexer);
            if (c == '\'') {
                in_single = false;
            }
            continue;
        }

        if (in_double) {
            advance(lexer);
            if (c == '"') {
                in_double = false;
            }
            continue;
        }

        if (in_comment) {
            advance(lexer);
            if (c == '*' && lexer->lookahead == '/') {
                advance(lexer);
                in_comment = false;
            }
            continue;
        }

        if (c == '/') {
            advance(lexer);
            if (lexer->lookahead == '*') {
                advance(lexer);
                in_comment = true;
            }
            ident_active = false;
            ident_len = 0;
            continue;
        }

        bool top_level = depth_paren == 0 && depth_bracket == 0 && depth_brace == 0;

        if (top_level && ident_active && !(is_ascii_alpha(c) || c == '-')) {
            if ((ident_len == 3 && strncmp(ident, "and", 3) == 0)
                || (ident_len == 2 && strncmp(ident, "or", 2) == 0)
                || (ident_len == 3 && strncmp(ident, "not", 3) == 0)) {
                if (c != '(') {
                    saw_top_level_boolean_keyword = true;
                }
            }
            ident_active = false;
            ident_len = 0;
        }

        switch (c) {
            case '\'':
                in_single = true;
                break;
            case '"':
                in_double = true;
                break;
            case '(':
                if (!top_level) {
                    saw_top_level_non_nested = true;
                }
                depth_paren++;
                break;
            case ')':
                if (depth_paren == 0 && depth_bracket == 0 && depth_brace == 0) {
                    lexer->mark_end(lexer);
                    lexer->result_symbol = GENERAL_ENCLOSED_VALUE;
                    return saw_top_level_non_nested && !saw_top_level_colon_or_comparison
                        && !saw_top_level_boolean_keyword;
                }
                if (depth_paren > 0) {
                    depth_paren--;
                }
                break;
            case '[':
                if (top_level) {
                    saw_top_level_non_nested = true;
                }
                depth_bracket++;
                break;
            case ']':
                if (depth_bracket > 0) {
                    depth_bracket--;
                }
                break;
            case '{':
                if (top_level) {
                    saw_top_level_non_nested = true;
                }
                depth_brace++;
                break;
            case '}':
                if (depth_brace > 0) {
                    depth_brace--;
                }
                break;
            case ':':
                if (top_level) {
                    saw_top_level_colon_or_comparison = true;
                    saw_top_level_non_nested = true;
                }
                break;
            case '<':
            case '>':
            case '=':
                if (top_level) {
                    saw_top_level_colon_or_comparison = true;
                    saw_top_level_non_nested = true;
                }
                break;
            default:
                if (top_level && !is_css_space(c)) {
                    saw_top_level_non_nested = true;
                }
                break;
        }

        if (top_level && is_ascii_alpha(c)) {
            if (!ident_active) {
                ident_active = true;
                ident_len = 0;
            }
            if (ident_len + 1 < sizeof(ident)) {
                char lower = (char)c;
                if (lower >= 'A' && lower <= 'Z') {
                    lower = (char)(lower - 'A' + 'a');
                }
                ident[ident_len++] = lower;
                ident[ident_len] = '\0';
            }
        } else if (top_level && ident_active && c != '-') {
            ident_active = false;
            ident_len = 0;
        }

        advance(lexer);
    }

    return false;
}

static bool scan_unicode_range_value(TSLexer *lexer) {
    while (is_css_space(lexer->lookahead)) {
        skip(lexer);
    }

    if (lexer->lookahead != 'u' && lexer->lookahead != 'U') {
        return false;
    }

    advance(lexer);
    if (lexer->lookahead != '+') {
        return false;
    }
    advance(lexer);

    unsigned count = 0;
    bool saw_question = false;
    while (count < 6) {
        int32_t c = lexer->lookahead;
        if (is_ascii_hex_digit(c)) {
            if (saw_question) {
                return false;
            }
            advance(lexer);
            count++;
            continue;
        }
        if (c == '?') {
            saw_question = true;
            advance(lexer);
            count++;
            continue;
        }
        break;
    }

    if (count == 0) {
        return false;
    }

    if (!saw_question && lexer->lookahead == '-') {
        advance(lexer);
        unsigned rhs_count = 0;
        while (rhs_count < 6 && is_ascii_hex_digit(lexer->lookahead)) {
            advance(lexer);
            rhs_count++;
        }
        if (rhs_count == 0) {
            return false;
        }
    }

    lexer->mark_end(lexer);
    lexer->result_symbol = UNICODE_RANGE_VALUE;
    return true;
}

static bool scan_forgiving_pseudo_element_recovery(TSLexer *lexer) {
    PROFILE_COUNT(scan_forgiving_pseudo_element_calls);

    if (lexer->lookahead != ':') {
        return false;
    }
    PROFILE_ADVANCE(scan_forgiving_pseudo_element_bytes, lexer);

    if (lexer->lookahead != ':') {
        return false;
    }
    PROFILE_ADVANCE(scan_forgiving_pseudo_element_bytes, lexer);

    bool has_content = true;
    unsigned depth_paren = 0;
    unsigned depth_bracket = 0;
    bool in_single = false;
    bool in_double = false;

    while (!lexer->eof(lexer)) {
        int32_t c = lexer->lookahead;

        if (!in_single && !in_double && depth_paren == 0 && depth_bracket == 0 && (c == ',' || c == ')')) {
            break;
        }

        if (c == '\\') {
            PROFILE_ADVANCE(scan_forgiving_pseudo_element_bytes, lexer);
            if (!lexer->eof(lexer)) {
                PROFILE_ADVANCE(scan_forgiving_pseudo_element_bytes, lexer);
            }
            continue;
        }

        if (in_single) {
            PROFILE_ADVANCE(scan_forgiving_pseudo_element_bytes, lexer);
            if (c == '\'') {
                in_single = false;
            }
            continue;
        }

        if (in_double) {
            PROFILE_ADVANCE(scan_forgiving_pseudo_element_bytes, lexer);
            if (c == '"') {
                in_double = false;
            }
            continue;
        }

        switch (c) {
            case '\'':
                in_single = true;
                break;
            case '"':
                in_double = true;
                break;
            case '(':
                depth_paren++;
                break;
            case ')':
                if (depth_paren > 0) depth_paren--;
                break;
            case '[':
                depth_bracket++;
                break;
            case ']':
                if (depth_bracket > 0) depth_bracket--;
                break;
        }

        PROFILE_ADVANCE(scan_forgiving_pseudo_element_bytes, lexer);
        has_content = true;
    }

    if (!has_content) {
        return false;
    }

    lexer->mark_end(lexer);
    lexer->result_symbol = FORGIVING_PSEUDO_ELEMENT_RECOVERY;
    PROFILE_COUNT(scan_forgiving_pseudo_element_successes);
    return true;
}

bool tree_sitter_css_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
    PROFILE_COUNT(scan_calls);

    if (valid_symbols[AT_RULE_PRELUDE]) {
        return scan_at_rule_prelude(lexer);
    }

    if (valid_symbols[GENERAL_ENCLOSED_VALUE]) {
        return scan_general_enclosed_value(lexer);
    }

    if (valid_symbols[UNICODE_RANGE_VALUE]) {
        return scan_unicode_range_value(lexer);
    }

    if (valid_symbols[BAD_URL_VALUE]) {
        return scan_bad_url_value(lexer);
    }

    if (valid_symbols[FORGIVING_PSEUDO_ELEMENT_RECOVERY]) {
        return scan_forgiving_pseudo_element_recovery(lexer);
    }

    if (valid_symbols[ERROR_RECOVERY]) {
        return false;
    }

    if (is_css_space(lexer->lookahead) && valid_symbols[DESCENDANT_OP]) {
        PROFILE_COUNT(scan_descendant_operator_calls);
        lexer->result_symbol = DESCENDANT_OP;

        PROFILE_SKIP(scan_descendant_operator_bytes, lexer);
        while (is_css_space(lexer->lookahead)) {
            PROFILE_SKIP(scan_descendant_operator_bytes, lexer);
        }
        lexer->mark_end(lexer);

        if (can_start_selector(lexer->lookahead)) {
            PROFILE_COUNT(scan_descendant_operator_successes);
            return true;
        }

        if (lexer->lookahead == ':') {
            PROFILE_ADVANCE(scan_descendant_operator_bytes, lexer);
            if (is_css_space(lexer->lookahead) || lexer->lookahead == ':' || lexer->lookahead == ';'
                || lexer->lookahead == '}' || lexer->eof(lexer)) {
                return false;
            }
            if (!can_start_pseudo_name(lexer->lookahead)) {
                return false;
            }
            for (;;) {
                if (lexer->lookahead == ';' || lexer->lookahead == '}' || lexer->eof(lexer)) {
                    return false;
                }
                if (lexer->lookahead == '{') {
                    PROFILE_COUNT(scan_descendant_operator_successes);
                    return true;
                }
                PROFILE_ADVANCE(scan_descendant_operator_bytes, lexer);
            }
        }
    }

    if (valid_symbols[PSEUDO_CLASS_SELECTOR_COLON]) {
        PROFILE_COUNT(scan_pseudo_class_colon_calls);
        while (is_css_space(lexer->lookahead)) {
            PROFILE_SKIP(scan_pseudo_class_colon_bytes, lexer);
        }
        if (lexer->lookahead == ':') {
            PROFILE_ADVANCE(scan_pseudo_class_colon_bytes, lexer);
            if (lexer->lookahead == ':') {
                return false;
            }
            if (!can_start_pseudo_name(lexer->lookahead)) {
                return false;
            }
            lexer->mark_end(lexer);
            lexer->result_symbol = PSEUDO_CLASS_SELECTOR_COLON;

            bool in_comment = false;
            bool in_single = false;
            bool in_double = false;
            unsigned depth_paren = 0;
            unsigned depth_bracket = 0;
            while (lexer->lookahead != ';' && lexer->lookahead != '}' && !lexer->eof(lexer)) {
                int32_t c = lexer->lookahead;

                if (c == '\\') {
                    PROFILE_ADVANCE(scan_pseudo_class_colon_bytes, lexer);
                    if (!lexer->eof(lexer)) {
                        PROFILE_ADVANCE(scan_pseudo_class_colon_bytes, lexer);
                    }
                    continue;
                }

                if (in_single) {
                    PROFILE_ADVANCE(scan_pseudo_class_colon_bytes, lexer);
                    if (c == '\'') {
                        in_single = false;
                    }
                    continue;
                }

                if (in_double) {
                    PROFILE_ADVANCE(scan_pseudo_class_colon_bytes, lexer);
                    if (c == '"') {
                        in_double = false;
                    }
                    continue;
                }

                if (in_comment) {
                    PROFILE_ADVANCE(scan_pseudo_class_colon_bytes, lexer);
                    if (c == '*' && lexer->lookahead == '/') {
                        PROFILE_ADVANCE(scan_pseudo_class_colon_bytes, lexer);
                        in_comment = false;
                    }
                    continue;
                }

                if (c == '/' && !in_comment) {
                    PROFILE_ADVANCE(scan_pseudo_class_colon_bytes, lexer);
                    if (lexer->lookahead == '*') {
                        PROFILE_ADVANCE(scan_pseudo_class_colon_bytes, lexer);
                        in_comment = true;
                    }
                    continue;
                }

                switch (c) {
                    case '\'':
                        in_single = true;
                        break;
                    case '"':
                        in_double = true;
                        break;
                    case '(':
                        depth_paren++;
                        break;
                    case ')':
                        if (depth_paren > 0) depth_paren--;
                        break;
                    case '[':
                        depth_bracket++;
                        break;
                    case ']':
                        if (depth_bracket > 0) depth_bracket--;
                        break;
                }

                PROFILE_ADVANCE(scan_pseudo_class_colon_bytes, lexer);
                if (!in_single && !in_double && !in_comment && lexer->lookahead == '{'
                    && depth_paren == 0 && depth_bracket == 0) {
                    PROFILE_COUNT(scan_pseudo_class_colon_successes);
                    return true;
                }
            }

            if (lexer->eof(lexer)) {
                PROFILE_COUNT(scan_pseudo_class_colon_successes);
                return true;
            }
            return false;
        }
    }

    return false;
}
