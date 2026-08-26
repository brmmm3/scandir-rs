GCC_BIN ?= $(shell which gcc)
CARGO_BIN ?= $(shell which cargo)

run: build-debug

clean:
	rm -rf bin

build:
	$(CARGO_BIN) build --release
	@mkdir -p bin
	$(GCC_BIN) -o ./bin/scandir_collect ./examples/scandir_collect.c -I. -L. -l:../target/release/libcscandir.so

build-debug:
	$(CARGO_BIN) build
	@mkdir -p bin
	$(GCC_BIN) -o ./bin/scandir_collect ./examples/scandir_collect.c -I. -L. -l:../target/debug/libcscandir.so
