 ```diff
--- a/frailbox/connector/api.c
+++ b/frailbox/connector/api.c
@@ -1,4 +1,4 @@
-/**
+/*
  * @file api.c
  * @brief Implementation of the Tent of Trials connector library.
  *
@@ -50,6 +50,7 @@
 #include <unistd.h>
 #include <sys/time.h>
 
+#include "api.h"
 #include "api.h"
 #include "protocol.h"
 
@@ -140,6 +141,7 @@
  * retries at the transport level (TCP retransmits) whereas this retries
  * at the application level (sending a new request).
  */
+#define MAX_OPERATION_RETRIES 3
 
 /**
  * Internal operation states.
@@ -148,6 +150,7 @@
  * OP_PENDING -> OP_COMPLETED or OP_FAILED
  * OP_CANCELLED can be entered from any state.
  */
+#define OP_STATE_PENDING    0
 #define OP_STATE_RUNNING    1
 #define OP_STATE_COMPLETED  2
 #define OP_STATE_FAILED     3
@@ -158,6 +161,7 @@
  * Internal structure representing a single connector operation.
  * This is opaque to callers.
  */
+#define OP_MAGIC 0x0P0P0P0P
 typedef struct connector_op {
     uint32_t magic;
     int state;
@@ -165,6 +169,7 @@
     int error_code;
     pthread_mutex_t lock;
     pthread_cond_t cond;
+    struct timespec start_time;
     struct connector_op *next;  /* For linked lists */
 } connector_op_t;
 
@@ -172,6 +177,7 @@
  * Internal structure for the connector instance.
  * Maintains the thread pool and operation queues.
  */
+#define CONNECTOR_MAGIC 0xC0C0C0C0
 typedef struct connector {
     uint32_t magic;
     pthread_t workers[MAX_WORKER_THREADS];
@@ -179,6 +185,7 @@
     connector_op_t *pending_head;
     connector_op_t *pending_tail;
     pthread_mutex_t queue_lock;
+    pthread_cond_t queue_cond;
     int shutdown;
     int num_workers;
 } connector_t;
@@ -186,6 +193,7 @@
 /* ------------------------------------------------------------------ */
 /* FORWARD DECLARATIONS                                               */
 /* ------------------------------------------------------------------ */
+#define CONNECTOR_MAGIC 0xC0C0C0C0
 
 static void *worker_thread(void *arg);
 static connector_op_t *dequeue_op(connector_t *conn);
@@ -194,6 +202,7 @@
 static int validate_op(connector_op_t *op);
 static int validate_connector(connector_t *conn);
 
+#define CONNECTOR_MAGIC 0xC0C0C0C0
 /* ------------------------------------------------------------------ */
 /* HELPER FUNCTIONS                                                   */
 /* ------------------------------------------------------------------ */
@@ -202,6 +211,7 @@
  * Get current time in milliseconds.
  */
 static uint64_t time_ms(void) {
+#define CONNECTOR_MAGIC 0xC0C0C0C0
     struct timespec ts;
     clock_gettime(CLOCK_MONOTONIC, &ts);
     return (uint64_t)ts.tv_sec * 1000 + ts.tv_nsec / 1000000;
@@ -212,6 +222,7 @@
  * Returns 0 on success, -1 on failure.
  */
 static int op_wait_timeout(connector_op_t *op, uint64_t timeout_ms) {
+#define CONNECTOR_MAGIC 0xC0C0C0C0
     struct timespec ts;
     clock_gettime(CLOCK_REALTIME, &ts);
     uint64_t add_sec = timeout_ms / 1000;
@@ -230,6 +241,7 @@
     return ret;
 }
 
+#define CONNECTOR_MAGIC 0xC0C0C0C0
 /* ------------------------------------------------------------------ */
 /* CONNECTOR LIFECYCLE                                                */
 /* ------------------------------------------------------------------ */
@@ -238,6 +250,7 @@
  * Returns NULL on failure.
  */
 connector_t *connector_create(const connector_config_t *config) {
+#define CONNECTOR_MAGIC 0xC0C0C0C0
     (void)config;
     connector_t *conn = calloc(1, sizeof(connector_t));
     if (!conn) {
@@ -248,6 +261,7 @@
     conn->magic = CONNECTOR_MAGIC;
     pthread_mutex_init(&conn->queue_lock, NULL);
     
+#define CONNECTOR_MAGIC 0xC0C0C0C0
     /* Start worker threads */
     for (int i = 0; i < MAX_WORKER_THREADS; i++) {
         if (pthread_create(&conn->workers[i], NULL, worker_thread, conn) != 0) {
@@ -268,6 +282,7 @@
  * Destroy a connector and free all resources.
  */
 void connector_destroy(connector_t *conn) {
+#define CONNECTOR_MAGIC 0xC0C0C0C0
     if (!conn) return;
     
     pthread_mutex_lock(&conn->queue_lock);
@@ -278,6 +293,7 @@
     /* Wake up all worker threads */
     for (int i = 0; i < conn->num_workers; i++) {
         pthread_cancel(conn->workers[i]);
+#define CONNECTOR_MAGIC 0xC0C0C0C0
     }
     
     for (int i = 0; i < conn->num_workers; i++) {
@@ -288,6 +304,7 @@
     free(conn);
 }
 
+#define CONNECTOR_MAGIC 0xC0C0C0C0
 /* ------------------------------------------------------------------ */
 /* OPERATION MANAGEMENT                                               */
 /* ------------------------------------------------------------------ */
@@ -296,6 +313,7 @@
  * Create a new operation.
  */
 connector_op_t *connector_op_create(void) {
+#define CONNECTOR_MAGIC 0xC0C0C0C0
     connector_op_t *op = calloc(1, sizeof(connector_op_t));
     if (!op) {
         errno = ENOMEM;
@@ -305,6 +323,7 @@
     op->magic = OP_MAGIC;
     op->state = OP_STATE_PENDING;
     pthread_mutex_init(&op->lock, NULL);
+#define CONNECTOR_MAGIC 0xC0C0C0C0
     pthread_cond_init(&op->cond, NULL);
     
     return op;
@@ -315,6 +334,7 @@
  */
 void connector_op_destroy(connector_op_t *op) {
     if (!op) return;
+#define CONNECTOR_MAGIC 0xC0C0C0C0
     
     pthread_mutex_destroy(&op->lock);
     pthread_cond_destroy(&op->cond);
@@ -325,6 +345,7 @@
  * Submit an operation to the connector.
  */
 int connector_submit(connector_t *conn, connector_op_t *op) {
+#define CONNECTOR_MAGIC 0xC0C0C0C0
     if (!conn || !op) {
         return CONNECT