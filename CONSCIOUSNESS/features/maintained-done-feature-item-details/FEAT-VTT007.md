---
id: FEAT-VTT007
status: deprecated
superseded_by: FEAT-VTT029
kano: must-have
---

# FEAT-VTT007: Homebrew cask distribution for macOS (deprecated — Python-era, not maintained for Rust)

## Description
**DONE / NOT ACTIVELY MAINTAINED.** A Homebrew cask formula was created for the original Python version of VTT. The formula is not currently updated for the Rust rewrite. There is no distributed macOS Rust binary, so users cannot install a working VTT via Homebrew today.

This feature will be re-delivered when STORY-VTT012 (macOS builds) is complete and a signed .app bundle is available.

## Why Done (not Maintained)
The original cask targeted the Python/ObjC binary. No Rust macOS binary is distributed. The cask is not kept current and is not tested. macOS distribution is on the backlog (FEAT-VTT029, STORY-VTT012).

## Historical Acceptance Criteria
- [x] `brew install --cask voice-to-text` installed the Python version on macOS — delivered
- [ ] Homebrew cask updated for Rust binary — NOT YET DONE

## Linked Tasks
- TASK-VTT007

## Parent Story
- STORY-VTT002
