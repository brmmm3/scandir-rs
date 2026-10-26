#include <cerrno>
#include <chrono>
#include <cstdlib>
#include <exception>
#include <iostream>
#include <stdexcept>
#include <string>

#include "cxxscandir/src/lib.rs.h"

namespace {

enum class Mode { Collect, CollectDiscard, Count, CollectExtended };

const char* mode_name(Mode mode) {
  switch (mode) {
    case Mode::Collect:
      return "collect";
    case Mode::CollectDiscard:
      return "collect-drop";
    case Mode::Count:
      return "count";
    case Mode::CollectExtended:
      return "collect-ext";
  }
  return "unknown";
}

bool parse_iterations(const char* value, long* iterations) {
  errno = 0;
  char* end = nullptr;
  const long parsed = std::strtol(value, &end, 10);
  if (errno != 0 || end == value || *end != '\0' || parsed < 1) {
    return false;
  }
  *iterations = parsed;
  return true;
}

std::size_t run_once(Mode mode, const std::string& root) {
  cxxscandir::ScanOptions options;
  options.root = rust::String(root);
  options.extended = mode == Mode::CollectExtended;

  if (mode == Mode::Count) {
    const auto stats = cxxscandir::count(options);
    if (!stats.errors.empty()) {
      throw std::runtime_error("count reported traversal errors");
    }
    return static_cast<std::size_t>(stats.dirs) +
           static_cast<std::size_t>(stats.files) +
           static_cast<std::size_t>(stats.symlinks);
  }

  const auto result = cxxscandir::scan(options);
  if (!result.errors.empty()) {
    throw std::runtime_error("scan reported traversal errors");
  }
  // The wrapper always returns entries; this measures dropping the collection
  // immediately, not a no-output scan. Output materialization remains included.
  return result.entries.size();
}

}  // namespace

int main(int argc, char** argv) {
  if (argc < 2 || argc > 3) {
    std::cerr << "Usage: " << argv[0] << " ROOT [ITERATIONS]\n";
    return 1;
  }
  long iterations = 10;
  if (argc == 3 && !parse_iterations(argv[2], &iterations)) {
    std::cerr << "ITERATIONS must be a positive integer\n";
    return 1;
  }

  const Mode modes[] = {Mode::Collect, Mode::CollectDiscard, Mode::Count,
                        Mode::CollectExtended};
  try {
    for (Mode mode : modes) {
      double elapsed_seconds = 0.0;
      std::size_t entries = 0;
      for (long i = -1; i < iterations; ++i) {
        const auto start = std::chrono::steady_clock::now();
        entries = run_once(mode, argv[1]);
        const auto elapsed = std::chrono::steady_clock::now() - start;
        if (i >= 0) {
          elapsed_seconds += std::chrono::duration<double>(elapsed).count();
        }
      }
      std::cout << mode_name(mode) << ": "
                << elapsed_seconds * 1000.0 / static_cast<double>(iterations)
                << " ms/scan (" << iterations << " iterations, " << entries
                << " entries)\n";
    }
  } catch (const std::exception& error) {
    std::cerr << "benchmark failed: " << error.what() << '\n';
    return 1;
  }
  return 0;
}