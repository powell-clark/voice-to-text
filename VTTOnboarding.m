// VTTOnboarding.m - Beautiful first-run onboarding window

#import "VTTOnboarding.h"
#import <AVFoundation/AVFoundation.h>
#import <ApplicationServices/ApplicationServices.h>

@implementation VTTOnboardingWindow

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
    alert.messageText = @"VTT Setup Required";
    alert.informativeText = @"VTT needs 3 permissions to work:\n\n"
                            @"1. Go to System Settings → Privacy & Security\n"
                            @"2. Grant these permissions:\n"
                            @"   • Microphone (to record your voice)\n"
                            @"   • Accessibility (to paste text)\n"
                            @"   • Input Monitoring (to detect Right Option key)\n\n"
                            @"3. Restart VTT after granting permissions\n\n"
                            @"Usage: Hold Right Option, speak, release to paste.";
    alert.alertStyle = NSAlertStyleInformational;
    [alert addButtonWithTitle:@"Open System Settings"];
    [alert addButtonWithTitle:@"I'll Do It Later"];

    NSModalResponse response = [alert runModal];
    if (response == NSAlertFirstButtonReturn) {
        [[NSWorkspace sharedWorkspace] openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy"]];
    }
}

+ (BOOL)hasAllPermissions {
    return [VTTOnboarding hasMicrophonePermission] &&
           [VTTOnboarding hasAccessibilityPermission] &&
           [VTTOnboarding hasInputMonitoringPermission];
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
