// VTT Daemon - Professional Objective-C implementation
#import <Cocoa/Cocoa.h>
#import <CoreAudio/CoreAudio.h>
#import <AudioToolbox/AudioToolbox.h>
#import <ApplicationServices/ApplicationServices.h>
#import <AVFoundation/AVFoundation.h>
#import <IOKit/hidsystem/ev_keymap.h>
#import "VTTOnboarding.h"
#include <unistd.h>
#include <sys/stat.h>

#ifdef USE_WHISPER_LIB
#include "whisper.h"
#endif

// Logging toggle (default ON for debugging). Use VTTLog instead of NSLog.
static volatile BOOL VTTLoggingEnabled = YES;
static NSString *VTTLogFilePath = nil;

static void VTTLogToFile(NSString *message) {
    if (!VTTLogFilePath) {
        NSString *logsDir = [NSTemporaryDirectory() stringByAppendingPathComponent:@"VTT"];
        [[NSFileManager defaultManager] createDirectoryAtPath:logsDir withIntermediateDirectories:YES attributes:nil error:nil];
        VTTLogFilePath = [logsDir stringByAppendingPathComponent:@"vtt.log"];
    }

    // Rotate log if > 10MB
    NSDictionary *attrs = [[NSFileManager defaultManager] attributesOfItemAtPath:VTTLogFilePath error:nil];
    if (attrs && [attrs fileSize] > 10 * 1024 * 1024) {
        // Keep last 3 rotated logs
        NSString *log3 = [VTTLogFilePath stringByAppendingString:@".3"];
        NSString *log2 = [VTTLogFilePath stringByAppendingString:@".2"];
        NSString *log1 = [VTTLogFilePath stringByAppendingString:@".1"];
        [[NSFileManager defaultManager] removeItemAtPath:log3 error:nil];
        [[NSFileManager defaultManager] moveItemAtPath:log2 toPath:log3 error:nil];
        [[NSFileManager defaultManager] moveItemAtPath:log1 toPath:log2 error:nil];
        [[NSFileManager defaultManager] moveItemAtPath:VTTLogFilePath toPath:log1 error:nil];
    }

    NSFileHandle *fh = [NSFileHandle fileHandleForWritingAtPath:VTTLogFilePath];
    if (!fh) {
        [@"" writeToFile:VTTLogFilePath atomically:YES encoding:NSUTF8StringEncoding error:nil];
        fh = [NSFileHandle fileHandleForWritingAtPath:VTTLogFilePath];
    }
    [fh seekToEndOfFile];
    NSString *timestamp = [NSDateFormatter localizedStringFromDate:[NSDate date] dateStyle:NSDateFormatterNoStyle timeStyle:NSDateFormatterMediumStyle];
    NSString *logLine = [NSString stringWithFormat:@"[%@] %@\n", timestamp, message];
    [fh writeData:[logLine dataUsingEncoding:NSUTF8StringEncoding]];
    [fh closeFile];
}

#define VTTLog(fmt, ...) do { \
    if (VTTLoggingEnabled) { \
        NSString *msg = [NSString stringWithFormat:fmt, ##__VA_ARGS__]; \
        NSLog(@"%@", msg); \
        VTTLogToFile(msg); \
    } \
} while (0)

#define SAMPLE_RATE 16000
#define CHANNELS 1
// Smaller buffer = lower callback latency (at 48 kHz, 4096 bytes ~ 42 ms)
#define BUFFER_SIZE 4096
#define MAX_RECORDING_SECONDS 300  // Maximum recording duration (5 minutes)

// C struct for audio state (for performance)
typedef struct {
    AudioQueueRef queue;
    AudioStreamBasicDescription format;
    AudioQueueBufferRef buffers[3];
    FILE* audioFile;
    BOOL isRecording;
    char tempFileName[256];
    size_t bytesCaptured;
    size_t maxBytesAllowed;      // Maximum bytes before stopping (actualSampleRate * CHANNELS * 2 * MAX_RECORDING_SECONDS)
    BOOL bufferFull;              // Set when max recording length is reached
    void* daemonRef;              // Weak reference to VTTDaemon for notifications
    Float64 actualSampleRate;     // Actual recording sample rate (48000 Hz, 24000 Hz, etc.)
} AudioState;

@interface VTTDaemon : NSObject <NSApplicationDelegate>
@property (strong) NSStatusItem *statusItem;
@property (nonatomic) AudioState *audioState;
@property (strong) NSMenu *menu;
@property (strong) NSMenuItem *statusMenuItem;
@property (strong) NSString *selectedModel;
@property (strong) NSMenuItem *modelMenuItem;
@property (strong) NSString *selectedLanguage; // "en" or "auto"
@property (strong) NSMenuItem *languageMenuItem;
@property (strong) NSTask *downloadTask;
@property (nonatomic) NSInteger downloadRetryCount;
@property (strong) NSString *downloadingModel;
@property (strong) dispatch_queue_t transcribeQueue;
@property (atomic) NSInteger pendingJobs;
@property (atomic) NSUInteger sessionCounter;
@property (atomic) BOOL isTranscribing;
@property (atomic) BOOL waitingForKeyRelease;  // Set when max length reached, cleared when PTT released
@property (nonatomic) BOOL loggingEnabled;
@property (strong) NSMenuItem *loggingToggleItem;
@property (strong) NSMenuItem *micMenuItem; // Read-only microphone status (like Linux)
@property (nonatomic) CGKeyCode hotkeyCode;
@property (nonatomic) CGEventFlags hotkeyModifiers;
@property (strong) NSMenuItem *hotkeyMenuItem;
@property (strong) NSWindow *logWindow;
@property (strong) NSString *initialPrompt;
@property (strong) NSString *voicePrefix;
@property (strong) NSMenuItem *promptMenuItem;
@property (strong) NSPanel *promptPanel;
#ifdef USE_WHISPER_LIB
@property (nonatomic) struct whisper_context *wctx;
#endif
@end

@implementation VTTDaemon

// Linear resampler: int16 -> int16, mono
static void resample_linear_i16_mono(const int16_t *in, size_t in_len,
                                     double in_rate, double out_rate,
                                     int16_t **out_data, size_t *out_len) {
    if (!in || in_len == 0 || in_rate <= 0 || out_rate <= 0 || !out_data || !out_len) {
        *out_data = NULL; *out_len = 0; return;
    }
    if (fabs(in_rate - out_rate) < 1e-6) {
        // No resample needed – copy
        *out_len = in_len;
        *out_data = (int16_t *)malloc(in_len * sizeof(int16_t));
        memcpy(*out_data, in, in_len * sizeof(int16_t));
        return;
    }
    double ratio = out_rate / in_rate;
    size_t n_out = (size_t)floor((double)in_len * ratio);
    if (n_out < 1) n_out = 1;
    int16_t *out = (int16_t *)malloc(n_out * sizeof(int16_t));
    for (size_t j = 0; j < n_out; ++j) {
        double src_pos = (double)j / ratio; // position in input samples
        size_t i0 = (size_t)floor(src_pos);
        size_t i1 = i0 + 1;
        double frac = src_pos - (double)i0;
        int16_t s0 = in[i0 < in_len ? i0 : in_len - 1];
        int16_t s1 = in[i1 < in_len ? i1 : in_len - 1];
        double val = (1.0 - frac) * (double)s0 + frac * (double)s1;
        if (val > 32767.0) val = 32767.0;
        if (val < -32768.0) val = -32768.0;
        out[j] = (int16_t)lrint(val);
    }
    *out_data = out;
    *out_len = n_out;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Hot-plug Detection for Microphones (ADDED: 2025-10-13 02:10 UTC)
// ═══════════════════════════════════════════════════════════════════════════════
// macOS Implementation: Core Audio property listener callback
//
// This C callback function is registered with AudioObjectAddPropertyListener() to
// receive instant notifications when audio devices are added/removed from the system.
//
// How it works:
// 1. Core Audio calls this function when kAudioHardwarePropertyDevices changes
// 2. Function is called on an arbitrary thread (hence @autoreleasepool)
// 3. We dispatch to main queue to safely update the UI (menu items)
// 4. updateMicrophoneDisplay shows current system default microphone name
//
// Registration happens in applicationDidFinishLaunching: after menu creation.
// See line ~291 where AudioObjectAddPropertyListener is called.
//
// Benefits over polling:
// - Instant notification (no delay)
// - No CPU overhead when devices don't change
// - Native macOS Core Audio API
//
// Note for debugging:
// - If build fails, check that updateMicrophoneDisplay method exists
// - Verify AudioToolbox framework is linked
// - Check that self is properly __bridged when passing to callback
// ═══════════════════════════════════════════════════════════════════════════════
static OSStatus audioDeviceChangeCallback(AudioObjectID inObjectID,
                                         UInt32 inNumberAddresses,
                                         const AudioObjectPropertyAddress* inAddresses,
                                         void* inClientData) {
    @autoreleasepool {
        VTTDaemon* daemon = (__bridge VTTDaemon*)inClientData;
        VTTLog(@"Audio devices changed, updating microphone display");
        dispatch_async(dispatch_get_main_queue(), ^{
            [daemon updateMicrophoneDisplay];
        });
    }
    return noErr;
}

// Pure C audio callback (for speed)
static void audioInputCallback(void* userData,
                              AudioQueueRef queue,
                              AudioQueueBufferRef buffer,
                              const AudioTimeStamp* startTime,
                              UInt32 numPackets,
                              const AudioStreamPacketDescription* packetDesc) {
    AudioState* state = (AudioState*)userData;

    if (state->isRecording && state->audioFile) {
        size_t bytesToWrite = buffer->mAudioDataByteSize;
        size_t spaceLeft = state->maxBytesAllowed - state->bytesCaptured;

        if (bytesToWrite > spaceLeft) {
            bytesToWrite = spaceLeft;
            // Mark that we've hit the buffer limit
            if (spaceLeft > 0 && !state->bufferFull) {
                state->bufferFull = YES;
                VTTLog(@"Recording buffer full - max length reached (%d seconds)", MAX_RECORDING_SECONDS);

                // Notify daemon to auto-stop recording
                if (state->daemonRef) {
                    VTTDaemon* daemon = (__bridge VTTDaemon*)state->daemonRef;
                    dispatch_async(dispatch_get_main_queue(), ^{
                        [daemon handleMaxLengthReached];
                    });
                }
            }
        }

        if (bytesToWrite > 0) {
            size_t written = fwrite(buffer->mAudioData, 1, bytesToWrite, state->audioFile);
            state->bytesCaptured += written;
            fflush(state->audioFile); // Force write to disk
        }
    }

    AudioQueueEnqueueBuffer(queue, buffer, 0, NULL);
}

- (void)applicationDidFinishLaunching:(NSNotification *)notification {
    // Initialize logging first
    NSString *logsDir = [NSTemporaryDirectory() stringByAppendingPathComponent:@"VTT"];
    NSError *dirError = nil;
    [[NSFileManager defaultManager] createDirectoryAtPath:logsDir withIntermediateDirectories:YES attributes:nil error:&dirError];
    if (dirError) {
        NSLog(@"Failed to create log directory: %@", dirError);
    }

    VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    VTTLog(@"VTT STARTING");
    VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Load preferences or set defaults
    NSUserDefaults *defaults = [NSUserDefaults standardUserDefaults];
    self.selectedModel = [defaults stringForKey:@"selectedModel"];
    if (!self.selectedModel) {
        self.selectedModel = @"CT2 small";  // Default to CT2 small (best for machines without GPU)
        [defaults setObject:self.selectedModel forKey:@"selectedModel"];
    }

    self.selectedLanguage = [defaults stringForKey:@"selectedLanguage"];
    if (!self.selectedLanguage) {
        self.selectedLanguage = @"en";  // Default to English-only (fastest)
        [defaults setObject:self.selectedLanguage forKey:@"selectedLanguage"];
    }

    VTTLog(@"Default model: %@", self.selectedModel);
    VTTLog(@"Default language: %@", self.selectedLanguage);

    // Load hotkey preference (default: Right Alt/Option = keycode 61)
    if ([defaults objectForKey:@"hotkeyCode"]) {
        self.hotkeyCode = (CGKeyCode)[defaults integerForKey:@"hotkeyCode"];
        self.hotkeyModifiers = (CGEventFlags)[defaults integerForKey:@"hotkeyModifiers"];
    } else {
        self.hotkeyCode = 61; // Default: Right Alt/Option
        self.hotkeyModifiers = 0; // No modifiers for Right Alt
        [defaults setInteger:self.hotkeyCode forKey:@"hotkeyCode"];
        [defaults setInteger:self.hotkeyModifiers forKey:@"hotkeyModifiers"];
    }

    // Load initial prompt preference (default: British English, programming)
    self.initialPrompt = [defaults stringForKey:@"initialPrompt"];
    if (!self.initialPrompt) {
        self.initialPrompt = @"British English, technical context. Git, GitHub, Claude, API, CLI, JSON, YAML, SSH, Docker, TypeScript, Python, Ubuntu, PPA, Launchpad. Powell-Clark, Emmanuel.";
        [defaults setObject:self.initialPrompt forKey:@"initialPrompt"];
    }

    // Load voice prefix preference (default: "[voice] ")
    self.voicePrefix = [defaults stringForKey:@"voicePrefix"];
    if (!self.voicePrefix) {
        self.voicePrefix = @"[voice] ";
        [defaults setObject:self.voicePrefix forKey:@"voicePrefix"];
    }

    // Set up menu bar
    self.statusItem = [[NSStatusBar systemStatusBar] statusItemWithLength:NSVariableStatusItemLength];
    self.statusItem.button.title = @"VTT ⏸";

    // Create menu
    self.menu = [[NSMenu alloc] init];
    self.statusMenuItem = [[NSMenuItem alloc] initWithTitle:@"Status: Initializing..."
                                                      action:nil
                                               keyEquivalent:@""];
    [self.menu addItem:self.statusMenuItem];
    [self.menu addItem:[NSMenuItem separatorItem]];

    // Language selection submenu (FIRST - affects model filtering)
    NSString *languageDisplay = [self.selectedLanguage isEqualToString:@"en"] ? @"English only" : @"Multilingual";
    self.languageMenuItem = [[NSMenuItem alloc] initWithTitle:[NSString stringWithFormat:@"Language: %@", languageDisplay]
                                                        action:nil
                                                 keyEquivalent:@""];
    NSMenu *languageMenu = [[NSMenu alloc] init];

    NSMenuItem *englishItem = [[NSMenuItem alloc] initWithTitle:@"English only (fastest)"
                                                         action:@selector(selectLanguage:)
                                                  keyEquivalent:@""];
    englishItem.target = self;
    englishItem.representedObject = @"en";
    englishItem.state = [self.selectedLanguage isEqualToString:@"en"] ? NSControlStateValueOn : NSControlStateValueOff;
    [languageMenu addItem:englishItem];

    NSMenuItem *multilingualItem = [[NSMenuItem alloc] initWithTitle:@"Multilingual (99 languages)"
                                                              action:@selector(selectLanguage:)
                                                       keyEquivalent:@""];
    multilingualItem.target = self;
    multilingualItem.representedObject = @"auto";
    multilingualItem.state = [self.selectedLanguage isEqualToString:@"auto"] ? NSControlStateValueOn : NSControlStateValueOff;
    [languageMenu addItem:multilingualItem];

    self.languageMenuItem.submenu = languageMenu;
    [self.menu addItem:self.languageMenuItem];

    // Model selection submenu (SECOND - filtered by language choice)
    self.modelMenuItem = [[NSMenuItem alloc] initWithTitle:[NSString stringWithFormat:@"Model: %@", self.selectedModel]
                                                     action:nil
                                              keyEquivalent:@""];
    NSMenu *modelMenu = [[NSMenu alloc] init];

    // Whisper.cpp models
    NSArray *baseModels = @[@"tiny", @"base", @"small", @"medium", @"large"];
    for (NSString *model in baseModels) {
        NSString *whisperModel = [NSString stringWithFormat:@"W %@", model];
        NSMenuItem *item = [[NSMenuItem alloc] initWithTitle:whisperModel
                                                       action:@selector(selectModel:)
                                                keyEquivalent:@""];
        item.target = self;
        item.representedObject = model; // Store base model name
        if ([model isEqualToString:self.selectedModel] && ![self.selectedModel hasPrefix:@"CT2 "]) {
            item.state = NSControlStateValueOn;
        }
        [modelMenu addItem:item];
    }

    // Separator
    [modelMenu addItem:[NSMenuItem separatorItem]];

    // CTranslate2 models
    NSArray *ct2Models = @[@"CT2 tiny", @"CT2 base", @"CT2 small", @"CT2 distil-large-v3", @"CT2 large-v3-turbo"];
    for (NSString *ct2Model in ct2Models) {
        NSMenuItem *item = [[NSMenuItem alloc] initWithTitle:ct2Model
                                                       action:@selector(selectModel:)
                                                keyEquivalent:@""];
        item.target = self;
        item.representedObject = ct2Model;
        if ([ct2Model isEqualToString:self.selectedModel]) {
            item.state = NSControlStateValueOn;
        }
        [modelMenu addItem:item];
    }

    self.modelMenuItem.submenu = modelMenu;
    [self.menu addItem:self.modelMenuItem];

    // Check which models exist and disable unavailable ones
    [self rebuildModelMenu];

    // Microphone status (read-only, like Linux)
    self.micMenuItem = [[NSMenuItem alloc] initWithTitle:@"Microphone: Default"
                                                   action:nil
                                            keyEquivalent:@""];
    [self.micMenuItem setEnabled:NO]; // Read-only status
    [self.micMenuItem setSubmenu:nil]; // Ensure no dropdown
    [self.menu addItem:self.micMenuItem];

    // Update microphone display and refresh periodically
    [self updateMicrophoneDisplay];
    [NSTimer scheduledTimerWithTimeInterval:3.0
                                     target:self
                                   selector:@selector(updateMicrophoneDisplay)
                                   userInfo:nil
                                    repeats:YES];

    // ═══════════════════════════════════════════════════════════════════════
    // ADDED 2025-10-13 02:10 UTC: Register for audio device changes
    // ═══════════════════════════════════════════════════════════════════════
    // Register for hot-plug detection of audio devices (USB mics, Bluetooth, etc.)
    // This registers the audioDeviceChangeCallback (see line ~169) to be called
    // whenever kAudioHardwarePropertyDevices changes.
    //
    // Listener is active for the lifetime of the application. No need to unregister
    // unless we need to stop monitoring device changes.
    // ═══════════════════════════════════════════════════════════════════════
    AudioObjectPropertyAddress devicesAddr = {
        kAudioHardwarePropertyDevices,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain
    };
    AudioObjectAddPropertyListener(kAudioObjectSystemObject,
                                   &devicesAddr,
                                   audioDeviceChangeCallback,
                                   (__bridge void*)self);
    VTTLog(@"Registered for audio device change notifications");

    // Hotkey customization menu item
    NSString *hotkeyName = [self hotkeyNameForCode:self.hotkeyCode];
    self.hotkeyMenuItem = [[NSMenuItem alloc] initWithTitle:[NSString stringWithFormat:@"Hotkey: %@", hotkeyName]
                                                      action:@selector(changeHotkey:)
                                               keyEquivalent:@""];
    self.hotkeyMenuItem.target = self;
    [self.menu addItem:self.hotkeyMenuItem];

    // Initial prompt customization menu item
    NSString *promptPreview = self.initialPrompt.length > 30 ? [[self.initialPrompt substringToIndex:27] stringByAppendingString:@"..."] : self.initialPrompt;
    self.promptMenuItem = [[NSMenuItem alloc] initWithTitle:[NSString stringWithFormat:@"Prompt: %@", promptPreview]
                                                     action:@selector(changePrompt:)
                                              keyEquivalent:@""];
    self.promptMenuItem.target = self;
    [self.menu addItem:self.promptMenuItem];

    [self.menu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *permissionsItem = [[NSMenuItem alloc] initWithTitle:@"Check Permissions..."
                                                              action:@selector(checkPermissions:)
                                                       keyEquivalent:@""];
    permissionsItem.target = self;
    [self.menu addItem:permissionsItem];

    // Logging toggle (default OFF)
    self.loggingEnabled = [[NSUserDefaults standardUserDefaults] objectForKey:@"loggingEnabled"] ? [[NSUserDefaults standardUserDefaults] boolForKey:@"loggingEnabled"] : YES;  // Default to ON for debugging
    VTTLoggingEnabled = self.loggingEnabled;
    NSString *logTitle = self.loggingEnabled ? @"Logging: On" : @"Logging: Off";
    self.loggingToggleItem = [[NSMenuItem alloc] initWithTitle:logTitle
                                                       action:@selector(toggleLogging:)
                                                keyEquivalent:@""];
    self.loggingToggleItem.target = self;
    self.loggingToggleItem.state = self.loggingEnabled ? NSControlStateValueOn : NSControlStateValueOff;
    [self.menu addItem:self.loggingToggleItem];

    NSMenuItem *showLogsItem = [[NSMenuItem alloc] initWithTitle:@"Show Logs"
                                                          action:@selector(showLogs:)
                                                   keyEquivalent:@""];
    showLogsItem.target = self;
    [self.menu addItem:showLogsItem];

    [self.menu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *aboutItem = [[NSMenuItem alloc] initWithTitle:@"About Voice to Text"
                                                        action:@selector(showAbout:)
                                                 keyEquivalent:@""];
    aboutItem.target = self;
    [self.menu addItem:aboutItem];

    [self.menu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *quitItem = [[NSMenuItem alloc] initWithTitle:@"Quit"
                                                       action:@selector(quit:)
                                                keyEquivalent:@"q"];
    quitItem.target = self;
    [self.menu addItem:quitItem];

    self.statusItem.menu = self.menu;

    // Initialize audio
    [self initializeAudio];

    // Show beautiful onboarding window if needed
    [VTTOnboardingWindow showIfNeeded];

    // Set up keyboard monitoring
    [self setupKeyboardMonitoring];

    // Create serial transcription queue (FIFO, no drop)
    self.transcribeQueue = dispatch_queue_create("com.powellclark.voice-to-text.transcribe", DISPATCH_QUEUE_SERIAL);
    self.pendingJobs = 0;
    self.sessionCounter = 0;

#ifdef USE_WHISPER_LIB
    // CT2 models don't need Whisper library preloading
    if ([self.selectedModel hasPrefix:@"CT2 "]) {
        VTTLog(@"Selected model is CT2 - skipping Whisper library preload");
        dispatch_async(dispatch_get_main_queue(), ^{
            self.statusMenuItem.title = [NSString stringWithFormat:@"Status: %@ ready", self.selectedModel];
            self.statusItem.button.title = @"VTT ✅";
            [self updateStatusIcon];
        });
    } else {
        // Preload whisper context in background for zero-latency transcription
        dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
            @autoreleasepool {
                // Resolve model path (bundled preferred)
                // Map model names (large → large-v3 which is multilingual)
                NSString *modelFile = self.selectedModel;
                NSString *extension;
                BOOL isEnglish = [self.selectedLanguage isEqualToString:@"en"];

                if ([self.selectedModel isEqualToString:@"large"]) {
                    modelFile = @"large-v3";
                    extension = @"bin"; // large-v3 is multilingual only
                } else if (isEnglish) {
                    extension = @"en.bin"; // English-only model
                } else {
                    extension = @"bin"; // Multilingual model
                }

            NSString *bundledModelPath = [[[NSBundle mainBundle] resourcePath] stringByAppendingPathComponent:[NSString stringWithFormat:@"ggml-%@.%@", modelFile, extension]];
            NSString *externalModelPath = [NSString stringWithFormat:@"%@/whisper.cpp/models/ggml-%@.%@", NSHomeDirectory(), modelFile, extension];
            NSString *modelPath = nil;
            if ([[NSFileManager defaultManager] fileExistsAtPath:bundledModelPath]) {
                modelPath = bundledModelPath;
            } else if ([[NSFileManager defaultManager] fileExistsAtPath:externalModelPath]) {
                modelPath = externalModelPath;
            }
            if (!modelPath) {
                VTTLog(@"No model found to preload");
                return;
            }

            dispatch_async(dispatch_get_main_queue(), ^{
                self.statusMenuItem.title = [NSString stringWithFormat:@"Status: Loading %@ model…", self.selectedModel];
                self.statusItem.button.title = @"VTT ⏳";
            });

            VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            VTTLog(@"📦 MODEL LOADING START");
            VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            // Check file exists and get size
            NSError *error = nil;
            NSDictionary *attrs = [[NSFileManager defaultManager] attributesOfItemAtPath:modelPath error:&error];
            if (attrs) {
                unsigned long long fileSize = [attrs fileSize];
                VTTLog(@"Model file: %@", [modelPath lastPathComponent]);
                VTTLog(@"Model path: %@", modelPath);
                VTTLog(@"Model size: %.1f MB", fileSize / 1024.0 / 1024.0);
            } else {
                VTTLog(@"❌ Cannot read model file attributes: %@", error);
            }

            struct whisper_context_params cparams = whisper_context_default_params();
            // Enable Metal acceleration for both Intel and Apple Silicon Macs
            // Metal shader must be present in Resources folder (bundled by Makefile)
            cparams.use_gpu = true;
            cparams.flash_attn = true; // Enable Flash Attention for memory efficiency (reduces VRAM usage 2-4x)
            VTTLog(@"Platform: macOS (attempting Metal GPU acceleration with Flash Attention)");
            const char *mp = [modelPath UTF8String];
            VTTLog(@"Attempting GPU load: %s", cparams.use_gpu ? "YES" : "NO");
            VTTLog(@"Calling whisper_init_from_file_with_params...");

            struct whisper_context *ctx = whisper_init_from_file_with_params(mp, cparams);

            // CRITICAL: Always fallback to CPU if GPU fails
            // Metal initialization can fail for various reasons (driver issues, incompatible GPU, etc.)
            if (!ctx && cparams.use_gpu) {
                VTTLog(@"⚠️  GPU/Metal initialization failed!");
                VTTLog(@"Falling back to CPU mode (this is normal on some systems)");
                cparams.use_gpu = false;
                ctx = whisper_init_from_file_with_params(mp, cparams);
            }

            // If STILL null after CPU attempt, try one more time with explicit CPU-only context
            if (!ctx) {
                VTTLog(@"⚠️  First CPU attempt failed, trying with minimal params...");
                cparams = whisper_context_default_params();
                cparams.use_gpu = false;
                ctx = whisper_init_from_file_with_params(mp, cparams);
            }

            if (!ctx) {
                VTTLog(@"❌ ❌ ❌ MODEL LOADING FAILED ❌ ❌ ❌");
                VTTLog(@"Tried: GPU=%@, CPU=%@",
                    cparams.use_gpu ? @"NO" : @"YES",
                    @"YES");
                VTTLog(@"Model file may be corrupted or incompatible");
                VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            } else {
                VTTLog(@"✅ ✅ ✅ MODEL LOADED SUCCESSFULLY ✅ ✅ ✅");
                VTTLog(@"Using: %s", cparams.use_gpu ? "Metal GPU" : "CPU");
                VTTLog(@"Context address: %p", ctx);
                VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            }
            self.wctx = ctx;

            dispatch_async(dispatch_get_main_queue(), ^{
                if (self.wctx) {
                    self.statusMenuItem.title = [NSString stringWithFormat:@"Status: %@ model ready", self.selectedModel];
                    self.statusItem.button.title = @"VTT ✅";
                } else {
                    self.statusMenuItem.title = @"Status: Whisper init failed";
                    self.statusItem.button.title = @"VTT ⚠️";
                }
            });
        }
        });
    }
#endif

    // Update status based on permissions
    [self updateStatusIcon];
}

- (void)updateStatusIcon {
    BOOL hasAccessibility = AXIsProcessTrusted();
    BOOL hasMicrophone = [self checkMicrophonePermission];
    BOOL isCT2Model = [self.selectedModel hasPrefix:@"CT2 "];

    // CT2 models don't use self.wctx (they're CLI-based)
    BOOL modelReady = isCT2Model || self.wctx != NULL;

    if (hasAccessibility && hasMicrophone && modelReady) {
        self.statusItem.button.title = @"VTT ✅";
        self.statusMenuItem.title = @"Status: Ready";
    } else if (!hasAccessibility || !hasMicrophone) {
        self.statusItem.button.title = @"VTT ⚠️";
        self.statusMenuItem.title = @"Status: Missing permissions";
    } else if (!modelReady) {
        self.statusItem.button.title = @"VTT ⏳";
        self.statusMenuItem.title = @"Status: Loading model...";
    }
}

- (void)initializeAudio {
    self.audioState = (AudioState *)calloc(1, sizeof(AudioState));

    // Configure audio format
    // Force use of built-in microphone to avoid interrupting AirPods music
    Float64 detectedRate = SAMPLE_RATE;
    AudioDeviceID builtInMicID = 0;

    // Get all audio devices
    AudioObjectPropertyAddress devicesAddr = {
        kAudioHardwarePropertyDevices,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain
    };

    UInt32 propSize = 0;
    OSStatus status = AudioObjectGetPropertyDataSize(kAudioObjectSystemObject, &devicesAddr, 0, NULL, &propSize);

    if (status == noErr) {
        int deviceCount = propSize / sizeof(AudioDeviceID);
        AudioDeviceID *devices = (AudioDeviceID *)malloc(propSize);
        status = AudioObjectGetPropertyData(kAudioObjectSystemObject, &devicesAddr, 0, NULL, &propSize, devices);

        if (status == noErr) {
            // Find the built-in microphone
            for (int i = 0; i < deviceCount; i++) {
                // Check if this is an input device
                AudioObjectPropertyAddress streamAddr = {
                    kAudioDevicePropertyStreams,
                    kAudioDevicePropertyScopeInput,
                    kAudioObjectPropertyElementMain
                };

                UInt32 streamSize = 0;
                status = AudioObjectGetPropertyDataSize(devices[i], &streamAddr, 0, NULL, &streamSize);
                if (status != noErr || streamSize == 0) continue; // Not an input device

                // Get device name
                CFStringRef deviceName = NULL;
                UInt32 nameSize = sizeof(deviceName);
                AudioObjectPropertyAddress nameAddr = {
                    kAudioDevicePropertyDeviceNameCFString,
                    kAudioObjectPropertyScopeGlobal,
                    kAudioObjectPropertyElementMain
                };
                status = AudioObjectGetPropertyData(devices[i], &nameAddr, 0, NULL, &nameSize, &deviceName);

                if (status == noErr && deviceName != NULL) {
                    NSString *name = (__bridge NSString *)deviceName;
                    VTTLog(@"Found audio device: %@ (ID: %d)", name, devices[i]);

                    // Look for MacBook Pro Microphone or built-in microphone
                    if ([name containsString:@"MacBook"] && [name containsString:@"Microphone"]) {
                        builtInMicID = devices[i];
                        VTTLog(@"Selected built-in microphone: %@ (ID: %d)", name, devices[i]);
                        CFRelease(deviceName);
                        break;
                    } else if ([name containsString:@"Built-in Microphone"]) {
                        builtInMicID = devices[i];
                        VTTLog(@"Selected built-in microphone: %@ (ID: %d)", name, devices[i]);
                        CFRelease(deviceName);
                        break;
                    }
                    CFRelease(deviceName);
                }
            }
        }
        free(devices);
    }

    // Get the system default input device
    // Note: We cannot programmatically select a specific device with AudioQueue on macOS.
    // AudioQueueSetProperty fails with error -66683. Users must set their preferred
    // microphone in System Settings > Sound > Input.
    AudioDeviceID deviceID = 0;
    UInt32 size = sizeof(deviceID);
    AudioObjectPropertyAddress addr = {
        kAudioHardwarePropertyDefaultInputDevice,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain
    };
    AudioObjectGetPropertyData(kAudioObjectSystemObject, &addr, 0, NULL, &size, &deviceID);

    // Log which device will be used
    if (deviceID != 0) {
        VTTLog(@"System default input device ID: %u", deviceID);

        // Get device name for logging
        AudioObjectPropertyAddress nameAddr = {
            kAudioDevicePropertyDeviceNameCFString,
            kAudioObjectPropertyScopeGlobal,
            kAudioObjectPropertyElementMain
        };
        CFStringRef deviceName = NULL;
        UInt32 nameSize = sizeof(deviceName);
        if (AudioObjectGetPropertyData(deviceID, &nameAddr, 0, NULL, &nameSize, &deviceName) == noErr && deviceName != NULL) {
            VTTLog(@"Will use: %@", (__bridge NSString *)deviceName);
            CFRelease(deviceName);
        }
    }

    // Get sample rate from selected device
    if (deviceID != 0 && deviceID != kAudioObjectUnknown) {
        Float64 rate = 0;
        UInt32 size = sizeof(rate);
        AudioObjectPropertyAddress addr2 = {
            kAudioDevicePropertyNominalSampleRate,
            kAudioDevicePropertyScopeInput,
            kAudioObjectPropertyElementMain
        };
        OSStatus q2 = AudioObjectGetPropertyData(deviceID, &addr2, 0, NULL, &size, &rate);
        if (q2 == noErr && rate > 0) detectedRate = rate;
    }

    self.audioState->format.mSampleRate = detectedRate;
    self.audioState->format.mFormatID = kAudioFormatLinearPCM;
    self.audioState->format.mFormatFlags = kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsPacked;
    self.audioState->format.mFramesPerPacket = 1;
    self.audioState->format.mChannelsPerFrame = CHANNELS;
    self.audioState->format.mBitsPerChannel = 16;
    self.audioState->format.mBytesPerPacket = 2;
    self.audioState->format.mBytesPerFrame = 2;

    // Create audio queue first with the device's native format
    OSStatus qstatus = AudioQueueNewInput(&self.audioState->format,
                                         audioInputCallback,
                                         self.audioState,
                                         NULL,
                                         kCFRunLoopCommonModes,
                                         0,
                                         &self.audioState->queue);

    if (qstatus != noErr) {
        VTTLog(@"Failed to create audio queue: %d", qstatus);
        return;
    }

    // NOTE: We DO NOT call AudioQueueSetProperty to set the device!
    // AudioQueue will automatically use the system default input device.
    // Calling AudioQueueSetProperty fails with error -66683 (kAudioQueueErr_InvalidDevice)
    // on macOS when trying to switch devices after creation.
    //
    // To use a specific microphone, the user must set it as their system default
    // in System Settings > Sound > Input.

    VTTLog(@"✅ Audio queue created at %.0f Hz", detectedRate);
    VTTLog(@"   Will use system default input device");
    VTTLog(@"   To select a specific microphone, set it in System Settings > Sound > Input");

    // Store actual sample rate for buffer size calculation
    self.audioState->actualSampleRate = detectedRate;

    // Allocate buffers
    for (int i = 0; i < 3; i++) {
        AudioQueueAllocateBuffer(self.audioState->queue, BUFFER_SIZE, &self.audioState->buffers[i]);
        AudioQueueEnqueueBuffer(self.audioState->queue, self.audioState->buffers[i], 0, NULL);
    }
}

- (void)requestPermissions {
    // Check all permissions
    BOOL hasAccessibility = AXIsProcessTrusted();
    BOOL hasMicrophone = [self checkMicrophonePermission];

    if (!hasAccessibility || !hasMicrophone) {
        [self showPermissionDialog:hasAccessibility hasMicrophone:hasMicrophone];
    }
}

- (BOOL)checkMicrophonePermission {
    // Check microphone permission status
    AVAuthorizationStatus status = [AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeAudio];

    if (status == AVAuthorizationStatusNotDetermined) {
        // Request permission
        [AVCaptureDevice requestAccessForMediaType:AVMediaTypeAudio completionHandler:^(BOOL granted) {
            if (!granted) {
                dispatch_async(dispatch_get_main_queue(), ^{
                    self.statusMenuItem.title = @"Status: Need Microphone Permission";
                });
            }
        }];
        return NO;
    }

    return status == AVAuthorizationStatusAuthorized;
}

- (void)showPermissionDialog:(BOOL)hasAccessibility hasMicrophone:(BOOL)hasMicrophone {
    NSAlert *alert = [[NSAlert alloc] init];
    alert.messageText = @"Voice to Text Needs Permissions";

    NSMutableString *info = [NSMutableString string];
    [info appendString:@"Voice to Text needs these permissions to work:\n\n"];

    if (!hasAccessibility) {
        [info appendString:@"❌ Accessibility - for keyboard monitoring\n"];
    } else {
        [info appendString:@"✅ Accessibility\n"];
    }

    if (!hasMicrophone) {
        [info appendString:@"❌ Microphone - for recording audio\n"];
    } else {
        [info appendString:@"✅ Microphone\n"];
    }

    [info appendString:@"\nClick 'Open Settings' to grant permissions."];
    [info appendString:@"\nYou'll need to:\n"];
    [info appendString:@"1. Add Voice to Text to each permission\n"];
    [info appendString:@"2. Toggle the switch ON\n"];
    [info appendString:@"3. Restart Voice to Text when done"];

    alert.informativeText = info;
    [alert addButtonWithTitle:@"Open Settings"];
    [alert addButtonWithTitle:@"Later"];

    NSModalResponse response = [alert runModal];

    if (response == NSAlertFirstButtonReturn) {
        // Open System Preferences
        if (!hasAccessibility) {
            [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"]];
        } else if (!hasMicrophone) {
            [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"]];
        }

        // Update status
        self.statusMenuItem.title = @"Status: Grant permissions & restart";
        self.statusItem.button.title = @"VTT ⚠️";
    }
}

- (void)setupKeyboardMonitoring {
    // Create event tap for keyboard AND modifier keys
    CGEventMask mask = (1 << kCGEventKeyDown) | (1 << kCGEventKeyUp) | (1 << kCGEventFlagsChanged);
    CFMachPortRef tap = CGEventTapCreate(kCGSessionEventTap,
                                         kCGHeadInsertEventTap,
                                         kCGEventTapOptionDefault,
                                         mask,
                                         keyboardCallback,
                                         (__bridge void *)self);

    if (!tap) {
        VTTLog(@"Failed to create event tap - Check Input Monitoring permissions for VTT.app");
        self.statusMenuItem.title = @"Status: Need Input Monitoring permission";
        self.statusItem.button.title = @"VTT ⚠️";

        // Show permission dialog specifically for Input Monitoring
        NSAlert *alert = [[NSAlert alloc] init];
        alert.messageText = @"Voice to Text Needs Input Monitoring Permission";
        alert.informativeText = @"Voice to Text needs Input Monitoring permission to detect when you hold the Right Option key.\n\n1. Click 'Open Settings'\n2. Find and select VTT.app\n3. Toggle the switch ON\n4. Restart Voice to Text";
        [alert addButtonWithTitle:@"Open Settings"];
        [alert addButtonWithTitle:@"Cancel"];

        if ([alert runModal] == NSAlertFirstButtonReturn) {
            [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"]];
        }
        return;
    }

    VTTLog(@"Event tap created successfully");
    CFRunLoopSourceRef runLoopSource = CFMachPortCreateRunLoopSource(NULL, tap, 0);
    CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource, kCFRunLoopCommonModes);
    CGEventTapEnable(tap, true);
}

// Keyboard event callback
static CGEventRef keyboardCallback(CGEventTapProxy proxy,
                                   CGEventType type,
                                   CGEventRef event,
                                   void* refcon) {
    VTTDaemon *self = (__bridge VTTDaemon *)refcon;

    CGKeyCode keyCode = (CGKeyCode)CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);

    // Debug: log Option key events with their type
    if (keyCode == 58 || keyCode == 61) {
        const char *typeName = "UNKNOWN";
        if (type == kCGEventKeyDown) typeName = "KeyDown";
        else if (type == kCGEventKeyUp) typeName = "KeyUp";
        else if (type == kCGEventFlagsChanged) typeName = "FlagsChanged";
        VTTLog(@"Option key event: code=%d, type=%s", keyCode, typeName);
    }

    // Prefer exact key down/up for Right Option (keycode 61 on ANSI)
    if ((type == kCGEventKeyDown || type == kCGEventKeyUp) && keyCode == (CGKeyCode)61) {
        if (type == kCGEventKeyDown) {
            VTTLog(@"PTT key DOWN (Right Option, code=61)");
            if (!self.audioState->isRecording) {
                [self startRecording];
            }
        } else {
            VTTLog(@"PTT key UP (Right Option, code=61)");
            if (self.audioState->isRecording) {
                [self stopRecording];
            }
        }
        // Swallow the PTT key so it does not affect foreground apps / beep
        return NULL;
    }

    // Check for hotkey activation (both standalone modifiers and combinations)
    CGEventFlags currentFlags = CGEventGetFlags(event);
    CGEventFlags relevantFlags = currentFlags & (kCGEventFlagMaskCommand | kCGEventFlagMaskAlternate | kCGEventFlagMaskShift | kCGEventFlagMaskControl);

    // Check if hotkey matches
    BOOL hotkeyMatches = NO;

    if (self.hotkeyModifiers == 0) {
        // Standalone modifier key (e.g., Right Alt)
        if (type == kCGEventFlagsChanged) {
            CGKeyCode kc = (CGKeyCode)CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);
            VTTLog(@"FlagsChanged event: keycode=%d, hotkeyCode=%d", kc, self.hotkeyCode);
            if (kc == self.hotkeyCode) {
                // Check if the corresponding modifier flag is set
                BOOL modifierDown = NO;
                if (kc == 58 || kc == 61) modifierDown = (currentFlags & kCGEventFlagMaskAlternate) != 0; // Option
                else if (kc == 59 || kc == 62) modifierDown = (currentFlags & kCGEventFlagMaskControl) != 0; // Control
                else if (kc == 55 || kc == 54) modifierDown = (currentFlags & kCGEventFlagMaskCommand) != 0; // Command
                else if (kc == 56 || kc == 60) modifierDown = (currentFlags & kCGEventFlagMaskShift) != 0; // Shift

                if (modifierDown && !self.audioState->isRecording) {
                    VTTLog(@"PTT (standalone modifier) DOWN - starting recording");
                    [self startRecording];
                    return NULL;
                } else if (!modifierDown) {
                    // Key released - handle both cases: during recording and during transcription
                    if (self.waitingForKeyRelease) {
                        VTTLog(@"PTT (standalone modifier) UP - key released while waiting for transcription");
                        self.waitingForKeyRelease = NO;
                    } else if (self.audioState->isRecording) {
                        VTTLog(@"PTT (standalone modifier) UP - stopping recording");
                        [self stopRecording];
                    }
                    return NULL;
                }
                // Always swallow the PTT key to prevent it affecting other apps
                return NULL;
            }
        }
    } else {
        // Combination (e.g., Cmd+Shift+R)
        if (type == kCGEventKeyDown) {
            if (keyCode == self.hotkeyCode && relevantFlags == self.hotkeyModifiers) {
                if (!self.audioState->isRecording) {
                    VTTLog(@"PTT (combination) DOWN - starting recording");
                    [self startRecording];
                    return NULL;
                }
            }
        } else if (type == kCGEventKeyUp) {
            if (keyCode == self.hotkeyCode && self.audioState->isRecording) {
                if (self.waitingForKeyRelease) {
                    VTTLog(@"PTT (combination) UP - key released after max length, now transcribing");
                    self.waitingForKeyRelease = NO;
                } else {
                    VTTLog(@"PTT (combination) UP - stopping recording");
                }
                [self stopRecording];
                return NULL;
            }
        }
    }

    // Debug: log other keys when logging is enabled (commented out - too verbose)
    // if (type == kCGEventKeyDown || type == kCGEventKeyUp) {
    //     VTTLog(@"Key event: code=%d, type=%s", keyCode, type == kCGEventKeyDown ? "DOWN" : "UP");
    // }

    return event;
}

- (void)startRecording {

    if (!self.audioState) {
        return;
    }

    // Check microphone permission before recording
    AVAuthorizationStatus micStatus = [AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeAudio];
    if (micStatus != AVAuthorizationStatusAuthorized) {
        dispatch_async(dispatch_get_main_queue(), ^{
            NSAlert *alert = [[NSAlert alloc] init];
            alert.messageText = @"Microphone Permission Required";
            alert.informativeText = @"Voice to Text needs microphone access to record audio.\n\nPlease grant permission in System Settings → Privacy & Security → Microphone";
            alert.alertStyle = NSAlertStyleWarning;
            [alert addButtonWithTitle:@"Open Settings"];
            [alert addButtonWithTitle:@"Cancel"];
            NSInteger response = [alert runModal];
            if (response == NSAlertFirstButtonReturn) {
                [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"]];
            }
        });
        return;
    }

    // Check accessibility permission (needed for paste)
    if (!AXIsProcessTrusted()) {
    }

    if (self.audioState->isRecording) {
        return;
    }

    // Allow new recordings even during transcription - serial dispatch queue will handle queuing
    // (Removed isTranscribing check to enable rapid-fire messages)

    // Create unique temp file for this session
    unsigned long long session = ++self.sessionCounter;
    snprintf(self.audioState->tempFileName, sizeof(self.audioState->tempFileName), "/tmp/vtt_%d_%llu.raw", getpid(), session);
    self.audioState->audioFile = fopen(self.audioState->tempFileName, "wb");

    if (!self.audioState->audioFile) {
        return;
    }

    // Arm recording and start the queue so callbacks deliver audio
    self.audioState->bytesCaptured = 0;
    // Use actual recording sample rate (not resampling target rate) for buffer calculation
    // actualSampleRate could be 48000 Hz (MacBook Pro), 24000 Hz (AirPods), etc.
    self.audioState->maxBytesAllowed = (size_t)(self.audioState->actualSampleRate * CHANNELS * 2 * MAX_RECORDING_SECONDS);
    self.audioState->bufferFull = NO;
    self.audioState->daemonRef = (__bridge void*)self;
    self.audioState->isRecording = YES;
    // Re-prime the input queue after a previous stop: return any buffers
    // to the queue and enqueue them again so the device can fill them.
    AudioQueueReset(self.audioState->queue);
    for (int i = 0; i < 3; i++) {
        AudioQueueEnqueueBuffer(self.audioState->queue, self.audioState->buffers[i], 0, NULL);
    }
    OSStatus qstart = AudioQueueStart(self.audioState->queue, NULL);

    // Update UI
    self.statusItem.button.title = @"VTT 🎤";
    self.statusMenuItem.title = @"Status: Recording...";
}

// Direct character typing function (like Linux XTest approach)
// Maps characters to CGKeyCodes and simulates keypresses without using clipboard
- (void)typeCharacter:(unichar)c {
    // Create event source and set keyboard type to isolate from current state
    CGEventSourceRef src = CGEventSourceCreate(kCGEventSourceStatePrivate);
    CGEventSourceSetKeyboardType(src, 40); // ANSI keyboard type
    CGKeyCode keyCode = 0;
    BOOL needsShift = NO;

    // Character to keycode mapping
    switch (c) {
        // Letters (lowercase)
        case 'a': keyCode = 0; break;
        case 'b': keyCode = 11; break;
        case 'c': keyCode = 8; break;
        case 'd': keyCode = 2; break;
        case 'e': keyCode = 14; break;
        case 'f': keyCode = 3; break;
        case 'g': keyCode = 5; break;
        case 'h': keyCode = 4; break;
        case 'i': keyCode = 34; break;
        case 'j': keyCode = 38; break;
        case 'k': keyCode = 40; break;
        case 'l': keyCode = 37; break;
        case 'm': keyCode = 46; break;
        case 'n': keyCode = 45; break;
        case 'o': keyCode = 31; break;
        case 'p': keyCode = 35; break;
        case 'q': keyCode = 12; break;
        case 'r': keyCode = 15; break;
        case 's': keyCode = 1; break;
        case 't': keyCode = 17; break;
        case 'u': keyCode = 32; break;
        case 'v': keyCode = 9; break;
        case 'w': keyCode = 13; break;
        case 'x': keyCode = 7; break;
        case 'y': keyCode = 16; break;
        case 'z': keyCode = 6; break;

        // Uppercase letters (same keycodes, need shift)
        case 'A': keyCode = 0; needsShift = YES; break;
        case 'B': keyCode = 11; needsShift = YES; break;
        case 'C': keyCode = 8; needsShift = YES; break;
        case 'D': keyCode = 2; needsShift = YES; break;
        case 'E': keyCode = 14; needsShift = YES; break;
        case 'F': keyCode = 3; needsShift = YES; break;
        case 'G': keyCode = 5; needsShift = YES; break;
        case 'H': keyCode = 4; needsShift = YES; break;
        case 'I': keyCode = 34; needsShift = YES; break;
        case 'J': keyCode = 38; needsShift = YES; break;
        case 'K': keyCode = 40; needsShift = YES; break;
        case 'L': keyCode = 37; needsShift = YES; break;
        case 'M': keyCode = 46; needsShift = YES; break;
        case 'N': keyCode = 45; needsShift = YES; break;
        case 'O': keyCode = 31; needsShift = YES; break;
        case 'P': keyCode = 35; needsShift = YES; break;
        case 'Q': keyCode = 12; needsShift = YES; break;
        case 'R': keyCode = 15; needsShift = YES; break;
        case 'S': keyCode = 1; needsShift = YES; break;
        case 'T': keyCode = 17; needsShift = YES; break;
        case 'U': keyCode = 32; needsShift = YES; break;
        case 'V': keyCode = 9; needsShift = YES; break;
        case 'W': keyCode = 13; needsShift = YES; break;
        case 'X': keyCode = 7; needsShift = YES; break;
        case 'Y': keyCode = 16; needsShift = YES; break;
        case 'Z': keyCode = 6; needsShift = YES; break;

        // Numbers
        case '0': keyCode = 29; break;
        case '1': keyCode = 18; break;
        case '2': keyCode = 19; break;
        case '3': keyCode = 20; break;
        case '4': keyCode = 21; break;
        case '5': keyCode = 23; break;
        case '6': keyCode = 22; break;
        case '7': keyCode = 26; break;
        case '8': keyCode = 28; break;
        case '9': keyCode = 25; break;

        // Special characters (no shift)
        case ' ': keyCode = 49; break;
        case '\n': keyCode = 36; break; // Return
        case '\t': keyCode = 48; break; // Tab
        case '-': keyCode = 27; break;
        case '=': keyCode = 24; break;
        case '[': keyCode = 33; break;
        case ']': keyCode = 30; break;
        case '\\': keyCode = 42; break;
        case ';': keyCode = 41; break;
        case '\'': keyCode = 39; break;
        case ',': keyCode = 43; break;
        case '.': keyCode = 47; break;
        case '/': keyCode = 44; break;
        case '`': keyCode = 50; break;

        // Special characters (with shift)
        case '!': keyCode = 18; needsShift = YES; break; // Shift+1
        case '@': keyCode = 19; needsShift = YES; break; // Shift+2
        case '#': keyCode = 20; needsShift = YES; break; // Shift+3
        case '$': keyCode = 21; needsShift = YES; break; // Shift+4
        case '%': keyCode = 23; needsShift = YES; break; // Shift+5
        case '^': keyCode = 22; needsShift = YES; break; // Shift+6
        case '&': keyCode = 26; needsShift = YES; break; // Shift+7
        case '*': keyCode = 28; needsShift = YES; break; // Shift+8
        case '(': keyCode = 25; needsShift = YES; break; // Shift+9
        case ')': keyCode = 29; needsShift = YES; break; // Shift+0
        case '_': keyCode = 27; needsShift = YES; break; // Shift+-
        case '+': keyCode = 24; needsShift = YES; break; // Shift+=
        case '{': keyCode = 33; needsShift = YES; break; // Shift+[
        case '}': keyCode = 30; needsShift = YES; break; // Shift+]
        case '|': keyCode = 42; needsShift = YES; break; // Shift+\
        case ':': keyCode = 41; needsShift = YES; break; // Shift+;
        case '"': keyCode = 39; needsShift = YES; break; // Shift+'
        case '<': keyCode = 43; needsShift = YES; break; // Shift+,
        case '>': keyCode = 47; needsShift = YES; break; // Shift+.
        case '?': keyCode = 44; needsShift = YES; break; // Shift+/
        case '~': keyCode = 50; needsShift = YES; break; // Shift+`

        default:
            // Unknown character - skip
            CFRelease(src);
            return;
    }

    // Press shift if needed
    if (needsShift) {
        CGEventRef shiftDown = CGEventCreateKeyboardEvent(src, 56, true); // Left Shift = 56
        CGEventPost(kCGHIDEventTap, shiftDown);
        CFRelease(shiftDown);
        usleep(1000); // 1ms
    }

    // Press key - explicitly clear modifier flags to prevent Option key interference
    CGEventRef keyDown = CGEventCreateKeyboardEvent(src, keyCode, true);
    CGEventSetFlags(keyDown, needsShift ? kCGEventFlagMaskShift : 0);  // Only set shift if needed, clear all other modifiers
    CGEventPost(kCGHIDEventTap, keyDown);
    CFRelease(keyDown);
    usleep(1000); // 1ms between press and release

    // Release key - also clear modifiers
    CGEventRef keyUp = CGEventCreateKeyboardEvent(src, keyCode, false);
    CGEventSetFlags(keyUp, needsShift ? kCGEventFlagMaskShift : 0);  // Only set shift if needed, clear all other modifiers
    CGEventPost(kCGHIDEventTap, keyUp);
    CFRelease(keyUp);

    // Release shift if needed
    if (needsShift) {
        CGEventRef shiftUp = CGEventCreateKeyboardEvent(src, 56, false);
        CGEventPost(kCGHIDEventTap, shiftUp);
        CFRelease(shiftUp);
    }

    usleep(1000); // 1ms delay between characters
    CFRelease(src);
}

- (void)typeText:(NSString *)text {
    if (!text || text.length == 0) {
        return;
    }

    VTTLog(@"⌨️  Typing %lu characters directly (waiting for modifier keys to clear)", (unsigned long)text.length);

    // Wait for all modifier keys to be released before typing
    // This prevents Option key from interfering with character generation
    CGEventFlags currentFlags;
    int maxWaitTime = 100; // Maximum 100 * 10ms = 1 second wait
    int waitCount = 0;

    do {
        currentFlags = CGEventSourceFlagsState(kCGEventSourceStateHIDSystemState);
        if (currentFlags & (kCGEventFlagMaskAlternate | kCGEventFlagMaskControl | kCGEventFlagMaskCommand)) {
            VTTLog(@"🔄 Waiting for modifier keys to release (flags: 0x%lx)...", (unsigned long)currentFlags);
            usleep(10000); // Wait 10ms
            waitCount++;
        } else {
            break;
        }
    } while (waitCount < maxWaitTime);

    if (waitCount >= maxWaitTime) {
        VTTLog(@"⚠️  Timeout waiting for modifier keys, typing anyway");
    } else if (waitCount > 0) {
        VTTLog(@"✅ Modifier keys cleared after %d attempts", waitCount);
    }

    // Character-by-character approach with isolated event source
    for (NSUInteger i = 0; i < text.length; i++) {
        unichar c = [text characterAtIndex:i];
        [self typeCharacter:c];
    }

    VTTLog(@"✅ Typing completed");
}

- (void)stopRecording {

    if (!self.audioState) {
        return;
    }

    if (!self.audioState->isRecording) {
        return;
    }

    // Allow a brief grace period to ensure we captured at least one buffer
    int spins = 0;
    while (self.audioState->bytesCaptured == 0 && spins < 30) { // up to ~300 ms
        usleep(10000);
        spins++;
    }

    // Stop the audio queue and flush outstanding buffers so the last data is written
    OSStatus qstop = AudioQueueStop(self.audioState->queue, true);

    // Now end recording state and close the file
    self.audioState->isRecording = NO;

    if (self.audioState->audioFile) {
        fclose(self.audioState->audioFile);
        self.audioState->audioFile = NULL;
    } else {
    }

    // Capture the just-finished raw file path and buffer full flag before new recordings begin
    NSString *rawPath = [NSString stringWithUTF8String:self.audioState->tempFileName];
    BOOL wasBufferFull = self.audioState->bufferFull;

    // Update UI state and enqueue job (FIFO, no drop)
    dispatch_async(dispatch_get_main_queue(), ^{
        self.statusItem.button.title = @"VTT ⏳";
        self.statusMenuItem.title = @"Status: Processing...";
    });

    dispatch_async(dispatch_get_main_queue(), ^{ self.pendingJobs++; });
    dispatch_async(self.transcribeQueue, ^{
        self.isTranscribing = YES;
        [self processAudioFileAtPath:rawPath wasBufferFull:wasBufferFull];
        self.isTranscribing = NO;
        dispatch_async(dispatch_get_main_queue(), ^{
            self.pendingJobs--;
            if (!self.audioState->isRecording && self.pendingJobs == 0) {
                self.statusItem.button.title = @"VTT ✅";
                self.statusMenuItem.title = @"Status: Ready";
            } else {
                self.statusItem.button.title = @"VTT ⏳";
                self.statusMenuItem.title = [NSString stringWithFormat:@"Status: Processing... (%ld)", (long)self.pendingJobs];
            }
        });
    });
}

- (void)handleMaxLengthReached {
    VTTLog(@"Max recording length reached - stopping recording, will transcribe immediately");

    // Show desktop notification telling user to release key (for paste)
    NSUserNotification *notification = [[NSUserNotification alloc] init];
    notification.title = @"Voice to Text";
    notification.informativeText = [NSString stringWithFormat:@"Recording limit reached (%ds) - transcribing... release key when ready", MAX_RECORDING_SECONDS];
    notification.soundName = NSUserNotificationDefaultSoundName;

    [[NSUserNotificationCenter defaultUserNotificationCenter] deliverNotification:notification];

    // Set flag to indicate paste should wait for key release
    self.waitingForKeyRelease = YES;

    // Stop recording and start transcription immediately
    [self stopRecording];
}

- (NSString *)transcribeWithCTranslate2:(NSString *)wavPath model:(NSString *)modelName {
    // Locate whisper-ctranslate2 CLI in common locations (Apple Silicon uses /opt/homebrew)
    NSArray<NSString *> *ct2Paths = @[
        @"/opt/homebrew/bin/whisper-ctranslate2",  // Apple Silicon (arm64)
        @"/usr/local/bin/whisper-ctranslate2"       // Intel Mac (x86_64)
    ];

    NSString *ct2Binary = nil;
    for (NSString *path in ct2Paths) {
        if ([[NSFileManager defaultManager] isExecutableFileAtPath:path]) {
            ct2Binary = path;
            break;
        }
    }

    if (!ct2Binary) {
        VTTLog(@"❌ whisper-ctranslate2 not found in /opt/homebrew/bin or /usr/local/bin");
        return nil;
    }

    VTTLog(@"Using CT2 binary: %@", ct2Binary);

    // Call whisper-ctranslate2 CLI for fast transcription
    NSTask *task = [[NSTask alloc] init];
    task.launchPath = ct2Binary;

    // Extract base model name (e.g., "CT2 small" → "small")
    NSString *baseModel = modelName;
    if ([modelName hasPrefix:@"CT2 "]) {
        baseModel = [modelName substringFromIndex:4];
    }

    // Map model names for CTranslate2 with language-aware .en suffix
    NSString *ct2Model = baseModel;
    BOOL isEnglish = [self.selectedLanguage isEqualToString:@"en"];

    // Auto-append .en suffix for English-only mode when .en model exists
    if (isEnglish) {
        if ([baseModel isEqualToString:@"tiny"] || [baseModel isEqualToString:@"base"] ||
            [baseModel isEqualToString:@"small"] || [baseModel isEqualToString:@"medium"]) {
            ct2Model = [NSString stringWithFormat:@"%@.en", baseModel];
            VTTLog(@"Auto-selected .en model: %@ (English mode)", ct2Model);
        } else if ([baseModel isEqualToString:@"large"]) {
            ct2Model = @"large-v3"; // large-v3 is multilingual, no .en version
        }
        // large-v3-turbo and distil-large-v3 have no .en variants
    } else {
        // Multilingual mode - use base models without .en suffix
        if ([baseModel isEqualToString:@"large"]) {
            ct2Model = @"large-v3";
        } else {
            ct2Model = baseModel; // tiny, base, small, medium (multilingual versions)
        }
    }

    // Map distil-large-v3 to HuggingFace model path
    if ([baseModel isEqualToString:@"distil-large-v3"]) {
        ct2Model = @"distil-whisper/distil-large-v3";
    }

    // whisper-ctranslate2 writes output to a file, not stdout
    // Output file will be: /tmp/vtt_XXX.txt (same name as wav but .txt extension)
    NSString *outputFile = [wavPath stringByReplacingOccurrencesOfString:@".wav" withString:@".txt"];

    // Build arguments with language parameter
    NSMutableArray *arguments = [NSMutableArray arrayWithArray:@[
        wavPath,
        @"--model", ct2Model,
        @"--device", @"cpu",
        @"--compute_type", @"int8",
        @"--output_format", @"txt",
        @"--output_dir", @"/tmp",
        @"--verbose", @"False",
        @"--initial_prompt", self.initialPrompt
    ]];

    // Add language parameter: "en" for English-only, omit for multilingual auto-detect
    if (isEnglish) {
        [arguments addObjectsFromArray:@[@"--language", @"en"]];
        VTTLog(@"Using English-only transcription");
    } else {
        VTTLog(@"Using multilingual auto-detection (99 languages)");
    }

    task.arguments = arguments;

    VTTLog(@"Launching whisper-ctranslate2: %@ --model %@ (output: %@)", wavPath, ct2Model, outputFile);

    @try {
        [task launch];
        [task waitUntilExit];

        if (task.terminationStatus != 0) {
            VTTLog(@"❌ whisper-ctranslate2 failed with status %d", task.terminationStatus);
            return nil;
        }

        // Read transcription from output file
        NSError *error = nil;
        NSString *transcription = [NSString stringWithContentsOfFile:outputFile encoding:NSUTF8StringEncoding error:&error];

        if (error || !transcription) {
            VTTLog(@"❌ Failed to read transcription file %@: %@", outputFile, error);
            return nil;
        }

        // Clean up output file
        [[NSFileManager defaultManager] removeItemAtPath:outputFile error:nil];

        transcription = [transcription stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]];
        VTTLog(@"✅ CTranslate2 transcription: %@", transcription);

        return transcription;

    } @catch (NSException *exception) {
        VTTLog(@"❌ Exception launching whisper-ctranslate2: %@", exception.reason);
        return nil;
    }
}

- (BOOL)shouldUseCTranslate2 {
    // Check if user selected a CT2 model (starts with "CT2 ")
    return [self.selectedModel hasPrefix:@"CT2 "];
}

- (void)processAudioFileAtPath:(NSString *)rawPath wasBufferFull:(BOOL)wasBufferFull {
    VTTLog(@"Processing audio file, wasBufferFull=%d", wasBufferFull);

    // Create unique WAV file based on the raw file name (preserves session uniqueness)
    char wavFile[256];
    NSString *rawFileName = [[rawPath lastPathComponent] stringByDeletingPathExtension];
    snprintf(wavFile, sizeof(wavFile), "/tmp/%s.wav", [rawFileName UTF8String]);

    // Read raw PCM (recorded at device sample rate) into memory
    const char *raw_c = [rawPath UTF8String];
    FILE* raw = fopen(raw_c, "rb");
    if (!raw) {
        VTTLog(@"❌ [ERROR] Failed to open raw audio file: %s", raw_c);
        self.statusMenuItem.title = @"Error: Audio file not found";
        dispatch_async(dispatch_get_main_queue(), ^{
            NSAlert *alert = [[NSAlert alloc] init];
            alert.messageText = @"Audio File Error";
            alert.informativeText = @"Failed to open the recorded audio file. It may have been deleted.";
            alert.alertStyle = NSAlertStyleWarning;
            [alert addButtonWithTitle:@"OK"];
            [alert runModal];
        });
        return;
    }

    fseek(raw, 0, SEEK_END);
    long bytesRaw = ftell(raw);
    fseek(raw, 0, SEEK_SET);

    size_t n_in = (size_t)bytesRaw / sizeof(int16_t);
    int16_t *in_pcm = (int16_t *)malloc(n_in * sizeof(int16_t));
    size_t rd = fread(in_pcm, sizeof(int16_t), n_in, raw);
    fclose(raw);

    // Resample to 16 kHz mono for Whisper
    double in_rate = self.audioState->format.mSampleRate;
    VTTLog(@"Resampling from %.0f Hz (%zu samples) to 16000 Hz", in_rate, n_in);
    int16_t *out_pcm = NULL; size_t n_out = 0;
    resample_linear_i16_mono(in_pcm, n_in, in_rate, 16000.0, &out_pcm, &n_out);
    free(in_pcm);
    if (!out_pcm || n_out == 0) {
        VTTLog(@"❌ [ERROR] Resampler failed (out_pcm=%p, n_out=%zu)", out_pcm, n_out);
        return;
    }
    VTTLog(@"✅ Resampled to %zu samples (%.2f seconds)", n_out, (float)n_out / 16000.0f);

    // Check if audio has actual content (not all zeros)
    int32_t sum = 0;
    int32_t max_val = 0;
    for (size_t i = 0; i < MIN(n_out, 1000); i++) {
        sum += abs(out_pcm[i]);
        if (abs(out_pcm[i]) > max_val) max_val = abs(out_pcm[i]);
    }
    int32_t avg = sum / MIN(n_out, 1000);
    VTTLog(@"Audio stats: avg=%d, max=%d (first 1000 samples)", avg, max_val);

    if (max_val == 0) {
        VTTLog(@"⚠️  WARNING: Audio appears to be silent (all zeros)");
    }

    // Check duration and amplitude thresholds
    float duration = (float)n_out / 16000.0f;
    NSString *rejectionMessage = nil;

    if (duration < 0.5f) {
        VTTLog(@"⚠️  Recording too short (%.2f seconds) - rejecting", duration);
        rejectionMessage = @"[Transcription activated: audio too short]";
    } else if (max_val < 100 && !wasBufferFull) {
        // Lower threshold (100) for quieter internal microphones
        VTTLog(@"⚠️  Audio too quiet (amplitude %d) - rejecting", max_val);
        rejectionMessage = @"[Transcription activated: no audio detected]";
    } else if (wasBufferFull && max_val < 100) {
        VTTLog(@"⚠️  Audio appears silent but max length was reached - attempting transcription anyway");
    }

    // If rejected, type rejection message and return
    if (rejectionMessage) {
        dispatch_semaphore_t sema = dispatch_semaphore_create(0);
        dispatch_async(dispatch_get_main_queue(), ^{
            [self typeText:rejectionMessage];
            dispatch_semaphore_signal(sema);
        });
        dispatch_semaphore_wait(sema, DISPATCH_TIME_FOREVER);

        // Clean up and return
        [[NSFileManager defaultManager] removeItemAtPath:rawPath error:nil];
        free(out_pcm);
        return;
    }

    // Write WAV @ 16 kHz
    FILE* wav = fopen(wavFile, "wb");
    int32_t dataSize = (int32_t)(n_out * sizeof(int16_t));
    fwrite("RIFF", 1, 4, wav);
    int32_t chunkSize = dataSize + 36; fwrite(&chunkSize, 4, 1, wav);
    fwrite("WAVEfmt ", 1, 8, wav);
    int32_t subchunk1Size = 16; fwrite(&subchunk1Size, 4, 1, wav);
    int16_t audioFormat = 1; fwrite(&audioFormat, 2, 1, wav);
    int16_t numChannels = CHANNELS; fwrite(&numChannels, 2, 1, wav);
    int32_t sampleRate = 16000; fwrite(&sampleRate, 4, 1, wav);
    int32_t byteRate = 16000 * CHANNELS * 2; fwrite(&byteRate, 4, 1, wav);
    int16_t blockAlign = CHANNELS * 2; fwrite(&blockAlign, 2, 1, wav);
    int16_t bitsPerSample = 16; fwrite(&bitsPerSample, 2, 1, wav);
    fwrite("data", 1, 4, wav);
    fwrite(&dataSize, 4, 1, wav);
    fwrite(out_pcm, sizeof(int16_t), n_out, wav);
    fclose(wav);

    // Determine backend: CTranslate2 for CT2-prefixed models, whisper.cpp for others
    if ([self shouldUseCTranslate2]) {
        VTTLog(@"Using CTranslate2 backend for model: %@", self.selectedModel);
        NSString *transcription = [self transcribeWithCTranslate2:@(wavFile) model:self.selectedModel];
        if (transcription && transcription.length > 0) {
            VTTLog(@"CTranslate2 transcription: %@", transcription);

            // Type text directly (with truncation indicator and voice prefix)
            NSString *textToType;
            if (wasBufferFull) {
                // Add truncation indicator before voice prefix (matches Linux behavior)
                textToType = [NSString stringWithFormat:@"[Truncated - %ds limit] %@%@",
                              MAX_RECORDING_SECONDS, self.voicePrefix ?: @"", transcription];
                VTTLog(@"📋 Truncation: YES - Full text: %@", textToType);
            } else {
                textToType = [NSString stringWithFormat:@"%@%@", self.voicePrefix ?: @"", transcription];
                VTTLog(@"📋 Truncation: NO - Full text: %@", textToType);
            }

            // If waiting for key release (max length reached), poll until key is released
            if (self.waitingForKeyRelease) {
                VTTLog(@"⏳ Transcription complete - waiting for PTT key release before typing");
                // Poll every 100ms until key is released
                while (self.waitingForKeyRelease) {
                    usleep(100000); // 100ms
                }
                VTTLog(@"✅ PTT key released - now typing");
            }

            // Type text directly on main queue (no clipboard, no delays needed)
            dispatch_semaphore_t sema = dispatch_semaphore_create(0);
            dispatch_async(dispatch_get_main_queue(), ^{
                [self typeText:textToType];
                dispatch_semaphore_signal(sema);
            });
            dispatch_semaphore_wait(sema, DISPATCH_TIME_FOREVER);

            // Clean up temp files
            [[NSFileManager defaultManager] removeItemAtPath:@(wavFile) error:nil];
            [[NSFileManager defaultManager] removeItemAtPath:rawPath error:nil];

            free(out_pcm);
            return;
        } else {
            VTTLog(@"⚠️  CTranslate2 failed, falling back to whisper.cpp");
        }
    }

    // Run whisper with selected model - check bundled model first, then external
    // Language-aware model selection: use .en models for English, base models for multilingual
    NSString *modelFile = self.selectedModel;
    NSString *extension;
    BOOL isEnglish = [self.selectedLanguage isEqualToString:@"en"];

    if ([self.selectedModel isEqualToString:@"large"]) {
        modelFile = @"large-v3";
        extension = @"bin"; // large-v3 is multilingual only
    } else if (isEnglish) {
        // English mode: use .en models (faster, better quality for English)
        extension = @"en.bin";
        VTTLog(@"Using English-only model: %@.%@", modelFile, extension);
    } else {
        // Multilingual mode: use base models without .en suffix
        extension = @"bin";
        VTTLog(@"Using multilingual model: %@.%@", modelFile, extension);
    }

    NSString *homeDir = NSHomeDirectory();
    NSString *bundledModelPath = [[[NSBundle mainBundle] resourcePath] stringByAppendingPathComponent:[NSString stringWithFormat:@"ggml-%@.%@", modelFile, extension]];
    NSString *externalModelPath = [NSString stringWithFormat:@"%@/whisper.cpp/models/ggml-%@.%@", homeDir, modelFile, extension];

    // Check for model: bundled first, then external location
    NSString *modelPath = nil;
    if ([[NSFileManager defaultManager] fileExistsAtPath:bundledModelPath]) {
        modelPath = bundledModelPath;
        VTTLog(@"Using bundled model: %@", modelPath);
    } else if ([[NSFileManager defaultManager] fileExistsAtPath:externalModelPath]) {
        modelPath = externalModelPath;
        VTTLog(@"Using external model: %@", modelPath);
    } else {
        VTTLog(@"Model not found in bundled or external locations");
        self.statusMenuItem.title = [NSString stringWithFormat:@"Model %@ not found", self.selectedModel];
        dispatch_async(dispatch_get_main_queue(), ^{
            NSAlert *alert = [[NSAlert alloc] init];
            alert.messageText = @"Model Not Found";
            alert.informativeText = [NSString stringWithFormat:@"Whisper model '%@' is not installed.\n\nPlease select a model from the Voice to Text menu bar to download it automatically.", self.selectedModel];
            alert.alertStyle = NSAlertStyleWarning;
            [alert addButtonWithTitle:@"OK"];
            [alert runModal];
        });
        // Clean up temp files and memory before returning
        [[NSFileManager defaultManager] removeItemAtPath:rawPath error:nil];
        [[NSFileManager defaultManager] removeItemAtPath:@(wavFile) error:nil];
        free(out_pcm);
        return;
    }

    // Try to locate whisper-cli in multiple known locations
    NSString *bundledBinDir = [[[NSBundle mainBundle] executablePath] stringByDeletingLastPathComponent];
    NSString *bundledWhisper = [bundledBinDir stringByAppendingPathComponent:@"whisper-cli"];
    NSArray<NSString *> *candidateCLIs = @[
        bundledWhisper,
        [homeDir stringByAppendingPathComponent:@"whisper.cpp/build/bin/whisper-cli"],
        @"/opt/homebrew/bin/whisper-cli",
        @"/opt/homebrew/bin/whisper-cpp",
        @"/usr/local/bin/whisper-cli",
        @"/usr/local/bin/whisper-cpp"
    ];

    NSString *whisperCli = nil;
    NSFileManager *fm = [NSFileManager defaultManager];
    for (NSString *path in candidateCLIs) {
        if ([fm isExecutableFileAtPath:path]) { whisperCli = path; break; }
    }

    if (!whisperCli) {
        // Last resort: try `which` for whisper-cli / whisper-cpp
        NSArray<NSString *> *whichCandidates = @[@"whisper-cli", @"whisper-cpp"]; 
        for (NSString *name in whichCandidates) {
            NSTask *whichTask = [[NSTask alloc] init];
            whichTask.launchPath = @"/usr/bin/which";
            whichTask.arguments = @[name];
            NSPipe *pipe = [NSPipe pipe];
            whichTask.standardOutput = pipe;
            @try { [whichTask launch]; [whichTask waitUntilExit]; } @catch(NSException *e) {}
            if (whichTask.terminationStatus == 0) {
                NSData *d = [[pipe fileHandleForReading] readDataToEndOfFile];
                NSString *found = [[[NSString alloc] initWithData:d encoding:NSUTF8StringEncoding] stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]];
                if (found.length && [fm isExecutableFileAtPath:found]) { whisperCli = found; break; }
            }
        }
    }

    if (!whisperCli) {
        VTTLog(@"whisper-cli not found in common locations");
        self.statusMenuItem.title = @"whisper-cli not installed";
        self.statusItem.button.title = @"VTT ⚠️";
        dispatch_async(dispatch_get_main_queue(), ^{
            NSAlert *alert = [[NSAlert alloc] init];
            alert.messageText = @"Whisper Not Installed";
            alert.informativeText = @"whisper-cli was not found on your system.\n\nPlease install whisper.cpp:\n  brew install whisper-cpp\n\nOr build from source and ensure whisper-cli is in your PATH.";
            alert.alertStyle = NSAlertStyleCritical;
            [alert addButtonWithTitle:@"OK"];
            [alert runModal];
        });
        // Clean up temp files and memory before returning
        [[NSFileManager defaultManager] removeItemAtPath:rawPath error:nil];
        [[NSFileManager defaultManager] removeItemAtPath:@(wavFile) error:nil];
        free(out_pcm);
        return;
    }

    NSMutableString *transcription = [NSMutableString string];

#ifdef USE_WHISPER_LIB
    VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    VTTLog(@"🔊 WHISPER TRANSCRIPTION START");
    VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    VTTLog(@"Using embedded whisper library (preloaded=%d)", self.wctx != NULL);
    VTTLog(@"Model: %@", self.selectedModel);
    VTTLog(@"Context pointer: %p", self.wctx);

    if (!self.wctx) {
        VTTLog(@"❌ Whisper context is NULL - model not loaded!");
        VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        dispatch_async(dispatch_get_main_queue(), ^{
            NSAlert *alert = [[NSAlert alloc] init];
            alert.messageText = @"Model Not Loaded";
            alert.informativeText = @"Whisper model context is not initialized.\n\nTry selecting the model again from the menu.";
            alert.alertStyle = NSAlertStyleCritical;
            [alert addButtonWithTitle:@"OK"];
            [alert runModal];
        });
    } else {
        VTTLog(@"✅ Context is valid, proceeding with transcription");

        // Convert to float [-1,1]
        VTTLog(@"Converting %zu int16 samples to float...", n_out);
        float *fsamples = (float *)malloc(n_out * sizeof(float));
        if (!fsamples) {
            VTTLog(@"❌ Failed to allocate float buffer for %zu samples", n_out);
            VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            return;
        }

        for (size_t i = 0; i < n_out; ++i) fsamples[i] = (float)out_pcm[i] / 32768.0f;

        // Calculate audio statistics
        float min_val = 0.0f, max_val = 0.0f, sum = 0.0f;
        for (size_t i = 0; i < n_out; i++) {
            if (fsamples[i] < min_val) min_val = fsamples[i];
            if (fsamples[i] > max_val) max_val = fsamples[i];
            sum += fabsf(fsamples[i]);
        }
        float avg = sum / n_out;
        float peak = MAX(fabsf(min_val), fabsf(max_val));

        VTTLog(@"Original audio stats: min=%.4f, max=%.4f, avg=%.4f, peak=%.4f", min_val, max_val, avg, peak);

        // Volume normalization - target peak at 0.7 to avoid clipping
        if (peak > 0.01f && peak < 0.95f) {
            float gain = 0.7f / peak;
            VTTLog(@"Normalizing volume with gain: %.2fx", gain);
            for (size_t i = 0; i < n_out; i++) {
                fsamples[i] *= gain;
            }
        } else if (peak <= 0.01f) {
            VTTLog(@"⚠️  Audio is very quiet (peak=%.4f), skipping normalization", peak);
        }

        // Trim silence from start and end (threshold: 0.02 = ~1% of full scale)
        const float silence_threshold = 0.02f;
        size_t start_idx = 0;
        size_t end_idx = n_out - 1;

        // Find first non-silent sample
        for (size_t i = 0; i < n_out; i++) {
            if (fabsf(fsamples[i]) > silence_threshold) {
                start_idx = i > 800 ? i - 800 : 0; // Keep 50ms lead-in (16000/sec * 0.05)
                break;
            }
        }

        // Find last non-silent sample
        for (size_t i = n_out - 1; i > start_idx; i--) {
            if (fabsf(fsamples[i]) > silence_threshold) {
                end_idx = MIN(i + 800, n_out - 1); // Keep 50ms tail (16000/sec * 0.05)
                break;
            }
        }

        size_t trimmed_len = end_idx - start_idx + 1;
        if (start_idx > 0 || end_idx < n_out - 1) {
            VTTLog(@"Trimming silence: %zu → %zu samples (removed %.2fs)",
                   n_out, trimmed_len, (n_out - trimmed_len) / 16000.0f);
            memmove(fsamples, fsamples + start_idx, trimmed_len * sizeof(float));
            n_out = trimmed_len;
        }

        VTTLog(@"Calling whisper_full with %zu samples (%.2f seconds)", n_out, (float)n_out / 16000.0f);

        struct whisper_full_params params = whisper_full_default_params(WHISPER_SAMPLING_GREEDY);
        params.print_progress = false;
        params.print_realtime = false;
        params.print_timestamps = false; // we only need text
        params.translate = false;
        params.no_context = true;
        params.single_segment = false;

        // Set language based on user preference
        BOOL isEnglish = [self.selectedLanguage isEqualToString:@"en"];
        params.language = isEnglish ? "en" : "auto";
        params.initial_prompt = [self.initialPrompt UTF8String];

        // Dynamic thread count based on CPU architecture
        int nth;
#if defined(__arm64__) || defined(__aarch64__)
        // Apple Silicon - GPU (Metal) does heavy lifting, fewer threads for pre/post processing
        nth = 4;
        VTTLog(@"Apple Silicon detected - using %d threads (GPU accelerated)", nth);
#else
        // Intel - No Metal GPU, use more CPU threads (physical cores)
        NSInteger physicalCores = [[NSProcessInfo processInfo] activeProcessorCount] / 2;
        nth = (int)MAX(4, physicalCores);
        VTTLog(@"Intel CPU detected - using %d threads (CPU only)", nth);
#endif
        params.n_threads = nth;

        VTTLog(@"Whisper params:");
        VTTLog(@"  - threads: %d", nth);
        VTTLog(@"  - language: %s", params.language);
        VTTLog(@"  - sampling: GREEDY");
        VTTLog(@"  - no_context: true");
        VTTLog(@"Invoking whisper_full(ctx=%p, params, samples=%p, n=%d)...", self.wctx, fsamples, (int)n_out);

        int rc = whisper_full(self.wctx, params, fsamples, (int)n_out);

        VTTLog(@"whisper_full returned: %d", rc);

        // If Metal GPU encoder fails (error -6), reload model with CPU and retry
        if (rc == -6) {
            VTTLog(@"⚠️  Metal GPU encoder failed (error -6), retrying with CPU...");
            free(fsamples);

            // Reload model with CPU-only
            const char *modelPathC = [modelPath UTF8String];
            struct whisper_context_params cparams = whisper_context_default_params();
            cparams.use_gpu = false; // Force CPU mode

            VTTLog(@"Reloading model with CPU-only mode...");
            struct whisper_context *cpu_ctx = whisper_init_from_file_with_params(modelPathC, cparams);

            if (cpu_ctx) {
                VTTLog(@"✅ Reloaded model in CPU mode, retrying transcription...");

                // Re-read and process audio (need to redo since we freed fsamples)
                FILE *f = fopen(wavFile, "rb");
                if (!f) {
                    VTTLog(@"❌ Failed to re-open audio file for CPU retry");
                    whisper_free(cpu_ctx);
                    return;
                }

                fseek(f, 0, SEEK_END);
                long fsize = ftell(f);
                fseek(f, 44, SEEK_SET); // skip WAV header
                long nsamples = (fsize - 44) / 2;

                int16_t *pcm16 = (int16_t *)malloc(nsamples * sizeof(int16_t));
                fread(pcm16, sizeof(int16_t), nsamples, f);
                fclose(f);

                // Convert to float
                float *fsamples_retry = (float *)malloc(nsamples * sizeof(float));
                for (long i = 0; i < nsamples; i++) {
                    fsamples_retry[i] = (float)pcm16[i] / 32768.0f;
                }
                free(pcm16);

                // Retry transcription with CPU context
                rc = whisper_full(cpu_ctx, params, fsamples_retry, (int)nsamples);
                VTTLog(@"CPU retry whisper_full returned: %d", rc);

                if (rc == 0) {
                    VTTLog(@"✅ CPU fallback succeeded!");
                    int nseg = whisper_full_n_segments(cpu_ctx);
                    for (int i = 0; i < nseg; i++) {
                        const char *txt = whisper_full_get_segment_text(cpu_ctx, i);
                        if (txt && txt[0]) {
                            [transcription appendFormat:@"%s\n", txt];
                        }
                    }
                } else {
                    VTTLog(@"❌ CPU fallback also failed with error: %d", rc);
                }

                free(fsamples_retry);
                whisper_free(cpu_ctx);

                if (rc == 0) {
                    // Success with CPU, return transcription
                    VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    goto transcription_complete;
                }
            } else {
                VTTLog(@"❌ Failed to reload model in CPU mode");
            }
        }

        free(fsamples);

        if (rc != 0) {
            VTTLog(@"❌ ❌ ❌ WHISPER_FULL FAILED ❌ ❌ ❌");
            VTTLog(@"Error code: %d", rc);
            VTTLog(@"Error -6 = Failed to encode (encoder failure)");
            VTTLog(@"Possible causes:");
            VTTLog(@"  1. Model file corrupted or incompatible");
            VTTLog(@"  2. GPU/Metal initialization failed");
            VTTLog(@"  3. Audio format issue (sample rate, channels)");
            VTTLog(@"  4. Insufficient memory");
            VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            dispatch_async(dispatch_get_main_queue(), ^{
                NSAlert *alert = [[NSAlert alloc] init];
                alert.messageText = @"Transcription Failed";
                alert.informativeText = [NSString stringWithFormat:@"Whisper transcription failed with error code %d.\n\nThis usually means:\n• Model file is corrupted\n• GPU/Metal failed to initialize\n• Audio format incompatibility\n\nTry:\n1. Selecting a different model\n2. Re-downloading the model\n3. Checking the logs (Show Logs from menu)", rc];
                alert.alertStyle = NSAlertStyleWarning;
                [alert addButtonWithTitle:@"OK"];
                [alert runModal];
            });
        } else {
            VTTLog(@"✅ whisper_full succeeded!");
            int nseg = whisper_full_n_segments(self.wctx);
            VTTLog(@"Number of segments: %d", nseg);

            for (int i = 0; i < nseg; ++i) {
                const char *txt = whisper_full_get_segment_text(self.wctx, i);
                if (txt && txt[0]) {
                    VTTLog(@"Segment %d: \"%s\"", i, txt);
                    [transcription appendFormat:@"%s\n", txt];
                }
            }

            if (nseg == 0) {
                VTTLog(@"⚠️  No segments returned (might be silence)");
            }
            VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }

transcription_complete:
        ; // Label for CPU fallback goto
    }
#else
    VTTLog(@"Using external whisper binary");
    VTTLog(@"Using whisper binary: %@", whisperCli);

    // Use NSTask to avoid shell-quoting pitfalls and capture output
    NSTask *task = [[NSTask alloc] init];
    task.launchPath = whisperCli;
    task.arguments = @[@"-m", modelPath, @"-f", [NSString stringWithUTF8String:wavFile], @"-np"]; // do not include -nt

    NSPipe *outPipe = [NSPipe pipe];
    task.standardOutput = outPipe;
    task.standardError = outPipe;

    NSFileHandle *readHandle = [outPipe fileHandleForReading];

    @try { [task launch]; } @catch (NSException *exception) {
        VTTLog(@"Failed to launch whisper-cli: %@", exception);
        self.statusItem.button.title = @"VTT ❌";
        self.statusMenuItem.title = @"Status: Whisper launch failed";
        free(out_pcm);
        return;
    }

    NSData *dout = [readHandle readDataToEndOfFile];
    [task waitUntilExit];
    NSString *output = [[NSString alloc] initWithData:dout encoding:NSUTF8StringEncoding];
    VTTLog(@"whisper-cli output (truncated): %@", output.length > 1000 ? [output substringToIndex:1000] : output);

    // Parse whisper output: prefer lines with timestamps [..] then fallback
    NSArray<NSString *> *lines = [output componentsSeparatedByCharactersInSet:[NSCharacterSet newlineCharacterSet]];
    for (NSString *line in lines) {
        NSRange sep = [line rangeOfString:@"] "];
        if (sep.location != NSNotFound && [line containsString:@"["]) {
            NSString *text = [line substringFromIndex:sep.location + sep.length];
            if (text.length) [transcription appendFormat:@"%@\n", text];
        }
    }
    if (transcription.length == 0) {
        // Fallback: try to capture any non-log lines
        for (NSString *line in lines) {
            if (line.length == 0) continue;
            if ([line hasPrefix:@"whisper_"] || [line hasPrefix:@"system_"] || [line hasPrefix:@"main_"]) continue;
            [transcription appendFormat:@"%@\n", line];
        }
    }
#endif

    VTTLog(@"Transcription result: %@", transcription);

    // Trim whitespace
    transcription = [[transcription stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]] mutableCopy];

    // Remove common Whisper artifacts
    NSArray *artifacts = @[@"bot", @"Bot", @"BOT", @"[BLANK_AUDIO]", @"(BLANK_AUDIO)", @"Thank you.", @"Thanks for watching!"];
    for (NSString *artifact in artifacts) {
        // Remove if it's the only word
        if ([transcription isEqualToString:artifact]) {
            transcription = [@"" mutableCopy];
            break;
        }
        // Remove from start
        if ([transcription hasPrefix:[NSString stringWithFormat:@"%@ ", artifact]]) {
            transcription = [[transcription substringFromIndex:artifact.length + 1] mutableCopy];
        }
        if ([transcription hasPrefix:[NSString stringWithFormat:@"%@.", artifact]]) {
            transcription = [[transcription substringFromIndex:artifact.length + 1] mutableCopy];
        }
    }

    // Trim again after artifact removal
    transcription = [[transcription stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]] mutableCopy];

    // Check if transcription is empty or failed
    if (transcription.length == 0) {
        VTTLog(@"Empty transcription - no speech detected or transcription failed");
        dispatch_async(dispatch_get_main_queue(), ^{
            self.statusMenuItem.title = @"Status: No speech detected";
            self.statusItem.button.title = @"VTT";
        });
        free(out_pcm);
        return;
    }

    // Type text directly (with truncation indicator and voice prefix)
    if (transcription.length > 0) {
        NSString *textToType;
        if (wasBufferFull) {
            // Add truncation indicator before voice prefix (matches Linux behavior)
            textToType = [NSString stringWithFormat:@"[Truncated - %ds limit] %@%@",
                          MAX_RECORDING_SECONDS, self.voicePrefix ?: @"", transcription];
            VTTLog(@"📋 Adding truncation prefix (whisper.cpp), textToType length: %lu", (unsigned long)textToType.length);
        } else {
            textToType = [NSString stringWithFormat:@"%@%@", self.voicePrefix ?: @"", transcription];
        }

        // Type text directly on main queue (no clipboard, no delays needed)
        dispatch_semaphore_t sema = dispatch_semaphore_create(0);
        dispatch_async(dispatch_get_main_queue(), ^{
            [self typeText:textToType];
            dispatch_semaphore_signal(sema);
        });
        dispatch_semaphore_wait(sema, DISPATCH_TIME_FOREVER);
    } else {
        VTTLog(@"No transcription text found");
        self.statusItem.button.title = @"VTT ❌";
        self.statusMenuItem.title = @"Status: No text transcribed";
    }

    // Clean up
    unlink(raw_c);
    unlink(wavFile);
    if (out_pcm) free(out_pcm);
}

- (void)checkPermissions:(id)sender {
    BOOL hasAccessibility = AXIsProcessTrusted();
    BOOL hasMicrophone = [self checkMicrophonePermission];

    NSAlert *alert = [[NSAlert alloc] init];

    if (hasAccessibility && hasMicrophone) {
        alert.messageText = @"Permissions Status";
        alert.informativeText = @"✅ Accessibility\n✅ Microphone\n✅ Input Monitoring (assumed)\n\nVoice to Text is ready to use.\n\n"
                               @"🔧 If VTT still doesn't work:\n"
                               @"1. Open System Settings → Privacy & Security\n"
                               @"2. Remove VTT from Microphone, Accessibility, and Input Monitoring\n"
                               @"3. Quit VTT completely\n"
                               @"4. Relaunch VTT and grant all permissions\n"
                               @"5. Restart VTT one more time\n\n"
                               @"This forces macOS to properly register the permissions.";
        [alert addButtonWithTitle:@"OK"];
        [alert addButtonWithTitle:@"Open System Settings"];
        NSModalResponse response = [alert runModal];
        if (response == NSAlertSecondButtonReturn) {
            [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy"]];
        }
    } else {
        alert.messageText = @"Missing Permissions";

        NSMutableString *info = [NSMutableString string];
        [info appendString:@"Voice to Text needs these permissions:\n\n"];

        if (!hasAccessibility) {
            [info appendString:@"❌ Accessibility - for keyboard monitoring and pasting text\n"];
        } else {
            [info appendString:@"✅ Accessibility\n"];
        }

        if (!hasMicrophone) {
            [info appendString:@"❌ Microphone - for recording audio\n"];
        } else {
            [info appendString:@"✅ Microphone\n"];
        }

        [info appendString:@"⚠️  Input Monitoring - for detecting hotkey presses\n\n"];

        [info appendString:@"📋 Setup Steps:\n"];
        [info appendString:@"1. Click 'Open System Settings'\n"];
        [info appendString:@"2. Grant missing permissions to VTT\n"];
        [info appendString:@"3. Restart VTT\n"];
        [info appendString:@"4. Click 'Check Permissions' again\n\n"];

        [info appendString:@"⚠️ If problems persist:\n"];
        [info appendString:@"• Remove VTT from all Privacy settings\n"];
        [info appendString:@"• Quit VTT completely\n"];
        [info appendString:@"• Relaunch and re-grant all permissions\n"];
        [info appendString:@"• Restart VTT one more time"];

        alert.informativeText = info;
        alert.alertStyle = NSAlertStyleWarning;
        [alert addButtonWithTitle:@"Open System Settings"];
        [alert addButtonWithTitle:@"Cancel"];

        NSModalResponse response = [alert runModal];
        if (response == NSAlertFirstButtonReturn) {
            if (!hasAccessibility) {
                [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"]];
            } else if (!hasMicrophone) {
                [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"]];
            } else {
                [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"]];
            }
        }
    }

    // Update status icon based on permission state
    [self updateStatusIcon];
}

- (void)showAbout:(id)sender {
    NSAlert *alert = [[NSAlert alloc] init];
    alert.messageText = @"Voice to Text";
    alert.informativeText = @"Pure C/Objective-C implementation\nHold Right Option to record\nRelease to transcribe";
    [alert addButtonWithTitle:@"OK"];
    [alert runModal];
}

- (void)selectModel:(id)sender {
    NSMenuItem *item = (NSMenuItem *)sender;
    NSString *newModel = item.representedObject; // Get the stored model name (e.g., "small" or "CT2 small")

    // Update UI checkmarks
    for (NSMenuItem *menuItem in item.menu.itemArray) {
        if (!menuItem.isSeparatorItem) {
            menuItem.state = NSControlStateValueOff;
        }
    }
    item.state = NSControlStateValueOn;

    VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    VTTLog(@"🔄 MODEL SWITCH REQUESTED: %@", newModel);
    VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // CT2 models don't need file checks or preloading - they download on-demand
    if ([newModel hasPrefix:@"CT2 "]) {
        VTTLog(@"✅ CT2 model selected (no preload needed)");
        self.selectedModel = newModel;
        self.modelMenuItem.title = [NSString stringWithFormat:@"Model: %@", newModel];

        // Save preference
        [[NSUserDefaults standardUserDefaults] setObject:newModel forKey:@"selectedModel"];

        self.statusMenuItem.title = [NSString stringWithFormat:@"Status: Ready (CT2 %@)", [newModel substringFromIndex:4]];
        self.statusItem.button.title = @"VTT ✅";
        VTTLog(@"Switched to CT2 model: %@", newModel);
        return; // Don't load whisper.cpp model for CT2
    }

    // Whisper.cpp models - need file check and preload
    NSString *baseModel = newModel;
    NSString *modelFile = baseModel;
    NSString *extension;
    BOOL isEnglish = [self.selectedLanguage isEqualToString:@"en"];

    if ([baseModel isEqualToString:@"large"]) {
        modelFile = @"large-v3";
        extension = @"bin"; // large-v3 is multilingual only
    } else if (isEnglish) {
        extension = @"en.bin"; // English-only models
    } else {
        extension = @"bin"; // Multilingual models
    }

    // Check if model exists - bundled first, then external
    NSString *bundledModelPath = [[[NSBundle mainBundle] resourcePath] stringByAppendingPathComponent:[NSString stringWithFormat:@"ggml-%@.%@", modelFile, extension]];
    NSString *externalModelPath = [NSString stringWithFormat:@"%@/whisper.cpp/models/ggml-%@.%@",
                          NSHomeDirectory(), modelFile, extension];

    VTTLog(@"Checking bundled path: %@", bundledModelPath);
    VTTLog(@"Bundled exists: %d", [[NSFileManager defaultManager] fileExistsAtPath:bundledModelPath]);
    VTTLog(@"Checking external path: %@", externalModelPath);
    VTTLog(@"External exists: %d", [[NSFileManager defaultManager] fileExistsAtPath:externalModelPath]);

    if ([[NSFileManager defaultManager] fileExistsAtPath:bundledModelPath] ||
        [[NSFileManager defaultManager] fileExistsAtPath:externalModelPath]) {
        // Model exists, just switch to it
        VTTLog(@"✅ Model file found, switching...");
        self.selectedModel = newModel;
        self.modelMenuItem.title = [NSString stringWithFormat:@"Model: %@", newModel];

        // Save preference
        [[NSUserDefaults standardUserDefaults] setObject:newModel forKey:@"selectedModel"];

        self.statusMenuItem.title = [NSString stringWithFormat:@"Status: Using %@ model", newModel];
        VTTLog(@"Switched to model: %@", newModel);

#ifdef USE_WHISPER_LIB
        // Reload whisper context for new model in background
        // Use transcribeQueue to ensure no transcription is using the old context
        dispatch_async(self.transcribeQueue, ^{
            if (self.wctx) {
                VTTLog(@"Freeing old whisper context before model switch...");
                whisper_free(self.wctx);
                self.wctx = NULL;
            }
        });

        dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
            NSString *bundledModelPath = [[[NSBundle mainBundle] resourcePath] stringByAppendingPathComponent:[NSString stringWithFormat:@"ggml-%@.%@", modelFile, extension]];
            NSString *externalModelPath = [NSString stringWithFormat:@"%@/whisper.cpp/models/ggml-%@.%@", NSHomeDirectory(), modelFile, extension];
            NSString *modelPath = [[NSFileManager defaultManager] fileExistsAtPath:bundledModelPath] ? bundledModelPath : externalModelPath;
            dispatch_async(dispatch_get_main_queue(), ^{
                self.statusMenuItem.title = [NSString stringWithFormat:@"Status: Loading %@ model…", newModel];
                self.statusItem.button.title = @"VTT ⏳";
            });
            struct whisper_context_params cparams = whisper_context_default_params();
            // Enable Metal acceleration (with automatic CPU fallback if Metal fails)
            cparams.use_gpu = true;
            cparams.flash_attn = true; // Enable Flash Attention for memory efficiency
            const char *mp = [modelPath UTF8String];

            VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            VTTLog(@"📦 LOADING MODEL INTO MEMORY");
            VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            VTTLog(@"Model path: %s", mp);
            VTTLog(@"GPU mode: %d", cparams.use_gpu);

            // Check file size
            struct stat st;
            if (stat(mp, &st) == 0) {
                VTTLog(@"Model file size: %.1f MB", st.st_size / 1024.0 / 1024.0);
            } else {
                VTTLog(@"⚠️  Cannot stat model file: %s", strerror(errno));
            }

            VTTLog(@"Calling whisper_init_from_file_with_params...");
            struct whisper_context *ctx = whisper_init_from_file_with_params(mp, cparams);
            VTTLog(@"whisper_init_from_file_with_params returned: %p", ctx);

            // Fallback to CPU if GPU fails
            if (!ctx && cparams.use_gpu) {
                VTTLog(@"GPU loading failed, trying CPU fallback...");
                cparams.use_gpu = false;
                ctx = whisper_init_from_file_with_params(mp, cparams);
                VTTLog(@"CPU fallback returned: %p", ctx);
            }

            if (!ctx) {
                VTTLog(@"❌ FAILED TO LOAD MODEL - whisper_init returned NULL");
                VTTLog(@"This could mean:");
                VTTLog(@"  - Not enough RAM available");
                VTTLog(@"  - Corrupted model file");
                VTTLog(@"  - Invalid model format");
            } else {
                VTTLog(@"✅ ✅ ✅ MODEL LOADED SUCCESSFULLY ✅ ✅ ✅");
                VTTLog(@"Using: %s", cparams.use_gpu ? "GPU" : "CPU");
                VTTLog(@"Context address: %p", ctx);
            }
            VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            self.wctx = ctx;
            dispatch_async(dispatch_get_main_queue(), ^{
                if (self.wctx) {
                    self.statusMenuItem.title = [NSString stringWithFormat:@"Status: %@ model ready", newModel];
                    self.statusItem.button.title = @"VTT ✅";
                } else {
                    self.statusMenuItem.title = @"Status: Whisper init failed";
                    self.statusItem.button.title = @"VTT ⚠️";
                }
            });
        });
#endif
    } else {
        // Need to download model
        VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        VTTLog(@"📥 MODEL DOWNLOAD START");
        VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        VTTLog(@"Model: %@", newModel);
        VTTLog(@"Target path: %@", externalModelPath);

        self.statusMenuItem.title = [NSString stringWithFormat:@"Downloading %@...", newModel];
        self.statusItem.button.title = @"VTT ⏬";

        // Download with curl showing progress
        NSString *downloadURL = [NSString stringWithFormat:
            @"https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-%@.%@", modelFile, extension];

        VTTLog(@"Download URL: %@", downloadURL);

        // Ensure directory exists
        NSString *modelsDir = [externalModelPath stringByDeletingLastPathComponent];
        [[NSFileManager defaultManager] createDirectoryAtPath:modelsDir withIntermediateDirectories:YES attributes:nil error:nil];
        VTTLog(@"Models directory: %@", modelsDir);

        // Check for partial download and enable resume
        NSString *partialPath = [externalModelPath stringByAppendingString:@".part"];
        BOOL resuming = NO;
        unsigned long long existingSize = 0;

        if ([[NSFileManager defaultManager] fileExistsAtPath:partialPath]) {
            NSDictionary *attrs = [[NSFileManager defaultManager] attributesOfItemAtPath:partialPath error:nil];
            if (attrs) {
                existingSize = [attrs fileSize];
                if (existingSize > 0) {
                    resuming = YES;
                    VTTLog(@"📦 Found partial download: %.1f MB - resuming", existingSize / 1024.0 / 1024.0);
                }
            }
        }

        self.downloadTask = [[NSTask alloc] init];
        self.downloadTask.launchPath = @"/usr/bin/curl";

        NSMutableArray *args = [@[@"-L", @"--fail", @"-#"] mutableCopy];

        // Add resume capability if partial file exists
        if (resuming) {
            [args addObjectsFromArray:@[@"-C", @"-"]]; // Continue from where we left off
            [args addObjectsFromArray:@[@"-o", partialPath]];
        } else {
            [args addObjectsFromArray:@[@"-o", partialPath]];
        }

        [args addObject:downloadURL];
        self.downloadTask.arguments = args;

        VTTLog(@"Executing: %@ %@", self.downloadTask.launchPath, [self.downloadTask.arguments componentsJoinedByString:@" "]);

        // Capture progress output
        NSPipe *pipe = [NSPipe pipe];
        self.downloadTask.standardError = pipe;

        // Monitor download progress
        NSFileHandle *file = [pipe fileHandleForReading];

        [[NSNotificationCenter defaultCenter] addObserverForName:NSFileHandleReadCompletionNotification
                                                          object:file
                                                           queue:nil
                                                      usingBlock:^(NSNotification *note) {
            NSData *data = note.userInfo[NSFileHandleNotificationDataItem];
            if (data.length > 0) {
                NSString *output = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];

                // Parse curl progress (looks for ###% pattern)
                NSRange range = [output rangeOfString:@"\\d+\\.\\d+%"
                                              options:NSRegularExpressionSearch];
                if (range.location != NSNotFound) {
                    NSString *percent = [output substringWithRange:range];
                    dispatch_async(dispatch_get_main_queue(), ^{
                        self.statusMenuItem.title = [NSString stringWithFormat:@"Downloading %@: %@", newModel, percent];

                        // Send notification at 25%, 50%, 75%, 100%
                        float percentValue = [percent floatValue];
                        static float lastNotified = 0;

                        // Reset progress tracking for new downloads (detect when progress goes backwards)
                        if (percentValue < lastNotified) {
                            lastNotified = 0;
                        }

                        if ((percentValue >= 25.0 && lastNotified < 25.0) ||
                            (percentValue >= 50.0 && lastNotified < 50.0) ||
                            (percentValue >= 75.0 && lastNotified < 75.0) ||
                            (percentValue >= 99.0 && lastNotified < 99.0)) {

                            NSUserNotification *notification = [[NSUserNotification alloc] init];
                            notification.title = @"VTT Model Download";
                            notification.informativeText = [NSString stringWithFormat:@"%@ model: %@", newModel, percent];
                            notification.soundName = nil;
                            [[NSUserNotificationCenter defaultUserNotificationCenter] deliverNotification:notification];

                            lastNotified = percentValue;
                        }
                    });
                }

                [file readInBackgroundAndNotify];
            }
        }];

        [file readInBackgroundAndNotify];

        __weak __typeof(self) weakSelf = self;
        self.downloadTask.terminationHandler = ^(NSTask *task) {
            dispatch_async(dispatch_get_main_queue(), ^{
                __strong __typeof(weakSelf) strongSelf = weakSelf;
                VTTLog(@"Download task terminated with status: %d", task.terminationStatus);

                if (task.terminationStatus == 0) {
                    // Move .part file to final location
                    NSError *moveError = nil;
                    [[NSFileManager defaultManager] removeItemAtPath:externalModelPath error:nil]; // Remove old file if exists
                    [[NSFileManager defaultManager] moveItemAtPath:partialPath toPath:externalModelPath error:&moveError];

                    if (moveError) {
                        VTTLog(@"❌ Failed to move downloaded file: %@", moveError);
                        strongSelf.statusMenuItem.title = @"Download failed: File move error";
                        strongSelf.statusItem.button.title = @"VTT ❌";
                        return;
                    }

                    // Check if file exists and has reasonable size
                    NSError *error = nil;
                    NSDictionary *attrs = [[NSFileManager defaultManager] attributesOfItemAtPath:externalModelPath error:&error];
                    if (attrs) {
                        unsigned long long fileSize = [attrs fileSize];
                        VTTLog(@"✅ Download successful! File size: %.1f MB", fileSize / 1024.0 / 1024.0);

                        if (fileSize < 1024 * 1024) {  // Less than 1MB is suspicious
                            VTTLog(@"⚠️  Warning: Downloaded file seems too small (%.1f MB)", fileSize / 1024.0 / 1024.0);
                        }

                        // Verify it's a binary file, not HTML
                        NSData *header = [NSData dataWithContentsOfFile:externalModelPath options:NSDataReadingMappedIfSafe error:nil];
                        if (header.length > 5) {
                            const unsigned char *bytes = (const unsigned char *)header.bytes;
                            // Check if it starts with "GGUF" or "ggml" (whisper model magic bytes)
                            BOOL isValid = (strncmp((const char *)bytes, "GGUF", 4) == 0 ||
                                          strncmp((const char *)bytes, "ggml", 4) == 0);

                            if (!isValid) {
                                VTTLog(@"❌ Downloaded file is not a valid GGML model (may be HTML error page)");
                                VTTLog(@"First bytes: %02x %02x %02x %02x", bytes[0], bytes[1], bytes[2], bytes[3]);
                                // Delete corrupted files
                                [[NSFileManager defaultManager] removeItemAtPath:externalModelPath error:nil];
                                [[NSFileManager defaultManager] removeItemAtPath:partialPath error:nil];

                                strongSelf.statusMenuItem.title = @"Download failed: Invalid file format";
                                strongSelf.statusItem.button.title = @"VTT ❌";
                                return;
                            }
                        }

                        // Clean up .part file on success
                        [[NSFileManager defaultManager] removeItemAtPath:partialPath error:nil];
                    } else {
                        VTTLog(@"❌ Downloaded file not found: %@", error);
                    }

                    // Success notification
                    NSUserNotification *notification = [[NSUserNotification alloc] init];
                    notification.title = @"VTT Model Ready";
                    notification.informativeText = [NSString stringWithFormat:@"%@ model downloaded successfully", newModel];
                    notification.soundName = NSUserNotificationDefaultSoundName;
                    [[NSUserNotificationCenter defaultUserNotificationCenter] deliverNotification:notification];

                    // Success
                    strongSelf.selectedModel = newModel;
                    strongSelf.modelMenuItem.title = [NSString stringWithFormat:@"Model: %@", newModel];
                    strongSelf.statusMenuItem.title = [NSString stringWithFormat:@"Status: %@ model ready", newModel];
                    strongSelf.statusItem.button.title = @"VTT ✅";

                    // Save preference
                    [[NSUserDefaults standardUserDefaults] setObject:newModel forKey:@"selectedModel"];

                    VTTLog(@"Downloaded and switched to model: %@", newModel);
                    VTTLog(@"━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                } else {
                    // Failed - partial file is kept for resume on retry
                    VTTLog(@"❌ Download failed with exit code %d (attempt %ld/3): %@", task.terminationStatus, (long)strongSelf.downloadRetryCount + 1, newModel);
                    VTTLog(@"Partial download saved to: %@", partialPath);

                    if (strongSelf.downloadRetryCount < 3) {
                        strongSelf.downloadRetryCount++;
                        strongSelf.statusMenuItem.title = [NSString stringWithFormat:@"Retrying download (%ld/3)...", (long)strongSelf.downloadRetryCount];

                        // Retry after 2 seconds (will resume from .part file)
                        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 2 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
                            [strongSelf selectModel:sender];
                        });
                    } else {
                        // All retries failed - clean up partial file
                        strongSelf.downloadRetryCount = 0;
                        [[NSFileManager defaultManager] removeItemAtPath:partialPath error:nil];
                        VTTLog(@"All retries exhausted, cleaned up partial file");

                        strongSelf.statusMenuItem.title = @"Status: Download failed";
                        strongSelf.statusItem.button.title = @"VTT ⚠️";

                        NSAlert *alert = [[NSAlert alloc] init];
                        alert.messageText = @"Download Failed";
                        alert.informativeText = [NSString stringWithFormat:@"Failed to download Whisper model '%@' after 3 attempts.\n\nPlease check your internet connection and try again.", newModel];
                        alert.alertStyle = NSAlertStyleCritical;
                        [alert addButtonWithTitle:@"OK"];
                        [alert runModal];

                        // Revert checkmark
                        item.state = NSControlStateValueOff;
                        for (NSMenuItem *menuItem in item.menu.itemArray) {
                            if ([menuItem.title isEqualToString:strongSelf.selectedModel]) {
                                menuItem.state = NSControlStateValueOn;
                                break;
                            }
                        }
                    }
                }
            });
        };

        @try {
            VTTLog(@"Launching download task...");
            [self.downloadTask launch];
            VTTLog(@"Download task launched successfully (PID: %d)", self.downloadTask.processIdentifier);
        } @catch (NSException *exception) {
            VTTLog(@"❌ Failed to launch download task: %@", exception);
            self.statusMenuItem.title = @"Status: Download failed to start";
            self.statusItem.button.title = @"VTT ⚠️";
        }
    }
}

- (void)selectLanguage:(id)sender {
    NSMenuItem *item = (NSMenuItem *)sender;
    NSString *newLanguage = item.representedObject; // "en" or "auto"

    // Update UI checkmarks
    for (NSMenuItem *menuItem in item.menu.itemArray) {
        menuItem.state = NSControlStateValueOff;
    }
    item.state = NSControlStateValueOn;

    // Update language setting
    self.selectedLanguage = newLanguage;
    NSString *languageDisplay = [newLanguage isEqualToString:@"en"] ? @"English only" : @"Multilingual";
    self.languageMenuItem.title = [NSString stringWithFormat:@"Language: %@", languageDisplay];

    // Save preference
    [[NSUserDefaults standardUserDefaults] setObject:newLanguage forKey:@"selectedLanguage"];
    [[NSUserDefaults standardUserDefaults] synchronize];

    VTTLog(@"Language switched to: %@ (%@)", languageDisplay, newLanguage);

    // Rebuild model menu to enable/disable models based on language
    [self rebuildModelMenu];

#ifdef USE_WHISPER_LIB
    // Reload whisper context for new language (requires different model file: .en.bin vs .bin)
    // Free old context on transcribeQueue to ensure no transcription is using it
    if (![self.selectedModel hasPrefix:@"CT2 "]) {
        dispatch_async(self.transcribeQueue, ^{
            if (self.wctx) {
                VTTLog(@"Freeing whisper context after language change...");
                whisper_free(self.wctx);
                self.wctx = NULL;
            }
        });

        // Trigger preload of new model file in background
        dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
            @autoreleasepool {
                NSString *modelFile = self.selectedModel;
                NSString *extension;
                BOOL isEnglish = [newLanguage isEqualToString:@"en"];

                if ([self.selectedModel isEqualToString:@"large"]) {
                    modelFile = @"large-v3";
                    extension = @"bin";
                } else if (isEnglish) {
                    extension = @"en.bin";
                } else {
                    extension = @"bin";
                }

                NSString *bundledModelPath = [[[NSBundle mainBundle] resourcePath] stringByAppendingPathComponent:[NSString stringWithFormat:@"ggml-%@.%@", modelFile, extension]];
                NSString *externalModelPath = [NSString stringWithFormat:@"%@/whisper.cpp/models/ggml-%@.%@", NSHomeDirectory(), modelFile, extension];
                NSString *modelPath = nil;

                if ([[NSFileManager defaultManager] fileExistsAtPath:bundledModelPath]) {
                    modelPath = bundledModelPath;
                } else if ([[NSFileManager defaultManager] fileExistsAtPath:externalModelPath]) {
                    modelPath = externalModelPath;
                }

                if (!modelPath) {
                    VTTLog(@"⚠️ Model file not found for new language: %@.%@", modelFile, extension);
                    dispatch_async(dispatch_get_main_queue(), ^{
                        self.statusMenuItem.title = @"Status: Model not found";
                        self.statusItem.button.title = @"VTT ⚠️";
                    });
                    return;
                }

                VTTLog(@"Reloading model for language change: %@", modelPath);
                dispatch_async(dispatch_get_main_queue(), ^{
                    self.statusMenuItem.title = [NSString stringWithFormat:@"Status: Loading %@ model…", self.selectedModel];
                    self.statusItem.button.title = @"VTT ⏳";
                });

                struct whisper_context_params cparams = whisper_context_default_params();
                cparams.use_gpu = true;
                cparams.flash_attn = true;

                const char *mp = [modelPath UTF8String];
                struct whisper_context *new_ctx = whisper_init_from_file_with_params(mp, cparams);

                if (new_ctx) {
                    VTTLog(@"✅ Whisper context reloaded for new language");
                    dispatch_async(self.transcribeQueue, ^{
                        self.wctx = new_ctx;
                    });

                    dispatch_async(dispatch_get_main_queue(), ^{
                        self.statusMenuItem.title = [NSString stringWithFormat:@"Status: Ready (%@)", languageDisplay];
                        self.statusItem.button.title = @"VTT ✅";
                    });
                } else {
                    VTTLog(@"❌ Failed to reload whisper context for new language");
                    dispatch_async(dispatch_get_main_queue(), ^{
                        self.statusMenuItem.title = @"Status: Model load failed";
                        self.statusItem.button.title = @"VTT ⚠️";
                    });
                }
            }
        });
    }
#endif
}

- (void)rebuildModelMenu {
    NSMenu *modelMenu = self.modelMenuItem.submenu;
    BOOL isEnglish = [self.selectedLanguage isEqualToString:@"en"];
    NSFileManager *fm = [NSFileManager defaultManager];

    VTTLog(@"Rebuilding model menu for language: %@ (isEnglish=%d)", self.selectedLanguage, isEnglish);

    // Update existing menu items instead of rebuilding (to preserve references)
    for (NSMenuItem *item in modelMenu.itemArray) {
        if (item.isSeparatorItem) continue;

        NSString *representedModel = item.representedObject;
        if (!representedModel) continue;

        // CT2 models don't need file checks (downloaded on-demand by faster-whisper)
        // But tiny/base don't support multilingual mode
        if ([representedModel hasPrefix:@"CT2"]) {
            BOOL isCT2TinyOrBase = [representedModel isEqualToString:@"CT2 tiny"] ||
                                    [representedModel isEqualToString:@"CT2 base"];
            BOOL shouldEnable = isEnglish || !isCT2TinyOrBase;
            [item setEnabled:shouldEnable];
            VTTLog(@"  %@ - enabled: %d (CT2 auto-download, isEnglish=%d, isTinyOrBase=%d)",
                   item.title, shouldEnable, isEnglish, isCT2TinyOrBase);
            continue;
        }

        // For W models, check if the required file exists
        NSString *modelFile = representedModel;
        NSString *extension;

        if ([modelFile isEqualToString:@"large"]) {
            modelFile = @"large-v3";
            extension = @"bin"; // large-v3 is multilingual only
        } else if (isEnglish) {
            extension = @"en.bin"; // English-only models
        } else {
            extension = @"bin"; // Multilingual models
        }

        // Check bundled location first, then external
        NSString *bundledPath = [[[NSBundle mainBundle] resourcePath] stringByAppendingPathComponent:
                                 [NSString stringWithFormat:@"ggml-%@.%@", modelFile, extension]];
        NSString *externalPath = [NSString stringWithFormat:@"%@/whisper.cpp/models/ggml-%@.%@",
                                 NSHomeDirectory(), modelFile, extension];

        BOOL fileExists = [fm fileExistsAtPath:bundledPath] || [fm fileExistsAtPath:externalPath];

        // Disable tiny/base in multilingual mode (poor quality) OR if file doesn't exist
        BOOL isTinyOrBase = [modelFile isEqualToString:@"tiny"] || [modelFile isEqualToString:@"base"];
        BOOL shouldEnable = fileExists && (isEnglish || !isTinyOrBase);

        [item setEnabled:shouldEnable];

        VTTLog(@"  %@ (ggml-%@.%@) - enabled: %d (exists=%d, isEnglish=%d, isTinyOrBase=%d)",
               item.title, modelFile, extension, shouldEnable, fileExists, isEnglish, isTinyOrBase);
    }

    // If currently selected model is now disabled, find first enabled model
    NSString *currentModel = self.selectedModel;
    BOOL currentStillEnabled = NO;

    for (NSMenuItem *item in modelMenu.itemArray) {
        if ([item.representedObject isEqualToString:currentModel] && item.enabled) {
            currentStillEnabled = YES;
            break;
        }
    }

    if (!currentStillEnabled) {
        VTTLog(@"⚠️  Current model (%@) no longer valid, finding first enabled model", currentModel);

        // Find first enabled model (prefer CT2 small, then small, then any)
        NSMenuItem *fallbackItem = nil;
        NSMenuItem *preferredItem = nil;

        for (NSMenuItem *item in modelMenu.itemArray) {
            if (item.enabled && item.representedObject) {
                if (!fallbackItem) fallbackItem = item;
                if ([item.representedObject isEqualToString:@"CT2 small"]) {
                    preferredItem = item;
                    break;
                } else if (!preferredItem && [item.representedObject isEqualToString:@"small"]) {
                    preferredItem = item;
                }
            }
        }

        NSMenuItem *selectedItem = preferredItem ?: fallbackItem;
        if (selectedItem) {
            VTTLog(@"Switching to: %@", selectedItem.title);
            [self selectModel:selectedItem];
        } else {
            VTTLog(@"❌ No enabled models found!");
        }
    }
}

- (void)updateMicrophoneDisplay {
    if (!self.micMenuItem) return;

    // Ensure no submenu is attached (remove any existing dropdown)
    [self.micMenuItem setSubmenu:nil];

    // Get the system default input device
    AudioDeviceID deviceID = 0;
    UInt32 size = sizeof(deviceID);
    AudioObjectPropertyAddress addr = {
        kAudioHardwarePropertyDefaultInputDevice,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain
    };

    OSStatus status = AudioObjectGetPropertyData(kAudioObjectSystemObject, &addr, 0, NULL, &size, &deviceID);

    if (status == noErr && deviceID != 0) {
        // Get device name
        AudioObjectPropertyAddress nameAddr = {
            kAudioDevicePropertyDeviceNameCFString,
            kAudioObjectPropertyScopeGlobal,
            kAudioObjectPropertyElementMain
        };
        CFStringRef deviceName = NULL;
        UInt32 nameSize = sizeof(deviceName);

        if (AudioObjectGetPropertyData(deviceID, &nameAddr, 0, NULL, &nameSize, &deviceName) == noErr && deviceName != NULL) {
            NSString *name = (__bridge NSString *)deviceName;
            self.micMenuItem.title = [NSString stringWithFormat:@"Microphone: %@", name];
            CFRelease(deviceName);
        } else {
            self.micMenuItem.title = @"Microphone: Default";
        }
    } else {
        self.micMenuItem.title = @"Microphone: Default";
    }
}

- (void)toggleLogging:(id)sender {
    self.loggingEnabled = !self.loggingEnabled;
    VTTLoggingEnabled = self.loggingEnabled;
    [[NSUserDefaults standardUserDefaults] setBool:self.loggingEnabled forKey:@"loggingEnabled"];
    [[NSUserDefaults standardUserDefaults] synchronize];
    self.loggingToggleItem.state = self.loggingEnabled ? NSControlStateValueOn : NSControlStateValueOff;
    self.loggingToggleItem.title = self.loggingEnabled ? @"Logging: On" : @"Logging: Off";
}

- (void)refreshLogContent {
    NSString *logsDir = [NSTemporaryDirectory() stringByAppendingPathComponent:@"VTT"];
    NSString *logPath = [logsDir stringByAppendingPathComponent:@"vtt.log"];
    NSString *logs = [NSString stringWithContentsOfFile:logPath encoding:NSUTF8StringEncoding error:nil];
    if (!logs) logs = @"(Error reading log file)";

    if (self.logWindow && [self.logWindow isVisible]) {
        NSScrollView *scrollView = [[self.logWindow.contentView subviews] firstObject];
        if ([scrollView isKindOfClass:[NSScrollView class]]) {
            NSTextView *textView = (NSTextView *)scrollView.documentView;
            textView.string = logs;
            [textView scrollRangeToVisible:NSMakeRange(logs.length, 0)];
        }
    }
}

- (void)showLogs:(id)sender {
    // Get log file path
    NSString *logsDir = [NSTemporaryDirectory() stringByAppendingPathComponent:@"VTT"];
    NSString *logPath = [logsDir stringByAppendingPathComponent:@"vtt.log"];

    // Check if log file exists
    if (![[NSFileManager defaultManager] fileExistsAtPath:logPath]) {
        NSAlert *alert = [[NSAlert alloc] init];
        alert.messageText = @"No Logs Yet";
        alert.informativeText = [NSString stringWithFormat:@"Logging is %@.\n\nLogs will appear in:\n%@\n\nTry recording something first, then check logs again.",
                                self.loggingEnabled ? @"enabled" : @"disabled", logPath];
        alert.alertStyle = NSAlertStyleInformational;
        [alert addButtonWithTitle:@"OK"];
        [alert runModal];
        return;
    }

    // Read log file
    NSString *logs = [NSString stringWithContentsOfFile:logPath encoding:NSUTF8StringEncoding error:nil];
    if (!logs) logs = @"(Error reading log file)";

    // Reuse existing window if it exists and is visible
    if (self.logWindow && [self.logWindow isVisible]) {
        [self refreshLogContent];
        [self.logWindow makeKeyAndOrderFront:nil];
        return;
    }

    // Create new log viewer window (regular NSWindow, not NSPanel)
    self.logWindow = [[NSWindow alloc] initWithContentRect:NSMakeRect(0, 0, 800, 600)
                                                  styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable)
                                                    backing:NSBackingStoreBuffered
                                                      defer:NO];
    self.logWindow.title = @"VTT Logs";
    [self.logWindow center];

    // Text view for logs
    NSScrollView *scrollView = [[NSScrollView alloc] initWithFrame:NSMakeRect(0, 50, 800, 550)];
    scrollView.hasVerticalScroller = YES;
    scrollView.hasHorizontalScroller = YES;
    scrollView.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;

    NSTextView *textView = [[NSTextView alloc] initWithFrame:scrollView.bounds];
    textView.string = logs;
    textView.editable = NO;
    textView.selectable = YES;  // Allow text selection
    textView.font = [NSFont fontWithName:@"Monaco" size:11];
    textView.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
    scrollView.documentView = textView;

    [self.logWindow.contentView addSubview:scrollView];

    // Buttons
    NSButton *refreshButton = [[NSButton alloc] initWithFrame:NSMakeRect(20, 10, 100, 32)];
    [refreshButton setButtonType:NSButtonTypeMomentaryPushIn];
    [refreshButton setBezelStyle:NSBezelStyleRounded];
    refreshButton.title = @"Refresh";
    refreshButton.target = self;
    refreshButton.action = @selector(refreshLogContent);
    [self.logWindow.contentView addSubview:refreshButton];

    NSButton *clearButton = [[NSButton alloc] initWithFrame:NSMakeRect(130, 10, 100, 32)];
    [clearButton setButtonType:NSButtonTypeMomentaryPushIn];
    [clearButton setBezelStyle:NSBezelStyleRounded];
    clearButton.title = @"Clear Logs";
    clearButton.target = self;
    clearButton.action = @selector(clearLogs:);
    [self.logWindow.contentView addSubview:clearButton];

    NSButton *copyAllButton = [[NSButton alloc] initWithFrame:NSMakeRect(240, 10, 100, 32)];
    [copyAllButton setButtonType:NSButtonTypeMomentaryPushIn];
    [copyAllButton setBezelStyle:NSBezelStyleRounded];
    copyAllButton.title = @"Copy All";
    [self.logWindow.contentView addSubview:copyAllButton];

    NSButton *copyPathButton = [[NSButton alloc] initWithFrame:NSMakeRect(350, 10, 120, 32)];
    [copyPathButton setButtonType:NSButtonTypeMomentaryPushIn];
    [copyPathButton setBezelStyle:NSBezelStyleRounded];
    copyPathButton.title = @"Copy Path";
    [self.logWindow.contentView addSubview:copyPathButton];

    // Copy all button action
    copyAllButton.target = nil;
    copyAllButton.action = nil;
    __weak NSButton *weakCopyAllButton = copyAllButton;

    // Copy path button action
    copyPathButton.target = nil;
    copyPathButton.action = nil;
    __weak NSButton *weakPathButton = copyPathButton;

    __block id mouseMonitor = [NSEvent addLocalMonitorForEventsMatchingMask:NSEventMaskLeftMouseDown handler:^NSEvent *(NSEvent *event) {
        NSPoint location = [self.logWindow.contentView convertPoint:event.locationInWindow fromView:nil];

        // Copy All button
        if (NSPointInRect(location, weakCopyAllButton.frame)) {
            NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
            [pasteboard clearContents];
            [pasteboard setString:logs forType:NSPasteboardTypeString];

            NSAlert *alert = [[NSAlert alloc] init];
            alert.messageText = @"Logs Copied";
            alert.informativeText = @"All logs have been copied to clipboard.";
            [alert addButtonWithTitle:@"OK"];
            [alert runModal];

            return nil;
        }

        // Copy Path button
        if (NSPointInRect(location, weakPathButton.frame)) {
            NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
            [pasteboard clearContents];
            [pasteboard setString:logPath forType:NSPasteboardTypeString];

            NSAlert *alert = [[NSAlert alloc] init];
            alert.messageText = @"Path Copied";
            alert.informativeText = [NSString stringWithFormat:@"Log file path copied to clipboard:\n\n%@\n\nYou can tail it with:\ntail -f %@", logPath, logPath];
            [alert addButtonWithTitle:@"OK"];
            [alert runModal];

            return nil;
        }
        return event;
    }];

    [self.logWindow makeKeyAndOrderFront:nil];

    // Scroll to bottom
    [textView scrollRangeToVisible:NSMakeRange(logs.length, 0)];
}

- (void)clearLogs:(id)sender {
    NSString *logsDir = [NSTemporaryDirectory() stringByAppendingPathComponent:@"VTT"];
    NSString *logPath = [logsDir stringByAppendingPathComponent:@"vtt.log"];
    [@"" writeToFile:logPath atomically:YES encoding:NSUTF8StringEncoding error:nil];

    NSAlert *alert = [[NSAlert alloc] init];
    alert.messageText = @"Logs Cleared";
    alert.informativeText = @"The log file has been cleared.";
    [alert addButtonWithTitle:@"OK"];
    [alert runModal];
}

- (NSString *)hotkeyNameForCode:(CGKeyCode)code withModifiers:(CGEventFlags)modifiers {
    NSMutableString *result = [NSMutableString string];

    // Add modifiers
    if (modifiers & kCGEventFlagMaskControl) [result appendString:@"⌃"];
    if (modifiers & kCGEventFlagMaskAlternate) [result appendString:@"⌥"];
    if (modifiers & kCGEventFlagMaskShift) [result appendString:@"⇧"];
    if (modifiers & kCGEventFlagMaskCommand) [result appendString:@"⌘"];

    // Add key name
    NSDictionary *keyNames = @{
        @58: @"Left Option",
        @61: @"Right Option",
        @59: @"Left Control",
        @62: @"Right Control",
        @55: @"Left Command",
        @54: @"Right Command",
        @56: @"Left Shift",
        @60: @"Right Shift",
        @63: @"Fn",
        @122: @"F1",
        @120: @"F2",
        @99: @"F3",
        @118: @"F4",
        @96: @"F5",
        @97: @"F6",
        @98: @"F7",
        @100: @"F8",
        @101: @"F9",
        @109: @"F10",
        @103: @"F11",
        @111: @"F12",
        @0: @"A", @11: @"B", @8: @"C", @2: @"D", @14: @"E", @3: @"F",
        @5: @"G", @4: @"H", @34: @"I", @38: @"J", @40: @"K", @37: @"L",
        @46: @"M", @45: @"N", @31: @"O", @35: @"P", @12: @"Q", @15: @"R",
        @1: @"S", @17: @"T", @32: @"U", @9: @"V", @13: @"W", @7: @"X",
        @16: @"Y", @6: @"Z",
        @18: @"1", @19: @"2", @20: @"3", @21: @"4", @22: @"5",
        @23: @"6", @24: @"7", @25: @"8", @26: @"9", @29: @"0",
        @49: @"Space", @36: @"Return", @51: @"Delete", @53: @"Escape",
    };

    NSString *keyName = keyNames[@(code)];
    if (keyName) {
        [result appendString:keyName];
    } else {
        [result appendFormat:@"Key %d", code];
    }

    return [result copy];
}

- (NSString *)hotkeyNameForCode:(CGKeyCode)code {
    return [self hotkeyNameForCode:code withModifiers:self.hotkeyModifiers];
}

- (BOOL)isLoginItem {
    NSString *appPath = [[NSBundle mainBundle] bundlePath];
    LSSharedFileListRef loginItems = LSSharedFileListCreate(NULL, kLSSharedFileListSessionLoginItems, NULL);
    if (!loginItems) return NO;

    UInt32 seed = 0U;
    NSArray *currentLoginItems = (__bridge_transfer NSArray *)LSSharedFileListCopySnapshot(loginItems, &seed);
    BOOL found = NO;

    for (id item in currentLoginItems) {
        LSSharedFileListItemRef itemRef = (__bridge LSSharedFileListItemRef)item;
        CFURLRef url = NULL;
        if (LSSharedFileListItemResolve(itemRef, 0, &url, NULL) == noErr) {
            NSString *itemPath = [(__bridge NSURL *)url path];
            if ([itemPath isEqualToString:appPath]) {
                found = YES;
            }
            if (url) CFRelease(url);
        }
        if (found) break;
    }

    CFRelease(loginItems);
    return found;
}

- (void)toggleStartup:(id)sender {
    NSMenuItem *item = (NSMenuItem *)sender;
    NSString *appPath = [[NSBundle mainBundle] bundlePath];
    NSURL *appURL = [NSURL fileURLWithPath:appPath];

    LSSharedFileListRef loginItems = LSSharedFileListCreate(NULL, kLSSharedFileListSessionLoginItems, NULL);
    if (!loginItems) return;

    if ([self isLoginItem]) {
        // Remove from login items
        UInt32 seed = 0U;
        NSArray *currentLoginItems = (__bridge_transfer NSArray *)LSSharedFileListCopySnapshot(loginItems, &seed);
        for (id currentItem in currentLoginItems) {
            LSSharedFileListItemRef itemRef = (__bridge LSSharedFileListItemRef)currentItem;
            CFURLRef url = NULL;
            if (LSSharedFileListItemResolve(itemRef, 0, &url, NULL) == noErr) {
                NSString *itemPath = [(__bridge NSURL *)url path];
                if ([itemPath isEqualToString:appPath]) {
                    LSSharedFileListItemRemove(loginItems, itemRef);
                }
                if (url) CFRelease(url);
            }
        }
        item.state = NSControlStateValueOff;
        item.title = @"Run on Startup: Off";
    } else {
        // Add to login items
        LSSharedFileListItemRef newItem = LSSharedFileListInsertItemURL(loginItems, kLSSharedFileListItemLast, NULL, NULL, (__bridge CFURLRef)appURL, NULL, NULL);
        if (newItem) CFRelease(newItem);
        item.state = NSControlStateValueOn;
        item.title = @"Run on Startup: On";
    }

    CFRelease(loginItems);
}

- (void)changeHotkey:(id)sender {
    // Create recording window
    NSPanel *panel = [[NSPanel alloc] initWithContentRect:NSMakeRect(0, 0, 400, 180)
                                                styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable)
                                                  backing:NSBackingStoreBuffered
                                                    defer:NO];
    panel.title = @"Record Hotkey";
    panel.level = NSFloatingWindowLevel;
    [panel center];

    // Instruction label
    NSTextField *instructionLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 130, 360, 40)];
    instructionLabel.stringValue = @"Press a key combination or single key\n(e.g., ⌘⇧R, Right Option, F5)";
    instructionLabel.bordered = NO;
    instructionLabel.editable = NO;
    instructionLabel.backgroundColor = [NSColor clearColor];
    instructionLabel.alignment = NSTextAlignmentCenter;
    instructionLabel.font = [NSFont systemFontOfSize:13];
    [panel.contentView addSubview:instructionLabel];

    // Display captured combination
    NSTextField *capturedLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 80, 360, 40)];
    capturedLabel.stringValue = @"Waiting...";
    capturedLabel.bordered = NO;
    capturedLabel.editable = NO;
    capturedLabel.backgroundColor = [NSColor clearColor];
    capturedLabel.alignment = NSTextAlignmentCenter;
    capturedLabel.font = [NSFont boldSystemFontOfSize:18];
    capturedLabel.textColor = [NSColor systemBlueColor];
    [panel.contentView addSubview:capturedLabel];

    // Buttons
    NSButton *cancelButton = [[NSButton alloc] initWithFrame:NSMakeRect(220, 20, 80, 32)];
    [cancelButton setButtonType:NSButtonTypeMomentaryPushIn];
    [cancelButton setBezelStyle:NSBezelStyleRounded];
    cancelButton.title = @"Cancel";
    cancelButton.keyEquivalent = @"\e"; // Escape key
    [panel.contentView addSubview:cancelButton];

    NSButton *confirmButton = [[NSButton alloc] initWithFrame:NSMakeRect(310, 20, 70, 32)];
    [confirmButton setButtonType:NSButtonTypeMomentaryPushIn];
    [confirmButton setBezelStyle:NSBezelStyleRounded];
    confirmButton.title = @"OK";
    confirmButton.keyEquivalent = @"\r"; // Return key
    confirmButton.enabled = NO;
    [panel.contentView addSubview:confirmButton];

    __block CGKeyCode capturedKey = 0;
    __block CGEventFlags capturedModifiers = 0;
    __block BOOL hasCaptured = NO;

    // Event monitor for key capture
    __block id eventMonitor = [NSEvent addLocalMonitorForEventsMatchingMask:(NSEventMaskFlagsChanged | NSEventMaskKeyDown) handler:^NSEvent *(NSEvent *event) {
        if (event.type == NSEventTypeKeyDown) {
            capturedKey = event.keyCode;
            capturedModifiers = event.modifierFlags & (kCGEventFlagMaskCommand | kCGEventFlagMaskAlternate | kCGEventFlagMaskShift | kCGEventFlagMaskControl);
            hasCaptured = YES;
        } else if (event.type == NSEventTypeFlagsChanged) {
            // Detect standalone modifier keys
            CGKeyCode kc = event.keyCode;
            // Only accept standalone modifiers if no other modifiers are pressed
            if (kc == 58 || kc == 61 || kc == 59 || kc == 62 || kc == 55 || kc == 54 || kc == 56 || kc == 60 || kc == 63) {
                capturedKey = kc;
                capturedModifiers = 0; // Standalone modifier
                hasCaptured = YES;
            }
        }

        if (hasCaptured) {
            NSString *displayName = [self hotkeyNameForCode:capturedKey withModifiers:capturedModifiers];
            capturedLabel.stringValue = displayName;
            confirmButton.enabled = YES;
        }

        return event;
    }];

    // Button actions
    __block BOOL confirmed = NO;
    __block BOOL buttonPressed = NO;

    confirmButton.target = nil;
    confirmButton.action = nil;
    [confirmButton setContinuous:NO];

    cancelButton.target = nil;
    cancelButton.action = nil;
    [cancelButton setContinuous:NO];

    __block id mouseMonitor = [NSEvent addLocalMonitorForEventsMatchingMask:NSEventMaskLeftMouseDown handler:^NSEvent *(NSEvent *event) {
        NSPoint location = [panel.contentView convertPoint:event.locationInWindow fromView:nil];
        if (NSPointInRect(location, confirmButton.frame) && hasCaptured) {
            confirmed = YES;
            buttonPressed = YES;
            [NSApp stopModal];
            return nil;
        }
        if (NSPointInRect(location, cancelButton.frame)) {
            confirmed = NO;
            buttonPressed = YES;
            [NSApp stopModal];
            return nil;
        }
        return event;
    }];

    [panel makeKeyAndOrderFront:nil];
    [NSApp runModalForWindow:panel];

    [NSEvent removeMonitor:eventMonitor];
    [NSEvent removeMonitor:mouseMonitor];
    [panel close];

    if (confirmed && hasCaptured) {
        // Save new hotkey
        self.hotkeyCode = capturedKey;
        self.hotkeyModifiers = capturedModifiers;
        [[NSUserDefaults standardUserDefaults] setInteger:self.hotkeyCode forKey:@"hotkeyCode"];
        [[NSUserDefaults standardUserDefaults] setInteger:self.hotkeyModifiers forKey:@"hotkeyModifiers"];
        [[NSUserDefaults standardUserDefaults] synchronize];

        // Update menu
        NSString *hotkeyName = [self hotkeyNameForCode:self.hotkeyCode withModifiers:self.hotkeyModifiers];
        self.hotkeyMenuItem.title = [NSString stringWithFormat:@"Hotkey: %@", hotkeyName];

        // Show confirmation
        NSAlert *confirm = [[NSAlert alloc] init];
        confirm.messageText = @"Hotkey Updated";
        confirm.informativeText = [NSString stringWithFormat:@"Your new hotkey is: %@\n\nHold this combination to record, release to transcribe.", hotkeyName];
        confirm.alertStyle = NSAlertStyleInformational;
        [confirm runModal];
    }
}

- (void)changePrompt:(id)sender {
    // If panel already exists, bring it to front
    if (self.promptPanel && [self.promptPanel isVisible]) {
        [self.promptPanel makeKeyAndOrderFront:nil];
        [NSApp activateIgnoringOtherApps:YES];
        return;
    }

    // Create prompt customization window (taller for two fields)
    NSPanel *panel = [[NSPanel alloc] initWithContentRect:NSMakeRect(0, 0, 500, 340)
                                                styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable)
                                                  backing:NSBackingStoreBuffered
                                                    defer:NO];
    panel.title = @"Customize Transcription Settings";
    panel.level = NSFloatingWindowLevel;
    [panel center];

    // === VOICE PREFIX SECTION ===
    NSTextField *prefixLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 290, 460, 20)];
    prefixLabel.stringValue = @"Voice Prefix (prepended to every transcription):";
    prefixLabel.bordered = NO;
    prefixLabel.editable = NO;
    prefixLabel.backgroundColor = [NSColor clearColor];
    prefixLabel.alignment = NSTextAlignmentLeft;
    prefixLabel.font = [NSFont boldSystemFontOfSize:12];
    [panel.contentView addSubview:prefixLabel];

    NSTextField *prefixField = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 260, 460, 24)];
    prefixField.stringValue = self.voicePrefix ?: @"";
    prefixField.placeholderString = @"e.g., [voice] ";
    prefixField.font = [NSFont systemFontOfSize:13];
    [panel.contentView addSubview:prefixField];

    // === INITIAL PROMPT SECTION ===
    NSTextField *promptLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 225, 460, 20)];
    promptLabel.stringValue = @"Initial Prompt (helps Whisper recognize your voice, max 240 chars):";
    promptLabel.bordered = NO;
    promptLabel.editable = NO;
    promptLabel.backgroundColor = [NSColor clearColor];
    promptLabel.alignment = NSTextAlignmentLeft;
    promptLabel.font = [NSFont boldSystemFontOfSize:12];
    [panel.contentView addSubview:promptLabel];

    NSTextField *textField = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 145, 460, 70)];
    textField.stringValue = self.initialPrompt ?: @"";
    textField.placeholderString = @"e.g., Male American English speaker. Business terminology.";
    textField.font = [NSFont systemFontOfSize:13];
    textField.maximumNumberOfLines = 3;
    [[textField cell] setWraps:YES];
    [[textField cell] setScrollable:NO];
    [panel.contentView addSubview:textField];

    // Character counter
    NSTextField *charCounter = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 115, 460, 20)];
    charCounter.stringValue = [NSString stringWithFormat:@"%lu / 240 characters", (unsigned long)textField.stringValue.length];
    charCounter.bordered = NO;
    charCounter.editable = NO;
    charCounter.backgroundColor = [NSColor clearColor];
    charCounter.alignment = NSTextAlignmentRight;
    charCounter.font = [NSFont systemFontOfSize:11];
    charCounter.textColor = [NSColor secondaryLabelColor];
    [panel.contentView addSubview:charCounter];

    // Update counter on text change
    [[NSNotificationCenter defaultCenter] addObserverForName:NSControlTextDidChangeNotification
                                                      object:textField
                                                       queue:[NSOperationQueue mainQueue]
                                                  usingBlock:^(NSNotification *note) {
        NSTextField *field = note.object;
        NSUInteger len = field.stringValue.length;
        charCounter.stringValue = [NSString stringWithFormat:@"%lu / 240 characters", (unsigned long)len];

        // Limit to 240 characters
        if (len > 240) {
            field.stringValue = [field.stringValue substringToIndex:240];
            NSBeep();
        }

        // Color code: yellow at 200, red at 230+
        if (len >= 230) {
            charCounter.textColor = [NSColor systemRedColor];
        } else if (len >= 200) {
            charCounter.textColor = [NSColor systemOrangeColor];
        } else {
            charCounter.textColor = [NSColor secondaryLabelColor];
        }
    }];

    __weak VTTDaemon *weakSelf = self;
    __weak NSTextField *weakTextField = textField;
    __weak NSTextField *weakPrefixField = prefixField;
    __weak NSPanel *weakPanel = panel;

    // Reset button
    NSButton *resetButton = [[NSButton alloc] initWithFrame:NSMakeRect(20, 20, 120, 32)];
    [resetButton setButtonType:NSButtonTypeMomentaryPushIn];
    [resetButton setBezelStyle:NSBezelStyleRounded];
    resetButton.title = @"Reset Default";
    resetButton.target = weakSelf;
    resetButton.action = @selector(resetPromptDefaults:);
    [panel.contentView addSubview:resetButton];

    // Cancel button
    NSButton *cancelButton = [[NSButton alloc] initWithFrame:NSMakeRect(300, 20, 80, 32)];
    [cancelButton setButtonType:NSButtonTypeMomentaryPushIn];
    [cancelButton setBezelStyle:NSBezelStyleRounded];
    cancelButton.title = @"Cancel";
    cancelButton.keyEquivalent = @"\e"; // Escape key
    cancelButton.target = weakPanel;
    cancelButton.action = @selector(close);
    [panel.contentView addSubview:cancelButton];

    // Save button
    NSButton *saveButton = [[NSButton alloc] initWithFrame:NSMakeRect(390, 20, 80, 32)];
    [saveButton setButtonType:NSButtonTypeMomentaryPushIn];
    [saveButton setBezelStyle:NSBezelStyleRounded];
    saveButton.title = @"Save";
    saveButton.keyEquivalent = @"\r"; // Return key
    saveButton.target = weakSelf;
    saveButton.action = @selector(savePromptSettings:);
    [panel.contentView addSubview:saveButton];

    // Store panel reference and clean up when closed
    self.promptPanel = panel;
    [[NSNotificationCenter defaultCenter] addObserverForName:NSWindowWillCloseNotification
                                                      object:panel
                                                       queue:[NSOperationQueue mainQueue]
                                                  usingBlock:^(NSNotification *note) {
        weakSelf.promptPanel = nil;
    }];

    [panel makeKeyAndOrderFront:nil];
    [panel makeFirstResponder:textField];
    [NSApp activateIgnoringOtherApps:YES];
}

- (void)resetPromptDefaults:(id)sender {
    if (!self.promptPanel) return;

    // Find the text fields in the panel
    NSTextField *promptField = nil;
    NSTextField *prefixField = nil;

    for (NSView *view in self.promptPanel.contentView.subviews) {
        if ([view isKindOfClass:[NSTextField class]]) {
            NSTextField *field = (NSTextField *)view;
            if (field.editable) {
                // Identify by frame position: prefix is higher (y=260), prompt is lower (y=145)
                if (field.frame.origin.y > 200) {
                    prefixField = field;
                } else if (field.frame.origin.y > 100) {
                    promptField = field;
                }
            }
        }
    }

    if (promptField) {
        promptField.stringValue = @"British English, technical context. Git, GitHub, Claude, API, CLI, JSON, YAML, SSH, Docker, TypeScript, Python, Ubuntu, PPA, Launchpad. Powell-Clark, Emmanuel.";
        [[NSNotificationCenter defaultCenter] postNotificationName:NSControlTextDidChangeNotification object:promptField];
    }
    if (prefixField) {
        prefixField.stringValue = @"[voice] ";
    }
}

- (void)savePromptSettings:(id)sender {
    if (!self.promptPanel) return;

    // Find the text fields in the panel
    NSTextField *promptField = nil;
    NSTextField *prefixField = nil;

    for (NSView *view in self.promptPanel.contentView.subviews) {
        if ([view isKindOfClass:[NSTextField class]]) {
            NSTextField *field = (NSTextField *)view;
            if (field.editable) {
                // Identify by frame position: prefix is higher (y=260), prompt is lower (y=145)
                if (field.frame.origin.y > 200) {
                    prefixField = field;
                } else if (field.frame.origin.y > 100) {
                    promptField = field;
                }
            }
        }
    }

    if (promptField) {
        NSString *newPrompt = promptField.stringValue;
        if (newPrompt.length > 240) {
            newPrompt = [newPrompt substringToIndex:240];
        }
        self.initialPrompt = newPrompt;
        [[NSUserDefaults standardUserDefaults] setObject:newPrompt forKey:@"initialPrompt"];

        // Update menu item
        NSString *promptPreview = newPrompt.length > 30 ? [[newPrompt substringToIndex:27] stringByAppendingString:@"..."] : newPrompt;
        self.promptMenuItem.title = [NSString stringWithFormat:@"Prompt: %@", promptPreview];

        VTTLog(@"Updated initial prompt: %@", newPrompt);
    }

    if (prefixField) {
        NSString *newPrefix = prefixField.stringValue;
        self.voicePrefix = newPrefix;
        [[NSUserDefaults standardUserDefaults] setObject:newPrefix forKey:@"voicePrefix"];
        VTTLog(@"Updated voice prefix: %@", newPrefix);
    }

    [[NSUserDefaults standardUserDefaults] synchronize];
    [self.promptPanel close];
}

- (void)quit:(id)sender {
    if (self.audioState) {
        AudioQueueDispose(self.audioState->queue, true);
        free(self.audioState);
    }
#ifdef USE_WHISPER_LIB
    if (self.wctx) { whisper_free(self.wctx); self.wctx = NULL; }
#endif
    [NSApp terminate:nil];
}

@end

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        VTTDaemon *daemon = [[VTTDaemon alloc] init];
        app.delegate = daemon;
        [app run];
    }
    return 0;
}
