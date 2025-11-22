#define _GNU_SOURCE
#include "crash_handler.h"
#include "logging.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include <time.h>
#include <unistd.h>
#include <sys/stat.h>

#ifdef __linux__
#include <execinfo.h>
#endif

static char crash_log_path[512] = {0};
static char crash_log_dir[512] = {0};

static void write_crash_log(int sig, const char *sig_name) {
    FILE *f = fopen(crash_log_path, "w");
    if (!f) return;

    // Write crash header
    time_t now = time(NULL);
    struct tm *tm_info = localtime(&now);
    char time_str[64];
    strftime(time_str, sizeof(time_str), "%Y-%m-%d %H:%M:%S", tm_info);

    fprintf(f, "=== VTT CRASH LOG ===\n");
    fprintf(f, "Time: %s\n", time_str);
    fprintf(f, "Signal: %d (%s)\n", sig, sig_name);
    fprintf(f, "PID: %d\n", getpid());
    fprintf(f, "\n");

#ifdef __linux__
    // Get backtrace
    fprintf(f, "Backtrace:\n");
    void *buffer[100];
    int count = backtrace(buffer, 100);
    char **strings = backtrace_symbols(buffer, count);

    if (strings) {
        for (int i = 0; i < count; i++) {
            fprintf(f, "  %s\n", strings[i]);
        }
        free(strings);
    } else {
        fprintf(f, "  (backtrace_symbols failed)\n");
    }
#else
    fprintf(f, "Backtrace: (not available on this platform)\n");
#endif

    fprintf(f, "\n=== END CRASH LOG ===\n");
    fclose(f);

    // Also log to main log
    vtt_log("CRASH: Signal %d (%s) - crash log written to %s", sig, sig_name, crash_log_path);
}

static void signal_handler(int sig) {
    const char *sig_name = "UNKNOWN";

    switch (sig) {
        case SIGSEGV: sig_name = "SIGSEGV (Segmentation Fault)"; break;
        case SIGABRT: sig_name = "SIGABRT (Abort)"; break;
        case SIGFPE:  sig_name = "SIGFPE (Floating Point Exception)"; break;
        case SIGILL:  sig_name = "SIGILL (Illegal Instruction)"; break;
        case SIGBUS:  sig_name = "SIGBUS (Bus Error)"; break;
    }

    write_crash_log(sig, sig_name);

    // Re-raise signal with default handler to actually crash
    signal(sig, SIG_DFL);
    raise(sig);
}

void crash_handler_init(const char *log_dir) {
    // Store crash log directory
    strncpy(crash_log_dir, log_dir, sizeof(crash_log_dir) - 1);

    // Create crash log directory
    mkdir(log_dir, 0755);

    // Set crash log path
    snprintf(crash_log_path, sizeof(crash_log_path), "%s/crash.log", log_dir);

    // Install signal handlers
    signal(SIGSEGV, signal_handler);
    signal(SIGABRT, signal_handler);
    signal(SIGFPE, signal_handler);
    signal(SIGILL, signal_handler);
    signal(SIGBUS, signal_handler);

    vtt_log("Crash handler initialized: %s", crash_log_path);
}

void crash_handler_log(const char *reason) {
    FILE *f = fopen(crash_log_path, "w");
    if (!f) return;

    time_t now = time(NULL);
    struct tm *tm_info = localtime(&now);
    char time_str[64];
    strftime(time_str, sizeof(time_str), "%Y-%m-%d %H:%M:%S", tm_info);

    fprintf(f, "=== VTT CRASH LOG ===\n");
    fprintf(f, "Time: %s\n", time_str);
    fprintf(f, "Reason: %s\n", reason);
    fprintf(f, "PID: %d\n", getpid());
    fprintf(f, "\n=== END CRASH LOG ===\n");
    fclose(f);

    vtt_log("Manual crash log: %s", reason);
}

const char *crash_handler_get_log_path(void) {
    return crash_log_path;
}

bool crash_handler_has_previous_crash(void) {
    struct stat st;
    return stat(crash_log_path, &st) == 0;
}

void crash_handler_clear_previous_crash(void) {
    remove(crash_log_path);
}
