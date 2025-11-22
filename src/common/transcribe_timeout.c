#define _POSIX_C_SOURCE 200809L
#include "transcribe_timeout.h"
#include "logging.h"
#include <stdlib.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <signal.h>
#include <errno.h>
#include <time.h>
#include <string.h>
#include <fcntl.h>

typedef struct {
    pid_t pid;
    int pipe_fd;
    time_t start_time;
    int timeout_seconds;
} timeout_process_t;

FILE *popen_with_timeout(const char *command, int timeout_seconds, int *timed_out, pid_t *pid_out) {
    *timed_out = 0;

    int pipefd[2];
    if (pipe(pipefd) == -1) {
        vtt_log("pipe() failed: %s", strerror(errno));
        return NULL;
    }

    pid_t pid = fork();
    if (pid == -1) {
        vtt_log("fork() failed: %s", strerror(errno));
        close(pipefd[0]);
        close(pipefd[1]);
        return NULL;
    }

    if (pid == 0) {
        // Child process
        close(pipefd[0]); // Close read end

        // Redirect stdout to pipe
        dup2(pipefd[1], STDOUT_FILENO);
        dup2(pipefd[1], STDERR_FILENO);
        close(pipefd[1]);

        // Execute command
        execl("/bin/sh", "sh", "-c", command, NULL);
        _exit(127); // execl failed
    }

    // Parent process
    close(pipefd[1]); // Close write end

    *pid_out = pid;

    // Convert fd to FILE*
    FILE *fp = fdopen(pipefd[0], "r");
    if (!fp) {
        vtt_log("fdopen() failed: %s", strerror(errno));
        close(pipefd[0]);
        kill(pid, SIGKILL);
        waitpid(pid, NULL, 0);
        return NULL;
    }

    return fp;
}

int pclose_with_timeout(FILE *fp, pid_t pid) {
    if (!fp) return -1;

    fclose(fp);

    // Wait for child with timeout
    int status;
    time_t start = time(NULL);
    int wait_timeout = 5; // 5 seconds to cleanup

    while (1) {
        pid_t result = waitpid(pid, &status, WNOHANG);

        if (result == pid) {
            // Child exited
            if (WIFEXITED(status)) {
                return WEXITSTATUS(status);
            }
            return -1;
        } else if (result == -1) {
            if (errno == ECHILD) {
                // Child already reaped
                return 0;
            }
            return -1;
        }

        // Still running, check timeout
        if (time(NULL) - start > wait_timeout) {
            vtt_log("pclose timeout, killing process %d", pid);
            kill(pid, SIGKILL);
            waitpid(pid, NULL, 0);
            return -1;
        }

        usleep(100000); // 100ms
    }
}
