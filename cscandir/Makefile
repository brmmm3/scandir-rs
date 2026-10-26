CC ?= cc
CARGO ?= cargo
PROFILE ?= release
TARGET_DIR ?= $(abspath ../target)
LIB_DIR = $(TARGET_DIR)/$(PROFILE)
CARGO_FLAGS = $(if $(filter release,$(PROFILE)),--release,)
CFLAGS ?= -O2 -std=c11 -Wall -Wextra -Werror
CPPFLAGS += -Iinclude
LDFLAGS += -L$(LIB_DIR) -Wl,-rpath,$(LIB_DIR)
LDLIBS += -lcscandir
ROOT ?= .
ITERATIONS ?= 20

.PHONY: build build-debug run run-count run-scandir run-walk run-cifs \
	test test-asan benchmark clean library examples
.DEFAULT_GOAL := build

library:
	$(CARGO) build -p cscandir $(CARGO_FLAGS) --target-dir $(TARGET_DIR)

build: examples

examples: library
	@mkdir -p bin
	$(CC) $(CPPFLAGS) $(CFLAGS) examples/count.c $(LDFLAGS) $(LDLIBS) -o bin/count
	$(CC) $(CPPFLAGS) $(CFLAGS) examples/scandir.c $(LDFLAGS) $(LDLIBS) -o bin/scandir
	$(CC) $(CPPFLAGS) $(CFLAGS) examples/walk.c $(LDFLAGS) $(LDLIBS) -o bin/walk
	$(CC) $(CPPFLAGS) $(CFLAGS) examples/cifs_ctime_check.c $(LDFLAGS) $(LDLIBS) -o bin/cifs_ctime_check

build-debug:
	$(MAKE) build PROFILE=debug

run: build
	./bin/scandir "$(ROOT)"

run-count: build
	./bin/count "$(ROOT)" $(if $(filter 1 true yes,$(EXT)),--ext,)

run-scandir: build
	./bin/scandir "$(ROOT)" $(if $(filter 1 true yes,$(EXT)),--ext,)

run-walk: build
	./bin/walk "$(ROOT)" $(if $(filter 1 true yes,$(EXT)),--ext,)

run-cifs: build
	./bin/cifs_ctime_check "$(ROOT)"

test: library
	$(CARGO) test -p cscandir --target-dir $(TARGET_DIR)
	@mkdir -p bin
	$(CC) $(CPPFLAGS) $(CFLAGS) tests/test_api.c $(LDFLAGS) $(LDLIBS) -o bin/test_api
	./bin/test_api

test-asan: library
	@mkdir -p bin
	$(CC) $(CPPFLAGS) $(CFLAGS) -g -fsanitize=address,undefined -fno-omit-frame-pointer tests/test_api.c $(LDFLAGS) $(LDLIBS) -o bin/test_api_asan
	ASAN_OPTIONS=detect_leaks=1 ./bin/test_api_asan

benchmark: library
	@mkdir -p bin
	$(CC) $(CPPFLAGS) $(CFLAGS) examples/benchmark.c $(LDFLAGS) $(LDLIBS) -o bin/benchmark
	./bin/benchmark "$(ROOT)" "$(ITERATIONS)"

clean:
	rm -rf bin
