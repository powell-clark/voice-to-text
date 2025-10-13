#include "transcribe.h"
#include "../common/logging.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_OUTPUT 8192

char *vtt_transcribe_audio(const char *audio_path) {
    if (!audio_path) {
        return NULL;
    }

    // Build command - use shared transcribe.py from src/common/
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "python3.12 src/common/transcribe.py '%s' large-v3 2>&1", audio_path);

    vtt_log("Transcribing: %s", audio_path);

    FILE *fp = popen(cmd, "r");
    if (!fp) {
        vtt_log("Failed to run transcription");
        return NULL;
    }

    // Read output
    char buffer[MAX_OUTPUT] = {0};
    size_t total_read = 0;

    while (total_read < MAX_OUTPUT - 1) {
        size_t read = fread(buffer + total_read, 1, MAX_OUTPUT - total_read - 1, fp);
        if (read == 0) break;
        total_read += read;
    }
    buffer[total_read] = '\0';

    int status = pclose(fp);

    if (status != 0) {
        vtt_log("Transcription failed with status %d", status);
        vtt_log("Output: %s", buffer);
        return NULL;
    }

    // Trim whitespace
    char *start = buffer;
    while (*start == ' ' || *start == '\n' || *start == '\r' || *start == '\t') {
        start++;
    }

    if (*start == '\0') {
        vtt_log("Empty transcription result");
        return NULL;
    }

    char *end = start + strlen(start) - 1;
    while (end > start && (*end == ' ' || *end == '\n' || *end == '\r' || *end == '\t')) {
        *end = '\0';
        end--;
    }

    vtt_log("Transcribed: %s", start);

    return strdup(start);
}
