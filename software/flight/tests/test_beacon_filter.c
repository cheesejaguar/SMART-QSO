/**
 * @file test_beacon_filter.c
 * @brief Beacon text content filter (SW-12, 47 CFR 97.113(a)(3)-(4))
 *
 * The accept/reject vectors are shared with the Jetson-side filter
 * (software/payload/tests/test_content_filter.py), so the two filters cannot
 * drift apart. The fuzz test checks properties of every accepted text rather
 * than re-implementing the rules.
 *
 * @requirement SYS-210 The spacecraft shall transmit AX.25 formatted beacons
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <ctype.h>
#include <stdio.h>
#include <string.h>

#include "beacon.h"
#include "beacon_filter.h"

#ifndef BEACON_FILTER_VECTORS
#error "BEACON_FILTER_VECTORS must name the shared vector file"
#endif

static BeaconFilterResult_t check(const char *text)
{
    return beacon_filter_check(text, strlen(text));
}

/** Every shared vector gives the expected verdict */
static void test_shared_vectors(void **state)
{
    (void)state;
    FILE *f = fopen(BEACON_FILTER_VECTORS, "r");
    char line[512];
    unsigned vectors = 0U;

    assert_non_null(f);
    while (fgets(line, (int)sizeof(line), f) != NULL) {
        size_t n = strlen(line);
        if ((n > 0U) && (line[n - 1U] == '\n')) {
            line[--n] = '\0';
        }
        if ((n == 0U) || (line[0] == '#')) {
            continue;
        }
        char *bar = strchr(line, '|');
        assert_non_null(bar);
        *bar = '\0';
        const char *text = bar + 1;
        const char *got = beacon_filter_reason(beacon_filter_check(text, strlen(text)));
        if (strcmp(got, line) != 0) {
            print_error("vector \"%s\": expected %s, got %s\n", text, line, got);
        }
        assert_string_equal(got, line);
        vectors++;
    }
    (void)fclose(f);
    assert_true(vectors >= 40U);
}

static void expect_list_rejected(const char *const *words, size_t count,
                                 BeaconFilterResult_t reason)
{
    char text[96];

    assert_true(count > 0U);
    for (size_t i = 0U; i < count; i++) {
        (void)snprintf(text, sizeof(text), "Hello %s world", words[i]);
        assert_int_equal(check(text), reason);

        /* Case does not matter */
        for (size_t j = 0U; text[j] != '\0'; j++) {
            text[j] = (char)toupper((unsigned char)text[j]);
        }
        assert_int_equal(check(text), reason);

        /* Whole words only: the word inside a longer word is not matched */
        (void)snprintf(text, sizeof(text), "Hello x%sx world", words[i]);
        assert_int_not_equal(check(text), reason);
    }
}

static void test_word_lists(void **state)
{
    (void)state;
    expect_list_rejected(beacon_filter_profanity, beacon_filter_profanity_count,
                         BEACON_FILTER_PROFANITY);
    expect_list_rejected(beacon_filter_commercial, beacon_filter_commercial_count,
                         BEACON_FILTER_COMMERCIAL);
}

/** The OBC's own templates pass the filter they fall back from */
static void test_templates_pass(void **state)
{
    (void)state;
    for (size_t i = 0U; i < beacon_template_count; i++) {
        assert_int_equal(check(beacon_templates[i]), BEACON_FILTER_OK);
    }
}

static void test_reason_names(void **state)
{
    (void)state;
    assert_string_equal(beacon_filter_reason(BEACON_FILTER_OK), "OK");
    assert_string_equal(beacon_filter_reason(BEACON_FILTER_COMMERCIAL), "COMMERCIAL");
    assert_string_equal(beacon_filter_reason((BeaconFilterResult_t)99), "UNKNOWN");
    assert_int_equal(beacon_filter_check(NULL, 5U), BEACON_FILTER_LENGTH);
    /* An embedded NUL is not text */
    assert_int_equal(beacon_filter_check("Hello\0world", 11U), BEACON_FILTER_CHARSET);
    /* The length given is honored: no read past it */
    assert_int_equal(beacon_filter_check("Hello W1AW", 5U), BEACON_FILTER_OK);
}

/*---------------------------------------------------------------------------*/
/* Fuzz: properties every accepted text must have                            */
/*---------------------------------------------------------------------------*/

static uint32_t s_rng = 0x2545F491U;

static uint32_t next_rand(void)
{
    s_rng ^= s_rng << 13;
    s_rng ^= s_rng >> 17;
    s_rng ^= s_rng << 5;
    return s_rng;
}

static bool icontains(const char *hay, const char *needle)
{
    size_t n = strlen(needle);
    for (size_t i = 0U; hay[i] != '\0'; i++) {
        size_t k = 0U;
        while ((k < n) && (hay[i + k] != '\0') &&
               (tolower((unsigned char)hay[i + k]) == tolower((unsigned char)needle[k]))) {
            k++;
        }
        if (k == n) {
            return true;
        }
    }
    return false;
}

static void assert_accepted_properties(const char *text, size_t len)
{
    static const char allowed_punct[] = " .,!?'\":;()/-&";
    char rest[BEACON_FILTER_MAX_LEN + 1U];

    assert_true((len > 0U) && (len <= BEACON_FILTER_MAX_LEN));
    for (size_t i = 0U; i < len; i++) {
        unsigned char c = (unsigned char)text[i];
        assert_true((isalnum(c) != 0) ||
                    ((c != 0U) && (strchr(allowed_punct, (int)c) != NULL)));
    }
    /* Never a URL or another station's call, even glued to other words */
    assert_false(icontains(text, "http") || icontains(text, "w1aw"));

    /* Digits appear only in the permitted words */
    (void)memcpy(rest, text, len);
    rest[len] = '\0';
    static const char *const permitted[] = {"KK6GDB-11", "KK6GDB", "73", "1U"};
    for (size_t p = 0U; p < (sizeof(permitted) / sizeof(permitted[0])); p++) {
        size_t n = strlen(permitted[p]);
        for (size_t i = 0U; (i + n) <= len; i++) {
            bool match = true;
            for (size_t k = 0U; k < n; k++) {
                if (tolower((unsigned char)rest[i + k]) != tolower((unsigned char)permitted[p][k])) {
                    match = false;
                    break;
                }
            }
            if (match) {
                (void)memset(&rest[i], 'x', n);
            }
        }
    }
    for (size_t i = 0U; i < len; i++) {
        assert_false(isdigit((unsigned char)rest[i]) != 0);
    }
}

static void test_fuzz_properties(void **state)
{
    (void)state;
    static const char *const pool[] = {
        "hello", "orbit", "from", "the", "beacon", "73", "1U", "KK6GDB-11", "KK6GDB",
        "sunlit", "transponder", "QSO", "listening", "tonight", "classic",
        "W1AW", "99", "$", "|", "\t", "deadbeefcafe", "SGVsbG", "http", "buy",
        "KK6GDB-12", "Supercalifragilisticexpialidocious", "www.", ".com", "5",
    };
    static const char *const seps[] = {" ", ", ", "-", "/", "", "! "};
    char text[BEACON_FILTER_MAX_LEN + 40U];
    unsigned accepted = 0U;

    for (unsigned iter = 0U; iter < 20000U; iter++) {
        size_t len = 0U;
        if ((iter % 4U) == 0U) {
            /* Raw bytes, including NUL and high-bit bytes */
            len = (size_t)(next_rand() % (BEACON_FILTER_MAX_LEN + 20U));
            for (size_t i = 0U; i < len; i++) {
                text[i] = (char)(next_rand() & 0xFFU);
            }
        } else {
            unsigned words = 1U + (next_rand() % 12U);
            for (unsigned w = 0U; w < words; w++) {
                const char *word = pool[next_rand() % (sizeof(pool) / sizeof(pool[0]))];
                const char *sep = seps[next_rand() % (sizeof(seps) / sizeof(seps[0]))];
                size_t need = strlen(word) + strlen(sep);
                if ((len + need) >= sizeof(text)) {
                    break;
                }
                (void)memcpy(&text[len], word, strlen(word));
                len += strlen(word);
                (void)memcpy(&text[len], sep, strlen(sep));
                len += strlen(sep);
            }
        }
        text[len] = '\0';

        BeaconFilterResult_t r = beacon_filter_check(text, len);
        assert_int_equal(r, beacon_filter_check(text, len));  /* deterministic */
        if (r == BEACON_FILTER_OK) {
            accepted++;
            assert_accepted_properties(text, len);

            /* Appending a blocked word to accepted text gets it rejected */
            char longer[BEACON_FILTER_MAX_LEN + 64U];
            int n = snprintf(longer, sizeof(longer), "%.*s %s", (int)len, text,
                             beacon_filter_profanity[iter % beacon_filter_profanity_count]);
            assert_true(n > 0);
            assert_int_not_equal(beacon_filter_check(longer, (size_t)n), BEACON_FILTER_OK);
        }
    }
    /* The generator exercises both sides of the filter */
    assert_true(accepted > 500U);
    assert_true(accepted < 15000U);
}

/*---------------------------------------------------------------------------*/
/* Beacon module uses the filter                                              */
/*---------------------------------------------------------------------------*/

static void test_validate_text_uses_filter(void **state)
{
    (void)state;
    assert_true(beacon_validate_text("Hello from orbit", 16U));
    assert_false(beacon_validate_text("Hello\tfrom orbit", 16U));
    assert_false(beacon_validate_text("Battery at 99 percent", 21U));
    assert_false(beacon_validate_text("de W1AW", 7U));
}

/** Rejected AI text falls back to a template and is counted */
static void test_rejected_ai_text_falls_back(void **state)
{
    (void)state;
    BeaconContent_t content;
    BeaconState_t st;
    const char *bad = "Battery at 99 percent";
    const char *good = "Hello from orbit";

    assert_int_equal(beacon_init(), SMART_QSO_OK);
    assert_int_equal(beacon_generate_content(&content, bad, strlen(bad)), SMART_QSO_OK);
    assert_int_equal(content.source, BEACON_SOURCE_TEMPLATE);
    assert_int_equal(check(content.text), BEACON_FILTER_OK);
    assert_int_equal(beacon_store_ai_text(bad, strlen(bad)), SMART_QSO_ERROR);
    assert_int_equal(beacon_get_state(&st), SMART_QSO_OK);
    assert_int_equal(st.ai_rejected_count, 2U);

    assert_int_equal(beacon_generate_content(&content, good, strlen(good)), SMART_QSO_OK);
    assert_int_equal(content.source, BEACON_SOURCE_AI);
    assert_string_equal(content.text, good);
    assert_int_equal(beacon_get_state(&st), SMART_QSO_OK);
    assert_int_equal(st.ai_rejected_count, 2U);
}

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_shared_vectors),
        cmocka_unit_test(test_word_lists),
        cmocka_unit_test(test_templates_pass),
        cmocka_unit_test(test_reason_names),
        cmocka_unit_test(test_fuzz_properties),
        cmocka_unit_test(test_validate_text_uses_filter),
        cmocka_unit_test(test_rejected_ai_text_falls_back),
    };

    return cmocka_run_group_tests_name("Beacon Content Filter Tests", tests, NULL, NULL);
}
