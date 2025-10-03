// VTT Daemon - Professional Objective-C implementation
#import <Cocoa/Cocoa.h>
#import <CoreAudio/CoreAudio.h>
#import <AudioToolbox/AudioToolbox.h>
#import <ApplicationServices/ApplicationServices.h>
#import <AVFoundation/AVFoundation.h>
#import <IOKit/hidsystem/ev_keymap.h>
#import "VTTOnboarding.h"
#include <unistd.h>

#ifdef USE_WHISPER_LIB
#include "whisper.h"
#endif

// Logging toggle (default OFF). Use VTTLog instead of NSLog.
static volatile BOOL VTTLoggingEnabled = NO;
#define VTTLog(fmt, ...) do { if (VTTLoggingEnabled) NSLog((fmt), ##__VA_ARGS__); } while (0)

#define SAMPLE_RATE 16000
#define CHANNELS 1
// Smaller buffer = lower callback latency (at 48 kHz, 4096 bytes ~ 42 ms)
#define BUFFER_SIZE 4096

// C struct for audio state (for performance)
typedef struct {
    AudioQueueRef queue;
    AudioStreamBasicDescription format;
    AudioQueueBufferRef buffers[3];
    FILE* audioFile;
    BOOL isRecording;
    char tempFileName[256];
    size_t bytesCaptured;
} AudioState;

@interface VTTDaemon : NSObject <NSApplicationDelegate>
@property (strong) NSStatusItem *statusItem;
@property (nonatomic) AudioState *audioState;
@property (strong) NSMenu *menu;
@property (strong) NSMenuItem *statusMenuItem;
@property (strong) NSString *selectedModel;
@property (strong) NSMenuItem *modelMenuItem;
@property (strong) NSTask *downloadTask;
@property (nonatomic) NSInteger downloadRetryCount;
@property (strong) NSString *downloadingModel;
@property (strong) dispatch_queue_t transcribeQueue;
@property (atomic) NSInteger pendingJobs;
@property (atomic) NSUInteger sessionCounter;
@property (nonatomic) BOOL loggingEnabled;
@property (strong) NSMenuItem *loggingToggleItem;
@property (nonatomic) AudioDeviceID selectedMicrophoneID;
@property (strong) NSMenuItem *micMenuItem;
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

// Pure C audio callback (for speed)
static void audioInputCallback(void* userData,
                              AudioQueueRef queue,
                              AudioQueueBufferRef buffer,
                              const AudioTimeStamp* startTime,
                              UInt32 numPackets,
                              const AudioStreamPacketDescription* packetDesc) {
    AudioState* state = (AudioState*)userData;

    VTTLog(@"🔥 [CALLBACK] Audio callback - bytes: %u, recording: %d, file: %p",
          buffer->mAudioDataByteSize, state->isRecording, state->audioFile);

    if (state->isRecording && state->audioFile) {
        size_t written = fwrite(buffer->mAudioData, 1, buffer->mAudioDataByteSize, state->audioFile);
        state->bytesCaptured += written;
        VTTLog(@"🔥 [CALLBACK] Wrote %zu bytes to file", written);
        fflush(state->audioFile); // Force write to disk
    } else {
        VTTLog(@"🔥 [CALLBACK] Not writing - recording: %d, file: %p", state->isRecording, state->audioFile);
    }

    AudioQueueEnqueueBuffer(queue, buffer, 0, NULL);
}

- (void)applicationDidFinishLaunching:(NSNotification *)notification {
    // Load preferences or set defaults
    NSUserDefaults *defaults = [NSUserDefaults standardUserDefaults];
    self.selectedModel = [defaults stringForKey:@"selectedModel"];
    if (!self.selectedModel) {
        self.selectedModel = @"small";  // Default to small model
        [defaults setObject:self.selectedModel forKey:@"selectedModel"];
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

    // Model selection submenu
    self.modelMenuItem = [[NSMenuItem alloc] initWithTitle:[NSString stringWithFormat:@"Model: %@", self.selectedModel]
                                                     action:nil
                                              keyEquivalent:@""];
    NSMenu *modelMenu = [[NSMenu alloc] init];

    NSArray *models = @[@"tiny", @"base", @"small", @"medium", @"large"];
    for (NSString *model in models) {
        NSMenuItem *item = [[NSMenuItem alloc] initWithTitle:model
                                                       action:@selector(selectModel:)
                                                keyEquivalent:@""];
        item.target = self;
        item.tag = [models indexOfObject:model];
        if ([model isEqualToString:self.selectedModel]) {
            item.state = NSControlStateValueOn;
        }
        [modelMenu addItem:item];
    }

    self.modelMenuItem.submenu = modelMenu;
    [self.menu addItem:self.modelMenuItem];

    // Microphone selection submenu
    self.micMenuItem = [[NSMenuItem alloc] initWithTitle:@"Microphone: Default"
                                                   action:nil
                                            keyEquivalent:@""];
    NSMenu *micMenu = [[NSMenu alloc] init];
    [self populateMicrophoneMenu:micMenu];
    self.micMenuItem.submenu = micMenu;
    [self.menu addItem:self.micMenuItem];

    [self.menu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *permissionsItem = [[NSMenuItem alloc] initWithTitle:@"Check Permissions..."
                                                              action:@selector(checkPermissions:)
                                                       keyEquivalent:@""];
    permissionsItem.target = self;
    [self.menu addItem:permissionsItem];

    // Logging toggle (default OFF)
    self.loggingEnabled = [[NSUserDefaults standardUserDefaults] objectForKey:@"loggingEnabled"] ? [[NSUserDefaults standardUserDefaults] boolForKey:@"loggingEnabled"] : NO;
    VTTLoggingEnabled = self.loggingEnabled;
    NSString *logTitle = self.loggingEnabled ? @"Logging: On" : @"Logging: Off";
    self.loggingToggleItem = [[NSMenuItem alloc] initWithTitle:logTitle
                                                       action:@selector(toggleLogging:)
                                                keyEquivalent:@""];
    self.loggingToggleItem.target = self;
    self.loggingToggleItem.state = self.loggingEnabled ? NSControlStateValueOn : NSControlStateValueOff;
    [self.menu addItem:self.loggingToggleItem];

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
    self.transcribeQueue = dispatch_queue_create("com.local.vtt.transcribe", DISPATCH_QUEUE_SERIAL);
    self.pendingJobs = 0;
    self.sessionCounter = 0;

#ifdef USE_WHISPER_LIB
    // Preload whisper context in background for zero-latency transcription
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
        @autoreleasepool {
            // Resolve model path (bundled preferred)
            NSString *bundledModelPath = [[[NSBundle mainBundle] resourcePath] stringByAppendingPathComponent:[NSString stringWithFormat:@"ggml-%@.en.bin", self.selectedModel]];
            NSString *externalModelPath = [NSString stringWithFormat:@"%@/whisper.cpp/models/ggml-%@.en.bin", NSHomeDirectory(), self.selectedModel];
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

            struct whisper_context_params cparams = whisper_context_default_params();
#if defined(__APPLE__) && defined(__aarch64__)
            cparams.use_gpu = true; // enable Metal on Apple Silicon
#else
            cparams.use_gpu = false;
#endif
            const char *mp = [modelPath UTF8String];
            struct whisper_context *ctx = whisper_init_from_file_with_params(mp, cparams);
            if (!ctx) {
                VTTLog(@"Failed to preload whisper context");
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
#endif

    // Update status
    self.statusItem.button.title = @"VTT ✅";
    self.statusMenuItem.title = @"Status: Ready";
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

    // Determine which device to use based on user preference
    AudioDeviceID deviceID = 0;

    if (self.selectedMicrophoneID != 0) {
        // User has explicitly selected a microphone
        deviceID = self.selectedMicrophoneID;
        VTTLog(@"Using user-selected microphone (ID: %u)", deviceID);
    } else {
        // Use built-in mic if found, otherwise fall back to default
        deviceID = (builtInMicID != 0) ? builtInMicID : 0;

        if (deviceID == 0) {
            // Fallback to default if built-in mic not found
            VTTLog(@"Built-in microphone not found, using default input device");
            UInt32 size = sizeof(deviceID);
            AudioObjectPropertyAddress addr = {
                kAudioHardwarePropertyDefaultInputDevice,
                kAudioObjectPropertyScopeGlobal,
                kAudioObjectPropertyElementMain
            };
            AudioObjectGetPropertyData(kAudioObjectSystemObject, &addr, 0, NULL, &size, &deviceID);
        } else {
            VTTLog(@"Auto-selected built-in microphone (ID: %u)", deviceID);
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

    // Create audio queue
    VTTLog(@"🔥 [INIT] Creating AudioQueue at %.0f Hz...", self.audioState->format.mSampleRate);
    OSStatus qstatus = AudioQueueNewInput(&self.audioState->format,
                                         audioInputCallback,
                                         self.audioState,
                                         NULL,
                                         kCFRunLoopCommonModes,
                                         0,
                                         &self.audioState->queue);
    VTTLog(@"🔥 [INIT] AudioQueueNewInput result: %d", (int)qstatus);

    if (qstatus != noErr) {
        VTTLog(@"Failed to create audio queue: %d", qstatus);
        return;
    }

    // Set the audio queue to use the specific device we selected
    if (deviceID != 0) {
        UInt32 size = sizeof(deviceID);
        OSStatus setStatus = AudioQueueSetProperty(self.audioState->queue,
                                                   kAudioQueueProperty_CurrentDevice,
                                                   &deviceID,
                                                   size);
        if (setStatus == noErr) {
            VTTLog(@"🔥 [INIT] Successfully set AudioQueue to use selected microphone (ID: %u)", deviceID);
        } else {
            VTTLog(@"🔥 [INIT] Failed to set specific device, using default: %d", (int)setStatus);
        }
    }

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

    // Prefer exact key down/up for Right Option (keycode 61 on ANSI)
    CGKeyCode keyCode = (CGKeyCode)CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);
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

    // Primary path for modifier keys: flagsChanged carries keycode for the modifier
    if (type == kCGEventFlagsChanged) {
        CGEventFlags flags = CGEventGetFlags(event);
        CGKeyCode kc = (CGKeyCode)CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);
        BOOL altDown = (flags & kCGEventFlagMaskAlternate) != 0;
        BOOL isRightOption = (kc == (CGKeyCode)61); // ANSI Right Option
        VTTLog(@"Modifier event: flags=%llu keyCode=%u rightOpt=%d altDown=%d", flags, kc, isRightOption, altDown);

        // Start only when Right Option goes down
        if (isRightOption && altDown && !self.audioState->isRecording) {
            VTTLog(@"PTT (flagsChanged) DOWN - starting recording");
            [self startRecording];
            return NULL; // swallow
        }

        // Stop when Alt is no longer down (release of the last Option key)
        if (!altDown && self.audioState->isRecording) {
            VTTLog(@"PTT (flagsChanged) UP - stopping recording");
            [self stopRecording];
            return NULL; // swallow
        }
    }

    // Debug: log other keys when logging is enabled
    if (type == kCGEventKeyDown || type == kCGEventKeyUp) {
        VTTLog(@"Key event: code=%d, type=%s", keyCode, type == kCGEventKeyDown ? "DOWN" : "UP");
    }

    return event;
}

- (void)startRecording {
    VTTLog(@"🔥 [START] startRecording called");

    if (!self.audioState) {
        VTTLog(@"🔥 [ERROR] audioState is NULL!");
        return;
    }

    // Check microphone permission before recording
    AVAuthorizationStatus micStatus = [AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeAudio];
    if (micStatus != AVAuthorizationStatusAuthorized) {
        VTTLog(@"🔥 [ERROR] Microphone permission not granted");
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
        VTTLog(@"🔥 [WARNING] Accessibility permission not granted - paste may fail");
    }

    if (self.audioState->isRecording) {
        VTTLog(@"🔥 [WARNING] Already recording, ignoring start request");
        return;
    }

    // Create unique temp file for this session
    unsigned long long session = ++self.sessionCounter;
    snprintf(self.audioState->tempFileName, sizeof(self.audioState->tempFileName), "/tmp/vtt_%d_%llu.raw", getpid(), session);
    VTTLog(@"🔥 [AUDIO] Temp file path: %s", self.audioState->tempFileName);
    self.audioState->audioFile = fopen(self.audioState->tempFileName, "wb");

    if (!self.audioState->audioFile) {
        VTTLog(@"🔥 [ERROR] Failed to create temp file: %s (errno: %d)", self.audioState->tempFileName, errno);
        return;
    }
    VTTLog(@"🔥 [SUCCESS] Temp file created successfully");

    // Arm recording and start the queue so callbacks deliver audio
    self.audioState->bytesCaptured = 0;
    self.audioState->isRecording = YES;
    // Re-prime the input queue after a previous stop: return any buffers
    // to the queue and enqueue them again so the device can fill them.
    AudioQueueReset(self.audioState->queue);
    for (int i = 0; i < 3; i++) {
        AudioQueueEnqueueBuffer(self.audioState->queue, self.audioState->buffers[i], 0, NULL);
    }
    OSStatus qstart = AudioQueueStart(self.audioState->queue, NULL);
    VTTLog(@"🔥 [AUDIO] AudioQueueStart (PTT) result: %d", (int)qstart);

    // Update UI
    self.statusItem.button.title = @"VTT 🎤";
    self.statusMenuItem.title = @"Status: Recording...";
}

- (void)stopRecording {
    VTTLog(@"🔥 [STOP] stopRecording called");

    if (!self.audioState) {
        VTTLog(@"🔥 [ERROR] audioState is NULL in stopRecording!");
        return;
    }

    if (!self.audioState->isRecording) {
        VTTLog(@"🔥 [WARNING] Not recording, ignoring stop request");
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
    VTTLog(@"🔥 [AUDIO] AudioQueueStop (PTT) result: %d (bytes=%zu)", (int)qstop, self.audioState->bytesCaptured);

    // Now end recording state and close the file
    self.audioState->isRecording = NO;

    if (self.audioState->audioFile) {
        fclose(self.audioState->audioFile);
        self.audioState->audioFile = NULL;
        VTTLog(@"🔥 [SUCCESS] Audio file closed");
    } else {
        VTTLog(@"🔥 [ERROR] No audio file to close!");
    }

    // Capture the just-finished raw file path before new recordings begin
    NSString *rawPath = [NSString stringWithUTF8String:self.audioState->tempFileName];

    // Update UI state and enqueue job (FIFO, no drop)
    dispatch_async(dispatch_get_main_queue(), ^{
        self.statusItem.button.title = @"VTT ⏳";
        self.statusMenuItem.title = @"Status: Processing...";
    });

    dispatch_async(dispatch_get_main_queue(), ^{ self.pendingJobs++; });
    dispatch_async(self.transcribeQueue, ^{
        VTTLog(@"🔥 [QUEUE] Processing job for %@ (pending=%ld)", rawPath, (long)self.pendingJobs);
        [self processAudioFileAtPath:rawPath];
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

- (void)processAudioFileAtPath:(NSString *)rawPath {
    VTTLog(@"🔥 [PROCESS] processAudioFile called");

    char wavFile[256];
    snprintf(wavFile, sizeof(wavFile), "/tmp/vtt_%d.wav", getpid());
    VTTLog(@"🔥 [PROCESS] WAV file path: %s", wavFile);

    // Read raw PCM (recorded at device sample rate) into memory
    const char *raw_c = [rawPath UTF8String];
    FILE* raw = fopen(raw_c, "rb");
    if (!raw) { VTTLog(@"🔥 [ERROR] Cannot open raw file: %@", rawPath); return; }

    fseek(raw, 0, SEEK_END);
    long bytesRaw = ftell(raw);
    fseek(raw, 0, SEEK_SET);
    if (bytesRaw <= 0) { fclose(raw); VTTLog(@"🔥 [ERROR] Raw file empty"); return; }

    size_t n_in = (size_t)bytesRaw / sizeof(int16_t);
    int16_t *in_pcm = (int16_t *)malloc(n_in * sizeof(int16_t));
    if (!in_pcm) { fclose(raw); VTTLog(@"🔥 [ERROR] OOM allocating %zu samples", n_in); return; }
    size_t rd = fread(in_pcm, sizeof(int16_t), n_in, raw);
    fclose(raw);
    if (rd != n_in) { free(in_pcm); VTTLog(@"🔥 [ERROR] Short read: %zu/%zu samples", rd, n_in); return; }

    // Resample to 16 kHz mono for Whisper
    double in_rate = self.audioState->format.mSampleRate;
    int16_t *out_pcm = NULL; size_t n_out = 0;
    resample_linear_i16_mono(in_pcm, n_in, in_rate, 16000.0, &out_pcm, &n_out);
    free(in_pcm);
    if (!out_pcm || n_out == 0) { VTTLog(@"🔥 [ERROR] Resampler failed"); return; }

    // Write WAV @ 16 kHz
    FILE* wav = fopen(wavFile, "wb");
    if (!wav) { free(out_pcm); VTTLog(@"🔥 [ERROR] Cannot create WAV file: %s", wavFile); return; }
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

    // Run whisper with selected model - check bundled model first, then external
    NSString *homeDir = NSHomeDirectory();
    NSString *bundledModelPath = [[[NSBundle mainBundle] resourcePath] stringByAppendingPathComponent:[NSString stringWithFormat:@"ggml-%@.en.bin", self.selectedModel]];
    NSString *externalModelPath = [NSString stringWithFormat:@"%@/whisper.cpp/models/ggml-%@.en.bin", homeDir, self.selectedModel];

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
        return;
    }

    NSMutableString *transcription = [NSMutableString string];

#ifdef USE_WHISPER_LIB
    VTTLog(@"Using embedded whisper library (preloaded=%d)", self.wctx != NULL);
    if (!self.wctx) {
        VTTLog(@"Whisper context not ready; skipping lib path");
    } else {
        // Convert to float [-1,1]
        float *fsamples = (float *)malloc(n_out * sizeof(float));
        for (size_t i = 0; i < n_out; ++i) fsamples[i] = (float)out_pcm[i] / 32768.0f;
        struct whisper_full_params params = whisper_full_default_params(WHISPER_SAMPLING_GREEDY);
        params.print_progress = false;
        params.print_realtime = false;
        params.print_timestamps = false; // we only need text
        params.translate = false;
        params.no_context = true;
        params.single_segment = false;
        params.language = "en";
        // threads: auto-detect
        int nth = (int)MAX(1, (int)sysconf(_SC_NPROCESSORS_ONLN) - 1);
        params.n_threads = nth;
        int rc = whisper_full(self.wctx, params, fsamples, (int)n_out);
        free(fsamples);
        if (rc != 0) {
            VTTLog(@"whisper_full failed: %d", rc);
            dispatch_async(dispatch_get_main_queue(), ^{
                NSAlert *alert = [[NSAlert alloc] init];
                alert.messageText = @"Transcription Failed";
                alert.informativeText = [NSString stringWithFormat:@"Whisper transcription failed with error code %d.\n\nPlease try again or select a different model.", rc];
                alert.alertStyle = NSAlertStyleWarning;
                [alert addButtonWithTitle:@"OK"];
                [alert runModal];
            });
        } else {
            int nseg = whisper_full_n_segments(self.wctx);
            for (int i = 0; i < nseg; ++i) {
                const char *txt = whisper_full_get_segment_text(self.wctx, i);
                if (txt && txt[0]) {
                    [transcription appendFormat:@"%s\n", txt];
                }
            }
        }
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

    // Copy to clipboard and paste
    if (transcription.length > 0) {
        [[NSPasteboard generalPasteboard] clearContents];
        [[NSPasteboard generalPasteboard] setString:transcription forType:NSPasteboardTypeString];

        // Simulate Cmd+V with proper modifier sequence and wait briefly
        // to avoid clipboard races across queued jobs.
        dispatch_semaphore_t sema = dispatch_semaphore_create(0);
        dispatch_async(dispatch_get_main_queue(), ^{
            CGEventSourceRef src = CGEventSourceCreate(kCGEventSourceStateHIDSystemState);
            CGEventRef cmdDown = CGEventCreateKeyboardEvent(src, (CGKeyCode)0x37, true);  // left cmd
            CGEventPost(kCGHIDEventTap, cmdDown);
            CGEventRef vDown = CGEventCreateKeyboardEvent(src, (CGKeyCode)9, true);
            CGEventRef vUp   = CGEventCreateKeyboardEvent(src, (CGKeyCode)9, false);
            CGEventSetFlags(vDown, kCGEventFlagMaskCommand);
            CGEventSetFlags(vUp,   kCGEventFlagMaskCommand);
            CGEventPost(kCGHIDEventTap, vDown);
            CGEventPost(kCGHIDEventTap, vUp);
            CGEventRef cmdUp = CGEventCreateKeyboardEvent(src, (CGKeyCode)0x37, false);
            CGEventPost(kCGHIDEventTap, cmdUp);
            if (src) CFRelease(src);
            CFRelease(cmdDown);
            CFRelease(vDown);
            CFRelease(vUp);
            CFRelease(cmdUp);
            // Signal completion shortly after events are posted
            dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(150 * NSEC_PER_MSEC)), dispatch_get_main_queue(), ^{
                dispatch_semaphore_signal(sema);
            });
        });
        // Block the serial transcribe queue briefly so the next job
        // does not overwrite the clipboard before this paste executes.
        dispatch_time_t tmo = dispatch_time(DISPATCH_TIME_NOW, (int64_t)(300 * NSEC_PER_MSEC));
        (void)dispatch_semaphore_wait(sema, tmo);
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

    if (hasAccessibility && hasMicrophone) {
        NSAlert *alert = [[NSAlert alloc] init];
        alert.messageText = @"Permissions OK";
        alert.informativeText = @"✅ All permissions granted!\n\nVoice to Text is ready to use.";
        [alert addButtonWithTitle:@"Great!"];
        [alert runModal];
    } else {
        [self showPermissionDialog:hasAccessibility hasMicrophone:hasMicrophone];
    }
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
    NSString *newModel = item.title;

    // Update UI checkmarks
    for (NSMenuItem *menuItem in item.menu.itemArray) {
        menuItem.state = NSControlStateValueOff;
    }
    item.state = NSControlStateValueOn;

    // Check if model exists - bundled first, then external
    NSString *bundledModelPath = [[[NSBundle mainBundle] resourcePath] stringByAppendingPathComponent:[NSString stringWithFormat:@"ggml-%@.en.bin", newModel]];
    NSString *externalModelPath = [NSString stringWithFormat:@"%@/whisper.cpp/models/ggml-%@.en.bin",
                          NSHomeDirectory(), newModel];

    if ([[NSFileManager defaultManager] fileExistsAtPath:bundledModelPath] ||
        [[NSFileManager defaultManager] fileExistsAtPath:externalModelPath]) {
        // Model exists, just switch to it
        self.selectedModel = newModel;
        self.modelMenuItem.title = [NSString stringWithFormat:@"Model: %@", newModel];

        // Save preference
        [[NSUserDefaults standardUserDefaults] setObject:newModel forKey:@"selectedModel"];

        self.statusMenuItem.title = [NSString stringWithFormat:@"Status: Using %@ model", newModel];
        VTTLog(@"Switched to model: %@", newModel);

#ifdef USE_WHISPER_LIB
        // Reload whisper context for new model in background
        if (self.wctx) { whisper_free(self.wctx); self.wctx = NULL; }
        dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
            NSString *bundledModelPath = [[[NSBundle mainBundle] resourcePath] stringByAppendingPathComponent:[NSString stringWithFormat:@"ggml-%@.en.bin", newModel]];
            NSString *externalModelPath = [NSString stringWithFormat:@"%@/whisper.cpp/models/ggml-%@.en.bin", NSHomeDirectory(), newModel];
            NSString *modelPath = [[NSFileManager defaultManager] fileExistsAtPath:bundledModelPath] ? bundledModelPath : externalModelPath;
            dispatch_async(dispatch_get_main_queue(), ^{
                self.statusMenuItem.title = [NSString stringWithFormat:@"Status: Loading %@ model…", newModel];
                self.statusItem.button.title = @"VTT ⏳";
            });
            struct whisper_context_params cparams = whisper_context_default_params();
#if defined(__APPLE__) && defined(__aarch64__)
            cparams.use_gpu = true;
#else
            cparams.use_gpu = false;
#endif
            struct whisper_context *ctx = whisper_init_from_file_with_params([modelPath UTF8String], cparams);
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
        self.statusMenuItem.title = [NSString stringWithFormat:@"Downloading %@...", newModel];
        self.statusItem.button.title = @"VTT ⏬";

        // Download with curl showing progress
        NSString *downloadURL = [NSString stringWithFormat:
            @"https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-%@.en.bin", newModel];

        self.downloadTask = [[NSTask alloc] init];
        self.downloadTask.launchPath = @"/usr/bin/curl";
        self.downloadTask.arguments = @[@"-L", @"-#", @"-o", externalModelPath, downloadURL];

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
                if (task.terminationStatus == 0) {
                    // Success
                    strongSelf.selectedModel = newModel;
                    strongSelf.modelMenuItem.title = [NSString stringWithFormat:@"Model: %@", newModel];
                    strongSelf.statusMenuItem.title = [NSString stringWithFormat:@"Status: %@ model ready", newModel];
                    strongSelf.statusItem.button.title = @"VTT ✅";

                    // Save preference
                    [[NSUserDefaults standardUserDefaults] setObject:newModel forKey:@"selectedModel"];

                    VTTLog(@"Downloaded and switched to model: %@", newModel);
                } else {
                    // Failed - retry up to 3 times
                    VTTLog(@"Download failed (attempt %ld/3): %@", (long)strongSelf.downloadRetryCount + 1, newModel);

                    if (strongSelf.downloadRetryCount < 3) {
                        strongSelf.downloadRetryCount++;
                        strongSelf.statusMenuItem.title = [NSString stringWithFormat:@"Retrying download (%ld/3)...", (long)strongSelf.downloadRetryCount];

                        // Retry after 2 seconds
                        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 2 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
                            [strongSelf selectModel:sender];
                        });
                    } else {
                        // All retries failed
                        strongSelf.downloadRetryCount = 0;
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

        [self.downloadTask launch];
    }
}

- (void)populateMicrophoneMenu:(NSMenu *)menu {
    // Get all audio input devices
    AudioObjectPropertyAddress devicesAddr = {
        kAudioHardwarePropertyDevices,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain
    };

    UInt32 propSize = 0;
    OSStatus status = AudioObjectGetPropertyDataSize(kAudioObjectSystemObject, &devicesAddr, 0, NULL, &propSize);
    if (status != noErr) return;

    int deviceCount = propSize / sizeof(AudioDeviceID);
    AudioDeviceID *devices = (AudioDeviceID *)malloc(propSize);
    status = AudioObjectGetPropertyData(kAudioObjectSystemObject, &devicesAddr, 0, NULL, &propSize, devices);
    if (status != noErr) { free(devices); return; }

    // Load saved preference
    NSNumber *savedID = [[NSUserDefaults standardUserDefaults] objectForKey:@"selectedMicrophoneID"];
    self.selectedMicrophoneID = savedID ? [savedID unsignedIntValue] : 0;

    // Add "Default" option
    NSMenuItem *defaultItem = [[NSMenuItem alloc] initWithTitle:@"Default (Auto-Select)"
                                                         action:@selector(selectMicrophone:)
                                                  keyEquivalent:@""];
    defaultItem.target = self;
    defaultItem.tag = 0;  // 0 = default
    defaultItem.state = (self.selectedMicrophoneID == 0) ? NSControlStateValueOn : NSControlStateValueOff;
    [menu addItem:defaultItem];

    [menu addItem:[NSMenuItem separatorItem]];

    // Add each input device
    for (int i = 0; i < deviceCount; i++) {
        // Check if this is an input device
        AudioObjectPropertyAddress streamAddr = {
            kAudioDevicePropertyStreams,
            kAudioDevicePropertyScopeInput,
            kAudioObjectPropertyElementMain
        };

        UInt32 streamSize = 0;
        status = AudioObjectGetPropertyDataSize(devices[i], &streamAddr, 0, NULL, &streamSize);
        if (status != noErr || streamSize == 0) continue;

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
            NSString *name = (__bridge_transfer NSString *)deviceName;

            NSMenuItem *item = [[NSMenuItem alloc] initWithTitle:name
                                                          action:@selector(selectMicrophone:)
                                                   keyEquivalent:@""];
            item.target = self;
            item.tag = devices[i];
            item.state = (devices[i] == self.selectedMicrophoneID) ? NSControlStateValueOn : NSControlStateValueOff;
            [menu addItem:item];

            // Update menu title if this is the selected device
            if (devices[i] == self.selectedMicrophoneID) {
                self.micMenuItem.title = [NSString stringWithFormat:@"Microphone: %@", name];
            }
        }
    }

    free(devices);

    // If default is selected, update title
    if (self.selectedMicrophoneID == 0) {
        self.micMenuItem.title = @"Microphone: Default";
    }
}

- (void)selectMicrophone:(id)sender {
    NSMenuItem *item = (NSMenuItem *)sender;
    AudioDeviceID deviceID = (AudioDeviceID)item.tag;

    // Update checkmarks
    for (NSMenuItem *menuItem in item.menu.itemArray) {
        menuItem.state = NSControlStateValueOff;
    }
    item.state = NSControlStateValueOn;

    // Save selection
    self.selectedMicrophoneID = deviceID;
    [[NSUserDefaults standardUserDefaults] setObject:@(deviceID) forKey:@"selectedMicrophoneID"];
    [[NSUserDefaults standardUserDefaults] synchronize];

    // Update menu title
    if (deviceID == 0) {
        self.micMenuItem.title = @"Microphone: Default";
    } else {
        self.micMenuItem.title = [NSString stringWithFormat:@"Microphone: %@", item.title];
    }

    // Reinitialize audio with new device
    if (self.audioState) {
        if (self.audioState->queue) {
            AudioQueueDispose(self.audioState->queue, true);
        }
        free(self.audioState);
        self.audioState = NULL;
    }
    [self initializeAudio];

    VTTLog(@"Switched to microphone: %@ (ID: %u)", item.title, deviceID);
    self.statusMenuItem.title = [NSString stringWithFormat:@"Status: Using %@", item.title];
}

- (void)toggleLogging:(id)sender {
    self.loggingEnabled = !self.loggingEnabled;
    VTTLoggingEnabled = self.loggingEnabled;
    [[NSUserDefaults standardUserDefaults] setBool:self.loggingEnabled forKey:@"loggingEnabled"];
    [[NSUserDefaults standardUserDefaults] synchronize];
    self.loggingToggleItem.state = self.loggingEnabled ? NSControlStateValueOn : NSControlStateValueOff;
    self.loggingToggleItem.title = self.loggingEnabled ? @"Logging: On" : @"Logging: Off";
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
