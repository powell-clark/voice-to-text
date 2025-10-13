#ifndef VTT_TYPING_LINUX_H
#define VTT_TYPING_LINUX_H

#include <stdbool.h>

typedef struct {
    void *display;
    int delay_ms;
} vtt_typing_t;

// Initialize typing system
int vtt_typing_init(vtt_typing_t *typing);

// Type text string
void vtt_typing_type_text(vtt_typing_t *typing, const char *text);

// Cleanup
void vtt_typing_cleanup(vtt_typing_t *typing);

#endif // VTT_TYPING_LINUX_H
