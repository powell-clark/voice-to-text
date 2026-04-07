#include "logging.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include <time.h>
#include <sys/stat.h>
#include <unistd.h>
#include <pthread.h>

static char log_file_path[512] = {0};
static bool logging_enabled = true;
static pthread_mutex_t log_mutex = PTHREAD_MUTEX_INITIALIZER;

void vtt_log_init(const char *log_dir) {
    // Create log directory
    mkdir(log_dir, 0755);

    // Set log file path
    snprintf(log_file_path, sizeof(log_file_path), "%s/vtt.log", log_dir);

    // Check if rotation needed (>10MB)
    struct stat st;
    if (stat(log_file_path, &st) == 0 && st.st_size > 10 * 1024 * 1024) {
        char log1[512], log2[512], log3[512];
        snprintf(log3, sizeof(log3), "%.507s.3", log_file_path);
        snprintf(log2, sizeof(log2), "%.507s.2", log_file_path);
        snprintf(log1, sizeof(log1), "%.507s.1", log_file_path);

        remove(log3);
        rename(log2, log3);
        rename(log1, log2);
        rename(log_file_path, log1);
    }
}

void vtt_log(const char *format, ...) {
    if (!logging_enabled || !log_file_path[0]) return;

    pthread_mutex_lock(&log_mutex);

    FILE *f = fopen(log_file_path, "a");
    if (!f) {
        pthread_mutex_unlock(&log_mutex);
        return;
    }

    // Get timestamp
    time_t now = time(NULL);
    struct tm *tm_info = localtime(&now);
    char time_str[64];
    strftime(time_str, sizeof(time_str), "%Y-%m-%d %H:%M:%S", tm_info);

    // Write to file
    fprintf(f, "[%s] ", time_str);
    va_list args;
    va_start(args, format);
    vfprintf(f, format, args);
    va_end(args);
    fprintf(f, "\n");
    fclose(f);

    // Write to console
    printf("[%s] ", time_str);
    va_list args2;
    va_start(args2, format);
    vprintf(format, args2);
    va_end(args2);
    printf("\n");

    pthread_mutex_unlock(&log_mutex);
}

void vtt_log_set_enabled(bool enabled) {
    logging_enabled = enabled;
}

bool vtt_log_is_enabled(void) {
    return logging_enabled;
}

const char *vtt_log_get_path(void) {
    return log_file_path;
}
