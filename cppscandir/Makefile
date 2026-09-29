# cppscandir Makefile

CXX := g++
CXXFLAGS := -std=c++17 -Wall -Wextra -Werror -I../cscandir/include -Iinclude
LDFLAGS := -L../target/release -lcscandir -Wl,-rpath,../target/release

EXAMPLES := scandir_collect scandir_walk scandir_count scandir_filter

.PHONY: all build test clean examples

all: build test

build: $(addprefix bin/,$(EXAMPLES))

bin/scandir_collect: examples/scandir_collect.cc src/cppscandir.cc include/cppscandir.h
	@mkdir -p bin
	$(CXX) $(CXXFLAGS) -o $@ examples/scandir_collect.cc src/cppscandir.cc $(LDFLAGS)

bin/scandir_walk: examples/scandir_walk.cc src/cppscandir.cc include/cppscandir.h
	@mkdir -p bin
	$(CXX) $(CXXFLAGS) -o $@ examples/scandir_walk.cc src/cppscandir.cc $(LDFLAGS)

bin/scandir_count: examples/scandir_count.cc src/cppscandir.cc include/cppscandir.h
	@mkdir -p bin
	$(CXX) $(CXXFLAGS) -o $@ examples/scandir_count.cc src/cppscandir.cc $(LDFLAGS)

bin/scandir_filter: examples/scandir_filter.cc src/cppscandir.cc include/cppscandir.h
	@mkdir -p bin
	$(CXX) $(CXXFLAGS) -o $@ examples/scandir_filter.cc src/cppscandir.cc $(LDFLAGS)

test: build
	@echo "Running tests..."
	g++ -std=c++17 -Wall -Wextra -Werror -I../cscandir/include -Iinclude \
		-o bin/cppscandir_test tests/cppscandir_test.cc src/cppscandir.cc \
		-lgtest -lgtest_main -lpthread \
		-L../target/release -lcscandir -Wl,-rpath,../target/release
	./bin/cppscandir_test

clean:
	rm -rf bin

run-collect: bin/scandir_collect
	./bin/scandir_collect .

run-walk: bin/scandir_walk
	./bin/scandir_walk .

run-count: bin/scandir_count
	./bin/scandir_count .

run-filter: bin/scandir_filter
	./bin/scandir_filter .

examples: build
	@echo "=== scandir_collect ==="
	./bin/scandir_collect .
	@echo ""
	@echo "=== scandir_walk ==="
	./bin/scandir_walk .
	@echo ""
	@echo "=== scandir_count ==="
	./bin/scandir_count .
	@echo ""
	@echo "=== scandir_filter ==="
	./bin/scandir_filter .

.PHONY: all build test clean examples run-collect run-walk run-count run-filter