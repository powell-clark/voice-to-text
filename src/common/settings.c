#include "settings.h"
#include "logging.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <errno.h>

#define MAX_LINE 1024

// Default settings
static const char *DEFAULT_MODEL = "small";
static const char *DEFAULT_LANGUAGE = "en";  // Default to English
static const char *DEFAULT_PREFIX = "[Voice] ";
static const char *DEFAULT_PROMPT = "Male British English speaker. Programming, business and technical terminology with frequent acronyms and spelled letters.";
static const int DEFAULT_DEVICE = -1;

void vtt_settings_init(vtt_settings_t *settings) {
    settings->selected_model = strdup(DEFAULT_MODEL);
    settings->selected_language = strdup(DEFAULT_LANGUAGE);
    settings->voice_prefix = strdup(DEFAULT_PREFIX);
    settings->initial_prompt = strdup(DEFAULT_PROMPT);
    settings->selected_device_index = DEFAULT_DEVICE;
    settings->hotkey_keycode = 0;  // 0 = use default Right Alt
}

static char* escape_string(const char *str) {
    if (!str) return strdup("");

    size_t len = strlen(str);
    char *escaped = malloc(len * 2 + 1);  // Worst case: every char needs escaping
    char *out = escaped;

    for (const char *in = str; *in; in++) {
        if (*in == '\\' || *in == '"' || *in == '\n') {
            *out++ = '\\';
            if (*in == '\n') {
                *out++ = 'n';
            } else {
                *out++ = *in;
            }
        } else {
            *out++ = *in;
        }
    }
    *out = '\0';
    return escaped;
}

static char* unescape_string(const char *str) {
    if (!str) return strdup("");

    size_t len = strlen(str);
    char *unescaped = malloc(len + 1);
    char *out = unescaped;

    for (const char *in = str; *in; in++) {
        if (*in == '\\' && *(in + 1)) {
            in++;
            if (*in == 'n') {
                *out++ = '\n';
            } else {
                *out++ = *in;
            }
        } else {
            *out++ = *in;
        }
    }
    *out = '\0';
    return unescaped;
}

int vtt_settings_load(vtt_settings_t *settings, const char *config_dir) {
    char path[512];
    snprintf(path, sizeof(path), "%s/settings.conf", config_dir);

    FILE *f = fopen(path, "r");
    if (!f) {
        vtt_log("No settings file found, using defaults");
        return -1;
    }

    char line[MAX_LINE];
    while (fgets(line, sizeof(line), f)) {
        // Trim newline
        size_t len = strlen(line);
        if (len > 0 && line[len-1] == '\n') {
            line[len-1] = '\0';
        }

        // Skip empty lines and comments
        if (line[0] == '\0' || line[0] == '#') continue;

        // Parse key=value
        char *equals = strchr(line, '=');
        if (!equals) continue;

        *equals = '\0';
        const char *key = line;
        const char *value = equals + 1;

        // Remove quotes from value if present
        if (value[0] == '"') {
            value++;
            char *end_quote = strrchr(value, '"');
            if (end_quote) *end_quote = '\0';
        }

        if (strcmp(key, "model") == 0) {
            free(settings->selected_model);
            settings->selected_model = strdup(value);
        } else if (strcmp(key, "language") == 0) {
            free(settings->selected_language);
            settings->selected_language = strdup(value);
        } else if (strcmp(key, "prefix") == 0) {
            free(settings->voice_prefix);
            settings->voice_prefix = unescape_string(value);
        } else if (strcmp(key, "prompt") == 0) {
            free(settings->initial_prompt);
            settings->initial_prompt = unescape_string(value);
        } else if (strcmp(key, "device") == 0) {
            settings->selected_device_index = atoi(value);
        } else if (strcmp(key, "hotkey") == 0) {
            settings->hotkey_keycode = atoi(value);
        }
    }

    fclose(f);
    vtt_log("Loaded settings from %s", path);
    return 0;
}

int vtt_settings_save(const vtt_settings_t *settings, const char *config_dir) {
    // Ensure config directory exists
    mkdir(config_dir, 0755);

    char path[512];
    snprintf(path, sizeof(path), "%s/settings.conf", config_dir);

    FILE *f = fopen(path, "w");
    if (!f) {
        vtt_log("Failed to save settings to %s: %s", path, strerror(errno));
        return -1;
    }

    fprintf(f, "# Voice to Text Linux Settings\n");
    fprintf(f, "# Auto-generated - edit at your own risk\n\n");

    fprintf(f, "model=%s\n", settings->selected_model ? settings->selected_model : DEFAULT_MODEL);
    fprintf(f, "language=%s\n", settings->selected_language ? settings->selected_language : DEFAULT_LANGUAGE);

    char *escaped_prefix = escape_string(settings->voice_prefix ? settings->voice_prefix : DEFAULT_PREFIX);
    fprintf(f, "prefix=\"%s\"\n", escaped_prefix);
    free(escaped_prefix);

    char *escaped_prompt = escape_string(settings->initial_prompt ? settings->initial_prompt : DEFAULT_PROMPT);
    fprintf(f, "prompt=\"%s\"\n", escaped_prompt);
    free(escaped_prompt);

    fprintf(f, "device=%d\n", settings->selected_device_index);

    if (settings->hotkey_keycode != 0) {
        fprintf(f, "hotkey=%d\n", settings->hotkey_keycode);
    }

    fclose(f);
    vtt_log("Saved settings to %s", path);
    return 0;
}

void vtt_settings_cleanup(vtt_settings_t *settings) {
    free(settings->selected_model);
    free(settings->selected_language);
    free(settings->voice_prefix);
    free(settings->initial_prompt);
    memset(settings, 0, sizeof(vtt_settings_t));
}
