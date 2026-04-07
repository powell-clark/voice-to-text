#include "transcribe.h"
#include "../common/logging.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define MAX_OUTPUT 8192

// Detect if model is whisper.cpp (W models) or CTranslate2
// W models: tiny, base, small, medium, large (not prefixed with CT2)
// CT2 models: start with "CT2 " prefix
static int is_whisper_cpp_model(const char *model) {
    if (!model) return 0;
    // If it starts with "CT2 ", it's a CTranslate2 model
    if (strncmp(model, "CT2 ", 4) == 0) return 0;
    // Otherwise it's a whisper.cpp model
    return 1;
}

// Shell-escape a string: replace single quotes with '\'' for safe embedding in '...'
static char *shell_escape(const char *str) {
    if (!str) return strdup("");
    size_t len = strlen(str);
    // Worst case: every char is a single quote → 4x expansion + quotes + null
    char *escaped = malloc(len * 4 + 3);
    if (!escaped) return strdup("");
    char *out = escaped;
    for (const char *p = str; *p; p++) {
        if (*p == '\'') {
            // End quote, escaped quote, start quote: '\''
            *out++ = '\'';
            *out++ = '\\';
            *out++ = '\'';
            *out++ = '\'';
        } else {
            *out++ = *p;
        }
    }
    *out = '\0';
    return escaped;
}

char *vtt_transcribe_audio(const char *audio_path, const char *model, const char *language, const char *initial_prompt) {
    if (!audio_path) {
        return NULL;
    }

    // Use provided model or default to CT2 small
    const char *model_to_use = model ? model : "CT2 small";

    // Use provided language or default to "en" (English)
    const char *language_to_use = language ? language : "en";

    // Use provided prompt or empty string
    const char *prompt_to_use = (initial_prompt && initial_prompt[0]) ? initial_prompt : "";

    // Auto-append .en suffix for English when .en model exists
    // Menu shows "W tiny", but backend uses "W tiny.en" for English
    char adjusted_model[128];
    strncpy(adjusted_model, model_to_use, sizeof(adjusted_model) - 1);
    adjusted_model[sizeof(adjusted_model) - 1] = '\0';

    if (strcmp(language_to_use, "en") == 0 && strstr(adjusted_model, ".en") == NULL) {
        // Check if this model has an .en version (tiny, base, small, medium do, large doesn't)
        int has_en_version = 0;
        if (strstr(adjusted_model, "tiny") || strstr(adjusted_model, "base") ||
            strstr(adjusted_model, "small") || strstr(adjusted_model, "medium")) {
            has_en_version = 1;
        }

        if (has_en_version) {
            // Append .en to model name
            size_t len = strlen(adjusted_model);
            if (len + 3 < sizeof(adjusted_model)) {
                strncat(adjusted_model, ".en", sizeof(adjusted_model) - len - 1);
                model_to_use = adjusted_model;
                vtt_log("Auto-selected .en model: %s (English mode)", model_to_use);
            }
        }
    }

    char cmd[1024];

    // Detect backend based on model name
    if (is_whisper_cpp_model(model_to_use)) {
        // ═══════════════════════════════════════════════════════════════
        // WHISPER.CPP BACKEND (W models: tiny, base, small, medium, large)
        // ═══════════════════════════════════════════════════════════════

        // Try multiple locations for whisper-cli (PPA install, manual install, dev mode)
        const char *whisper_cli_paths[] = {
            "/usr/bin/whisper-cli",                          // PPA/system install
            "/usr/local/bin/whisper-cli",                    // Manual install
            "./third_party/whisper.cpp/build/bin/whisper-cli",  // Relative (dev)
            NULL
        };

        const char *whisper_cli = NULL;
        for (int i = 0; whisper_cli_paths[i] != NULL; i++) {
            if (access(whisper_cli_paths[i], X_OK) == 0) {
                whisper_cli = whisper_cli_paths[i];
                break;
            }
        }

        if (!whisper_cli) {
            vtt_log("ERROR: whisper-cli not found. Try installing whisper.cpp or use CT2 models.");
            vtt_log("Searched: /usr/bin, /usr/local/bin, ./third_party/whisper.cpp/build/bin");
            return NULL;
        }

        vtt_log("Using whisper-cli: %s", whisper_cli);

        // Extract base model name (strip "W " prefix if present)
        const char *base_model = model_to_use;
        if (strncmp(model_to_use, "W ", 2) == 0) {
            base_model = model_to_use + 2; // Skip "W "
        }

        // Determine if model is English-only (.en suffix) or multilingual
        int is_english_only = (strstr(base_model, ".en") != NULL);

        // Map model names and determine file extension
        const char *model_file_name = base_model;
        const char *extension = is_english_only ? ".en.bin" : ".bin";

        // Remove .en suffix from model name for file lookup
        char clean_model_name[64];
        if (is_english_only) {
            // Copy model name without .en suffix
            size_t len = strlen(base_model) - 3; // Remove ".en"
            if (len < sizeof(clean_model_name)) {
                strncpy(clean_model_name, base_model, len);
                clean_model_name[len] = '\0';
                model_file_name = clean_model_name;
            }
        }

        // Special case: large → large-v3
        if (strcmp(model_file_name, "large") == 0) {
            model_file_name = "large-v3";
        }

        // Model file path: ~/.cache/whisper/ggml-{model}{extension}
        char model_file[512];
        const char *home = getenv("HOME");
        snprintf(model_file, sizeof(model_file), "%s/.cache/whisper/ggml-%s%s",
                 home, model_file_name, extension);

        if (access(model_file, R_OK) != 0) {
            vtt_log("ERROR: Model file not found: %s", model_file);
            vtt_log("Download with: bash third_party/whisper.cpp/models/download-ggml-model.sh %s", model_file_name);
            return NULL;
        }

        // Run whisper-cli with user-selected language and prompt
        char *escaped_prompt = shell_escape(prompt_to_use);
        if (escaped_prompt[0]) {
            snprintf(cmd, sizeof(cmd),
                     "%s -m '%s' -f '%s' --no-timestamps --language %s --threads 4 --prompt '%s' 2>/dev/null",
                     whisper_cli, model_file, audio_path, language_to_use, escaped_prompt);
        } else {
            snprintf(cmd, sizeof(cmd),
                     "%s -m '%s' -f '%s' --no-timestamps --language %s --threads 4 2>/dev/null",
                     whisper_cli, model_file, audio_path, language_to_use);
        }
        free(escaped_prompt);

    } else {
        // ═══════════════════════════════════════════════════════════════
        // CTRANSLATE2 BACKEND (CT2 models: tiny, base, small, medium, large-v3)
        // ═══════════════════════════════════════════════════════════════

        // Try multiple locations for transcribe.py (PPA install, dev mode)
        const char *script_paths[] = {
            "/usr/share/voice-to-text/transcribe.py",       // PPA/system install
            "./src/common/transcribe.py",                   // Relative (dev)
            "src/common/transcribe.py",                     // Relative alt
            NULL
        };

        const char *script_path = NULL;
        for (int i = 0; script_paths[i] != NULL; i++) {
            if (access(script_paths[i], R_OK) == 0) {
                script_path = script_paths[i];
                break;
            }
        }

        if (!script_path) {
            vtt_log("ERROR: transcribe.py not found.");
            vtt_log("Searched: /usr/share/voice-to-text, ./src/common, src/common");
            return NULL;
        }

        vtt_log("Using transcribe.py: %s", script_path);
        const char *python_path = "python3";

        // Extract base model name (strip "CT2 " prefix if present)
        const char *base_model = model_to_use;
        if (strncmp(model_to_use, "CT2 ", 4) == 0) {
            base_model = model_to_use + 4; // Skip "CT2 " prefix
        }

        // Map model names for CT2 (large → large-v3)
        // Preserve .en suffix if present (e.g., "small.en" → "small.en")
        char ct2_model[64];
        strncpy(ct2_model, base_model, sizeof(ct2_model) - 1);
        ct2_model[sizeof(ct2_model) - 1] = '\0';

        // Replace "large" with "large-v3" (unless it's "large.en")
        if (strcmp(base_model, "large") == 0) {
            strncpy(ct2_model, "large-v3", sizeof(ct2_model));
        }

        // Run Python faster-whisper script with model, language, and prompt
        char *escaped_prompt = shell_escape(prompt_to_use);
        if (escaped_prompt[0]) {
            snprintf(cmd, sizeof(cmd), "%s '%s' '%s' %s %s '%s' 2>/dev/null",
                     python_path, script_path, audio_path, ct2_model, language_to_use, escaped_prompt);
        } else {
            snprintf(cmd, sizeof(cmd), "%s '%s' '%s' %s %s 2>/dev/null",
                     python_path, script_path, audio_path, ct2_model, language_to_use);
        }
        free(escaped_prompt);
    }

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

    return strdup(start);
}
