#ifndef VTT_RECORDING_INDICATOR_H
#define VTT_RECORDING_INDICATOR_H

#include <stdbool.h>

// Recording indicator context (for showing recording progress)
typedef struct {
    bool active;
    int duration_seconds;
    int max_duration;
    void *notification;
} vtt_recording_indicator_t;

// Show recording started notification
void vtt_recording_indicator_start(vtt_recording_indicator_t *indicator, int max_duration);

// Update recording duration
void vtt_recording_indicator_update(vtt_recording_indicator_t *indicator, int duration_seconds);

// Show recording stopped
void vtt_recording_indicator_stop(vtt_recording_indicator_t *indicator);

// Show transcribing notification
void vtt_recording_indicator_transcribing(vtt_recording_indicator_t *indicator);

#endif // VTT_RECORDING_INDICATOR_H
