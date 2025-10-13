#!/usr/bin/env python3
import os
from PIL import Image, ImageDraw, ImageFont
import subprocess

# Create a simple icon with "VTT" text
size = 1024
img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# Background circle
margin = 100
draw.ellipse([margin, margin, size-margin, size-margin], fill=(52, 152, 219), outline=(41, 128, 185), width=20)

# Try to use system font
try:
    font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 300)
except:
    font = ImageFont.load_default()

# Draw "VTT" text
text = "VTT"
bbox = draw.textbbox((0, 0), text, font=font)
text_width = bbox[2] - bbox[0]
text_height = bbox[3] - bbox[1]
position = ((size - text_width) / 2, (size - text_height) / 2 - 50)
draw.text(position, text, fill=(255, 255, 255), font=font)

# Draw small microphone icon
mic_x = size // 2
mic_y = size - 350
# Microphone body
draw.rounded_rectangle([mic_x-40, mic_y-80, mic_x+40, mic_y+80], radius=20, fill=(255, 255, 255))
# Microphone stand
draw.rectangle([mic_x-10, mic_y+80, mic_x+10, mic_y+120], fill=(255, 255, 255))
draw.rectangle([mic_x-30, mic_y+120, mic_x+30, mic_y+130], fill=(255, 255, 255))

# Save the icon
img.save('/tmp/icon_1024.png')

# Create iconset directory
os.makedirs('/tmp/VTT.iconset', exist_ok=True)

# Generate all required sizes
sizes = [16, 32, 128, 256, 512, 1024]
for size in sizes:
    img_resized = img.resize((size, size), Image.Resampling.LANCZOS)
    img_resized.save(f'/tmp/VTT.iconset/icon_{size}x{size}.png')
    if size <= 512:
        img_resized_2x = img.resize((size*2, size*2), Image.Resampling.LANCZOS)
        img_resized_2x.save(f'/tmp/VTT.iconset/icon_{size}x{size}@2x.png')

print("Icon files created in /tmp/VTT.iconset")