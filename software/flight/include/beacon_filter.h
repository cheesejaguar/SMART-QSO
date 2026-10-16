/**
 * @file beacon_filter.h
 * @brief Content filter for beacon text (SW-12)
 *
 * Every beacon text that did not come from the OBC's own templates passes
 * this filter before it is transmitted. It enforces the 47 CFR 97.113 limits
 * that can be checked mechanically (docs/BEACON_SPEC.md section 6):
 *
 *   LENGTH      1 to BEACON_FILTER_MAX_LEN characters
 *   CHARSET     letters, digits, space and . , ! ? ' " : ; ( ) / - & only
 *               (no '|', which separates the telemetry fields, no '$' or '@')
 *   COMMERCIAL  no URLs ("http", "www.", ".com", ".net", ".org")
 *
 * then, word by word (a word is a maximal run of letters and digits):
 *
 *   ENCODED     longer than BEACON_FILTER_MAX_WORD
 *   CALLSIGN    shaped like a call sign but not this station's (false
 *               identification, 97.113(a)(4)), or an SSID after this
 *               station's call that is not its own
 *   NUMERIC     digits other than "73", "1U" and this station's call sign
 *               and SSID: only the OBC writes numbers, from measured
 *               telemetry, so a generated text cannot state a false value
 *   ENCODED     five or more letters and no vowel, eight or more hex digits
 *               only, or more than two lower-to-upper case changes
 *               (97.113(a)(4) messages encoded to obscure their meaning)
 *   PROFANITY   a word from beacon_filter_profanity (97.113(a)(4))
 *   COMMERCIAL  a word from beacon_filter_commercial (97.113(a)(3))
 *
 * The Jetson applies the same rules (software/payload/content_filter.py);
 * both suites run the vectors in tests/vectors/beacon_filter_vectors.txt.
 */

#ifndef BEACON_FILTER_H
#define BEACON_FILTER_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Longest beacon text: what fits beside the telemetry fields (beacon.c) */
#define BEACON_FILTER_MAX_LEN   176U

/** Longest word (run of letters and digits) */
#define BEACON_FILTER_MAX_WORD  20U

typedef enum {
    BEACON_FILTER_OK = 0,
    BEACON_FILTER_LENGTH,
    BEACON_FILTER_CHARSET,
    BEACON_FILTER_ENCODED,
    BEACON_FILTER_NUMERIC,
    BEACON_FILTER_CALLSIGN,
    BEACON_FILTER_PROFANITY,
    BEACON_FILTER_COMMERCIAL
} BeaconFilterResult_t;

/** Lower-case word lists (whole-word, case-insensitive match) */
extern const char *const beacon_filter_profanity[];
extern const size_t beacon_filter_profanity_count;
extern const char *const beacon_filter_commercial[];
extern const size_t beacon_filter_commercial_count;

/**
 * @brief Check a beacon text
 * @param text Text (need not be NUL-terminated; NUL bytes are rejected)
 * @param len  Number of bytes to check
 * @return BEACON_FILTER_OK, or the first rule the text breaks
 */
BeaconFilterResult_t beacon_filter_check(const char *text, size_t len);

/**
 * @brief Name of a result ("OK", "LENGTH", ...), as used in the vectors
 */
const char *beacon_filter_reason(BeaconFilterResult_t result);

#ifdef __cplusplus
}
#endif

#endif /* BEACON_FILTER_H */
