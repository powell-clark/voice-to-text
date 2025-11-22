CC = gcc
CFLAGS = -Wall -Wextra -g -I../src/common
LDFLAGS = -lm

.PHONY: all clean

all: crash_test

crash_test: crash_test.c ../src/common/logging.c ../src/common/crash_handler.c
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

clean:
	rm -f crash_test
