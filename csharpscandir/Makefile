CARGO ?= cargo
DOTNET ?= dotnet
PROFILE ?= release
TARGET_DIR ?= $(abspath ../target)
ROOT ?= .
ITERATIONS ?= 20
EXT ?= 0
CONFIGURATION = $(if $(filter release,$(PROFILE)),Release,Debug)
RUN_EXT = $(if $(filter 1 true yes,$(EXT)),--ext,)

.PHONY: build library examples test run run-count run-scandir run-walk run-cifs benchmark clean
.DEFAULT_GOAL := build

library:
	$(CARGO) build -p cscandir $(if $(filter release,$(PROFILE)),--release,) --target-dir $(TARGET_DIR)

build: library examples

examples: library
	$(DOTNET) build examples/examples.csproj --configuration $(CONFIGURATION)

test: library
	$(DOTNET) build tests/tests.csproj --configuration $(CONFIGURATION)
	$(DOTNET) run --no-restore --project tests/tests.csproj --configuration $(CONFIGURATION)

run: run-scandir

run-count: build
	$(DOTNET) run --no-build --project examples/examples.csproj --configuration $(CONFIGURATION) -- count "$(ROOT)" $(RUN_EXT)

run-scandir: build
	$(DOTNET) run --no-build --project examples/examples.csproj --configuration $(CONFIGURATION) -- scandir "$(ROOT)" $(RUN_EXT)

run-walk: build
	$(DOTNET) run --no-build --project examples/examples.csproj --configuration $(CONFIGURATION) -- walk "$(ROOT)" $(RUN_EXT)

run-cifs: build
	$(DOTNET) run --no-build --project examples/examples.csproj --configuration $(CONFIGURATION) -- cifs "$(ROOT)"

benchmark: build
	$(DOTNET) run --no-build --project examples/examples.csproj --configuration $(CONFIGURATION) -- benchmark "$(ROOT)" "$(ITERATIONS)"

clean:
	rm -rf examples/bin examples/obj tests/bin tests/obj bin obj