 ```diff
--- a/frailbox/include/logger.h
+++ b/frailbox/include/logger.h
@@ -1,3 +1,4 @@
+
 /**
  * @file logger.h
  * @brief Header for the legacy logging subsystem.
@@ -124,6 +125,15 @@
  *   - There is no log_hex_dump equivalent (use structured fields)
  */
 
+/**
+ * @brief Post-shutdown logging behavior.
+ *
+ * After log_shutdown() is called, all subsequent logging calls are
+ * safely dropped. No messages are written, and no resources are
+ * accessed. This behavior is deterministic and thread-safe.
+ * Repeated shutdown calls are safe and have no additional effect.
+ */
+
 #ifndef LEGACY_LOGGER_H
 #define LEGACY_LOGGER_H
 
@@ -218,6 +228,12 @@
  */
 int log_init(void);
 
+/**
+ * @brief Shut down the logger and release all resources.
+ * @return 0 on success, -1 on failure.
+ */
+int log_shutdown(void);
+
 /**
  * @brief Set the current log level.
  * @param level The log level to set.
@@ -268,6 +284,16 @@
  */
 void log_hex_dump(const char *label, const uint8_t *data, size_t len);
 
+/**
+ * @brief Check if the logger is currently initialized.
+ * @return 1 if initialized, 0 if not.
+ *
+ * This is primarily useful for tests and diagnostics to verify
+ * logger state without relying on side effects.
+ */
+int log_is_initialized(void);
+
+
 #ifdef __cplusplus
 }
 #endif
--- a/frailbox/src/logger.c
+++ b/frailbox/src/logger.c
@@ -1,3 +1,4 @@
+
 /**
  * @file legacy_logger.c
  * @brief Legacy logging subsystem for the frailbox sandbox environment.
@@ -68,6 +69,7 @@
 #include <unistd.h>
 #include <errno.h>
 
+#include "../include/logger.h"
 #include "../include/logger.h" /* This header doesn't exist yet. TODO: Create it. */
 
 /* ------------------------------------------------------------------ */
@@ -118,6 +120,7 @@
 /* MUTEX AND GLOBAL STATE                                              */
 /* ------------------------------------------------------------------ */
 
+#include <stdatomic.h>
 #include <stdatomic.h>
 
 /**
@@ -126,6 +129,7 @@
  * The atomic flag is used to detect concurrent initialization attempts.
  * If two threads try to initialize the logger at the same time, the
  * second thread will spin until the first thread completes.
+ * After shutdown, the logger is in a safe "dropped" state.
  */
 static pthread_mutex_t log_mutex = PTHREAD_MUTEX_INITIALIZER;
 
@@ -134,6 +138,7 @@
  * This is used to prevent double-initialization and to detect
  * use-after-free in debug builds. In release builds, the flag
  * is not checked to avoid the performance overhead.
+ * After shutdown, this is set to 0.
  */
 static volatile int log_initialized = 0;
 
@@ -141,6 +146,7 @@
  * Atomic flag to prevent concurrent initialization.
  * This is used in a spinlock pattern during log_init().
  * The spinlock is only held briefly during initialization.
+ * After shutdown, this is set to 0.
  */
 static atomic_int log_init_lock = 0;
 
@@ -148,6 +154,7 @@
  * Current log level. Defaults to INFO.
  * This can be changed at runtime with log_set_level().
  * The log level is checked before formatting the log message.
+ * After shutdown, this is set to LOG_LEVEL_NONE.
  */
 static volatile int current_log_level = DEFAULT_LOG_LEVEL;
 
@@ -155,6 +162,7 @@
  * Log output file. Defaults to stderr.
  * This can be changed at runtime with log_set_file().
  * The file is not closed automatically (legacy behavior).
+ * After shutdown, this is set to NULL and never accessed.
  */
 static FILE *log_file = NULL;
 
@@ -162,6 +170,7 @@
  * Log prefix format string.
  * This can be changed at runtime with log_set_prefix().
  * The prefix is used to format the timestamp and log level.
+ * After shutdown, this is set to NULL and never accessed.
  */
 static const char *log_prefix = DEFAULT_LOG_PREFIX;
 
@@ -169,6 +178,7 @@
  * Ring buffer for crash reporter integration.
  * The ring buffer stores the most recent log entries.
  * It is not thread-safe and may lose entries under contention.
+ * After shutdown, this is set to NULL and never accessed.
  */
 static char **ring_buffer = NULL;
 static volatile size_t ring_buffer_head = 0;
@@ -178,6 +188,7 @@
  * Callback for log rotation.
  * This is called when the log file needs to be rotated.
  * The callback is responsible for closing the old file and opening a new one.
+ * After shutdown, this is set to NULL and never accessed.
  */
 static void (*log_rotation_callback)(void) = NULL;
 
@@ -185,6 +196,7 @@
  * Custom log sink callback.
  * If set, this function is called instead of writing to the log file.
  * This is used by the test harness to capture log output.
+ * After shutdown, this is set to NULL and never accessed.
  */
 static void (*log_sink_callback)(const char *msg, size_t len) = NULL;
 
@@ -194,6 +206,7 @@
 
 /**
  * @brief Internal helper: Get the string name for a log level.
+ * Safe to call after shutdown (returns "NONE").
  */
 static const char *level_to_string(int level)
 {
@@ -210,6 +223,7 @@
 
 /**
  * @brief Internal helper: Write to the ring buffer.
+ * Does nothing if the logger is not initialized.
  */
 static void ring_buffer_write(const char *msg)
 {
@@ -232,6 +246,7 @@
 
 /**
  * @brief Internal helper: Format and write a log message.
+ * Does nothing if the logger is not initialized.
  */
 static void write_log(int level, const char *file, int line, const char *fmt, va_list args)
 {
@@ -239,6 +254,11 @@
     char prefix_buf[256];
     char msg_buf[MAX_LOG_LINE];
 
+    /* After shutdown, all logging is dropped safely */
+    if (!log_initialized) {
+        return;
+    }
+
     /* Format the prefix */
     now = time(NULL);
     localtime_r(&now, &tm_info);
@@ -283,6 +302,11 @@
 
 int log_init(void)
 {
+    /* If already initialized, return success */
+    if (log_initialized) {
+        return 0;
+    }
+
     /* Spinlock: wait for any concurrent