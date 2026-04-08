#include "transcribe.h"
#include "../common/logging.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <fcntl.h>

#define MAX_OUTPUT 8192

// Detect if model is whisper.cpp (W models) or CTranslate2
static int is_whisper_cpp_model(const char *model) {
    if (!model) return 0;
    if (strncmp(model, "CT2 ", 4) == 0) return 0;
    return 1;
}

// Execute a command with arguments, capture stdout, no shell involved.
// Returns malloc'd output string or NULL on failure. Sets *exit_status.
static char *exec_capture(char *const argv[], int *exit_status) {
    int pipefd[2];
    if (pipe(pipefd) < 0) {
        vtt_log("exec_capture: pipe() failed");
        return NULL;
    }

    pid_t pid = fork();
    if (pid < 0) {
        vtt_log("exec_capture: fork() failed");
        close(pipefd[0]);
        close(pipefd[1]);
        return NULL;
    }

    if (pid == 0) {
        // Child: redirect stdout to pipe, suppress stderr
        close(pipefd[0]);
        dup2(pipefd[1], STDOUT_FILENO);
        close(pipefd[1]);

        int devnull = open("/dev/null", O_WRONLY);
        if (devnull >= 0) {
            dup2(devnull, STDERR_FILENO);
            close(devnull);
        }

        execvp(argv[0], argv);
        _exit(127);
    }

    // Parent: read from pipe
    close(pipefd[1]);

    char *output = malloc(MAX_OUTPUT);
    if (!output) {
        close(pipefd[0]);
        waitpid(pid, NULL, 0);
        return NULL;
    }

    size_t total = 0;
    while (total < MAX_OUTPUT - 1) {
        ssize_t n = read(pipefd[0], output + total, MAX_OUTPUT - total - 1);
        if (n <= 0) break;
        total += n;
    }
    output[total] = '\0';
    close(pipefd[0]);

    int status;
    waitpid(pid, &status, 0);
    if (exit_status) {
        *exit_status = WIFEXITED(status) ? WEXITSTATUS(status) : -1;
    }

    return output;
}

char *vtt_transcribe_audio(const char *audio_path, const char *model, const char *language, const char *initial_prompt) {
    if (!audio_path) {
        return NULL;
    }

    // Use provided model or default to CT2 small
    const char *model_to_use = model ? model : "CT2 small";

    // Use provided language or default to "en" (English)
    const char *language_to_use = language ? language : "en";

    // Use provided prompt or NULL
    const char *prompt_to_use = (initial_prompt && initial_prompt[0]) ? initial_prompt : NULL;

    // Auto-append .en suffix for English when .en model exists
    char adjusted_model[128];
    strncpy(adjusted_model, model_to_use, sizeof(adjusted_model) - 1);
    adjusted_model[sizeof(adjusted_model) - 1] = '\0';

    if (strcmp(language_to_use, "en") == 0 && strstr(adjusted_model, ".en") == NULL) {
        int has_en_version = 0;
        if (strstr(adjusted_model, "tiny") || strstr(adjusted_model, "base") ||
            strstr(adjusted_model, "small") || strstr(adjusted_model, "medium")) {
            has_en_version = 1;
        }

        if (has_en_version) {
            size_t len = strlen(adjusted_model);
            if (len + 3 < sizeof(adjusted_model)) {
                strncat(adjusted_model, ".en", sizeof(adjusted_model) - len - 1);
                model_to_use = adjusted_model;
                vtt_log("Auto-selected .en model: %s (English mode)", model_to_use);
            }
        }
    }

    char *output = NULL;
    int exit_status = -1;

    if (is_whisper_cpp_model(model_to_use)) {
        // ═══════════════════════════════════════════════════════════════
        // WHISPER.CPP BACKEND
        // ═══════════════════════════════════════════════════════════════

        const char *whisper_cli_paths[] = {
            "/usr/bin/whisper-cli",
            "/usr/local/bin/whisper-cli",
            "./third_party/whisper.cpp/build/bin/whisper-cli",
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
            vtt_log("ERROR: whisper-cli not found.");
            return NULL;
        }

        vtt_log("Using whisper-cli: %s", whisper_cli);

        // Extract base model name (strip "W " prefix)
        const char *base_model = model_to_use;
        if (strncmp(model_to_use, "W ", 2) == 0) {
            base_model = model_to_use + 2;
        }

        int is_english_only = (strstr(base_model, ".en") != NULL);
        const char *model_file_name = base_model;
        const char *extension = is_english_only ? ".en.bin" : ".bin";

        char clean_model_name[64];
        if (is_english_only) {
            size_t len = strlen(base_model) - 3;
            if (len < sizeof(clean_model_name)) {
                memcpy(clean_model_name, base_model, len);
                clean_model_name[len] = '\0';
                model_file_name = clean_model_name;
            }
        }

        if (strcmp(model_file_name, "large") == 0) {
            model_file_name = "large-v3";
        }

        char model_file[512];
        const char *home = getenv("HOME");
        if (!home) home = "/tmp";
        snprintf(model_file, sizeof(model_file), "%s/.cache/whisper/ggml-%s%s",
                 home, model_file_name, extension);

        if (access(model_file, R_OK) != 0) {
            vtt_log("ERROR: Model file not found: %s", model_file);
            return NULL;
        }

        // Build argv — no shell, no escaping needed
        if (prompt_to_use) {
            char *argv[] = {
                (char *)whisper_cli,
                "-m", model_file,
                "-f", (char *)audio_path,
                "--no-timestamps",
                "--language", (char *)language_to_use,
                "--threads", "4",
                "--prompt", (char *)prompt_to_use,
                NULL
            };
            output = exec_capture(argv, &exit_status);
        } else {
            char *argv[] = {
                (char *)whisper_cli,
                "-m", model_file,
                "-f", (char *)audio_path,
                "--no-timestamps",
                "--language", (char *)language_to_use,
                "--threads", "4",
                NULL
            };
            output = exec_capture(argv, &exit_status);
        }

    } else {
        // ═══════════════════════════════════════════════════════════════
        // CTRANSLATE2 BACKEND
        // ═══════════════════════════════════════════════════════════════

        const char *script_paths[] = {
            "/usr/share/voice-to-text/transcribe.py",
            "./src/common/transcribe.py",
            "src/common/transcribe.py",
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
            return NULL;
        }

        vtt_log("Using transcribe.py: %s", script_path);

        const char *base_model = model_to_use;
        if (strncmp(model_to_use, "CT2 ", 4) == 0) {
            base_model = model_to_use + 4;
        }

        char ct2_model[64];
        strncpy(ct2_model, base_model, sizeof(ct2_model) - 1);
        ct2_model[sizeof(ct2_model) - 1] = '\0';

        if (strcmp(base_model, "large") == 0) {
            strncpy(ct2_model, "large-v3", sizeof(ct2_model));
        }
        if (strcmp(base_model, "distil-large-v3") == 0) {
            strncpy(ct2_model, "distil-whisper/distil-large-v3", sizeof(ct2_model));
            ct2_model[sizeof(ct2_model) - 1] = '\0';
        }
        if (strcmp(base_model, "distil-large-v3.5") == 0) {
            strncpy(ct2_model, "distil-whisper/distil-large-v3.5-ct2", sizeof(ct2_model));
            ct2_model[sizeof(ct2_model) - 1] = '\0';
        }

        // Build argv — no shell, no escaping needed
        if (prompt_to_use) {
            char *argv[] = {
                "python3",
                (char *)script_path,
                (char *)audio_path,
                ct2_model,
                (char *)language_to_use,
                (char *)prompt_to_use,
                NULL
            };
            output = exec_capture(argv, &exit_status);
        } else {
            char *argv[] = {
                "python3",
                (char *)script_path,
                (char *)audio_path,
                ct2_model,
                (char *)language_to_use,
                NULL
            };
            output = exec_capture(argv, &exit_status);
        }
    }

    if (!output) {
        vtt_log("Failed to run transcription");
        return NULL;
    }

    if (exit_status != 0) {
        vtt_log("Transcription failed with status %d", exit_status);
        free(output);
        return NULL;
    }

    // Trim whitespace
    char *start = output;
    while (*start == ' ' || *start == '\n' || *start == '\r' || *start == '\t') {
        start++;
    }

    if (*start == '\0') {
        vtt_log("Empty transcription result");
        free(output);
        return NULL;
    }

    char *end = start + strlen(start) - 1;
    while (end > start && (*end == ' ' || *end == '\n' || *end == '\r' || *end == '\t')) {
        *end = '\0';
        end--;
    }

    char *result = strdup(start);
    free(output);
    return result;
}
