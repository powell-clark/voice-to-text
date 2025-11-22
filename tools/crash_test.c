// Crash test utility for Voice to Text
// Tests crash handler by triggering various crashes

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>

#include "../src/common/logging.h"
#include "../src/common/crash_handler.h"

void test_segfault() {
    printf("Testing SIGSEGV (segmentation fault)...\n");
    int *ptr = NULL;
    *ptr = 42;  // BOOM
}

void test_abort() {
    printf("Testing SIGABRT (abort)...\n");
    abort();
}

void test_fpe() {
    printf("Testing SIGFPE (floating point exception)...\n");
    int x = 1;
    int y = 0;
    int z = x / y;  // Division by zero
    printf("%d\n", z);
}

void test_stack_overflow() {
    printf("Testing stack overflow...\n");
    test_stack_overflow();  // Infinite recursion
}

int main(int argc, char *argv[]) {
    // Initialize logging and crash handler
    char log_dir[512];
    snprintf(log_dir, sizeof(log_dir), "/tmp/vtt-crash-test");
    vtt_log_init(log_dir);
    crash_handler_init(log_dir);

    printf("===========================================\n");
    printf("VTT Crash Test Utility\n");
    printf("===========================================\n");
    printf("Crash logs will be written to: %s\n", crash_handler_get_log_path());
    printf("\n");

    if (argc < 2) {
        printf("Usage: %s <test>\n", argv[0]);
        printf("\nAvailable tests:\n");
        printf("  segfault       - Null pointer dereference\n");
        printf("  abort          - Call abort()\n");
        printf("  fpe            - Division by zero\n");
        printf("  stackoverflow  - Infinite recursion\n");
        printf("\nExample: %s segfault\n", argv[0]);
        return 1;
    }

    const char *test = argv[1];

    printf("Running test: %s\n", test);
    printf("Crash handler installed. Expect crash log...\n\n");

    if (strcmp(test, "segfault") == 0) {
        test_segfault();
    } else if (strcmp(test, "abort") == 0) {
        test_abort();
    } else if (strcmp(test, "fpe") == 0) {
        test_fpe();
    } else if (strcmp(test, "stackoverflow") == 0) {
        test_stack_overflow();
    } else {
        printf("Unknown test: %s\n", test);
        return 1;
    }

    printf("Test completed (should not reach here)\n");
    return 0;
}
