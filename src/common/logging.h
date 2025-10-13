#ifndef VTT_LOGGING_H
#define VTT_LOGGING_H

#include <stdio.h>
#include <stdbool.h>

// Initialize logging system
void vtt_log_init(const char *log_dir);

// Log a message
void vtt_log(const char *format, ...);

// Enable/disable logging
void vtt_log_set_enabled(bool enabled);

// Check if logging is enabled
bool vtt_log_is_enabled(void);

// Get log file path
const char *vtt_log_get_path(void);

#endif // VTT_LOGGING_H
