# goscandir Makefile

GO := go
CGO_ENABLED := 1
BUILD_DIR := bin

.PHONY: all build test test-race test-sanitized clean examples install

all: build test

build:
	@mkdir -p $(BUILD_DIR)
	$(GO) build -o $(BUILD_DIR)/goscandir_collect ./examples/scandir_collect.go
	$(GO) build -o $(BUILD_DIR)/goscandir_walk ./examples/scandir_walk.go

test:
	$(GO) test -v .

test-race:
	$(GO) test -race .

test-sanitized:
	$(GO) test -race -count=1 ./...

examples: build
	@echo "Running examples..."
	@echo "=== scandir_collect ==="
	./bin/goscandir_collect .
	@echo ""
	@echo "=== scandir_walk ==="
	./bin/goscandir_walk .

bench:
	$(GO) test -bench=. -benchmem ./...

install:
	$(GO) install ./examples/scandir_collect.go
	$(GO) install ./examples/scandir_walk.go

clean:
	rm -rf $(BUILD_DIR)

vet:
	$(GO) vet .

fmt:
	$(GO) fmt .

# Build with CGO debug flags for debugging
debug:
	$(GO) build -gcflags="all=-N -l" -o $(BUILD_DIR)/goscandir_collect_d ./examples/scandir_collect.go
	$(GO) build -gcflags="all=-N -l" -o $(BUILD_DIR)/goscandir_walk_d ./examples/scandir_walk.go

# Run tests with memory sanitizer (requires clang)
msan:
	CC=clang CGO_CFLAGS="-fsanitize=memory -fno-omit-frame-pointer" \
	CGO_LDFLAGS="-fsanitize=memory" \
	$(GO) test -race ./...

# Run tests with address sanitizer
asan:
	CGO_CFLAGS="-fsanitize=address -fno-omit-frame-pointer" \
	CGO_LDFLAGS="-fsanitize=address" \
	$(GO) test -race ./...

# CI target - runs all checks
ci: fmt vet test-race

.PHONY: all build test test-race test-sanitized clean examples install bench vet fmt debug msan asan ci