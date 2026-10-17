/**
 * @file beacon_filter.c
 * @brief Content filter for beacon text (SW-12)
 *
 * Rules and their order are documented in beacon_filter.h. The Jetson-side
 * copy is software/payload/content_filter.py; keep the two in step (the
 * shared vectors and the word-list comparison in the payload suite fail
 * otherwise).
 */

#include "beacon_filter.h"
#include "station_id.h"

#include <stdint.h>
#include <string.h>

/*===========================================================================*/
/* Word lists (lower case, whole words)                                      */
/*===========================================================================*/

/** Obscene or indecent words (97.113(a)(4)) */
const char *const beacon_filter_profanity[] = {
    "ass", "asshole", "bastard", "bitch", "bollocks", "bullshit", "cock",
    "cocksucker", "crap", "cunt", "damn", "dick", "dickhead", "dyke", "fag",
    "faggot", "fuck", "fucked", "fucker", "fucking", "goddamn", "motherfucker",
    "nigga", "nigger", "piss", "pissed", "prick", "pussy", "retard", "shit",
    "shitty", "slut", "spic", "twat", "wank", "wanker", "whore"
};
const size_t beacon_filter_profanity_count =
    sizeof(beacon_filter_profanity) / sizeof(beacon_filter_profanity[0]);

/** Business, advertising or pecuniary words (97.113(a)(3)) */
const char *const beacon_filter_commercial[] = {
    "advert", "advertise", "advertisement", "bitcoin", "buy", "buying", "cheap",
    "coupon", "crypto", "deal", "deals", "discount", "dollar", "dollars",
    "invest", "investment", "price", "prices", "profit", "promo", "promotion",
    "purchase", "sale", "sales", "sell", "selling", "shop", "sponsor",
    "sponsored", "subscribe", "usd"
};
const size_t beacon_filter_commercial_count =
    sizeof(beacon_filter_commercial) / sizeof(beacon_filter_commercial[0]);

/** URL fragments (97.113(a)(3) advertising; checked anywhere in the text) */
static const char *const s_url_marks[] = {"http", "www.", ".com", ".net", ".org"};

/** Punctuation allowed besides letters and digits */
static const char s_punct[] = " .,!?'\":;()/-&";

/*===========================================================================*/
/* Character helpers (ASCII only, locale-independent)                        */
/*===========================================================================*/

static bool is_upper(char c) { return (c >= 'A') && (c <= 'Z'); }
static bool is_lower(char c) { return (c >= 'a') && (c <= 'z'); }
static bool is_alpha(char c) { return is_upper(c) || is_lower(c); }
static bool is_digit(char c) { return (c >= '0') && (c <= '9'); }
static bool is_alnum(char c) { return is_alpha(c) || is_digit(c); }

static char to_lower(char c)
{
    return is_upper(c) ? (char)(c + ('a' - 'A')) : c;
}

static bool is_vowel(char c)
{
    char l = to_lower(c);
    return (l == 'a') || (l == 'e') || (l == 'i') || (l == 'o') || (l == 'u') || (l == 'y');
}

static bool is_hex(char c)
{
    char l = to_lower(c);
    return is_digit(c) || ((l >= 'a') && (l <= 'f'));
}

static bool is_allowed(char c)
{
    return is_alnum(c) || ((c != '\0') && (strchr(s_punct, (int)c) != NULL));
}

/** Case-insensitive comparison of w[0..n) with a lower-case literal */
static bool word_is(const char *w, size_t n, const char *lit)
{
    if (strlen(lit) != n) {
        return false;
    }
    for (size_t i = 0U; i < n; i++) {
        if (to_lower(w[i]) != lit[i]) {
            return false;
        }
    }
    return true;
}

static bool word_in(const char *w, size_t n, const char *const *list, size_t count)
{
    for (size_t i = 0U; i < count; i++) {
        if (word_is(w, n, list[i])) {
            return true;
        }
    }
    return false;
}

/** Case-insensitive search for a lower-case literal in text[0..len) */
static bool text_has(const char *text, size_t len, const char *lit)
{
    size_t n = strlen(lit);
    for (size_t i = 0U; (i + n) <= len; i++) {
        if (word_is(&text[i], n, lit)) {
            return true;
        }
    }
    return false;
}

/*===========================================================================*/
/* Word rules                                                                 */
/*===========================================================================*/

/** This station's call sign without SSID, in lower case */
static bool is_own_call(const char *w, size_t n)
{
    static const char s_call[] = SQSO_CALLSIGN;
    if (n != (sizeof(s_call) - 1U)) {
        return false;
    }
    for (size_t i = 0U; i < n; i++) {
        if (to_lower(w[i]) != to_lower(s_call[i])) {
            return false;
        }
    }
    return true;
}

/** Prefix (letter, two letters, digit-letter or letter-digit), one digit,
 *  then one to four letters: the ITU call-sign pattern */
static bool is_callsign_shape(const char *w, size_t n)
{
    size_t s = n;
    while ((s > 0U) && is_alpha(w[s - 1U])) {
        s--;
    }
    size_t suffix = n - s;
    if ((suffix < 1U) || (suffix > 4U) || (s < 2U) || !is_digit(w[s - 1U])) {
        return false;
    }
    size_t prefix = s - 1U;
    if (prefix == 1U) {
        return is_alpha(w[0]);
    }
    return (prefix == 2U) && (is_alpha(w[0]) || is_alpha(w[1]));
}

/** Digits only, at most two of them, equal to this station's SSID */
static BeaconFilterResult_t check_ssid(const char *w, size_t n)
{
    uint32_t value = 0U;
    for (size_t i = 0U; i < n; i++) {
        if (!is_digit(w[i])) {
            return BEACON_FILTER_NUMERIC;
        }
        value = (value * 10U) + (uint32_t)(w[i] - '0');
    }
    if (n > 2U) {
        return BEACON_FILTER_NUMERIC;
    }
    return (value == (uint32_t)SQSO_SSID) ? BEACON_FILTER_OK : BEACON_FILTER_CALLSIGN;
}

static bool looks_encoded(const char *w, size_t n)
{
    bool all_alpha = true;
    bool all_hex = true;
    bool vowel = false;
    unsigned case_changes = 0U;

    for (size_t i = 0U; i < n; i++) {
        all_alpha = all_alpha && is_alpha(w[i]);
        all_hex = all_hex && is_hex(w[i]);
        vowel = vowel || is_vowel(w[i]);
        if ((i > 0U) && is_lower(w[i - 1U]) && is_upper(w[i])) {
            case_changes++;
        }
    }
    return (all_alpha && (n >= 5U) && !vowel) || (all_hex && (n >= 8U)) || (case_changes > 2U);
}

static bool has_digit(const char *w, size_t n)
{
    for (size_t i = 0U; i < n; i++) {
        if (is_digit(w[i])) {
            return true;
        }
    }
    return false;
}

/**
 * @param after_own_call The word follows "<own call>-" directly (an SSID)
 */
static BeaconFilterResult_t check_word(const char *w, size_t n, bool after_own_call)
{
    if (n > BEACON_FILTER_MAX_WORD) {
        return BEACON_FILTER_ENCODED;
    }
    if (after_own_call && is_digit(w[0])) {
        return check_ssid(w, n);
    }
    if (is_callsign_shape(w, n) && !is_own_call(w, n)) {
        return BEACON_FILTER_CALLSIGN;
    }
    if (has_digit(w, n) && !word_is(w, n, "73") && !word_is(w, n, "1u") && !is_own_call(w, n)) {
        return BEACON_FILTER_NUMERIC;
    }
    if (looks_encoded(w, n)) {
        return BEACON_FILTER_ENCODED;
    }
    if (word_in(w, n, beacon_filter_profanity, beacon_filter_profanity_count)) {
        return BEACON_FILTER_PROFANITY;
    }
    if (word_in(w, n, beacon_filter_commercial, beacon_filter_commercial_count)) {
        return BEACON_FILTER_COMMERCIAL;
    }
    return BEACON_FILTER_OK;
}

static BeaconFilterResult_t check_words(const char *text, size_t len)
{
    size_t i = 0U;
    size_t prev_end = 0U;
    bool prev_own = false;

    while (i < len) {
        if (!is_alnum(text[i])) {
            i++;
            continue;
        }
        size_t start = i;
        while ((i < len) && is_alnum(text[i])) {
            i++;
        }
        bool after_own = prev_own && (start == (prev_end + 1U)) && (text[prev_end] == '-');
        BeaconFilterResult_t r = check_word(&text[start], i - start, after_own);
        if (r != BEACON_FILTER_OK) {
            return r;
        }
        prev_own = is_own_call(&text[start], i - start);
        prev_end = i;
    }
    return BEACON_FILTER_OK;
}

/*===========================================================================*/
/* Public                                                                     */
/*===========================================================================*/

BeaconFilterResult_t beacon_filter_check(const char *text, size_t len)
{
    if ((text == NULL) || (len == 0U) || (len > BEACON_FILTER_MAX_LEN)) {
        return BEACON_FILTER_LENGTH;
    }
    for (size_t i = 0U; i < len; i++) {
        if (!is_allowed(text[i])) {
            return BEACON_FILTER_CHARSET;
        }
    }
    for (size_t i = 0U; i < (sizeof(s_url_marks) / sizeof(s_url_marks[0])); i++) {
        if (text_has(text, len, s_url_marks[i])) {
            return BEACON_FILTER_COMMERCIAL;
        }
    }
    return check_words(text, len);
}

const char *beacon_filter_reason(BeaconFilterResult_t result)
{
    switch (result) {
        case BEACON_FILTER_OK:          return "OK";
        case BEACON_FILTER_LENGTH:      return "LENGTH";
        case BEACON_FILTER_CHARSET:     return "CHARSET";
        case BEACON_FILTER_ENCODED:     return "ENCODED";
        case BEACON_FILTER_NUMERIC:     return "NUMERIC";
        case BEACON_FILTER_CALLSIGN:    return "CALLSIGN";
        case BEACON_FILTER_PROFANITY:   return "PROFANITY";
        case BEACON_FILTER_COMMERCIAL:  return "COMMERCIAL";
        default:                        return "UNKNOWN";
    }
}
