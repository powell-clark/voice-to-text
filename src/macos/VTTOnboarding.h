// VTTOnboarding.h - Beautiful first-run onboarding window

#import <Cocoa/Cocoa.h>

@interface VTTOnboardingWindow : NSWindowController

// Show onboarding if needed (first run or missing permissions)
+ (void)showIfNeeded;

// Show onboarding manually
+ (void)show;

// Check if all permissions are granted
+ (BOOL)hasAllPermissions;

@end

// Onboarding helper methods
@interface VTTOnboarding : NSObject

// Individual permission checks
+ (BOOL)hasMicrophonePermission;
+ (BOOL)hasAccessibilityPermission;
+ (BOOL)hasInputMonitoringPermission;

// Request individual permissions
+ (void)requestMicrophonePermission;
+ (void)requestAccessibilityPermission;
+ (void)requestInputMonitoringPermission;

// Open System Settings to specific permission pane
+ (void)openMicrophoneSettings;
+ (void)openAccessibilitySettings;
+ (void)openInputMonitoringSettings;

@end
