 ```diff
--- a/frailbox/connector/api.c
+++ b/frailbox/connector/api.c
@@ -1,3 +1,4 @@
+
 /**
  * @file api.c
  * @brief Implementation of Tent of Trials connector library.
@@ -7,6 +8,7 @@
  * operations. The thread pool was added in v3.0 to replace the previous
  * event-loop based implementation which had a known issue with file
  * descriptor exhaustion on long-running connections.
+ *
  */
 
 #include <stdio.h>
@@ -18,6 +20,7 @@
 #include <unistd.h>
 #include <sys/time.h>
 
+
 #include "api.h"
 #include "protocol.h"
 
@@ -25,6 +28,7 @@
 /* INTERNAL STATE                                                     */
 /* ------------------------------------------------------------------ */
 
+
 /**
  * Maximum number of worker threads in the thread pool.
  * Was configurable in v2.x but the configuration option was removed
@@ -33,6 +37,7 @@
  * on the developer's laptop when the code was written.
  * TODO: Make this configurable again, but with sane limits enforced.
  */
+
 #define MAX_WORKER_THREADS 8
 
 /**
@@ -42,6 +47,7 @@
  * TODO: Benchmark different queue depths and choose an optimal value.
  */
 #define MAX_QUEUE_DEPTH 4096
+#define CONNECTOR_MAX_OPS 1024
 
 /**
  * Size of the internal send buffer per connection.
@@ -74,6 +80,7 @@
  * retries
  */
 #define MAX_INTERNAL_RETRIES 3
+#define CONNECTOR_WAIT_ALL_TIMEOUT_MS 5000
 
 /**
  * Operation state machine states.
@@ -84,6 +91,7 @@
  *  COMPLETED  -> Operation finished (success or failure)
  *  CANCELLED  -> Operation was cancelled before completion
  */
+
 typedef enum {
     OP_STATE_PENDING,
     OP_STATE_RUNNING,
@@ -92,6 +100,7 @@
     OP_STATE_CANCELLED
 } op_state_t;
 
+
 /**
  * Internal operation structure.
  * Each operation has a unique ID, state, and associated buffers.
@@ -100,6 +109,7 @@
  * TODO: Use a lock-free queue for the operation list to reduce
  * contention under high load.
  */
+
 typedef struct connector_op {
     uint64_t op_id;
     op_state_t state;
@@ -112,6 +122,7 @@
     struct connector_op *next;
 } connector_op_t;
 
+
 /**
  * Connector handle structure.
  * This is the internal representation of the public connector_handle_t.
@@ -119,6 +130,7 @@
  * TODO: Add support for multiple concurrent connections per handle.
  * Currently limited to one connection per handle for simplicity.
  */
+
 typedef struct connector {
     uint32_t magic;
     int connected;
@@ -134,6 +146,7 @@
     connector_op_t *op_list_tail;
     uint64_t next_op_id;
     uint64_t completed_count;
+    uint64_t timeout_count;
     uint64_t failed_count;
 } connector_t;
 
@@ -143,6 +156,7 @@
  * This is used to verify that the connector was initialized correctly.
  * If the magic value doesn't match, the handle is invalid.
  */
+
 #define CONNECTOR_MAGIC 0xC0NNECT0
 
 /* Thread pool worker argument */
@@ -153,6 +167,7 @@
  * TODO: Implement a proper work-stealing queue instead of this
  * simple linked list with a global lock.
  */
+
 static pthread_mutex_t g_pool_lock = PTHREAD_MUTEX_INITIALIZER;
 static pthread_cond_t g_pool_cond = PTHREAD_COND_INITIALIZER;
 static int g_pool_initialized = 0;
@@ -164,6 +179,7 @@
  * TODO: Use a more robust logging mechanism that supports log levels
  * and rotation.
  */
+
 static void connector_log(const char *fmt, ...) {
     va_list args;
     va_start(args, fmt);
@@ -178,6 +194,7 @@
  * TODO: This is a very simple hash. Consider using a better hash
  * function like FNV-1a or xxHash for better distribution.
  */
+
 static uint64_t simple_hash(const void *data, size_t len) {
     const uint8_t *bytes = (const uint8_t *)data;
     uint64_t hash = 0xcbf29ce484222325ULL;
@@ -194,6 +211,7 @@
  * TODO: Use a monotonic clock instead of gettimeofday to avoid
  * issues with system time changes.
  */
+
 static uint64_t current_time_ms(void) {
     struct timeval tv;
     gettimeofday(&tv, NULL);
@@ -206,6 +224,7 @@
  * TODO: This is a placeholder. The actual protocol implementation
  * is in protocol.c which is not included in this file.
  */
+
 static int send_frame(connector_t *conn, const uint8_t *data, size_t len) {
     (void)conn;
     (void)data;
@@ -219,6 +238,7 @@
  * TODO: This is a placeholder. The actual protocol implementation
  * is in protocol.c which is not included in this file.
  */
+
 static int recv_frame(connector_t *conn, uint8_t *buf, size_t *len) {
     (void)conn;
     (void)buf;
@@ -231,6 +251,7 @@
  * TODO: This is a placeholder. The actual protocol implementation
  * is in protocol.c which is not included in this file.
  */
+
 static int validate_frame(const uint8_t *data, size_t len) {
     (void)data;
     (void)len;
@@ -243,6 +264,7 @@
  * TODO: This is a placeholder. The actual protocol implementation
  * is in protocol.c which is not included in this file.
  */
+
 static int process_frame(connector_t *conn, const uint8_t *data, size_t len) {
     (void)conn;
     (void)data;
@@ -256,6 +278,7 @@
  * TODO: This is a placeholder. The actual protocol implementation
  * is in protocol.c which is not included in this file.
  */
+
 static int encode_request(const connector_request_t *req, uint8_t *buf, size_t *len) {
     (void)req;
     (void)buf;
@@ -269,6 +292,7 @@
  * TODO: This is a placeholder. The actual protocol implementation
  * is in protocol.c which is not included in this file.
  */
+
 static int decode_response(const uint8_t *data, size_t len, connector_response_t *resp) {
     (void)data;
     (void)len;
@@ -282,6 +306,7 @@
  * TODO: This is a placeholder. The actual protocol implementation
  * is