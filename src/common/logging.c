#include "logging.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include <time.h>
#include <sys/stat.h>
#include <unistd.h>
#include <dirent.h>
#include <pthread.h>

#define MAX_LOG_DAYS 7  // Keep logs for 7 days

static char log_dir_path[512] = {0};
static char log_file_path[512] = {0};
static int current_day = -1;
static bool logging_enabled = true;
static pthread_mutex_t log_mutex = PTHREAD_MUTEX_INITIALIZER;

// Get today's date as yyyymmdd int for comparison
static int get_today(void) {
    time_t now = time(NULL);
    struct tm *tm = localtime(&now);
    return (tm->tm_year + 1900) * 10000 + (tm->tm_mon + 1) * 100 + tm->tm_mday;
}

// Update log file path for today's date
static void update_log_path(void) {
    time_t now = time(NULL);
    struct tm *tm = localtime(&now);
    char date_str[16];
    strftime(date_str, sizeof(date_str), "%Y-%m-%d", tm);
    snprintf(log_file_path, sizeof(log_file_path), "%s/vtt-%s.log", log_dir_path, date_str);
    current_day = get_today();
}

// Purge log files older than MAX_LOG_DAYS
static void purge_old_logs(void) {
    DIR *dir = opendir(log_dir_path);
    if (!dir) return;

    time_t cutoff = time(NULL) - (MAX_LOG_DAYS * 86400);
    struct dirent *entry;
    int purged = 0;

    while ((entry = readdir(dir)) != NULL) {
        if (strncmp(entry->d_name, "vtt-", 4) != 0) continue;
        if (!strstr(entry->d_name, ".log")) continue;

        char path[512];
        snprintf(path, sizeof(path), "%.480s/%s", log_dir_path, entry->d_name);

        struct stat st;
        if (stat(path, &st) == 0 && st.st_mtime < cutoff) {
            if (unlink(path) == 0) purged++;
        }
    }
    closedir(dir);

    // Also remove legacy vtt.log and rotated files
    char legacy[512];
    snprintf(legacy, sizeof(legacy), "%s/vtt.log", log_dir_path);
    unlink(legacy);
    for (int i = 1; i <= 3; i++) {
        snprintf(legacy, sizeof(legacy), "%s/vtt.log.%d", log_dir_path, i);
        unlink(legacy);
    }
}

void vtt_log_init(const char *log_dir) {
    mkdir(log_dir, 0755);
    strncpy(log_dir_path, log_dir, sizeof(log_dir_path) - 1);
    log_dir_path[sizeof(log_dir_path) - 1] = '\0';

    update_log_path();
    purge_old_logs();
}

void vtt_log(const char *format, ...) {
    if (!logging_enabled || !log_dir_path[0]) return;

    pthread_mutex_lock(&log_mutex);

    // Roll to new day's file if needed
    int today = get_today();
    if (today != current_day) {
        update_log_path();
    }

    FILE *f = fopen(log_file_path, "a");
    if (!f) {
        pthread_mutex_unlock(&log_mutex);
        return;
    }

    time_t now = time(NULL);
    struct tm *tm_info = localtime(&now);
    char time_str[64];
    strftime(time_str, sizeof(time_str), "%Y-%m-%d %H:%M:%S", tm_info);

    fprintf(f, "[%s] ", time_str);
    va_list args;
    va_start(args, format);
    vfprintf(f, format, args);
    va_end(args);
    fprintf(f, "\n");
    fclose(f);

    // Console output
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

const char *vtt_log_get_dir(void) {
    return log_dir_path;
}
