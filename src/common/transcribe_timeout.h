#ifndef VTT_TRANSCRIBE_TIMEOUT_H
#define VTT_TRANSCRIBE_TIMEOUT_H

#include <stdio.h>

// Run command with timeout (in seconds)
// Returns: FILE* on success, NULL on timeout/error
// Sets *timed_out to 1 if timeout occurred, 0 otherwise
FILE *popen_with_timeout(const char *command, int timeout_seconds, int *timed_out, pid_t *pid_out);

// Close popen_with_timeout handle
int pclose_with_timeout(FILE *fp, pid_t pid);

#endif // VTT_TRANSCRIBE_TIMEOUT_H
