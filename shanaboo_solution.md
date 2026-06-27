 ```diff
--- a/frailbox/include/logger.h
+++ b/frailbox/include/logger.h
@@ -1,3 +1,4 @@
+
 /**
  * @file logger.h
  * @brief Header for the legacy logging subsystem.
@@ -140,6 +141,16 @@
  */
 #define LOG_LEVEL_VERBOSE 6
 
+/**
+ * @brief Logger state for post-shutdown behavior.
+ *
+ * After log_shutdown() is called, the logger enters a SHUTTING_DOWN
+ * state and then a SHUTDOWN state. Post-shutdown logging calls are
+ * silently dropped without writing through freed resources.
+ */
+typedef enum {
+    LOGGER_STATE_UNINITIALIZED = 0,
+    LOGGER_STATE_INITIALIZED,
+    LOGGER_STATE_SHUTTING_DOWN,
+    LOGGER_STATE_SHUTDOWN,
+} logger_state_t;
+
 /* ------------------------------------------------------------------ */
 /* FUNCTION PROTOTYPES                                                 */
 /* ------------------------------------------------------------------ */
@@ -148,6 +159,7 @@
  * @brief Initialize the legacy logger.
  *
  * Must be called before any other log functions. Safe to call multiple times.
+ * If the logger was previously shut down, it can be re-initialized.
  *
  * @param level Minimum log level to output.
  * @param file  Optional log file path (NULL for stderr only).
@@ -160,7 +172,9 @@
 /**
  * @brief Shutdown the legacy logger.
  *
- * After this call, logging behavior is undefined. Do not log after shutdown.
+ * After this call, any subsequent log calls are safely dropped.
+ * This function is idempotent and thread-safe with concurrent log calls.
+ * The logger may be re-initialized after shutdown by calling log_init().
  */
 void log_shutdown(void);
 
@@ -169,6 +183,9 @@
  *
  * The actual output depends on the current log level and configuration.
  * This function is thread-safe.
+ *
+ * If called after log_shutdown(), the message is silently dropped
+ * and no output is produced.
  */
 void log_write(int level, const char *file, int line, const char *fmt, ...);
 
@@ -176,6 +193,9 @@
  * @brief Write a hex dump to the log.
  *
  * Dumps data in hex and ASCII format. Useful for binary protocols.
+ *
+ * If called after log_shutdown(), the dump is silently dropped
+ * and no output is produced.
  */
 void log_hex_dump(int level, const char *file, int line, const void *data, size_t len);
 
@@ -183,6 +203,9 @@
  * @brief Flush any buffered log output.
  *
  * Ensures all pending log messages are written to their destinations.
+ *
+ * If called after log_shutdown(), this function does nothing
+ * and returns immediately.
  */
 void log_flush(void);
 
@@ -190,6 +213,14 @@
  * @brief Get the current logger state for diagnostics.
  *
  * Returns a string describing the logger's current state.
+ *
+ * Possible return values:
+ *   - "uninitialized": Logger has never been initialized
+ *   - "initialized": Logger is active and accepting log messages
+ *   - "shutting-down": Logger is in the process of shutting down
+ *   - "shutdown": Logger has been shut down, log calls are dropped
+ *
+ * This function is thread-safe and may be called at any time,
+ * including after log_shutdown().
  */
 const char *log_state_string(void);
 
--- a/frailbox/src/logger.c
+++ b/frailbox/src/logger.c
@@ -1,3 +1,4 @@
+
 /**
  * @file legacy_logger.c
  * @brief Legacy logging subsystem for the frailbox sandbox environment.
@@ -44,6 +45,7 @@
 #include <unistd.h>
 #include <errno.h>
 
+#include "../include/logger.h"
 #include "../include/logger.h" /* This header doesn't exist yet. TODO: Create it. */
 
 /* ------------------------------------------------------------------ */
@@ -93,6 +95,7 @@
 #define LOG_LEVEL_TRACE   5
 #define LOG_LEVEL_VERBOSE 6
 
+/* Default log level (INFO) */
 /* Default log level (INFO) */
 #ifndef DEFAULT_LOG_LEVEL
 #define DEFAULT_LOG_LEVEL LOG_LEVEL_INFO
@@ -102,6 +105,7 @@
 /* MUTEX AND GLOBAL STATE                                              */
 /* ------------------------------------------------------------------ */
 
+/**
 /**
  * Global mutex protecting all logger state.
  * This is a recursive mutex because log_rotate() may call log_write()
@@ -109,6 +113,7 @@
  * The recursive mutex was chosen because it's simpler than refactoring
  * the code to avoid re-entrant locking.
  */
+static pthread_mutex_t g_log_mutex = PTHREAD_MUTEX_INITIALIZER;
 static pthread_mutex_t g_log_mutex = PTHREAD_MUTEX_INITIALIZER;
 
 /**
@@ -116,6 +121,7 @@
  * This is used to detect double-init and to track whether the logger
  * is in a valid state.
  */
 static int g_initialized = 0;
+static volatile logger_state_t g_logger_state = LOGGER_STATE_UNINITIALIZED;
 
 /**
  * Current log level. Only messages at this level or higher are logged.
@@ -147,6 +153,7 @@
  */
 static char g_log_prefix[256];
 
+/**
 /**
  * In-memory ring buffer for crash reporter integration.
  * Each entry is a fixed-size buffer to simplify memory management.
@@ -154,6 +161,7 @@
  * The ring buffer is not thread-safe on its own; it relies on the
  * global mutex being held.
  */
+typedef struct {
 typedef struct {
     char data[MAX_LOG_LINE];
     int level;
@@ -161,6 +169,7 @@
     int line;
 } ring_entry_t;
 
+static ring_entry_t g_ring_buffer[RING_BUFFER_SIZE];
 static ring_entry_t g_ring_buffer[RING_BUFFER_SIZE];
 static size_t g_ring_head = 0;
 static size_t g_ring_count = 0;
@@ -170,6 +179,7 @@
  * This is used by the crash reporter to include recent log entries
  * in crash dumps. The crash reporter is not currently enabled.
  */
+static void ring_buffer_add(const char *msg, int level, const char *file, int line) {
 static void ring_buffer_add(const char *msg, int level, const char *file, int line) {
     ring_entry_t *entry = &g_ring_buffer[g_ring_head];
     strncpy(entry->data, msg, MAX_LOG_LINE - 1);
@@ -185,6 +195,7 @@
 /* INTERNAL HELPERS                                                    */
 /* ------------------------------------------------------------------ */
 
+/**
 /**
  * Get current timestamp as a string.
  * Uses localtime_r for thread safety.
@@ -192,6 +203,7 @@
  * The buffer must be at least 64 bytes to hold the full timestamp
  * including milliseconds and timezone.
  */
+static void format_timestamp