CARGO ?= cargo
CXX ?= c++
TARGET_DIR ?= ../target
PROFILE ?= release
CXXFLAGS ?= -O2 -std=c++11 -Wall -Wextra -pedantic
ROOT ?= .

ifeq ($(PROFILE),release)
CARGO_FLAGS := --release
endif

LIB_DIR := $(TARGET_DIR)/$(PROFILE)
EXAMPLE := bin/scandir

.PHONY: all build run clean

all: build

build:
	$(CARGO) build -p cxxscandir $(CARGO_FLAGS)
	@mkdir -p bin
	@bridge_dir=$$(find "$(LIB_DIR)/build" -path '*/out/cxxbridge/include' -type d -print -quit); \
	 test -n "$$bridge_dir" || (echo "cxx generated include directory not found" >&2; exit 1); \
	 $(CXX) $(CXXFLAGS) -I"$$bridge_dir" \
		examples/scandir.cc "$(LIB_DIR)/libcxxscandir.a" -lstdc++ -lpthread -ldl -lm \
		-Wl,-rpath,"$(abspath $(LIB_DIR))" -o $(EXAMPLE)

run: build
	./$(EXAMPLE) "$(ROOT)"

clean:
	rm -rf bin