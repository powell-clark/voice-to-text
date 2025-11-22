#ifndef VTT_ERROR_HANDLER_H
#define VTT_ERROR_HANDLER_H

// Error types
typedef enum {
    VTT_ERROR_MICROPHONE_INIT,
    VTT_ERROR_MICROPHONE_ACCESS,
    VTT_ERROR_TRANSCRIPTION_TIMEOUT,
    VTT_ERROR_TRANSCRIPTION_FAILED,
    VTT_ERROR_MODEL_LOAD_FAILED,
    VTT_ERROR_AUDIO_TOO_SHORT,
    VTT_ERROR_AUDIO_TOO_QUIET,
    VTT_ERROR_GENERIC
} vtt_error_type_t;

// Show error notification to user
void vtt_error_notify(vtt_error_type_t error, const char *details);

// Get human-readable error message
const char *vtt_error_get_message(vtt_error_type_t error);

#endif // VTT_ERROR_HANDLER_H
