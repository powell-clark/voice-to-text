# TASK-VTT102: Rename binary — drop the misleading "vtt-linux" name

The cargo `[[bin]]` is named `vtt-linux` on every platform (historical), so the
Windows/macOS build output is `vtt-linux.exe` / `vtt-linux` — confusing on a
Windows machine ("why is it called linux?"). The Windows MSI already renames it to
`vtt.exe` on install, but the raw build artifact and the release asset still read
`vtt-linux`.

Proposal: rename the bin to a platform-neutral `vtt` (or `voice-to-text`). Cross-
cutting — touches `Cargo.toml [[bin]]`, `debian/` install paths + `vtt.service`,
`wix/main.wxs` source path, `ci.yml`/`release.yml` verify+package steps that grep
`vtt-linux`, `gen-release-notes.sh`/`build-windows.ps1`, and CLAUDE.md. Do as one
atomic rename with all consumers updated; verify each platform still builds/packages.

- [x] Single bin name, platform-neutral, no "linux" on Windows/macOS
- [~] deb, msi, and release assets all use the new name; CI green on all three
      — deb and release assets done and verified; the `.msi` source path is
      updated in `wix/main.wxs` but NOT verified, see "Not verified" below
- Story: STORY-VTT013 · Directive: DIRECT-VTT002

## Chosen name

`vtt`, not `voice-to-text`. Not a coin toss: the MSI already installed the
binary as `vtt.exe`, and README's troubleshooting section already told users to
run `vtt --version`. The rename makes the build artefact agree with what the
product already claimed to be.

## Evidence

Implementation: commit 2d08c06, 27 files, one atomic rename.

- Local gate, all clean: `cargo build --release`, `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --release`
  (208 passed, 0 failed, 3 ignored — 2 tests new).
- Binary identity: `target/release/vtt --version` → `voice-to-text 2.4.0`;
  both checks CI performs pass locally (`file … | grep ELF 64-bit LSB.*x86-64`
  and `ldd … | grep libvulkan.so.1`).
- Debian packaging exercised for real, not read: running the actual
  `debian/rules` install targets produced `/usr/bin/vtt`, a working
  `/usr/bin/vtt-linux -> vtt` symlink (invoking the symlink prints the version),
  `ExecStart=/usr/bin/vtt` in the unit and `Exec=/usr/bin/vtt` in the desktop
  entry.
- CI run 33937869511 on commit 2d08c06: `conclusion: success`, all five jobs
  green — ubuntu-24.04 (101229254583), windows-latest x86_64 (101229254771),
  windows-11-arm (101229254822), macos-latest (101229254678), cargo audit
  (101229254723). The ubuntu job's binary-shape step is the rename's own
  regression test: it greps `target/release/vtt`, which would fail outright if
  the `[[bin]]` name and the workflow had drifted apart.

## Decisions taken (both reversible, both deliberate)

1. **The Linux package keeps a `/usr/bin/vtt-linux -> vtt` symlink.** That path
   shipped in every release up to 2.4.0 and lives in users' scripts and shell
   history. The name was only ever misleading on Windows and macOS, which never
   saw this path, so breaking Linux users buys nothing. Removing the symlink is
   a one-line change in `debian/rules` whenever a release wants the clean break.
2. **The single-instance lock filename stays `vtt-linux.lock`.** The guard only
   works if both instances agree on the path, and during an upgrade the running
   process is the OLD build holding the OLD name. Renaming it would let two
   instances run simultaneously, each grabbing the same global hotkey. The path
   is internal and no user types it.

## Bug found and fixed by this task

`--doctor` located running instances with `target.contains("vtt-linux")`. After
the rename the exe path is `/usr/bin/vtt`, so the check would have silently
stopped matching and the doctor would have reported "no process found" while the
app was plainly running. Replaced with `doctor::is_our_binary`, which matches on
the exe *file name* and accepts both `vtt` and `vtt-linux` — the old name still
counts because a pre-upgrade instance is the one actually holding the hotkey,
which is precisely what the doctor exists to explain. A file-name match rather
than a substring test also stops an unrelated binary under a path like
`~/vtt-experiments/` being mistaken for ours. Two tests cover both halves.

Falsified against reality rather than only unit tests: run against a live
pre-rename instance, `--doctor` found pid 1935134 and correctly reported it as
running `/usr/bin/vtt-linux` rather than the installed `/usr/bin/vtt` — the
exact state a user occupies mid-upgrade.

## Not verified

The `.msi`. `wix/main.wxs` now sources `vtt.exe` instead of `vtt-linux.exe`, and
the Windows binary itself is CI-verified, but the installer is built only by the
tag-triggered `build-windows-msi` job in `release.yml` (`cargo wix`), which no
ordinary push exercises. Cutting a release to prove it is the operator's call,
not an autonomous one, so this leg is honestly outstanding and is what keeps
criterion 2 at partial rather than done. It will be exercised by the next real
release; TASK-VTT168 (Verify Windows .msi in-place upgrade) already owns
hands-on `.msi` verification and should confirm the installed binary is named
`vtt.exe` when it runs.
