#ifndef VTT_CRASH_HANDLER_H
#define VTT_CRASH_HANDLER_H

#include <stdbool.h>

// Initialize crash handler with signal handlers
void crash_handler_init(const char *crash_log_dir);

// Log a crash with backtrace
void crash_handler_log(const char *reason);

// Get crash log path
const char *crash_handler_get_log_path(void);

// Check if previous crash log exists
bool crash_handler_has_previous_crash(void);

// Clear previous crash log
void crash_handler_clear_previous_crash(void);

#endif // VTT_CRASH_HANDLER_H
