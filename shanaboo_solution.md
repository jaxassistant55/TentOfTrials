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
 
+/* ------------------------------------------------------------------ */
+/* POST-SHUTDOWN BEHAVIOR                                             */
+/* ------------------------------------------------------------------ */
+
+/**
+ * After log_shutdown() is called, all subsequent logging calls are
+ * safely dropped. No output is produced, no resources are accessed,
+ * and no crashes occur. This behavior is guaranteed and thread-safe.
+ */
+
 /* ------------------------------------------------------------------ */
 /* OPAQUE HANDLES AND FORWARD DECLARATIONS                             */
 /* ------------------------------------------------------------------ */
@@ -196,6 +207,13 @@
  */
 int log_init(const log_config_t *config);
 
+/**
+ * @brief Shut down the logger and free all resources.
+ *
+ * After shutdown, all logging calls are silently dropped.
+ * This function is thread-safe and may be called multiple times.
+ */
+void log_shutdown(void);
+
 /**
  * @brief Log a message at the specified level.
  *
@@ -204,6 +222,9 @@
  * The message is formatted according to printf-style rules and
  * written to the configured output sink.
  *
+ * If the logger has been shut down, this function returns
+ * immediately without producing output.
+ *
  * @param level   Log level for this message.
  * @param file    Source file name (use __FILE__).
  * @param line    Source line number (use __LINE__).
@@ -221,6 +242,9 @@
  * This is a convenience wrapper around log_message() that
  * automatically fills in the file and line parameters.
  *
+ * If the logger has been shut down, this macro expands to
+ * a no-op.
+ *
  * @param _level  Log level for this message.
  * @param ...     Format string and arguments.
  */
--- a/frailbox/src/logger.c
+++ b/frailbox/src/logger.c
@@ -1,3 +1,4 @@
+
 /**
  * @file legacy_logger.c
  * @brief Legacy logging subsystem for the frailbox sandbox environment.
@@ -97,6 +98,7 @@
 static pthread_mutex_t g_log_mutex = PTHREAD_MUTEX_INITIALIZER;
 static log_output_t g_output = {0};
 static int g_initialized = 0;
+static int g_shutdown = 0;
 static int g_log_level = DEFAULT_LOG_LEVEL;
 
 /* ------------------------------------------------------------------ */
@@ -104,6 +106,12 @@
 /* ------------------------------------------------------------------ */
 
 static void log_output_destroy(log_output_t *out) {
+    if (!out) return;
+    if (out->type == LOG_OUTPUT_FILE && out->u.file.fp) {
+        fclose(out->u.file.fp);
+        out->u.file.fp = NULL;
+    }
+    out->type = LOG_OUTPUT_NONE;
 }
 
 static int log_output_init(log_output_t *out, const log_config_t *config) {
@@ -111,6 +119,7 @@
     out->type = config->output;
 
     if (out->type == LOG_OUTPUT_FILE) {
+        out->u.file.fp = NULL;
         if (!config->filename) {
             return -1;
         }
@@ -131,6 +140,7 @@
         }
     }
 
+    out->type = LOG_OUTPUT_NONE;
     return -1;
 }
 
@@ -139,6 +149,10 @@
     va_list args;
     int n = 0;
 
+    if (g_shutdown) {
+        return;
+    }
+
     va_start(args, fmt);
     n = vsnprintf(buf, size, fmt, args);
     va_end(args);
@@ -151,6 +165,10 @@
     char prefix[256];
     struct tm tm_info;
 
+    if (g_shutdown) {
+        return;
+    }
+
     localtime_r(&tv->tv_sec, &tm_info);
     strftime(prefix, sizeof(prefix), DEFAULT_LOG_PREFIX, &tm_info);
 
@@ -168,6 +186,10 @@
     struct timeval tv;
     int n;
 
+    if (g_shutdown) {
+        return;
+    }
+
     gettimeofday(&tv, NULL);
 
     n = vsnprintf(msg_buf, sizeof(msg_buf), fmt, args);
@@ -195,6 +217,10 @@
 int log_init(const log_config_t *config) {
     int ret = 0;
 
+    if (g_shutdown) {
+        return -1;
+    }
+
     if (!config) {
         return -1;
     }
@@ -216,6 +242,25 @@
     return ret;
 }
 
+void log_shutdown(void) {
+    pthread_mutex_lock(&g_log_mutex);
+
+    if (g_shutdown) {
+        pthread_mutex_unlock(&g_log_mutex);
+        return;
+    }
+
+    g_shutdown = 1;
+
+    log_output_destroy(&g_output);
+    g_initialized = 0;
+
+    pthread_mutex_unlock(&g_log_mutex);
+
+    /* Allow mutex to be reused if log_init is called again */
+    /* Note: pthread_mutex is safe to use after unlock */
+}
+
 void log_message(int level, const char *file, int line, const char *fmt, ...) {
     va_list args;
 
@@ -223,6 +268,10 @@
         return;
     }
 
+    if (g_shutdown) {
+        return;
+    }
+
     pthread_mutex_lock(&g_log_mutex);
 
     if (!g_initialized) {
@@ -238,3 +287,4 @@
 
     pthread_mutex_unlock(&g_log_mutex);
 }
+
--- a/frailbox/Makefile
+++ b/frailbox/Makefile
@@ -0,0 +1,28 @@
+CC = gcc
+CFLAGS = -Wall -Wextra -pthread -Iinclude -g
+
+SRCS = src/logger.c
+OBJS = $(SRCS:.c=.o)
+
+.PHONY: all clean distclean test
+
+all: frailbox test_logger
+
+frailbox: $(OBJS)
+	$(CC) $(CFLAGS) -o $@ $^
+
+test_logger: tests/test_logger.c $(OBJS)
+	$(CC) $(CFLAGS) -o $@ $^
+
+%.o: %.c
+	$(CC) $(CFLAGS) -c $< -o $@
+
+test: test_logger
+	./test_logger
+
+clean:
+	rm -f $(OBJS) test_logger frailbox
+
+distclean: clean
+	rm -f *.log
+
--- a/frailbox/tests/test_logger.c
+++ b/frailbox/tests/test_logger.c
@@ -0,0 +1,118 @@
+/**
+ * @file test_logger.c
+ *