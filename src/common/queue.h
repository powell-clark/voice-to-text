#ifndef VTT_QUEUE_H
#define VTT_QUEUE_H

#include <pthread.h>
#include <stdbool.h>

typedef struct queue_node {
    char *data;
    struct queue_node *next;
} queue_node_t;

typedef struct {
    queue_node_t *head;
    queue_node_t *tail;
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    bool shutdown;
} vtt_queue_t;

// Initialize queue
void vtt_queue_init(vtt_queue_t *q);

// Push item (blocking if queue full, but we use unbounded queue)
void vtt_queue_push(vtt_queue_t *q, const char *data);

// Pop item (blocking until available)
char *vtt_queue_pop(vtt_queue_t *q);

// Shutdown queue
void vtt_queue_shutdown(vtt_queue_t *q);

// Cleanup queue
void vtt_queue_destroy(vtt_queue_t *q);

#endif // VTT_QUEUE_H
