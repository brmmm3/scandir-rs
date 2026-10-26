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
TEST := bin/test_api
RUST_LIBS := "$(LIB_DIR)/libcxxscandir.a" -lstdc++ -lpthread -ldl -lm

.PHONY: all build run test clean

all: build

build:
	$(CARGO) build -p cxxscandir $(CARGO_FLAGS)
	@mkdir -p bin
	@bridge_dir=$$(find "$(LIB_DIR)/build" -path '*/out/cxxbridge/include' -type d -print -quit); \
	 test -n "$$bridge_dir" || (echo "cxx generated include directory not found" >&2; exit 1); \
	 for source in examples/*.cc; do \
		name=$$(basename "$${source%.cc}"); \
		$(CXX) $(CXXFLAGS) -I"$$bridge_dir" "$$source" $(RUST_LIBS) \
			-Wl,-rpath,"$(abspath $(LIB_DIR))" -o "bin/$$name" || exit $$?; \
	 done

run: build
	./$(EXAMPLE) "$(ROOT)"

test: build
	@bridge_dir=$$(find "$(LIB_DIR)/build" -path '*/out/cxxbridge/include' -type d -print -quit); \
	 test -n "$$bridge_dir" || (echo "cxx generated include directory not found" >&2; exit 1); \
	 $(CXX) $(CXXFLAGS) -I"$$bridge_dir" tests/test_api.cc $(RUST_LIBS) \
		-Wl,-rpath,"$(abspath $(LIB_DIR))" -o $(TEST)
	./$(TEST)

clean:
	rm -rf bin