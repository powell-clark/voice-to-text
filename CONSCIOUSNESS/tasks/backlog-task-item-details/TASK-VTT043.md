# TASK-VTT043: Apple developer signing and notarisation

## Acceptance Criteria
1. The `.app` bundle is signed with a Developer ID Application certificate
2. The bundle is notarised via `xcrun notarytool` and stapled
3. Gatekeeper clears the app on a fresh macOS install with no security override needed
4. CI signs and notarises artefacts automatically on tag builds using credentials in GitHub Secrets

## Parked (2026-07-17 operator steer)

Parked indefinitely: requires a paid Apple Developer licence the operator is deferring for budget reasons. See ADR-0007 "Priority & sequencing". Un-park by operator decision only.
