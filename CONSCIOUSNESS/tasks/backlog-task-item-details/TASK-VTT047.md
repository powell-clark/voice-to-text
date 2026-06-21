# TASK-VTT047: Windows Authenticode code signing

## Acceptance Criteria
1. The release `.exe` and `.msi` carry a valid Authenticode signature
2. Windows SmartScreen does not block the installer for first-time users
3. `signtool verify /pa vtt-windows.exe` exits 0
4. CI signs artefacts automatically on tag builds using the certificate stored in GitHub Secrets
