#include "recording_indicator.h"
#include "logging.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef __linux__
#include <libnotify/notify.h>
#endif

void vtt_recording_indicator_start(vtt_recording_indicator_t *indicator, int max_duration) {
    indicator->active = true;
    indicator->duration_seconds = 0;
    indicator->max_duration = max_duration;

#ifdef __linux__
    NotifyNotification *notification = notify_notification_new(
        "🎤 Recording...",
        "Speak now. Release key to transcribe.",
        "audio-input-microphone"
    );

    notify_notification_set_urgency(notification, NOTIFY_URGENCY_LOW);
    notify_notification_set_timeout(notification, NOTIFY_EXPIRES_NEVER);

    GError *error = NULL;
    if (!notify_notification_show(notification, &error)) {
        vtt_log("Failed to show recording notification: %s", error->message);
        g_error_free(error);
        g_object_unref(G_OBJECT(notification));
        return;
    }

    indicator->notification = notification;
#endif

    vtt_log("Recording indicator started");
}

void vtt_recording_indicator_update(vtt_recording_indicator_t *indicator, int duration_seconds) {
    if (!indicator->active) return;

    indicator->duration_seconds = duration_seconds;

#ifdef __linux__
    if (!indicator->notification) return;

    NotifyNotification *notification = (NotifyNotification *)indicator->notification;

    char summary[128];
    char body[256];

    snprintf(summary, sizeof(summary), "🎤 Recording... %ds", duration_seconds);

    int remaining = indicator->max_duration - duration_seconds;
    if (remaining <= 10 && remaining > 0) {
        snprintf(body, sizeof(body),
            "⚠️ %d seconds remaining. Release key soon!", remaining);
        notify_notification_set_urgency(notification, NOTIFY_URGENCY_NORMAL);
    } else {
        snprintf(body, sizeof(body), "Speak now. Release key to transcribe.");
        notify_notification_set_urgency(notification, NOTIFY_URGENCY_LOW);
    }

    notify_notification_update(notification, summary, body, "audio-input-microphone");

    GError *error = NULL;
    if (!notify_notification_show(notification, &error)) {
        vtt_log("Failed to update recording notification: %s", error->message);
        g_error_free(error);
    }
#endif
}

void vtt_recording_indicator_stop(vtt_recording_indicator_t *indicator) {
    if (!indicator->active) return;

    indicator->active = false;

#ifdef __linux__
    if (indicator->notification) {
        NotifyNotification *notification = (NotifyNotification *)indicator->notification;
        notify_notification_close(notification, NULL);
        g_object_unref(G_OBJECT(notification));
        indicator->notification = NULL;
    }
#endif

    vtt_log("Recording indicator stopped");
}

void vtt_recording_indicator_transcribing(vtt_recording_indicator_t *indicator) {
    vtt_recording_indicator_stop(indicator);

#ifdef __linux__
    NotifyNotification *notification = notify_notification_new(
        "⏳ Transcribing...",
        "Processing your speech. Please wait.",
        "system-run"
    );

    notify_notification_set_urgency(notification, NOTIFY_URGENCY_LOW);
    notify_notification_set_timeout(notification, 5000); // 5 seconds

    GError *error = NULL;
    if (!notify_notification_show(notification, &error)) {
        vtt_log("Failed to show transcribing notification: %s", error->message);
        g_error_free(error);
    }

    g_object_unref(G_OBJECT(notification));
#endif

    vtt_log("Showing transcribing indicator");
}
