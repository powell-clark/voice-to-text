# VTT Makefile - Clean Objective-C Menu Bar App

CC = clang
# Build as Objective-C++; separate compile and link flags
COMPILE_FLAGS = -fobjc-arc -x objective-c++ -std=c++17
LINK_FLAGS = -framework Cocoa -framework CoreAudio -framework AudioToolbox -framework ApplicationServices -framework AVFoundation -framework IOKit -framework Metal -framework Accelerate -lc++
APP_NAME = VTT

# Vendor paths
WHISPER_DIR := third_party/whisper.cpp
WHISPER_BUILD_DIR := $(WHISPER_DIR)/build
WHISPER_VENDOR_LIB := $(WHISPER_BUILD_DIR)/src/libwhisper.a
GGML_LIBS := $(WHISPER_BUILD_DIR)/ggml/src/libggml.a $(WHISPER_BUILD_DIR)/ggml/src/libggml-base.a $(WHISPER_BUILD_DIR)/ggml/src/libggml-cpu.a $(WHISPER_BUILD_DIR)/ggml/src/ggml-metal/libggml-metal.a $(WHISPER_BUILD_DIR)/ggml/src/ggml-blas/libggml-blas.a

# Optional: link with libwhisper if available to avoid external CLI
WHISPER_LIB_CANDIDATES = $(WHISPER_VENDOR_LIB) \
                         $(HOME)/whisper.cpp/build/libwhisper.a \
                         $(HOME)/whisper.cpp/build/lib/libwhisper.a \
                         /opt/homebrew/lib/libwhisper.a \
                         /usr/local/lib/libwhisper.a
WHISPER_LIB := $(firstword $(wildcard $(WHISPER_LIB_CANDIDATES)))
ifneq ($(WHISPER_LIB),)
  COMPILE_FLAGS += -DUSE_WHISPER_LIB
  LDFLAGS += $(WHISPER_LIB) $(GGML_LIBS)
  # Common include locations for whisper.h (prefer vendor)
  COMPILE_FLAGS += -I$(WHISPER_DIR)/include -I$(WHISPER_DIR)/ggml/include -I$(WHISPER_DIR) -I$(HOME)/whisper.cpp -I/opt/homebrew/include -I/usr/local/include
  $(info ✅ Using libwhisper: $(WHISPER_LIB))
else
  $(info ℹ️  libwhisper not found; will use external whisper-cli at runtime)
endif

all: whisper-lib app

# Build the menu bar app binary (Objective-C)
app: $(APP_NAME)

$(APP_NAME): VTTDaemon.m
	$(CC) $(COMPILE_FLAGS) -c VTTDaemon.m -o VTTDaemon.o
	$(CC) $(LINK_FLAGS) VTTDaemon.o $(LDFLAGS) -o $(APP_NAME)
	rm -f VTTDaemon.o
	@echo "✅ Built VTT menu bar app"

# Create the complete app bundle
bundle: app
	mkdir -p $(APP_NAME).app/Contents/MacOS
	mkdir -p $(APP_NAME).app/Contents/Resources
	cp $(APP_NAME) $(APP_NAME).app/Contents/MacOS/$(APP_NAME)
	# Optionally bundle whisper-cli if present (set WHISPER_CLI or use common locations)
	@if [ -n "$(WHISPER_CLI)" ] && [ -x "$(WHISPER_CLI)" ]; then \
		cp "$(WHISPER_CLI)" $(APP_NAME).app/Contents/MacOS/whisper-cli && chmod +x $(APP_NAME).app/Contents/MacOS/whisper-cli && echo "✅ Bundled whisper-cli (from $(WHISPER_CLI))"; \
	elif [ -x $$HOME/whisper.cpp/build/bin/whisper-cli ]; then \
		cp $$HOME/whisper.cpp/build/bin/whisper-cli $(APP_NAME).app/Contents/MacOS/whisper-cli && chmod +x $(APP_NAME).app/Contents/MacOS/whisper-cli && echo "✅ Bundled whisper-cli (from ~/whisper.cpp)"; \
	elif [ -x /opt/homebrew/bin/whisper-cli ]; then \
		cp /opt/homebrew/bin/whisper-cli $(APP_NAME).app/Contents/MacOS/whisper-cli && chmod +x $(APP_NAME).app/Contents/MacOS/whisper-cli && echo "✅ Bundled whisper-cli (from Homebrew)"; \
	elif [ -x /opt/homebrew/bin/whisper-cpp ]; then \
		cp /opt/homebrew/bin/whisper-cpp $(APP_NAME).app/Contents/MacOS/whisper-cli && chmod +x $(APP_NAME).app/Contents/MacOS/whisper-cli && echo "✅ Bundled whisper-cpp as whisper-cli"; \
	else \
		echo "ℹ️  whisper-cli not bundled (set WHISPER_CLI or install whisper.cpp)"; \
	fi
	# Create Info.plist
	@echo '<?xml version="1.0" encoding="UTF-8"?>' > $(APP_NAME).app/Contents/Info.plist
	@echo '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' >> $(APP_NAME).app/Contents/Info.plist
	@echo '<plist version="1.0"><dict>' >> $(APP_NAME).app/Contents/Info.plist
	@echo '    <key>CFBundleExecutable</key><string>VTT</string>' >> $(APP_NAME).app/Contents/Info.plist
	@echo '    <key>CFBundleIconFile</key><string>AppIcon</string>' >> $(APP_NAME).app/Contents/Info.plist
	@echo '    <key>CFBundleIdentifier</key><string>com.local.vtt</string>' >> $(APP_NAME).app/Contents/Info.plist
	@echo '    <key>CFBundleName</key><string>VTT</string>' >> $(APP_NAME).app/Contents/Info.plist
	@echo '    <key>CFBundlePackageType</key><string>APPL</string>' >> $(APP_NAME).app/Contents/Info.plist
	@echo '    <key>CFBundleVersion</key><string>1.0</string>' >> $(APP_NAME).app/Contents/Info.plist
	@echo '    <key>LSUIElement</key><true/>' >> $(APP_NAME).app/Contents/Info.plist
	@echo '    <key>NSMicrophoneUsageDescription</key><string>VTT needs microphone access.</string>' >> $(APP_NAME).app/Contents/Info.plist
	@echo '</dict></plist>' >> $(APP_NAME).app/Contents/Info.plist
	# Bundle small model if it exists
	@if [ -f $(APP_NAME).app/Contents/Resources/ggml-small.en.bin ]; then \
		echo "✅ Small model already bundled"; \
	elif [ -f $(WHISPER_DIR)/models/ggml-small.en.bin ]; then \
		cp $(WHISPER_DIR)/models/ggml-small.en.bin $(APP_NAME).app/Contents/Resources/; \
		echo "✅ Bundled small model (vendor)"; \
	elif [ -f $$HOME/whisper.cpp/models/ggml-small.en.bin ]; then \
		cp $$HOME/whisper.cpp/models/ggml-small.en.bin $(APP_NAME).app/Contents/Resources/; \
		echo "✅ Bundled small model"; \
	else \
		echo "⚠️  Small model not found to bundle; place ggml-small.en.bin in Resources"; \
	fi
	@echo "✅ Created VTT.app bundle"

# Create app icon
icon:
	python3 create_icon.py
	iconutil -c icns /tmp/VTT.iconset -o $(APP_NAME).app/Contents/Resources/AppIcon.icns
	@echo "✅ Created app icon"

# Build everything including icon
complete: app bundle icon
	@echo "✅ VTT.app ready for installation"

# Test the Objective-C implementation
test: test_vtt_daemon
	./test_vtt_daemon

test_vtt_daemon: test_vtt_daemon.m
	$(CC) $(COMPILE_FLAGS) $(LINK_FLAGS) -o test_vtt_daemon test_vtt_daemon.m
	@echo "✅ Built VTT tests"

clean:
	rm -f $(APP_NAME)
	rm -rf $(APP_NAME).app
	rm -rf /tmp/VTT.iconset
	rm -f test_vtt_daemon

.PHONY: all app bundle icon complete test clean vendor-whisper whisper-lib

# Helper to fetch vendor whisper.cpp source
vendor-whisper:
	bash scripts/vendor_whisper.sh

# Build vendor libwhisper if vendor is present
whisper-lib:
	@if [ -d $(WHISPER_DIR) ]; then \
		set -e; \
		if command -v cmake >/dev/null 2>&1; then \
			mkdir -p $(WHISPER_BUILD_DIR); \
			cd $(WHISPER_BUILD_DIR) && cmake -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_EXAMPLES=OFF -DBUILD_SHARED_LIBS=OFF ..; \
			cd $(WHISPER_BUILD_DIR) && cmake --build . -j; \
			echo "✅ Built libwhisper with CMake"; \
		elif [ -f $(WHISPER_DIR)/Makefile ]; then \
			$(MAKE) -C $(WHISPER_DIR) libwhisper.a || true; \
			echo "✅ Built libwhisper with Make"; \
		else \
			echo "⚠️  whisper.cpp present but no build system found"; \
		fi; \
	else \
		echo "ℹ️  No vendor whisper.cpp; skipping whisper-lib"; \
	fi
