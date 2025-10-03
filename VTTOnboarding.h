// VTTOnboarding.h - First-run walkthrough and permission helper

#import <Cocoa/Cocoa.h>

@interface VTTOnboarding : NSObject

// Show onboarding if first run
+ (void)showIfNeeded;

// Show onboarding manually
+ (void)show;

// Check if all permissions are granted
+ (BOOL)hasAllPermissions;

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
