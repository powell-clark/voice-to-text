#include "queue.h"
#include <stdlib.h>
#include <string.h>

void vtt_queue_init(vtt_queue_t *q) {
    q->head = NULL;
    q->tail = NULL;
    q->shutdown = false;
    pthread_mutex_init(&q->mutex, NULL);
    pthread_cond_init(&q->cond, NULL);
}

void vtt_queue_push(vtt_queue_t *q, const char *data) {
    queue_node_t *node = malloc(sizeof(queue_node_t));
    node->data = strdup(data);
    node->next = NULL;

    pthread_mutex_lock(&q->mutex);

    if (q->tail) {
        q->tail->next = node;
    } else {
        q->head = node;
    }
    q->tail = node;

    pthread_cond_signal(&q->cond);
    pthread_mutex_unlock(&q->mutex);
}

char *vtt_queue_pop(vtt_queue_t *q) {
    pthread_mutex_lock(&q->mutex);

    while (!q->head && !q->shutdown) {
        pthread_cond_wait(&q->cond, &q->mutex);
    }

    if (q->shutdown && !q->head) {
        pthread_mutex_unlock(&q->mutex);
        return NULL;
    }

    queue_node_t *node = q->head;
    q->head = node->next;
    if (!q->head) {
        q->tail = NULL;
    }

    pthread_mutex_unlock(&q->mutex);

    char *data = node->data;
    free(node);
    return data;
}

void vtt_queue_shutdown(vtt_queue_t *q) {
    pthread_mutex_lock(&q->mutex);
    q->shutdown = true;
    pthread_cond_broadcast(&q->cond);
    pthread_mutex_unlock(&q->mutex);
}

void vtt_queue_destroy(vtt_queue_t *q) {
    pthread_mutex_destroy(&q->mutex);
    pthread_cond_destroy(&q->cond);

    // Free remaining items
    while (q->head) {
        queue_node_t *node = q->head;
        q->head = node->next;
        free(node->data);
        free(node);
    }
}
