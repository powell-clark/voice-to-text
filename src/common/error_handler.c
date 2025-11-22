#include "error_handler.h"
#include "logging.h"
#include <stdio.h>
#include <string.h>

#ifdef __linux__
#include <libnotify/notify.h>
#endif

const char *vtt_error_get_message(vtt_error_type_t error) {
    switch (error) {
        case VTT_ERROR_MICROPHONE_INIT:
            return "Failed to initialize microphone";
        case VTT_ERROR_MICROPHONE_ACCESS:
            return "Cannot access microphone - check permissions";
        case VTT_ERROR_TRANSCRIPTION_TIMEOUT:
            return "Transcription timed out (>60s)";
        case VTT_ERROR_TRANSCRIPTION_FAILED:
            return "Transcription failed - check model installation";
        case VTT_ERROR_MODEL_LOAD_FAILED:
            return "Failed to load transcription model";
        case VTT_ERROR_AUDIO_TOO_SHORT:
            return "Recording too short (<0.5s)";
        case VTT_ERROR_AUDIO_TOO_QUIET:
            return "Audio too quiet - speak louder";
        case VTT_ERROR_GENERIC:
        default:
            return "An error occurred";
    }
}

void vtt_error_notify(vtt_error_type_t error, const char *details) {
    const char *message = vtt_error_get_message(error);
    vtt_log("ERROR: %s%s%s", message, details ? " - " : "", details ? details : "");

#ifdef __linux__
    // Show desktop notification
    NotifyNotification *notification = notify_notification_new(
        "Voice to Text Error",
        details ? details : message,
        "dialog-error"
    );

    notify_notification_set_urgency(notification, NOTIFY_URGENCY_NORMAL);
    notify_notification_set_timeout(notification, 5000); // 5 seconds

    GError *error_obj = NULL;
    if (!notify_notification_show(notification, &error_obj)) {
        vtt_log("Failed to show error notification: %s", error_obj->message);
        g_error_free(error_obj);
    }

    g_object_unref(G_OBJECT(notification));
#endif
}
