/**
 * @file test_logger_shutdown.c
 * @brief Test harness for logger post-shutdown behavior.
 *
 * Compile:
 *   gcc -DTEST_LOGGER_SHUTDOWN -I../include ../src/logger.c -o test_logger_shutdown -lpthread
 *
 * Run:
 *   ./test_logger_shutdown
 *
 * Tests:
 *   1. log-before-shutdown  — messages appear before shutdown
 *   2. log-after-shutdown   — messages are silently dropped after shutdown
 *   3. repeated-shutdown    — calling shutdown twice is safe
 *   4. concurrent-shutdown  — log and shutdown from threads simultaneously
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <pthread.h>
#include <unistd.h>

#include "../include/logger.h"

/* ------------------------------------------------------------------ */
/* HELPERS                                                            */
/* ------------------------------------------------------------------ */

static int test_count = 0;
static int pass_count = 0;

#define TEST_ASSERT(cond, msg) do {                                     \
    test_count++;                                                       \
    if (cond) {                                                         \
        pass_count++;                                                   \
        printf("  PASS: %s\n", msg);                                    \
    } else {                                                            \
        printf("  FAIL: %s (line %d)\n", msg, __LINE__);               \
    }                                                                   \
} while(0)

/* ------------------------------------------------------------------ */
/* TEST: Log before shutdown                                          */
/* ------------------------------------------------------------------ */
static void test_log_before_shutdown(void)
{
    printf("\n[TEST] Log before shutdown\n");
    log_init();
    TEST_ASSERT(log_is_shutdown() == 0, "logger is not shut down after init");

    /* Messages should be produced — we can't easily capture stderr in
     * this harness, but we verify the logger doesn't crash. */
    LOG_INFO("test: message before shutdown");
    LOG_ERROR("test: error before shutdown");
    TEST_ASSERT(1, "log messages before shutdown did not crash");

    log_shutdown();
}

/* ------------------------------------------------------------------ */
/* TEST: Log after shutdown                                           */
/* ------------------------------------------------------------------ */
static void test_log_after_shutdown(void)
{
    printf("\n[TEST] Log after shutdown\n");
    log_init();
    log_shutdown();
    TEST_ASSERT(log_is_shutdown() == 1, "logger is shut down after shutdown call");

    /* These should be silently dropped — no crash, no write to freed resources */
    LOG_ERROR("test: this should be silently dropped");
    LOG_INFO("test: this too should be silently dropped");
    LOG_DEBUG("test: and this");
    TEST_ASSERT(1, "log messages after shutdown did not crash");

    /* log_message direct call */
    log_message(LOG_LEVEL_ERROR, __FILE__, __LINE__,
                "direct call after shutdown — should be dropped");
    TEST_ASSERT(1, "direct log_message after shutdown did not crash");
}

/* ------------------------------------------------------------------ */
/* TEST: Repeated shutdown                                            */
/* ------------------------------------------------------------------ */
static void test_repeated_shutdown(void)
{
    printf("\n[TEST] Repeated shutdown\n");
    log_init();
    LOG_INFO("test: before first shutdown");

    log_shutdown();
    TEST_ASSERT(log_is_shutdown() == 1, "shut down after first call");

    /* Second shutdown should be safe */
    log_shutdown();
    TEST_ASSERT(log_is_shutdown() == 1, "still shut down after second call");

    /* Log after repeated shutdown */
    LOG_ERROR("test: after repeated shutdown");
    TEST_ASSERT(1, "log after repeated shutdown did not crash");
}

/* ------------------------------------------------------------------ */
/* TEST: Reinitialize after shutdown                                  */
/* ------------------------------------------------------------------ */
static void test_reinit_after_shutdown(void)
{
    printf("\n[TEST] Reinitialize after shutdown\n");
    log_init();
    LOG_INFO("test: first init");

    log_shutdown();
    TEST_ASSERT(log_is_shutdown() == 1, "shut down");

    /* Re-init should clear shutdown state */
    log_init();
    TEST_ASSERT(log_is_shutdown() == 0, "not shut down after re-init");

    LOG_INFO("test: after re-init");
    TEST_ASSERT(1, "log after re-init did not crash");

    log_shutdown();
}

/* ------------------------------------------------------------------ */
/* TEST: Concurrent log and shutdown                                  */
/* ------------------------------------------------------------------ */

static volatile int concurrent_stop = 0;

static void *concurrent_logger_thread(void *arg)
{
    (void)arg;
    while (!concurrent_stop) {
        LOG_INFO("concurrent thread logging");
    }
    return NULL;
}

static void test_concurrent_log_shutdown(void)
{
    printf("\n[TEST] Concurrent log and shutdown\n");

    log_init();

    pthread_t threads[4];
    for (int i = 0; i < 4; i++) {
        pthread_create(&threads[i], NULL, concurrent_logger_thread, NULL);
    }

    /* Let threads log for a bit */
    usleep(50000); /* 50ms */

    /* Shutdown while threads are still logging */
    log_shutdown();
    TEST_ASSERT(log_is_shutdown() == 1, "shut down while threads logging");

    /* Signal threads to stop */
    concurrent_stop = 1;

    for (int i = 0; i < 4; i++) {
        pthread_join(threads[i], NULL);
    }

    TEST_ASSERT(1, "concurrent log + shutdown did not crash");
    concurrent_stop = 0;
}

/* ------------------------------------------------------------------ */
/* MAIN                                                               */
/* ------------------------------------------------------------------ */

int main(void)
{
    printf("=== Logger Shutdown Behavior Tests ===\n");

    test_log_before_shutdown();
    test_log_after_shutdown();
    test_repeated_shutdown();
    test_reinit_after_shutdown();
    test_concurrent_log_shutdown();

    printf("\n=== Results: %d/%d passed ===\n", pass_count, test_count);

    return (pass_count == test_count) ? 0 : 1;
}
