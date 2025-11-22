# Test Makefile
CC = gcc
CFLAGS = -Wall -Wextra -g -I../src/common
LDFLAGS = -lm

TEST_BINARIES = test_settings

.PHONY: all clean test

all: $(TEST_BINARIES)

test_settings: test_settings.c ../src/common/settings.c
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

test: all
	@echo "Running C tests..."
	@./test_settings
	@echo ""
	@echo "Running Python tests..."
	@python3 test_transcribe.py

clean:
	rm -f $(TEST_BINARIES)
	find . -name "*.pyc" -delete
	find . -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
