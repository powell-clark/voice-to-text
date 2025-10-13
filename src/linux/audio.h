#ifndef VTT_AUDIO_LINUX_H
#define VTT_AUDIO_LINUX_H

#include <portaudio.h>
#include <stdbool.h>

typedef struct {
    PaStream *stream;
    short *buffer;
    int buffer_size;
    int buffer_pos;
    bool recording;
    int sample_rate;
} vtt_audio_t;

// Initialize audio system
int vtt_audio_init(vtt_audio_t *audio);

// Start recording
int vtt_audio_start_recording(vtt_audio_t *audio);

// Stop recording and save to WAV file
char *vtt_audio_stop_recording(vtt_audio_t *audio);

// Cleanup
void vtt_audio_cleanup(vtt_audio_t *audio);

#endif // VTT_AUDIO_LINUX_H
