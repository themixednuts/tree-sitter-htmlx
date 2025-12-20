/**
 * External scanner for HTML grammar
 *
 * Follows the WHATWG HTML Living Standard:
 * https://html.spec.whatwg.org/
 *
 * Handles:
 * - Tag names (start, end, special elements)
 * - Raw text content (script, style)
 * - Escapable raw text content (textarea, title)
 * - Implicit end tags (§13.1.2.4)
 * - Comments (§13.6)
 * - Self-closing tag delimiter
 *
 * Performance optimizations:
 * - Zero-copy tag name scanning with stack buffers
 * - ASCII-only character operations (no wchar overhead)
 * - Optimized delimiter matching with early exit
 * - Branch prediction hints
 */

#include "tag.h"
#include "tree_sitter/parser.h"

#include <stdbool.h>
#include <stdint.h>
#include <string.h>

// ============================================================================
// Performance macros
// ============================================================================

#if defined(__GNUC__) || defined(__clang__)
#define LIKELY(x) __builtin_expect(!!(x), 1)
#define UNLIKELY(x) __builtin_expect(!!(x), 0)
#define ALWAYS_INLINE __attribute__((always_inline)) inline
#else
#define LIKELY(x) (x)
#define UNLIKELY(x) (x)
#define ALWAYS_INLINE inline
#endif

// Maximum tag name length we handle on stack (covers all standard HTML tags)
#define MAX_TAG_NAME_STACK 32

// ============================================================================
// Token types - must match grammar.js externals order
// ============================================================================

enum TokenType {
  START_TAG_NAME,             // 0
  SCRIPT_START_TAG_NAME,      // 1
  STYLE_START_TAG_NAME,       // 2
  TEXTAREA_START_TAG_NAME,    // 3
  TITLE_START_TAG_NAME,       // 4
  END_TAG_NAME,               // 5
  ERRONEOUS_END_TAG_NAME,     // 6
  SELF_CLOSING_TAG_DELIMITER, // 7
  IMPLICIT_END_TAG,           // 8
  RAW_TEXT,                   // 9
  COMMENT,                    // 10
};

// ============================================================================
// Scanner state
// ============================================================================

typedef struct {
  Array(Tag) tags;
} Scanner;

// ============================================================================
// ASCII-optimized character operations (no wchar overhead)
// ============================================================================

static ALWAYS_INLINE bool is_ascii_alpha(int32_t c) {
  return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z');
}

static ALWAYS_INLINE bool is_ascii_alnum(int32_t c) {
  return is_ascii_alpha(c) || (c >= '0' && c <= '9');
}

static ALWAYS_INLINE bool is_tag_name_char(int32_t c) {
  // Tag names: ASCII alphanumeric, hyphen, colon (for namespaces)
  return is_ascii_alnum(c) || c == '-' || c == ':';
}

static ALWAYS_INLINE char to_ascii_upper(int32_t c) {
  return (c >= 'a' && c <= 'z') ? (char)(c - 32) : (char)c;
}

static ALWAYS_INLINE bool is_ascii_space(int32_t c) {
  // HTML ASCII whitespace: space, tab, LF, FF, CR
  return c == ' ' || c == '\t' || c == '\n' || c == '\f' || c == '\r';
}

// ============================================================================
// Lexer helpers
// ============================================================================

static ALWAYS_INLINE void advance(TSLexer *lexer) {
  lexer->advance(lexer, false);
}

static ALWAYS_INLINE void skip_whitespace(TSLexer *lexer) {
  lexer->advance(lexer, true);
}

// ============================================================================
// Serialization (state persistence across parse calls)
// ============================================================================

static unsigned serialize(Scanner *scanner, char *buffer) {
  uint16_t tag_count = scanner->tags.size > UINT16_MAX
                           ? UINT16_MAX
                           : (uint16_t)scanner->tags.size;
  uint16_t serialized_tag_count = 0;

  unsigned size = sizeof(tag_count);
  memcpy(&buffer[size], &tag_count, sizeof(tag_count));
  size += sizeof(tag_count);

  for (; serialized_tag_count < tag_count; serialized_tag_count++) {
    Tag tag = scanner->tags.contents[serialized_tag_count];
    if (tag.type == CUSTOM) {
      unsigned name_length = tag.custom_tag_name.size;
      if (name_length > UINT8_MAX) {
        name_length = UINT8_MAX;
      }
      if (size + 2 + name_length >= TREE_SITTER_SERIALIZATION_BUFFER_SIZE) {
        break;
      }
      buffer[size++] = (char)tag.type;
      buffer[size++] = (char)name_length;
      memcpy(&buffer[size], tag.custom_tag_name.contents, name_length);
      size += name_length;
    } else {
      if (size + 1 >= TREE_SITTER_SERIALIZATION_BUFFER_SIZE) {
        break;
      }
      buffer[size++] = (char)tag.type;
    }
  }

  memcpy(&buffer[0], &serialized_tag_count, sizeof(serialized_tag_count));
  return size;
}

static void deserialize(Scanner *scanner, const char *buffer, unsigned length) {
  // Free existing tags
  for (unsigned i = 0; i < scanner->tags.size; i++) {
    tag_free(&scanner->tags.contents[i]);
  }
  array_clear(&scanner->tags);

  if (UNLIKELY(length == 0)) {
    return;
  }

  unsigned size = 0;
  uint16_t tag_count = 0;
  uint16_t serialized_tag_count = 0;

  memcpy(&serialized_tag_count, &buffer[size], sizeof(serialized_tag_count));
  size += sizeof(serialized_tag_count);

  memcpy(&tag_count, &buffer[size], sizeof(tag_count));
  size += sizeof(tag_count);

  array_reserve(&scanner->tags, tag_count);

  for (unsigned iter = 0; iter < serialized_tag_count; iter++) {
    Tag tag = tag_new();
    tag.type = (TagType)buffer[size++];
    if (tag.type == CUSTOM) {
      uint16_t name_length = (uint8_t)buffer[size++];
      array_reserve(&tag.custom_tag_name, name_length);
      tag.custom_tag_name.size = name_length;
      memcpy(tag.custom_tag_name.contents, &buffer[size], name_length);
      size += name_length;
    }
    array_push(&scanner->tags, tag);
  }

  // Add zero tags if we didn't read enough (buffer overflow protection)
  for (unsigned iter = serialized_tag_count; iter < tag_count; iter++) {
    array_push(&scanner->tags, tag_new());
  }
}

// ============================================================================
// Tag name scanning - optimized with stack buffer
// ============================================================================

/**
 * Scan tag name into stack buffer (zero-copy for standard tags)
 * Returns length of tag name, or 0 if no valid tag name
 */
static unsigned scan_tag_name_into(TSLexer *lexer, char *buffer,
                                   unsigned max_len) {
  unsigned len = 0;

  while (is_tag_name_char(lexer->lookahead) && len < max_len) {
    buffer[len++] = to_ascii_upper(lexer->lookahead);
    advance(lexer);
  }

  return len;
}

/**
 * Scan tag name - returns String (caller owns)
 * Only allocates heap for custom/long tag names
 */
static String scan_tag_name(TSLexer *lexer) {
  // Try stack buffer first
  char stack_buffer[MAX_TAG_NAME_STACK];
  unsigned len = scan_tag_name_into(lexer, stack_buffer, MAX_TAG_NAME_STACK);

  // If we hit the limit and there's more, we need dynamic allocation
  if (UNLIKELY(len == MAX_TAG_NAME_STACK &&
               is_tag_name_char(lexer->lookahead))) {
    String tag_name = array_new();
    array_reserve(&tag_name, MAX_TAG_NAME_STACK * 2);
    memcpy(tag_name.contents, stack_buffer, len);
    tag_name.size = len;

    while (is_tag_name_char(lexer->lookahead)) {
      array_push(&tag_name, to_ascii_upper(lexer->lookahead));
      advance(lexer);
    }
    return tag_name;
  }

  // Copy from stack to heap String
  String tag_name = array_new();
  if (len > 0) {
    array_reserve(&tag_name, len);
    memcpy(tag_name.contents, stack_buffer, len);
    tag_name.size = len;
  }
  return tag_name;
}

// ============================================================================
// Comment scanning
// ============================================================================

/**
 * Scan HTML comment
 * Per §13.6 - Comments start with <!-- and end with -->
 */
static bool scan_comment(TSLexer *lexer) {
  // Already consumed '<!'
  if (UNLIKELY(lexer->lookahead != '-')) {
    return false;
  }
  advance(lexer);

  if (UNLIKELY(lexer->lookahead != '-')) {
    return false;
  }
  advance(lexer);

  // Scan until we find -->
  unsigned dashes = 0;
  while (lexer->lookahead != 0) {
    int32_t c = lexer->lookahead;
    advance(lexer);

    if (c == '-') {
      dashes++;
    } else if (c == '>' && dashes >= 2) {
      lexer->result_symbol = COMMENT;
      lexer->mark_end(lexer);
      return true;
    } else {
      dashes = 0;
    }
  }

  return false;
}

// ============================================================================
// Raw text content scanning
// ============================================================================

// Pre-computed raw text end delimiters (uppercase for case-insensitive
// matching)
static const struct {
  TagType type;
  const char *delimiter;
  uint8_t length;
} RAW_TEXT_DELIMITERS[] = {
    {SCRIPT, "</SCRIPT", 8},
    {STYLE, "</STYLE", 7},
    {TEXTAREA, "</TEXTAREA", 10},
    {TITLE, "</TITLE", 7},
};

#define RAW_TEXT_DELIMITER_COUNT 4

/**
 * Scan raw text content for script, style, textarea, title elements
 * Per §13.1.2.1 and §13.1.2.2
 *
 * Optimized: uses pre-computed delimiter info, avoids strlen in hot path
 */
static bool scan_raw_text(Scanner *scanner, TSLexer *lexer) {
  if (UNLIKELY(scanner->tags.size == 0)) {
    return false;
  }

  Tag *current_tag = array_back(&scanner->tags);
  TagType tag_type = current_tag->type;

  // Find delimiter info
  const char *delimiter = NULL;
  unsigned delimiter_len = 0;

  for (int i = 0; i < RAW_TEXT_DELIMITER_COUNT; i++) {
    if (RAW_TEXT_DELIMITERS[i].type == tag_type) {
      delimiter = RAW_TEXT_DELIMITERS[i].delimiter;
      delimiter_len = RAW_TEXT_DELIMITERS[i].length;
      break;
    }
  }

  if (UNLIKELY(delimiter == NULL)) {
    return false;
  }

  lexer->mark_end(lexer);

  unsigned match_index = 0;

  while (lexer->lookahead != 0) {
    char upper = to_ascii_upper(lexer->lookahead);

    if (upper == delimiter[match_index]) {
      match_index++;
      if (match_index == delimiter_len) {
        // Found end delimiter - don't consume it
        break;
      }
      advance(lexer);
    } else {
      // Reset matching, mark position as content end
      match_index = 0;
      advance(lexer);
      lexer->mark_end(lexer);
    }
  }

  lexer->result_symbol = RAW_TEXT;
  return true;
}

// ============================================================================
// Tag stack management
// ============================================================================

static ALWAYS_INLINE void pop_tag(Scanner *scanner) {
  Tag popped_tag = array_pop(&scanner->tags);
  tag_free(&popped_tag);
}

// ============================================================================
// Implicit end tag scanning
// ============================================================================

/**
 * Scan for implicit end tags
 * Per §13.1.2.4 - Optional tags
 */
static bool scan_implicit_end_tag(Scanner *scanner, TSLexer *lexer) {
  Tag *parent = scanner->tags.size == 0 ? NULL : array_back(&scanner->tags);

  bool is_closing_tag = false;
  if (lexer->lookahead == '/') {
    is_closing_tag = true;
    advance(lexer);
  } else {
    // Void elements implicitly close themselves
    if (parent && tag_is_void(parent)) {
      pop_tag(scanner);
      lexer->result_symbol = IMPLICIT_END_TAG;
      return true;
    }
  }

  String tag_name = scan_tag_name(lexer);

  if (tag_name.size == 0 && !lexer->eof(lexer)) {
    array_delete(&tag_name);
    return false;
  }

  Tag next_tag = tag_for_name(tag_name);

  if (is_closing_tag) {
    // Check if tag correctly closes the topmost element
    if (scanner->tags.size > 0 &&
        tag_eq(array_back(&scanner->tags), &next_tag)) {
      tag_free(&next_tag);
      return false;
    }

    // Search stack for matching tag - emit implicit end tags
    for (unsigned i = scanner->tags.size; i > 0; i--) {
      if (scanner->tags.contents[i - 1].type == next_tag.type) {
        pop_tag(scanner);
        lexer->result_symbol = IMPLICIT_END_TAG;
        tag_free(&next_tag);
        return true;
      }
    }
  } else if (parent != NULL) {
    // Check content model - does parent allow this child?
    bool should_close = !tag_can_contain(parent, &next_tag);

    // Also close html/head/body at EOF
    if (!should_close && lexer->eof(lexer)) {
      TagType pt = parent->type;
      should_close = (pt == HTML || pt == HEAD || pt == BODY);
    }

    if (should_close) {
      pop_tag(scanner);
      lexer->result_symbol = IMPLICIT_END_TAG;
      tag_free(&next_tag);
      return true;
    }
  }

  tag_free(&next_tag);
  return false;
}

// ============================================================================
// Start tag scanning
// ============================================================================

static bool scan_start_tag_name(Scanner *scanner, TSLexer *lexer) {
  String tag_name = scan_tag_name(lexer);

  if (UNLIKELY(tag_name.size == 0)) {
    array_delete(&tag_name);
    return false;
  }

  Tag tag = tag_for_name(tag_name);
  array_push(&scanner->tags, tag);

  // Determine token type based on element category
  switch (tag.type) {
  case SCRIPT:
    lexer->result_symbol = SCRIPT_START_TAG_NAME;
    break;
  case STYLE:
    lexer->result_symbol = STYLE_START_TAG_NAME;
    break;
  case TEXTAREA:
    lexer->result_symbol = TEXTAREA_START_TAG_NAME;
    break;
  case TITLE:
    lexer->result_symbol = TITLE_START_TAG_NAME;
    break;
  default:
    lexer->result_symbol = START_TAG_NAME;
    break;
  }

  return true;
}

// ============================================================================
// End tag scanning
// ============================================================================

static bool scan_end_tag_name(Scanner *scanner, TSLexer *lexer) {
  String tag_name = scan_tag_name(lexer);

  if (UNLIKELY(tag_name.size == 0)) {
    array_delete(&tag_name);
    return false;
  }

  Tag tag = tag_for_name(tag_name);

  // Check if this closes the current element
  if (scanner->tags.size > 0 && tag_eq(array_back(&scanner->tags), &tag)) {
    pop_tag(scanner);
    lexer->result_symbol = END_TAG_NAME;
  } else {
    lexer->result_symbol = ERRONEOUS_END_TAG_NAME;
  }

  tag_free(&tag);
  return true;
}

// ============================================================================
// Self-closing tag delimiter
// ============================================================================

static bool scan_self_closing_tag_delimiter(Scanner *scanner, TSLexer *lexer) {
  advance(lexer); // consume '/'

  if (LIKELY(lexer->lookahead == '>')) {
    advance(lexer);
    if (scanner->tags.size > 0) {
      pop_tag(scanner);
      lexer->result_symbol = SELF_CLOSING_TAG_DELIMITER;
    }
    return true;
  }

  return false;
}

// ============================================================================
// Main scan function
// ============================================================================

static bool scan(Scanner *scanner, TSLexer *lexer, const bool *valid_symbols) {
  // Priority 1: Raw text mode - for script, style, textarea, title content
  if (valid_symbols[RAW_TEXT] && !valid_symbols[START_TAG_NAME] &&
      !valid_symbols[END_TAG_NAME]) {
    return scan_raw_text(scanner, lexer);
  }

  // Skip whitespace
  while (is_ascii_space(lexer->lookahead)) {
    skip_whitespace(lexer);
  }

  int32_t lookahead = lexer->lookahead;

  // Priority 2: Check for tag/comment start
  if (lookahead == '<') {
    lexer->mark_end(lexer);
    advance(lexer);

    if (lexer->lookahead == '!') {
      advance(lexer);
      return scan_comment(lexer);
    }

    if (valid_symbols[IMPLICIT_END_TAG]) {
      return scan_implicit_end_tag(scanner, lexer);
    }

    return false;
  }

  // Priority 3: EOF handling
  if (lookahead == 0) {
    if (valid_symbols[IMPLICIT_END_TAG]) {
      return scan_implicit_end_tag(scanner, lexer);
    }
    return false;
  }

  // Priority 4: Self-closing tag delimiter
  if (lookahead == '/' && valid_symbols[SELF_CLOSING_TAG_DELIMITER]) {
    return scan_self_closing_tag_delimiter(scanner, lexer);
  }

  // Priority 5: Tag names (after < or </)
  if ((valid_symbols[START_TAG_NAME] || valid_symbols[END_TAG_NAME]) &&
      !valid_symbols[RAW_TEXT]) {
    if (valid_symbols[START_TAG_NAME]) {
      return scan_start_tag_name(scanner, lexer);
    } else {
      return scan_end_tag_name(scanner, lexer);
    }
  }

  return false;
}

// ============================================================================
// Tree-sitter external scanner interface
// ============================================================================

void *tree_sitter_html_external_scanner_create(void) {
  Scanner *scanner = (Scanner *)ts_calloc(1, sizeof(Scanner));
  return scanner;
}

bool tree_sitter_html_external_scanner_scan(void *payload, TSLexer *lexer,
                                            const bool *valid_symbols) {
  Scanner *scanner = (Scanner *)payload;
  return scan(scanner, lexer, valid_symbols);
}

unsigned tree_sitter_html_external_scanner_serialize(void *payload,
                                                     char *buffer) {
  Scanner *scanner = (Scanner *)payload;
  return serialize(scanner, buffer);
}

void tree_sitter_html_external_scanner_deserialize(void *payload,
                                                   const char *buffer,
                                                   unsigned length) {
  Scanner *scanner = (Scanner *)payload;
  deserialize(scanner, buffer, length);
}

void tree_sitter_html_external_scanner_destroy(void *payload) {
  Scanner *scanner = (Scanner *)payload;
  for (unsigned i = 0; i < scanner->tags.size; i++) {
    tag_free(&scanner->tags.contents[i]);
  }
  array_delete(&scanner->tags);
  ts_free(scanner);
}
