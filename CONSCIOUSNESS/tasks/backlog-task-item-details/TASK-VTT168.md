# TASK-VTT168: Verify Windows .msi in-place upgrade

## Context

Deferred from TASK-VTT095 (Update mechanism) -- its acceptance criterion 3 needs a real Windows machine to install vX over vX-1 and confirm the WiX MajorUpgrade path preserves settings.conf and the downloaded model cache, which this Linux dev environment cannot exercise. WiX template already uses MajorUpgrade (wix/ at repo root); verification is the only remaining gap.

## Acceptance criteria

- [ ] Installing a newer `.msi` over an existing install upgrades in place with
      no manual uninstall step, on a real Windows machine
- [ ] `settings.conf` survives the upgrade with its contents intact
- [ ] Downloaded models are still present after the upgrade and are not
      re-downloaded on first use
- [ ] The installed binary is named `vtt.exe` and `vtt --version` reports the
      new version — added by TASK-VTT102 (Rename binary — drop the misleading
      vtt-linux name), which changed the WiX source path from `vtt-linux.exe`
      to `vtt.exe`. That path change is code-complete and the Windows binary is
      CI-verified, but the `.msi` itself is built only by the tag-triggered
      `build-windows-msi` job, so no push has ever exercised it
- [ ] The Start Menu shortcut and any `PATH` entry still resolve after the
      upgrade rather than pointing at the removed previous install

## Dependencies

- Directive: DIRECT-VTT004
- Story: STORY-VTT013
- Deferred from: TASK-VTT095 (Update mechanism — how Windows/macOS users get
  new versions)
- Also carries the unverified `.msi` leg of TASK-VTT102 (Rename binary — drop
  the misleading vtt-linux name)

## Pre-mortem

### Failure modes

- The upgrade succeeds but the binary rename breaks a shortcut or `PATH` entry
  still pointing at `vtt-linux.exe`, so the app appears to vanish after upgrade.
- WiX `MajorUpgrade` sequencing removes the old install *after* laying down the
  new files, deleting the new ones — the classic scheduling mistake. Symptom is
  a "successful" install with no binary present.
- Settings or models live under a per-user path the installer does not touch,
  so the test passes for the tester and still loses data for a user whose
  install was made by a different account.

### Weak assumptions

- That a machine with a previous version installed is available. If not, the
  test has to install an older release first, which needs that release's `.msi`
  to still be downloadable.
- That the upgrade path from the LAST SHIPPED version is what matters. Since
  the binary rename lands in an unreleased commit, the first real upgrade test
  must go from a pre-rename release to a post-rename one, not between two
  post-rename builds — the latter would silently skip the rename risk entirely.
