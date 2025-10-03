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
    NSRect frame = NSMakeRect(0, 0, 600, 500);
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

    // Large emoji/icon
    NSTextField *icon = [[NSTextField alloc] initWithFrame:NSMakeRect(250, 350, 100, 80)];
    icon.stringValue = @"🎙️";
    icon.font = [NSFont systemFontOfSize:72];
    icon.bordered = NO;
    icon.editable = NO;
    icon.selectable = NO;
    icon.backgroundColor = [NSColor clearColor];
    icon.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:icon];

    // Title
    NSTextField *title = [NSTextField labelWithString:@"Welcome to VTT"];
    title.frame = NSMakeRect(50, 280, 500, 40);
    title.font = [NSFont boldSystemFontOfSize:32];
    title.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:title];

    // Subtitle
    NSTextField *subtitle = [NSTextField labelWithString:@"Voice to Text - Real-time transcription for macOS\nHold Right Option, speak, release to paste."];
    subtitle.frame = NSMakeRect(50, 220, 500, 50);
    subtitle.font = [NSFont systemFontOfSize:16];
    subtitle.textColor = [NSColor grayColor];
    subtitle.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:subtitle];

    // Features list
    NSArray *features = @[
        @"✓ Real-time transcription using Whisper AI",
        @"✓ Works completely offline - no cloud required",
        @"✓ Lightweight menu bar app - always ready",
        @"✓ Multiple model sizes - tiny to large"
    ];

    CGFloat yPos = 150;
    for (NSString *feature in features) {
        NSTextField *featureLabel = [NSTextField labelWithString:feature];
        featureLabel.frame = NSMakeRect(100, yPos, 400, 20);
        featureLabel.font = [NSFont systemFontOfSize:14];
        [self.contentView addSubview:featureLabel];
        yPos -= 25;
    }

    // Get Started button
    NSButton *startButton = [[NSButton alloc] initWithFrame:NSMakeRect(200, 40, 200, 40)];
    [startButton setTitle:@"Get Started"];
    [startButton setBezelStyle:NSBezelStyleRounded];
    [startButton setFont:[NSFont boldSystemFontOfSize:16]];
    [startButton setTarget:self];
    [startButton setAction:@selector(startPermissionFlow)];
    [self.contentView addSubview:startButton];
}

- (void)startPermissionFlow {
    self.currentStep = 0;
    [self showPermissionStep];

    // Start timer to check permissions
    self.permissionCheckTimer = [NSTimer scheduledTimerWithTimeInterval:0.5
                                                                 target:self
                                                               selector:@selector(checkPermissions)
                                                               userInfo:nil
                                                                repeats:YES];
}

- (void)showPermissionStep {
    // Clear content
    for (NSView *subview in self.contentView.subviews) {
        [subview removeFromSuperview];
    }

    // Progress bar
    self.progressBar = [[NSProgressIndicator alloc] initWithFrame:NSMakeRect(50, 440, 500, 20)];
    self.progressBar.style = NSProgressIndicatorStyleBar;
    self.progressBar.indeterminate = NO;
    self.progressBar.minValue = 0;
    self.progressBar.maxValue = 3;
    self.progressBar.doubleValue = self.currentStep;
    [self.contentView addSubview:self.progressBar];

    // Step indicator
    NSTextField *stepLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 410, 500, 20)];
    stepLabel.stringValue = [NSString stringWithFormat:@"Step %ld of 3", (long)self.currentStep + 1];
    stepLabel.font = [NSFont systemFontOfSize:14];
    stepLabel.textColor = [NSColor grayColor];
    stepLabel.bordered = NO;
    stepLabel.editable = NO;
    stepLabel.selectable = NO;
    stepLabel.backgroundColor = [NSColor clearColor];
    [self.contentView addSubview:stepLabel];

    // Show current permission step
    if (self.currentStep == 0) {
        [self showMicrophoneStep];
    } else if (self.currentStep == 1) {
        [self showAccessibilityStep];
    } else if (self.currentStep == 2) {
        [self showInputMonitoringStep];
    } else {
        [self showCompletionScreen];
    }
}

- (void)showMicrophoneStep {
    [self showPermissionStepWithIcon:@"🎤"
                               title:@"Microphone Access"
                         description:@"VTT needs microphone access to record your voice.\n\nClick the button below to grant permission."
                              action:@selector(requestMicrophone)
                          buttonTitle:@"Grant Microphone Access"];
}

- (void)showAccessibilityStep {
    [self showPermissionStepWithIcon:@"♿️"
                               title:@"Accessibility Access"
                         description:@"VTT needs accessibility access to automatically paste transcribed text.\n\nYou'll be taken to System Settings. Find VTT in the list and check the box."
                              action:@selector(requestAccessibility)
                          buttonTitle:@"Open System Settings"];
}

- (void)showInputMonitoringStep {
    [self showPermissionStepWithIcon:@"⌨️"
                               title:@"Input Monitoring"
                         description:@"VTT needs input monitoring to detect when you hold the Right Option key.\n\nYou'll be taken to System Settings. Find VTT in the list and check the box."
                              action:@selector(requestInputMonitoring)
                          buttonTitle:@"Open System Settings"];
}

- (void)showPermissionStepWithIcon:(NSString *)icon
                             title:(NSString *)title
                       description:(NSString *)description
                            action:(SEL)action
                        buttonTitle:(NSString *)buttonTitle {
    // Icon
    NSTextField *iconLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(250, 300, 100, 80)];
    iconLabel.stringValue = icon;
    iconLabel.font = [NSFont systemFontOfSize:64];
    iconLabel.bordered = NO;
    iconLabel.editable = NO;
    iconLabel.selectable = NO;
    iconLabel.backgroundColor = [NSColor clearColor];
    iconLabel.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:iconLabel];

    // Title
    self.titleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 240, 500, 40)];
    self.titleLabel.stringValue = title;
    self.titleLabel.font = [NSFont boldSystemFontOfSize:28];
    self.titleLabel.bordered = NO;
    self.titleLabel.editable = NO;
    self.titleLabel.selectable = NO;
    self.titleLabel.backgroundColor = [NSColor clearColor];
    self.titleLabel.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:self.titleLabel];

    // Description
    self.subtitleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 140, 500, 90)];
    self.subtitleLabel.stringValue = description;
    self.subtitleLabel.font = [NSFont systemFontOfSize:14];
    self.subtitleLabel.textColor = [NSColor grayColor];
    self.subtitleLabel.bordered = NO;
    self.subtitleLabel.editable = NO;
    self.subtitleLabel.selectable = NO;
    self.subtitleLabel.backgroundColor = [NSColor clearColor];
    self.subtitleLabel.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:self.subtitleLabel];

    // Grant button
    NSButton *grantButton = [[NSButton alloc] initWithFrame:NSMakeRect(150, 70, 300, 40)];
    [grantButton setTitle:buttonTitle];
    [grantButton setBezelStyle:NSBezelStyleRounded];
    [grantButton setFont:[NSFont boldSystemFontOfSize:14]];
    [grantButton setTarget:self];
    [grantButton setAction:action];
    [self.contentView addSubview:grantButton];

    // Status label (for checking)
    NSTextField *statusLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 30, 500, 20)];
    statusLabel.stringValue = @"Waiting for permission...";
    statusLabel.font = [NSFont systemFontOfSize:12];
    statusLabel.textColor = [NSColor grayColor];
    statusLabel.bordered = NO;
    statusLabel.editable = NO;
    statusLabel.selectable = NO;
    statusLabel.backgroundColor = [NSColor clearColor];
    statusLabel.alignment = NSTextAlignmentCenter;
    statusLabel.tag = 999; // Tag for updating
    [self.contentView addSubview:statusLabel];
}

- (void)showCompletionScreen {
    [self.permissionCheckTimer invalidate];
    self.permissionCheckTimer = nil;

    // Clear content
    for (NSView *subview in self.contentView.subviews) {
        [subview removeFromSuperview];
    }

    // Success icon
    NSTextField *icon = [[NSTextField alloc] initWithFrame:NSMakeRect(250, 320, 100, 80)];
    icon.stringValue = @"✅";
    icon.font = [NSFont systemFontOfSize:72];
    icon.bordered = NO;
    icon.editable = NO;
    icon.selectable = NO;
    icon.backgroundColor = [NSColor clearColor];
    icon.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:icon];

    // Title
    NSTextField *title = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 260, 500, 40)];
    title.stringValue = @"All Set!";
    title.font = [NSFont boldSystemFontOfSize:32];
    title.bordered = NO;
    title.editable = NO;
    title.selectable = NO;
    title.backgroundColor = [NSColor clearColor];
    title.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:title];

    // Instructions
    NSTextField *instructions = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 160, 500, 90)];
    instructions.stringValue = @"VTT is ready to use!\n\nHow to use:\n1. Hold the Right Option key\n2. Speak clearly\n3. Release the key\n4. Your text will be pasted automatically";
    instructions.font = [NSFont systemFontOfSize:14];
    instructions.textColor = [NSColor grayColor];
    instructions.bordered = NO;
    instructions.editable = NO;
    instructions.selectable = NO;
    instructions.backgroundColor = [NSColor clearColor];
    instructions.alignment = NSTextAlignmentCenter;
    [self.contentView addSubview:instructions];

    // Done button
    NSButton *doneButton = [[NSButton alloc] initWithFrame:NSMakeRect(200, 80, 200, 40)];
    [doneButton setTitle:@"Start Using VTT"];
    [doneButton setBezelStyle:NSBezelStyleRounded];
    [doneButton setFont:[NSFont boldSystemFontOfSize:16]];
    [doneButton setTarget:self];
    [doneButton setAction:@selector(finishOnboarding)];
    [self.contentView addSubview:doneButton];

    // Mark onboarding as seen
    NSUserDefaults *defaults = [NSUserDefaults standardUserDefaults];
    [defaults setBool:YES forKey:@"VTTHasSeenOnboarding"];
    [defaults synchronize];
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

- (void)checkPermissions {
    BOOL hasCurrentPermission = NO;
    NSTextField *statusLabel = [self.contentView viewWithTag:999];

    if (self.currentStep == 0) {
        hasCurrentPermission = [VTTOnboarding hasMicrophonePermission];
        if (hasCurrentPermission) {
            statusLabel.stringValue = @"✓ Microphone access granted!";
            statusLabel.textColor = [NSColor systemGreenColor];
        }
    } else if (self.currentStep == 1) {
        hasCurrentPermission = [VTTOnboarding hasAccessibilityPermission];
        if (hasCurrentPermission) {
            statusLabel.stringValue = @"✓ Accessibility access granted!";
            statusLabel.textColor = [NSColor systemGreenColor];
        }
    } else if (self.currentStep == 2) {
        hasCurrentPermission = [VTTOnboarding hasInputMonitoringPermission];
        if (hasCurrentPermission) {
            statusLabel.stringValue = @"✓ Input monitoring granted!";
            statusLabel.textColor = [NSColor systemGreenColor];
        }
    }

    // Auto-advance when permission granted
    if (hasCurrentPermission && self.currentStep < 3) {
        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 1 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
            self.currentStep++;
            [self showPermissionStep];
        });
    }
}

- (void)finishOnboarding {
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
