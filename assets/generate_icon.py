#!/usr/bin/env python3
"""Generate assets/vtt.ico — the Windows app/installer icon (TASK-VTT108).

First-pass brand mark: a simple microphone glyph on a slate rounded-square,
matching the restrained look of the app's other UI rather than the tray's
transient state colours (green/red/amber are recording-state indicators, not
the app's identity). Re-run this script after editing to regenerate the ICO.

Requires Pillow (`pip install pillow`).
"""

from pathlib import Path

from PIL import Image, ImageDraw

SUPERSAMPLE = 1024
SIZES = [16, 32, 48, 256]

BG = (30, 41, 59, 255)  # slate-800
GLYPH = (248, 250, 252, 255)  # slate-50


def draw_icon(size: int) -> Image.Image:
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    corner = round(size * 0.22)
    draw.rounded_rectangle([0, 0, size - 1, size - 1], radius=corner, fill=BG)

    # A bold, simple capsule+stem+base silhouette — no thin arc. Legibility at
    # 16x16 (Explorer list view, taskbar, title bar) is the binding constraint;
    # an earlier draft's cradling arc anti-aliased into an unreadable smudge at
    # that size, so every remaining stroke here is thick relative to its size.
    cap_w = size * 0.34
    cap_h = size * 0.40
    cx = size / 2
    cap_top = size * 0.18
    draw.rounded_rectangle(
        [cx - cap_w / 2, cap_top, cx + cap_w / 2, cap_top + cap_h],
        radius=cap_w / 2,
        fill=GLYPH,
    )

    stem_top = cap_top + cap_h - size * 0.02  # slight overlap, no seam at small sizes
    stem_bottom = size * 0.80
    stem_w = size * 0.11
    draw.rounded_rectangle(
        [cx - stem_w / 2, stem_top, cx + stem_w / 2, stem_bottom],
        radius=stem_w / 2,
        fill=GLYPH,
    )

    foot_w = size * 0.34
    foot_h = size * 0.09
    draw.rounded_rectangle(
        [cx - foot_w / 2, stem_bottom - foot_h / 2, cx + foot_w / 2, stem_bottom + foot_h / 2],
        radius=foot_h / 2,
        fill=GLYPH,
    )

    return img


def main() -> None:
    out_dir = Path(__file__).parent
    master = draw_icon(SUPERSAMPLE)
    base = master.resize((max(SIZES), max(SIZES)), Image.LANCZOS)

    ico_path = out_dir / "vtt.ico"
    # Pillow's ICO writer resizes `base` itself for every entry in `sizes` —
    # passing the largest pre-rendered frame as the base (rather than the raw
    # supersample) keeps every embedded size a clean power-of-two downscale.
    base.save(ico_path, format="ICO", sizes=[(s, s) for s in SIZES])
    print(f"wrote {ico_path} ({ico_path.stat().st_size} bytes)")

    png_path = out_dir / "vtt-preview.png"
    base.save(png_path)
    print(f"wrote {png_path} (preview only, not shipped)")


if __name__ == "__main__":
    main()
