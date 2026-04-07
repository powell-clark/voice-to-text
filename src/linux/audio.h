#ifndef VTT_AUDIO_LINUX_H
#define VTT_AUDIO_LINUX_H

#include <portaudio.h>
#include <stdbool.h>
#include <stdatomic.h>

typedef struct {
    char *name;
    int index;
    bool is_default;
} vtt_audio_device_t;

// Callback function type for buffer full notification
typedef void (*vtt_audio_buffer_full_callback_t)(void *user_data);

typedef struct {
    PaStream *stream;
    short *buffer;
    int buffer_size;
    atomic_int buffer_pos;        // Shared between audio callback and main thread
    atomic_bool recording;        // Shared between audio callback and main thread
    int sample_rate;
    int selected_device_index;    // -1 for default
    atomic_bool buffer_full;      // Shared between audio callback and main thread
    vtt_audio_buffer_full_callback_t buffer_full_callback;
    void *callback_user_data;
} vtt_audio_t;

// Initialize audio system
int vtt_audio_init(vtt_audio_t *audio);

// Open audio stream (call after init and after selecting device)
int vtt_audio_open_stream(vtt_audio_t *audio);

// Start recording
int vtt_audio_start_recording(vtt_audio_t *audio);

// Stop recording and save to WAV file
char *vtt_audio_stop_recording(vtt_audio_t *audio);

// Get list of available input devices
vtt_audio_device_t **vtt_audio_get_devices(int *count);

// Free device list
void vtt_audio_free_devices(vtt_audio_device_t **devices, int count);

// Set selected device by index (-1 for default)
void vtt_audio_set_device(vtt_audio_t *audio, int device_index);

// Set callback for buffer full notification
void vtt_audio_set_buffer_full_callback(vtt_audio_t *audio,
                                         vtt_audio_buffer_full_callback_t callback,
                                         void *user_data);

// Cleanup
void vtt_audio_cleanup(vtt_audio_t *audio);

#endif // VTT_AUDIO_LINUX_H
