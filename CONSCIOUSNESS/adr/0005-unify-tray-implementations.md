# 5. Evaluate unifying the Linux GTK tray onto the portable tray

Date: 2026-07-17

## Status

Accepted — 2026-07-17, alternative (b): keep the split and mirror parity
now; alternative (a) unify deferred pending the hotkey-dialog spike
(spike succeeds → revisit this ADR before any GTK retirement; spike fails
→ (b) becomes the standing decision). Approved under the operator's
explicit in-session delegation of sign-off (2026-07-17 15:41 BST,
recorded in REVIEW-INDEX). No GTK code is retired under this acceptance.
Filed per TASK-VTT103 (STORY-VTT013, DIRECT-VTT002).

## Operator note on risk (2026-07-17)

The operator correctly observes that **local rollback is trivial**: install
the unified-tray `.deb`, judge it in seconds, and if it regresses, reinstall
the prior `.deb` — one command, as demonstrated installing 2.3.10 this
session. So the regression risk this ADR weighs is **not** local
experimentation (cheap and reversible) — it is confined to the
**PPA-ship gate**, where a regression reaches other users who cannot roll
back as easily. This *raises* the appetite for building the unified tray and
test-installing it locally, and narrows the only real guardrail to: do not
push the unified tray to the PPA until a local install has confirmed it. The
build-before-testable cost (the five Linux-only features must exist on
`tray-icon`/`muda` before the tray is judgeable) remains the real effort; the
spike (TASK-VTT138) is the smallest slice that reaches "install and see."

## Context

The repo ships two independent tray implementations:

- `src/tray/linux.rs` — GTK3 + `libappindicator` 0.9 / `gtk` 0.18, 1064
  lines, the mature Linux experience.
- `src/tray/portable.rs` — `tray-icon` 0.19 + `muda` 0.15, 379 lines,
  used on Windows and macOS (`cfg(any(target_os = "macos", target_os =
  "windows"))` in `Cargo.toml` and `src/tray/mod.rs`).

`tray-icon`/`muda` also support Linux via StatusNotifierItem, so in
principle one cross-platform tray could replace both, removing the
~1000-line GTK module and the permanent risk of the two menus drifting
apart. `src/tray/quit.rs` already shows the alternative pattern working
today — a single shared helper both `linux.rs` and `portable.rs` call
into (added in TASK-VTT122 specifically to stop quit-behaviour drift).

Reading both files line-by-line against `src/tray/mod.rs` shows the
drift is not hypothetical — it is already substantial and asymmetric.

### Menu items present in `linux.rs` but absent from `portable.rs`

- **Hotkey rebinding.** `linux.rs` has a `hotkey_item` menu entry wired
  to `show_hotkey_dialog()` (lines 802-905), a modal GTK dialog that
  captures a raw X11 keycode via `connect_key_press_event` /
  `connect_key_release_event`, validates it against the 8-255 X11
  keycode range, and pushes `HotkeyCmd::SetKeycode` to the hotkey
  thread. `portable.rs` has no hotkey menu item, no dialog, and no
  `HotkeyCmd` usage at all (confirmed by grep — `HotkeyCmd` only
  appears in `src/hotkey/mod.rs`, `src/hotkey/portable.rs`, `src/main.rs`,
  and `linux.rs`). `src/hotkey/portable.rs::keycode_to_rdev` hardcodes a
  hidden default (`Key::ScrollLock` for keycode 0 or 78) and a small
  F-key/Insert/Home/etc. table with no way to change it from the tray on
  Windows or macOS. This is not a cosmetic gap — Windows/macOS users
  cannot rebind their push-to-talk key at all today.
- **Customize Transcription Settings dialog.** `linux.rs`'s
  `prompt_item` opens `show_prompt_dialog()` (lines 590-771): voice
  prefix entry, a 240-char initial-prompt text view with live
  colour-coded counter (`char_counter_markup`, orange at 200+, red at
  230+), and newline-behaviour radios (plain vs Shift+Return). None of
  this exists in `portable.rs` — no menu item, no settings surface for
  `voice_prefix`, `initial_prompt`, `append_newline`, or `newline_type`
  on Windows/macOS.
- **Logs submenu.** `linux.rs` has a `logs_item` whose submenu is
  rebuilt on every `menu.connect_show` (lines 120-149, 451-504),
  listing files matching `vtt-*.log` from `logging::get_dir()`,
  friendly-labelled via the pure `format_log_label()` helper (Today /
  Yesterday / ISO date), opened via `xdg-open` on click. `portable.rs`
  has no Logs entry — no way to reach the log files from the tray on
  Windows/macOS.
- **Live microphone label.** `linux.rs` has a `mic_item` info row
  refreshed every 3 seconds via `glib::timeout_add_seconds_local`,
  shelling out to `pactl get-default-source` / `pactl list sources`
  (`get_default_mic_description`, lines 518-551). `portable.rs` has no
  equivalent — no microphone-name display at all.
- **Real About dialog.** `linux.rs`'s `about_item` opens a modal
  `gtk::Dialog` with selectable version/URL text (`show_about_dialog`,
  lines 555-588). `portable.rs`'s `MenuCmd::About` handler only writes
  a `vtt_log!` line — clicking About on Windows/macOS produces no
  visible window at all.

### Menu items present in `portable.rs` but absent from `linux.rs`

- **Start-at-login toggle.** `portable.rs` conditionally adds an
  `autostart` `CheckMenuItem` when `crate::autostart::SUPPORTED`
  (lines 119-126), backed by `crate::autostart::toggle()`/`is_enabled()`.
  `linux.rs` has no autostart item — the code comment in
  `portable.rs` explains this is intentional (Linux relies on the
  systemd `--user` unit for autostart instead of an in-app toggle), so
  this asymmetry is a deliberate platform difference, not drift.

### Other divergences

- **Model menu construction differs.** `linux.rs::rebuild_model_menu`
  filters `crate::models::MODELS` to `multilingual` entries only, maps
  each through `display_name_for_model()` for title-cased labels, and
  auto-strips `.en` from the active model when switching to
  multilingual (with a logged auto-switch). `portable.rs` iterates all
  of `crate::models::MODELS` unfiltered and displays the raw
  `info.name` with no title-casing and no `.en` auto-switch logic —
  Windows/macOS may show both `.en` and non-`.en` variants as separate,
  differently-behaved entries where Linux collapses them into one
  radio group with a language toggle.
- **Icon language differs entirely.** `linux.rs` sets AppIndicator
  icon names from the system theme (`audio-input-microphone`,
  `media-record`, `emblem-synchronizing`). `portable.rs` renders a
  flat-colour circle in-process (`create_icon`, green/red/amber) with
  no theme integration. Unifying trays does not by itself unify this —
  `tray-icon` on Linux would still need a chosen icon strategy.
- **Quit is already unified.** Both call `super::quit::quit()` — proof
  that the shared-helper-behind-`cfg` pattern already works for
  single-behaviour logic; the menu *structure* differs far more than
  its quit behaviour ever did.

None of the above five Linux-only items (hotkey, transcription
settings, logs, mic label, real About window) are on backlog cards to
be added to `portable.rs` — they simply don't exist there. Any unify
path must either build all five in `tray-icon`/`muda`, or ship a
console/settings-window equivalent, before Linux users lose the GTK
tray without regression.

## Decision

**Alternative (b) — keep the split, mirror parity — accepted 2026-07-17.**
Retiring `linux.rs` (alternative (a)) stays deferred because it is
effectively irreversible in practice (a full rewrite of the Linux-only
surfaces above, tested manually across X11 and Wayland, would be required
to reverse it later); it is re-opened only by the time-boxed hotkey-dialog
spike succeeding, per the Recommendation below.

## Considered Alternatives

### (a) Unify on the portable (`tray-icon`) tray for all three platforms — retire `linux.rs`

**Pros:**

- Single ~400-line implementation instead of ~1000 (GTK) + ~380
  (portable) ≈ 1440 lines today; permanent parity by construction —
  no future PR can add a menu item to one platform and forget the
  other.
- Removes the `libappindicator`/`gtk`/`glib` build dependency from the
  Linux target entirely, simplifying `debian/control` Depends and the
  pbuilder build environment.
- `tray-icon` on Linux talks StatusNotifierItem, which is the modern
  standard (GNOME Shell via AppIndicator extension, KDE Plasma native,
  most Wayland compositors) — AppIndicator/`libayatana-appindicator`
  itself is a fork lineage with inconsistent distro packaging.

**Cons / risks:**

- All five Linux-only features (hotkey rebind dialog, transcription
  settings dialog, logs submenu, live mic label, real About window)
  must be rebuilt on `muda`/`tray-icon`, which has no built-in dialog
  widget toolkit — `muda` menus are menus only; any modal dialog
  (hotkey capture, prompt editor) needs a separate windowing solution
  (e.g. `tao`/`winit` + a minimal immediate-mode UI, or shelling out to
  a native file/zenity-style dialog on Linux). This is materially more
  engineering than "swap the tray crate."
  - Enigo/rdev-based hotkey capture (already used for the portable
    hotkey monitor) is much lower-fidelity than GTK's
    `connect_key_press_event` for interactive "press a key now" UX —
    needs prototyping before committing.
- StatusNotifierItem support varies by desktop environment; some
  minimal window managers (i3, sway without waybar's SNI host, etc.)
  have no StatusNotifierHost at all, whereas AppIndicator has been the
  de facto standard on Ubuntu (this project's primary PPA target,
  Ubuntu Noble) for over a decade. Regression risk is concentrated
  exactly on the platform this project cares most about (Launchpad PPA
  → Ubuntu desktop).
- Wayland-specific AppIndicator quirks (icon not appearing in GNOME
  without the AppIndicator/KStatusNotifierItem Support extension) is a
  known current pain point either way — unifying does not obviously
  fix or worsen it, but changes which failure mode users hit.
- No current automated test coverage exercises either tray's GTK/native
  widget behaviour (ADR-0004 explicitly excludes GTK widget tests as
  low ROI) — a Linux tray rewrite ships with the same "manual QA only"
  exposure that produced the v2.0.x regression string ADR-0004
  describes for the rest of the app.

### (b) Keep the split and mirror features between the two trays as parity work

**Pros:**

- Lower risk — Linux keeps its mature, battle-tested AppIndicator
  behaviour untouched.
- Incremental: each parity gap (hotkey dialog, transcription settings,
  logs, mic label, real About window) becomes its own small task on
  `portable.rs`, reviewable independently, revertible independently.
- No new dependency on `tray-icon`'s StatusNotifierItem behaviour on
  Linux, avoiding the DE-compatibility risk above.

**Cons / risks:**

- Does not fix the structural cause — two independent menu-building
  code paths will keep drifting unless a process (checklist, lint, or
  a shared "menu spec" data structure both trays render from) enforces
  parity going forward. `quit.rs` shows the shared-helper pattern
  scales to single-behaviour logic; it does not by itself prevent a
  future PR from adding one more Linux-only or Windows-only menu item.
  This alternative should be paired with a lightweight parity
  guard — e.g. a task-level checklist in every tray-touching PR, or a
  small shared "menu item list" the two impls both iterate over even
  while rendering via different crates.
- Ongoing double maintenance cost (mirrors ADR-CLAUDE.md's own
  distinction between "done" and "maintained" — this is squarely a
  "maintained" burden, not a one-off).

### (c) Unify but keep a GTK fallback behind a cfg/feature flag

- Ship the portable tray as the default on Linux, but retain
  `linux.rs` behind a Cargo feature (e.g. `gtk-tray`) or a runtime
  env-var escape hatch, so a user hitting a StatusNotifierItem
  regression can opt back into the known-good AppIndicator path while
  the portable Linux tray matures.

**Pros:**

- De-risks the cutover — a working rollback path exists in the same
  binary rather than requiring a full revert-and-rebuild.
- Lets the portable-tray Linux path get real-world exposure gradually.

**Cons / risks:**

- Doubles the maintenance burden this whole ADR exists to eliminate,
  at least for a transition window — two Linux tray code paths, one of
  which (GTK) still needs the same `libappindicator`/`gtk` build deps
  the unify path was trying to drop.
- Feature flags that are meant to be temporary have a strong tendency
  to become permanent (see `engineering-first-principles`: "delete
  first" — a permanent dual-path option is the opposite of deleting).
  Would need an explicit removal date/task, not an open-ended flag.
- Packaging complexity: the Debian package would need to decide at
  build time (not runtime) which deps to pull in, since `libappindicator`
  cannot be made a purely optional runtime dependency without either
  dynamic loading (`dlopen`) or two separate `.deb` builds.

## Consequences

**If (a) unify is chosen and executed well:** permanent parity, ~600
fewer lines to maintain, simpler Linux packaging — at the cost of a
significant one-time engineering effort to rebuild five UI surfaces
that currently only exist in GTK, plus real Wayland/DE-compatibility
risk on the platform (Ubuntu PPA) this project treats as primary.

**If (b) keep-split is chosen:** zero regression risk today, but the
maintainability problem this ADR was raised to solve persists
indefinitely unless a parity-enforcement habit (checklist or shared
menu-item list) is adopted alongside it.

**If (c) unify-with-fallback is chosen:** safest migration path in
principle, but only if the fallback has an explicit sunset task from
day one — otherwise it becomes a permanent third state (two Linux tray
paths forever), which is worse than either (a) or (b) alone.

## Recommendation (proposal — pending operator sign-off)

Recommend **(b) now, (a) later, gated on a scoped spike** — not a
straight cutover. Concretely:

1. Do **not** retire `linux.rs` yet. The five Linux-only features
   (hotkey dialog, transcription settings dialog, logs submenu, mic
   label, real About window) represent real, currently-shipped user
   functionality with no equivalent design in `tray-icon`/`muda` today;
   committing to (a) before that design exists risks shipping a Linux
   regression to the PPA, which ADR-0004 shows is expensive to recover
   from once released.
2. File a small time-boxed spike task (not a task under this ADR,
   filed separately once approved) to prototype exactly one of the
   five gaps — the hotkey capture dialog, the highest-risk one because
   it has no dialog toolkit equivalent in `muda` — on Linux using
   `tray-icon` + a minimal window (e.g. `tao`, which `tray-icon` and
   `muda` already pull in transitively for their event loop) or a
   native dialog shim. If that spike cannot produce an acceptably
   equivalent UX, alternative (a) is likely not viable without a much
   larger scope, and (b) becomes the standing decision.
3. If the spike succeeds, come back to this ADR, flip Status to
   Accepted, and execute the migration plan below.

This keeps the mature, working Linux tray untouched today while
converting "should we unify" from a theoretical judgement call into an
evidence-based one (gensho over risho, per this repo's own philosophy
precept) — build the hardest missing piece first, then decide.

## Migration plan sketch (only if (a) is approved after the spike)

1. **Parity checklist before any deletion.** For each of the five
   Linux-only surfaces, land the `tray-icon`/`muda` equivalent as an
   additive change first (portable tray on Linux, gated behind a
   feature flag or `--tray=portable` dev flag), verified manually
   against `linux.rs`'s current behaviour item-by-item, before deleting
   any GTK code.
2. **Explicit Linux regression checks**, run on both a Wayland session
   (GNOME with and without the AppIndicator extension installed) and
   an X11 session, before merging the cutover:
   - Icon appears at all (StatusNotifierItem host present) on each
     target desktop environment (GNOME/Wayland, GNOME/X11, KDE
     Plasma, XFCE at minimum — matches typical Ubuntu-derivative
     coverage for the PPA).
   - Status label updates live (recording / processing / ready).
   - Icon colour/state changes correctly for each of the three states.
   - Language submenu radio behaviour (English-only vs multilingual)
     matches, including the `.en` auto-strip-on-switch behaviour
     currently unique to `linux.rs::rebuild_model_menu`.
   - Model submenu: display names, selection state, and the
     multilingual-only filter all match current Linux behaviour.
   - Hotkey item opens a working capture dialog; keycode is saved,
     applied to the running hotkey thread without restart, and
     persisted across restart.
   - Customize Transcription Settings dialog: voice prefix, initial
     prompt (240-char limit + colour thresholds), newline
     type — all read/write correctly.
   - Copy last transcription — behaves identically including the
     "nothing transcribed yet" no-op log line.
   - Re-transcribe last recording — behaves identically.
   - Logging toggle — behaves identically.
   - Logs submenu — file list, Today/Yesterday/ISO labelling
     (`format_log_label`), opens via the platform-correct file opener
     (`xdg-open` equivalent).
   - Microphone label — either ported (pactl-based) or explicitly
     descoped with operator sign-off if not portable-tray-feasible.
   - About — a real, visible window with selectable text, not a log
     line.
   - Quit — continues to route through `systemctl --user stop vtt`
     under systemd exactly as `quit.rs` does today; unaffected by the
     tray-layer change but must be re-verified end-to-end after the
     cutover.
3. **Packaging.** Remove `libappindicator`/`gtk`/`glib` from the Linux
   target deps in `Cargo.toml` and from `debian/control` only after
   every check above passes on a rebuilt `.deb` installed via
   `scripts/release-local.sh --install` on a real Ubuntu Noble
   machine, not just `cargo build`.
4. **Rollback.** Land the cutover on its own branch; if any regression
   check fails post-release, `git revert` the cutover commit — the
   additive-first approach in step 1 means `linux.rs` still exists in
   history and, if the feature flag from step 1 wasn't yet deleted,
   possibly still in the tree, minimising rollback cost.

## References

- ADR-0000 — Use Architecture Decision Records
- ADR-0004 — Lightweight regression testing (the "manual QA is the
  only net for GTK/native widget behaviour" gap this ADR inherits)
- TASK-VTT103 — Evaluate unifying the Linux GTK tray onto the portable
  tray (ADR)
- STORY-VTT013 · DIRECT-VTT002
- TASK-VTT122 — shared `quit()` helper, the existing precedent for
  cross-tray behaviour unification
- `src/tray/linux.rs`, `src/tray/portable.rs`, `src/tray/mod.rs`,
  `src/tray/quit.rs`, `src/hotkey/portable.rs`
- `tray-icon` — https://github.com/tauri-apps/tray-icon
- `muda` — https://github.com/tauri-apps/muda
