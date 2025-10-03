This directory holds vendored third-party code used by Voice to Text.

whisper.cpp (https://github.com/ggerganov/whisper.cpp)
- We include it as a git submodule or a plain clone.
- The build integrates a static library (libwhisper.a) for a zero-dependency Voice to Text.
- The bundled model (ggml-small.en.bin) is used by default at runtime.

Fetch options:
- Submodule: git submodule add --depth=1 https://github.com/ggerganov/whisper.cpp third_party/whisper.cpp
- Clone:     git clone --depth=1 https://github.com/ggerganov/whisper.cpp third_party/whisper.cpp

Build options (Makefile supports both):
- CMake (recommended): builds libwhisper.a into third_party/whisper.cpp/build/
- Make (fallback): attempts `make -C third_party/whisper.cpp libwhisper.a`

Note: Large model files are not kept under version control here. Place
ggml-small.en.bin in VTT.app/Contents/Resources during bundling, or use your
local copy at ~/whisper.cpp/models/.

