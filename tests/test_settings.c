// Unit tests for settings.c
#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <unistd.h>
#include "../src/common/settings.h"

void test_load_default_settings() {
    Settings settings;
    memset(&settings, 0, sizeof(Settings));

    load_settings(&settings);

    // Verify defaults
    assert(strcmp(settings.model, "CT2 small.en") == 0);
    assert(settings.language_mode == LANG_MODE_ENGLISH);

    printf("✅ test_load_default_settings passed\n");
}

void test_save_and_load_settings() {
    Settings settings_write, settings_read;
    memset(&settings_write, 0, sizeof(Settings));
    memset(&settings_read, 0, sizeof(Settings));

    // Set custom values
    strncpy(settings_write.model, "W large-v3", sizeof(settings_write.model) - 1);
    settings_write.language_mode = LANG_MODE_MULTILINGUAL;

    // Save
    save_settings(&settings_write);

    // Load
    load_settings(&settings_read);

    // Verify persistence
    assert(strcmp(settings_read.model, "W large-v3") == 0);
    assert(settings_read.language_mode == LANG_MODE_MULTILINGUAL);

    printf("✅ test_save_and_load_settings passed\n");
}

void test_model_backend_detection() {
    assert(is_ct2_model("CT2 small.en"));
    assert(is_ct2_model("CT2 large-v3"));
    assert(!is_ct2_model("W tiny.en"));
    assert(!is_ct2_model("W base"));

    printf("✅ test_model_backend_detection passed\n");
}

int main() {
    printf("Running settings tests...\n");

    test_load_default_settings();
    test_model_backend_detection();
    test_save_and_load_settings();

    printf("\n✅ All tests passed!\n");
    return 0;
}
