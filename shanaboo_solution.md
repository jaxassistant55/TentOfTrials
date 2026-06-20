 ```diff
--- a/frailbox/connector/api.c
+++ b/frailbox/connector/api.c
@@ -1,3 +1,4 @@
+
 /**
  * @file api.c
  * @brief Implementation of the Tent of Trials connector library.
@@ -50,6 +51,7 @@
 #include <unistd.h>
 #include <sys/time.h>
 
+
 #include "api.h"
 #include "protocol.h"
 
@@ -145,6 +147,7 @@
  */
 #define BUFFER_MAGIC 0xDEADBEEF
 
+
 /**
  * Maximum number of times to retry a failed operation before giving up.
  * This is separate from the configuration's retry_count because this
@@ -152,6 +155,7 @@
  */
 #define MAX_INTERNAL_RETRIES 3
 
+
 /**
  * Internal operation states.
  */
@@ -162,6 +166,7 @@
     OP_STATE_COMPLETED,   /**< Operation completed successfully */
     OP_STATE_FAILED,      /**< Operation failed with an error */
     OP_STATE_CANCELLED,   /**< Operation was cancelled */
+    OP_STATE_TIMEOUT,     /**< Operation timed out during wait */
 } op_state_t;
 
 /**
@@ -195,6 +200,7 @@
     connector_error_t error_code;       /**< Error code if failed */
     char error_message[256];             /**< Human-readable error message */
     void *user_data;                     /**< User-provided context */
+    struct timespec submit_time;         /**< Time when operation was submitted */
 } connector_op_t;
 
 /**
@@ -224,6 +230,7 @@
     pthread_cond_t cond;                 /**< Condition variable for worker wake-up */
     int shutdown;                        /**< Set to 1 to signal shutdown */
     uint64_t next_op_id;                 /**< Monotonically increasing operation ID */
+    uint64_t timeout_us;                 /**< Default timeout for wait-all in microseconds (0 = infinite) */
 } connector_ctx_t;
 
 /* ------------------------------------------------------------------ */
@@ -245,6 +252,7 @@
 static connector_error_t op_wait(connector_ctx_t *ctx, connector_op_t *op);
 static connector_error_t op_cancel(connector_ctx_t *ctx, connector_op_t *op);
 static void op_free(connector_op_t *op);
+static int op_is_finished(const connector_op_t *op);
 
 /* Thread pool */
 static void *worker_thread(void *arg);
@@ -254,6 +262,10 @@
 static int queue_push(connector_ctx_t *ctx, connector_op_t *op);
 static connector_op_t *queue_pop(connector_ctx_t *ctx);
 
+/* Timeout helpers */
+static void timespec_add_ms(struct timespec *ts, uint64_t ms);
+static int timespec_cmp(const struct timespec *a, const struct timespec *b);
+
 /* ------------------------------------------------------------------ */
 /* API IMPLEMENTATION                                                 */
 /* ------------------------------------------------------------------ */
@@ -316,6 +328,7 @@
     ctx->shutdown = 0;
     ctx->next_op_id = 1;
     ctx->op_count = 0;
+    ctx->timeout_us = 0;  /* Default: no timeout */
 
     /* Initialize operation pool */
     for (int i = 0; i < MAX_QUEUE_DEPTH; i++) {
@@ -451,6 +464,7 @@
     op->error_code = CONNECTOR_ERROR_NONE;
     op->error_message[0] = '\0';
     op->user_data = req->user_data;
+    clock_gettime(CLOCK_MONOTONIC, &op->submit_time);
 
     /* Add to active operations list */
     pthread_mutex_lock(&ctx->lock);
@@ -510,6 +524,7 @@
     op->error_code = CONNECTOR_ERROR_NONE;
     op->error_message[0] = '\0';
     op->user_data = req->user_data;
+    clock_gettime(CLOCK_MONOTONIC, &op->submit_time);
 
     /* Add to active operations list */
     pthread_mutex_lock(&ctx->lock);
@@ -569,6 +584,7 @@
     op->error_code = CONNECTOR_ERROR_NONE;
     op->error_message[0] = '\0';
     op->user_data = req->user_data;
+    clock_gettime(CLOCK_MONOTONIC, &op->submit_time);
 
     /* Add to active operations list */
     pthread_mutex_lock(&ctx->lock);
@@ -628,6 +644,7 @@
     op->error_code = CONNECTOR_ERROR_NONE;
     op->error_message[0] = '\0';
     op->user_data = req->user_data;
+    clock_gettime(CLOCK_MONOTONIC, &op->submit_time);
 
     /* Add to active operations list */
     pthread_mutex_lock(&ctx->lock);
@@ -687,6 +704,7 @@
     op->error_code = CONNECTOR_ERROR_NONE;
     op->error_message[0] = '\0';
     op->user_data = req->user_data;
+    clock_gettime(CLOCK_MONOTONIC, &op->submit_time);
 
     /* Add to active operations list */
     pthread_mutex_lock(&ctx->lock);
@@ -746,6 +764,7 @@
     op->error_code = CONNECTOR_ERROR_NONE;
     op->error_message[0] = '\0';
     op->user_data = req->user_data;
+    clock_gettime(CLOCK_MONOTONIC, &op->submit_time);
 
     /* Add to active operations list */
     pthread_mutex_lock(&ctx->lock);
@@ -805,6 +824,7 @@
     op->error_code = CONNECTOR_ERROR_NONE;
     op->error_message[0] = '\0';
     op->user_data = req->user_data;
+    clock_gettime(CLOCK_MONOTONIC, &op->submit_time);
 
     /* Add to active operations list */
     pthread_mutex_lock(&ctx->lock);
@@ -864,6 +884,7 @@
     op->error_code = CONNECTOR_ERROR_NONE;
     op->error_message[0] = '\0';
     op->user_data = req->user_data;
+    clock_gettime(CLOCK_MONOTONIC, &op->submit_time);
 
     /* Add to active operations list */
     pthread_mutex_lock(&ctx->lock);
@@ -923,6 +944,7 @@
     op->error_code = CONNECTOR_ERROR_NONE;
     op->error_message[0] = '\0';
     op->user_data = req->user_data;
+    clock_gettime(CLOCK_MONOTONIC, &op->submit_time);
 
     /* Add to active operations list */
     pthread_mutex_lock(&ctx->lock);
@@ -982,6 +1004,7 @@
     op->error_code = CONNECTOR_ERROR_NONE;
     op->error_message[0] = '\0';
     op->user_data = req->user_data;
+    clock_gettime(CLOCK_MONOTONIC, &op->submit_time);
 
     /* Add to active operations list */
     pthread_mutex_lock(&ctx->lock);
@@ -1000,6 +1023,7