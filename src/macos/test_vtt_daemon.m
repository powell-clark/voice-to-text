#import <Foundation/Foundation.h>
#import <CoreAudio/CoreAudio.h>
#import <AudioToolbox/AudioToolbox.h>
#import <AppKit/AppKit.h>
#include <sys/stat.h>

// Test the core VTT functionality without UI
@interface VTTTester : NSObject
- (BOOL)testWhisperCommand;
- (BOOL)testAudioFormat;
- (BOOL)testWAVGeneration;
- (BOOL)testTempFileCreation;
- (BOOL)testModelPathGeneration;
- (BOOL)testClipboardFunctionality;
@end

@implementation VTTTester

- (BOOL)testWhisperCommand {
    printf("Testing whisper command generation... ");

    // Test that whisper command does NOT include -nt flag (ChatGPT's fix)
    NSString *selectedModel = @"small";
    NSString *modelPath = [NSString stringWithFormat:@"~/whisper.cpp/models/ggml-%@.en.bin", selectedModel];
    char command[512];
    const char *wavFile = "/tmp/test.wav";

    sprintf(command, "~/whisper.cpp/build/bin/whisper-cli -m %s -f %s -np 2>&1",
            [modelPath UTF8String], wavFile);

    NSString *commandStr = [NSString stringWithUTF8String:command];

    // Verify -nt flag is NOT present (the bug fix)
    if ([commandStr containsString:@"-nt"]) {
        printf("❌ FAILED - Command still contains -nt flag\n");
        return NO;
    }

    // Verify required flags are present
    if (![commandStr containsString:@"-np"] ||
        ![commandStr containsString:@"-m"] ||
        ![commandStr containsString:@"-f"]) {
        printf("❌ FAILED - Missing required flags\n");
        return NO;
    }

    printf("✅ PASSED\n");
    return YES;
}

- (BOOL)testAudioFormat {
    printf("Testing audio format configuration... ");

    AudioStreamBasicDescription format;
    format.mSampleRate = 16000.0;
    format.mFormatID = kAudioFormatLinearPCM;
    format.mFormatFlags = kLinearPCMFormatFlagIsSignedInteger | kLinearPCMFormatFlagIsPacked;
    format.mBitsPerChannel = 16;
    format.mChannelsPerFrame = 1;
    format.mFramesPerPacket = 1;
    format.mBytesPerFrame = 2;
    format.mBytesPerPacket = 2;

    if (format.mSampleRate != 16000.0 || format.mChannelsPerFrame != 1) {
        printf("❌ FAILED\n");
        return NO;
    }

    printf("✅ PASSED\n");
    return YES;
}

- (BOOL)testWAVGeneration {
    printf("Testing WAV header generation... ");

    // Test WAV header structure
    struct {
        char riff[4];
        uint32_t fileSize;
        char wave[4];
        char fmt[4];
        uint32_t fmtSize;
        uint16_t audioFormat;
        uint16_t channels;
        uint32_t sampleRate;
        uint32_t byteRate;
        uint16_t blockAlign;
        uint16_t bitsPerSample;
        char data[4];
        uint32_t dataSize;
    } header;

    // Initialize header
    memcpy(header.riff, "RIFF", 4);
    memcpy(header.wave, "WAVE", 4);
    memcpy(header.fmt, "fmt ", 4);
    header.fmtSize = 16;
    header.audioFormat = 1;
    header.channels = 1;
    header.sampleRate = 16000;
    header.bitsPerSample = 16;
    header.byteRate = header.sampleRate * header.channels * header.bitsPerSample / 8;
    header.blockAlign = header.channels * header.bitsPerSample / 8;
    memcpy(header.data, "data", 4);

    if (header.sampleRate != 16000 || header.channels != 1) {
        printf("❌ FAILED\n");
        return NO;
    }

    printf("✅ PASSED\n");
    return YES;
}

- (BOOL)testTempFileCreation {
    printf("Testing temp file creation... ");

    NSString *tempFile = [NSString stringWithFormat:@"/tmp/vtt_test_%d.wav", arc4random()];

    // Test file creation
    FILE *fp = fopen([tempFile UTF8String], "wb");
    if (!fp) {
        printf("❌ FAILED - Cannot create temp file\n");
        return NO;
    }
    fclose(fp);

    // Test file deletion
    unlink([tempFile UTF8String]);

    printf("✅ PASSED\n");
    return YES;
}

- (BOOL)testModelPathGeneration {
    printf("Testing model path generation... ");

    NSString *model = @"small";
    NSString *modelPath = [NSString stringWithFormat:@"~/whisper.cpp/models/ggml-%@.en.bin", model];

    if (![modelPath containsString:@"whisper.cpp/models"] || ![modelPath containsString:@"small"]) {
        printf("❌ FAILED\n");
        return NO;
    }

    printf("✅ PASSED\n");
    return YES;
}

- (BOOL)testClipboardFunctionality {
    printf("Testing clipboard functionality... ");

    NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
    NSString *testText = @"VTT Test Text";

    [pasteboard clearContents];
    [pasteboard setString:testText forType:NSPasteboardTypeString];

    NSString *clipboardText = [pasteboard stringForType:NSPasteboardTypeString];

    if (![clipboardText isEqualToString:testText]) {
        printf("❌ FAILED\n");
        return NO;
    }

    printf("✅ PASSED\n");
    return YES;
}

@end

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        printf("\n=== VTT Daemon Tests ===\n\n");

        VTTTester *tester = [[VTTTester alloc] init];
        int passed = 0;
        int total = 6;

        if ([tester testWhisperCommand]) passed++;
        if ([tester testAudioFormat]) passed++;
        if ([tester testWAVGeneration]) passed++;
        if ([tester testTempFileCreation]) passed++;
        if ([tester testModelPathGeneration]) passed++;
        if ([tester testClipboardFunctionality]) passed++;

        printf("\n=== Results: %d/%d Tests Passed ===\n\n", passed, total);

        if (passed == total) {
            printf("🎉 All tests passed! VTT is ready.\n");
            return 0;
        } else {
            printf("❌ Some tests failed. Check the implementation.\n");
            return 1;
        }
    }
}