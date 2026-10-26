# goscandir Makefile; run targets from the module directory.

GO := go
BUILD_DIR := bin
ROOT ?= ..

.PHONY: all build test test-race clean examples install bench vet fmt debug msan asan ci

all: build test

build:
	@mkdir -p $(BUILD_DIR)
	$(GO) build -o $(BUILD_DIR)/goscandir_collect ./examples/collect
	$(GO) build -o $(BUILD_DIR)/goscandir_count ./examples/count
	$(GO) build -o $(BUILD_DIR)/goscandir_walk ./examples/walk
	$(GO) build -o $(BUILD_DIR)/goscandir_filter ./examples/filter

test:
	CGO_ENABLED=1 $(GO) test -v ./...

test-race:
	CGO_ENABLED=1 $(GO) test -race ./...

examples: build
	@echo "Running examples..."
	@echo "=== scandir_collect ==="
	./bin/goscandir_collect "$(ROOT)"
	@echo "=== scandir_count ==="
	./bin/goscandir_count "$(ROOT)"
	@echo ""
	@echo "=== scandir_walk ==="
	./bin/goscandir_walk "$(ROOT)"
	@echo "=== scandir_filter ==="
	./bin/goscandir_filter "$(ROOT)"

bench:
	$(GO) test -bench=. -benchmem ./...

install:
	$(GO) install ./examples/collect
	$(GO) install ./examples/count
	$(GO) install ./examples/walk
	$(GO) install ./examples/filter

clean:
	rm -rf $(BUILD_DIR)

vet:
	$(GO) vet ./...

fmt:
	$(GO) fmt ./...

# Build with CGO debug flags for debugging
debug:
	$(GO) build -gcflags="all=-N -l" -o $(BUILD_DIR)/goscandir_collect_d ./examples/collect
	$(GO) build -gcflags="all=-N -l" -o $(BUILD_DIR)/goscandir_count_d ./examples/count
	$(GO) build -gcflags="all=-N -l" -o $(BUILD_DIR)/goscandir_walk_d ./examples/walk
	$(GO) build -gcflags="all=-N -l" -o $(BUILD_DIR)/goscandir_filter_d ./examples/filter

# Run tests with memory sanitizer (requires clang)
msan:
	CC=clang CGO_CFLAGS="-fsanitize=memory -fno-omit-frame-pointer" \
	CGO_LDFLAGS="-fsanitize=memory" \
	$(GO) test ./...

# Run tests with address sanitizer
asan:
	CGO_CFLAGS="-fsanitize=address -fno-omit-frame-pointer" \
	CGO_LDFLAGS="-fsanitize=address" \
	$(GO) test ./...

# CI target - runs all checks
ci: fmt vet test-race

.PHONY: all build test test-race clean examples install bench vet fmt debug msan asan ci