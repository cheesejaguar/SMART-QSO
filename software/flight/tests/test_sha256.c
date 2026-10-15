/**
 * @file test_sha256.c
 * @brief Unit tests for SHA-256 and HMAC-SHA-256
 *
 * Expected values are the FIPS 180-2 examples and RFC 4231 test cases,
 * cross-checked with Python hashlib/hmac.
 *
 * @requirement SYS-325 Execute only authenticated, fresh telecommands (47 CFR 97.211(b))
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <setjmp.h>
#include <cmocka.h>
#include <string.h>

#include "sha256.h"

/*===========================================================================*/
/* Helpers                                                                    */
/*===========================================================================*/

static uint8_t hex_nibble(char c)
{
    if ((c >= '0') && (c <= '9')) {
        return (uint8_t)(c - '0');
    }
    return (uint8_t)((c - 'a') + 10);
}

/** Assert that a 32-byte digest equals a 64-character lowercase hex string */
static void assert_digest(const uint8_t digest[SHA256_DIGEST_SIZE], const char *hex)
{
    uint8_t expected[SHA256_DIGEST_SIZE];
    for (size_t i = 0U; i < SHA256_DIGEST_SIZE; i++) {
        expected[i] = (uint8_t)((hex_nibble(hex[2U * i]) << 4) | hex_nibble(hex[(2U * i) + 1U]));
    }
    assert_memory_equal(digest, expected, SHA256_DIGEST_SIZE);
}

/*===========================================================================*/
/* SHA-256                                                                    */
/*===========================================================================*/

static void test_sha256_abc(void **state)
{
    (void)state;
    uint8_t digest[SHA256_DIGEST_SIZE];

    assert_int_equal(sha256((const uint8_t *)"abc", 3U, digest), SMART_QSO_OK);
    assert_digest(digest, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

static void test_sha256_empty(void **state)
{
    (void)state;
    uint8_t digest[SHA256_DIGEST_SIZE];

    assert_int_equal(sha256(NULL, 0U, digest), SMART_QSO_OK);
    assert_digest(digest, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}

static void test_sha256_two_blocks(void **state)
{
    (void)state;
    const char *msg = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    uint8_t digest[SHA256_DIGEST_SIZE];

    assert_int_equal(sha256((const uint8_t *)msg, strlen(msg), digest), SMART_QSO_OK);
    assert_digest(digest, "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
}

/** One million 'a' fed in uneven chunks exercises partial-block buffering */
static void test_sha256_million_a_incremental(void **state)
{
    (void)state;
    static uint8_t chunk[997];
    Sha256Ctx_t ctx;
    uint8_t digest[SHA256_DIGEST_SIZE];
    size_t remaining = 1000000U;

    memset(chunk, 'a', sizeof(chunk));
    assert_int_equal(sha256_init(&ctx), SMART_QSO_OK);
    while (remaining > 0U) {
        size_t n = (remaining < sizeof(chunk)) ? remaining : sizeof(chunk);
        assert_int_equal(sha256_update(&ctx, chunk, n), SMART_QSO_OK);
        remaining -= n;
    }
    assert_int_equal(sha256_final(&ctx, digest), SMART_QSO_OK);
    assert_digest(digest, "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0");
}

static void test_sha256_null_arguments(void **state)
{
    (void)state;
    uint8_t digest[SHA256_DIGEST_SIZE];
    Sha256Ctx_t ctx;

    assert_int_equal(sha256((const uint8_t *)"abc", 3U, NULL), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(sha256(NULL, 3U, digest), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(sha256_init(NULL), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(sha256_init(&ctx), SMART_QSO_OK);
    assert_int_equal(sha256_update(&ctx, NULL, 1U), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(sha256_final(&ctx, NULL), SMART_QSO_ERROR_NULL_PTR);
}

/*===========================================================================*/
/* HMAC-SHA-256 (RFC 4231)                                                    */
/*===========================================================================*/

static void test_hmac_rfc4231_case1(void **state)
{
    (void)state;
    uint8_t key[20];
    uint8_t mac[SHA256_DIGEST_SIZE];

    memset(key, 0x0b, sizeof(key));
    assert_int_equal(hmac_sha256(key, sizeof(key), (const uint8_t *)"Hi There", 8U, mac),
                     SMART_QSO_OK);
    assert_digest(mac, "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7");
}

static void test_hmac_rfc4231_case2(void **state)
{
    (void)state;
    const char *msg = "what do ya want for nothing?";
    uint8_t mac[SHA256_DIGEST_SIZE];

    assert_int_equal(hmac_sha256((const uint8_t *)"Jefe", 4U,
                                 (const uint8_t *)msg, strlen(msg), mac),
                     SMART_QSO_OK);
    assert_digest(mac, "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843");
}

/** Key longer than the block size must be hashed first */
static void test_hmac_rfc4231_case6_long_key(void **state)
{
    (void)state;
    const char *msg = "Test Using Larger Than Block-Size Key - Hash Key First";
    uint8_t key[131];
    uint8_t mac[SHA256_DIGEST_SIZE];

    memset(key, 0xaa, sizeof(key));
    assert_int_equal(hmac_sha256(key, sizeof(key), (const uint8_t *)msg, strlen(msg), mac),
                     SMART_QSO_OK);
    assert_digest(mac, "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54");
}

static void test_hmac_null_arguments(void **state)
{
    (void)state;
    uint8_t mac[SHA256_DIGEST_SIZE];
    const uint8_t key[4] = {1U, 2U, 3U, 4U};

    assert_int_equal(hmac_sha256(NULL, 4U, key, 4U, mac), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(hmac_sha256(key, 4U, NULL, 4U, mac), SMART_QSO_ERROR_NULL_PTR);
    assert_int_equal(hmac_sha256(key, 4U, key, 4U, NULL), SMART_QSO_ERROR_NULL_PTR);
}

/*===========================================================================*/
/* Test Suite                                                                 */
/*===========================================================================*/

int main(void)
{
    const struct CMUnitTest tests[] = {
        cmocka_unit_test(test_sha256_abc),
        cmocka_unit_test(test_sha256_empty),
        cmocka_unit_test(test_sha256_two_blocks),
        cmocka_unit_test(test_sha256_million_a_incremental),
        cmocka_unit_test(test_sha256_null_arguments),
        cmocka_unit_test(test_hmac_rfc4231_case1),
        cmocka_unit_test(test_hmac_rfc4231_case2),
        cmocka_unit_test(test_hmac_rfc4231_case6_long_key),
        cmocka_unit_test(test_hmac_null_arguments),
    };

    return cmocka_run_group_tests_name("SHA-256 / HMAC Tests", tests, NULL, NULL);
}
