// VTTOnboarding.m - Beautiful first-run onboarding window

#import "VTTOnboarding.h"
#import <AVFoundation/AVFoundation.h>
#import <ApplicationServices/ApplicationServices.h>

// Onboarding window controller
@interface VTTOnboardingWindow ()
@property (strong) NSWindow *onboardingWindow;
@property (strong) NSView *contentView;
@property (strong) NSTextField *titleLabel;
@property (strong) NSTextField *subtitleLabel;
@property (strong) NSProgressIndicator *progressBar;
@property (strong) NSView *permissionView;
@property (strong) NSTimer *permissionCheckTimer;
@property (nonatomic) NSInteger currentStep;
@end

@implementation VTTOnboardingWindow

+ (void)showIfNeeded {
    NSUserDefaults *defaults = [NSUserDefaults standardUserDefaults];
    BOOL hasSeenOnboarding = [defaults boolForKey:@"VTTHasSeenOnboarding"];

    if (!hasSeenOnboarding || ![self hasAllPermissions]) {
        [self show];
    }
}

+ (void)show {
    VTTOnboardingWindow *controller = [[VTTOnboardingWindow alloc] init];
    [controller showWindow:nil];
}

+ (BOOL)hasAllPermissions {
    return [VTTOnboarding hasMicrophonePermission] &&
           [VTTOnboarding hasAccessibilityPermission] &&
           [VTTOnboarding hasInputMonitoringPermission];
}

- (instancetype)init {
    self = [super init];
    if (self) {
        [self createWindow];
        _currentStep = 0;
    }
    return self;
}

- (void)createWindow {
    // Create window
    NSRect frame = NSMakeRect(0, 0, 600, 550);
    self.onboardingWindow = [[NSWindow alloc] initWithContentRect:frame
                                                        styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable)
                                                          backing:NSBackingStoreBuffered
                                                            defer:NO];
    self.onboardingWindow.title = @"Welcome to VTT";
    self.onboardingWindow.backgroundColor = [NSColor whiteColor];
    [self.onboardingWindow center];
    self.onboardingWindow.delegate = (id<NSWindowDelegate>)self;

    // Content view
    self.contentView = [[NSView alloc] initWithFrame:frame];
    [self.onboardingWindow.contentView addSubview:self.contentView];

    // Show welcome screen first
    [self showWelcomeScreen];
}

- (void)showWelcomeScreen {
    // Clear content
    for (NSView *subview in self.contentView.subviews) {
        [subview removeFromSuperview];
    }

    // Title
    NSTextField *title = [NSTextField labelWithString:@"Welcome to VTT"];
    title.frame = NSMakeRect(50, 480, 500, 40);
    title.font = [NSFont boldSystemFontOfSize:32];
    title.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:title];

    // Subtitle
    NSTextField *subtitle = [NSTextField labelWithString:@"VTT needs 3 permissions to work. Click each button below:"];
    subtitle.frame = NSMakeRect(50, 440, 500, 30);
    subtitle.font = [NSFont systemFontOfSize:14];
    subtitle.textColor = [NSColor grayColor];
    subtitle.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:subtitle];

    // Microphone permission
    NSTextField *mic1 = [NSTextField labelWithString:@"🎤 Microphone"];
    mic1.frame = NSMakeRect(50, 380, 200, 30);
    mic1.font = [NSFont boldSystemFontOfSize:16];
    [self.contentView addSubview:mic1];

    NSTextField *mic2 = [NSTextField labelWithString:@"To record your voice"];
    mic2.frame = NSMakeRect(50, 360, 200, 20);
    mic2.font = [NSFont systemFontOfSize:12];
    mic2.textColor = [NSColor grayColor];
    [self.contentView addSubview:mic2];

    NSButton *micButton = [[NSButton alloc] initWithFrame:NSMakeRect(280, 360, 280, 40)];
    [micButton setTitle:@"Grant Microphone Access"];
    [micButton setBezelStyle:NSBezelStyleRounded];
    [micButton setTarget:self];
    [micButton setAction:@selector(requestMicrophone)];
    micButton.tag = 1;
    [self.contentView addSubview:micButton];

    // Accessibility permission
    NSTextField *acc1 = [NSTextField labelWithString:@"♿️ Accessibility"];
    acc1.frame = NSMakeRect(50, 300, 200, 30);
    acc1.font = [NSFont boldSystemFontOfSize:16];
    [self.contentView addSubview:acc1];

    NSTextField *acc2 = [NSTextField labelWithString:@"To paste transcribed text"];
    acc2.frame = NSMakeRect(50, 280, 200, 20);
    acc2.font = [NSFont systemFontOfSize:12];
    acc2.textColor = [NSColor grayColor];
    [self.contentView addSubview:acc2];

    NSButton *accButton = [[NSButton alloc] initWithFrame:NSMakeRect(280, 280, 280, 40)];
    [accButton setTitle:@"Open System Settings"];
    [accButton setBezelStyle:NSBezelStyleRounded];
    [accButton setTarget:self];
    [accButton setAction:@selector(requestAccessibility)];
    accButton.tag = 2;
    [self.contentView addSubview:accButton];

    // Input Monitoring permission
    NSTextField *input1 = [NSTextField labelWithString:@"⌨️ Input Monitoring"];
    input1.frame = NSMakeRect(50, 220, 200, 30);
    input1.font = [NSFont boldSystemFontOfSize:16];
    [self.contentView addSubview:input1];

    NSTextField *input2 = [NSTextField labelWithString:@"To detect Right Option key"];
    input2.frame = NSMakeRect(50, 200, 200, 20);
    input2.font = [NSFont systemFontOfSize:12];
    input2.textColor = [NSColor grayColor];
    [self.contentView addSubview:input2];

    NSButton *inputButton = [[NSButton alloc] initWithFrame:NSMakeRect(280, 200, 280, 40)];
    [inputButton setTitle:@"Open System Settings"];
    [inputButton setBezelStyle:NSBezelStyleRounded];
    [inputButton setTarget:self];
    [inputButton setAction:@selector(requestInputMonitoring)];
    inputButton.tag = 3;
    [self.contentView addSubview:inputButton];

    // Instructions
    NSTextField *instructions = [NSTextField labelWithString:@"After granting all permissions, click Done below."];
    instructions.frame = NSMakeRect(50, 150, 500, 30);
    instructions.font = [NSFont systemFontOfSize:12];
    instructions.textColor = [NSColor grayColor];
    instructions.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:instructions];

    // Done button
    NSButton *doneButton = [[NSButton alloc] initWithFrame:NSMakeRect(200, 80, 200, 40)];
    [doneButton setTitle:@"Done"];
    [doneButton setBezelStyle:NSBezelStyleRounded];
    [doneButton setFont:[NSFont boldSystemFontOfSize:16]];
    [doneButton setTarget:self];
    [doneButton setAction:@selector(finishOnboarding)];
    [self.contentView addSubview:doneButton];

    // Start timer to update button states
    self.permissionCheckTimer = [NSTimer scheduledTimerWithTimeInterval:0.5
                                                                 target:self
                                                               selector:@selector(updateButtonStates)
                                                               userInfo:nil
                                                                repeats:YES];
}

- (void)updateButtonStates {
    NSButton *micButton = [self.contentView viewWithTag:1];
    NSButton *accButton = [self.contentView viewWithTag:2];
    NSButton *inputButton = [self.contentView viewWithTag:3];

    if ([VTTOnboarding hasMicrophonePermission]) {
        [micButton setTitle:@"✓ Microphone Granted"];
        [micButton setEnabled:NO];
    }
    if ([VTTOnboarding hasAccessibilityPermission]) {
        [accButton setTitle:@"✓ Accessibility Granted"];
        [accButton setEnabled:NO];
    }
    if ([VTTOnboarding hasInputMonitoringPermission]) {
        [inputButton setTitle:@"✓ Input Monitoring Granted"];
        [inputButton setEnabled:NO];
    }
}


- (void)requestMicrophone {
    [VTTOnboarding requestMicrophonePermission];
}

- (void)requestAccessibility {
    [VTTOnboarding openAccessibilitySettings];
}

- (void)requestInputMonitoring {
    [VTTOnboarding openInputMonitoringSettings];
}

- (void)finishOnboarding {
    NSUserDefaults *defaults = [NSUserDefaults standardUserDefaults];
    [defaults setBool:YES forKey:@"VTTHasSeenOnboarding"];
    [defaults synchronize];
    [self.onboardingWindow close];
}

- (void)showWindow:(id)sender {
    [self.onboardingWindow makeKeyAndOrderFront:sender];
    [NSApp activateIgnoringOtherApps:YES];
}

- (void)windowWillClose:(NSNotification *)notification {
    [self.permissionCheckTimer invalidate];
    self.permissionCheckTimer = nil;
}

@end

// Helper methods
@implementation VTTOnboarding

+ (void)showIfNeeded {
    NSUserDefaults *defaults = [NSUserDefaults standardUserDefaults];
    BOOL hasSeenOnboarding = [defaults boolForKey:@"VTTHasSeenOnboarding"];

    if (!hasSeenOnboarding || ![self hasAllPermissions]) {
        [self show];
        [defaults setBool:YES forKey:@"VTTHasSeenOnboarding"];
        [defaults synchronize];
    }
}

+ (void)show {
    NSAlert *alert = [[NSAlert alloc] init];
    alert.messageText = @"Welcome to VTT! 🎙️";
    alert.informativeText = @"VTT needs a few permissions to work:\n\n"
                            @"✓ Microphone - Record your voice\n"
                            @"✓ Accessibility - Paste transcribed text\n"
                            @"✓ Input Monitoring - Detect Right Option key\n\n"
                            @"Click 'Grant Permissions' to get started.";
    alert.alertStyle = NSAlertStyleInformational;
    [alert addButtonWithTitle:@"Grant Permissions"];
    [alert addButtonWithTitle:@"Later"];

    NSInteger response = [alert runModal];
    if (response == NSAlertFirstButtonReturn) {
        [self requestAllPermissions];
    }
}

+ (void)requestAllPermissions {
    // Request microphone first (easiest)
    [self requestMicrophonePermission];

    // Show detailed instructions for manual permissions
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 1 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
        [self showPermissionInstructions];
    });
}

+ (void)showPermissionInstructions {
    NSAlert *alert = [[NSAlert alloc] init];
    alert.messageText = @"Grant Permissions";
    alert.informativeText = @"Click each button below to open System Settings:\n\n"
                            @"1️⃣ Microphone - Allow VTT to record\n"
                            @"2️⃣ Accessibility - Allow VTT to paste text\n"
                            @"3️⃣ Input Monitoring - Allow VTT to detect keys\n\n"
                            @"After granting each permission, restart VTT.";
    alert.alertStyle = NSAlertStyleInformational;

    // Create custom view with buttons for each permission
    NSView *accessoryView = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 400, 100)];

    // Microphone button
    NSButton *micButton = [[NSButton alloc] initWithFrame:NSMakeRect(10, 60, 380, 30)];
    [micButton setTitle:[self hasMicrophonePermission] ? @"✓ Microphone Access Granted" : @"Open Microphone Settings"];
    [micButton setBezelStyle:NSBezelStyleRounded];
    [micButton setTarget:self];
    [micButton setAction:@selector(openMicrophoneSettings)];
    [micButton setEnabled:![self hasMicrophonePermission]];
    [accessoryView addSubview:micButton];

    // Accessibility button
    NSButton *accessButton = [[NSButton alloc] initWithFrame:NSMakeRect(10, 30, 380, 30)];
    [accessButton setTitle:[self hasAccessibilityPermission] ? @"✓ Accessibility Access Granted" : @"Open Accessibility Settings"];
    [accessButton setBezelStyle:NSBezelStyleRounded];
    [accessButton setTarget:self];
    [accessButton setAction:@selector(openAccessibilitySettings)];
    [accessButton setEnabled:![self hasAccessibilityPermission]];
    [accessoryView addSubview:accessButton];

    // Input Monitoring button
    NSButton *inputButton = [[NSButton alloc] initWithFrame:NSMakeRect(10, 0, 380, 30)];
    [inputButton setTitle:[self hasInputMonitoringPermission] ? @"✓ Input Monitoring Granted" : @"Open Input Monitoring Settings"];
    [inputButton setBezelStyle:NSBezelStyleRounded];
    [inputButton setTarget:self];
    [inputButton setAction:@selector(openInputMonitoringSettings)];
    [inputButton setEnabled:![self hasInputMonitoringPermission]];
    [accessoryView addSubview:inputButton];

    [alert setAccessoryView:accessoryView];
    [alert addButtonWithTitle:@"Done"];
    [alert runModal];
}

+ (BOOL)hasAllPermissions {
    return [self hasMicrophonePermission] &&
           [self hasAccessibilityPermission] &&
           [self hasInputMonitoringPermission];
}

+ (BOOL)hasMicrophonePermission {
    if (@available(macOS 10.14, *)) {
        AVAuthorizationStatus status = [AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeAudio];
        return status == AVAuthorizationStatusAuthorized;
    }
    return YES; // Older macOS doesn't need permission
}

+ (BOOL)hasAccessibilityPermission {
    NSDictionary *options = @{(__bridge id)kAXTrustedCheckOptionPrompt: @NO};
    return AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)options);
}

static CGEventRef dummyEventCallback(CGEventTapProxy proxy, CGEventType type, CGEventRef event, void *refcon) {
    return event;
}

+ (BOOL)hasInputMonitoringPermission {
    // Input Monitoring is part of Accessibility on macOS < 10.15
    if (@available(macOS 10.15, *)) {
        // Check if we can create an event tap (requires Input Monitoring)
        CGEventMask mask = CGEventMaskBit(kCGEventFlagsChanged);
        CFMachPortRef eventTap = CGEventTapCreate(
            kCGSessionEventTap,
            kCGHeadInsertEventTap,
            kCGEventTapOptionListenOnly,
            mask,
            dummyEventCallback,
            NULL
        );

        if (eventTap) {
            CFRelease(eventTap);
            return YES;
        }
        return NO;
    }
    return [self hasAccessibilityPermission];
}

+ (void)requestMicrophonePermission {
    if (@available(macOS 10.14, *)) {
        [AVCaptureDevice requestAccessForMediaType:AVMediaTypeAudio completionHandler:^(BOOL granted) {
            if (granted) {
                NSLog(@"Microphone permission granted");
            } else {
                dispatch_async(dispatch_get_main_queue(), ^{
                    [self openMicrophoneSettings];
                });
            }
        }];
    }
}

+ (void)requestAccessibilityPermission {
    // This will show the system prompt
    NSDictionary *options = @{(__bridge id)kAXTrustedCheckOptionPrompt: @YES};
    AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)options);
}

+ (void)requestInputMonitoringPermission {
    // No direct API - must open System Settings
    [self openInputMonitoringSettings];
}

+ (void)openMicrophoneSettings {
    if (@available(macOS 13.0, *)) {
        [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"]];
    } else {
        [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"]];
    }
}

+ (void)openAccessibilitySettings {
    if (@available(macOS 13.0, *)) {
        [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"]];
    } else {
        [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"]];
    }
}

+ (void)openInputMonitoringSettings {
    if (@available(macOS 13.0, *)) {
        [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"]];
    } else if (@available(macOS 10.15, *)) {
        [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"]];
    } else {
        // Fallback to Accessibility on older macOS
        [self openAccessibilitySettings];
    }
}

@end
