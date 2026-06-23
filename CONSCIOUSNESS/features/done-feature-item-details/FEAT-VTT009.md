---
id: FEAT-VTT009
status: done
superseded_by: FEAT-VTT024
kano: performance
---

# FEAT-VTT009: CUDA GPU acceleration with cuDNN auto-detection (done — superseded)

## Description
**SUPERSEDED.** The original Python backend used CTranslate2 with CUDA and cuDNN for GPU-accelerated inference on NVIDIA cards. This required the CUDA Toolkit and cuDNN libraries to be installed. After the Rust rewrite (ADR-0003), whisper-rs uses Vulkan for GPU acceleration, which works on NVIDIA, AMD, and Intel GPUs without the CUDA Toolkit.

**Successor:** FEAT-VTT024 (Vulkan GPU acceleration, in backlog for Windows; Linux Vulkan delivered as part of v2.0.x).

## Why Done (not Maintained)
The Python CT2 backend that provided CUDA was deleted in TASK-VTT031. whisper-rs uses the Vulkan compute path. No CUDA or cuDNN dependency remains.

## Historical Acceptance Criteria
- [x] Python faster-whisper used CUDA when NVIDIA GPU and cuDNN were present — delivered in original version
- [x] CUDA path retired and deleted in v2.0.0 — verified via TASK-VTT031
- [x] Vulkan GPU path (Linux) works on NVIDIA RTX 2060 SUPER — verified in daily use on the Linux machine

## Linked Tasks
- TASK-VTT002, TASK-VTT031

## Parent Story
- STORY-VTT001
