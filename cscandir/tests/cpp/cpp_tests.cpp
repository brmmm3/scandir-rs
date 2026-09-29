// Tests for the C++ wrapper.
//
// These exercise the wrapper the way a C++ caller would: through the public
// API only, with no manual cleanup, since ownership is handled by RAII.

#include "cscandir.hpp"

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <functional>
#include <string>
#include <vector>

namespace fs = std::filesystem;

namespace {

int g_failures = 0;
int g_checks = 0;
std::string g_current;

void check(bool condition, const char* expression, int line) {
    ++g_checks;
    if (!condition) {
        ++g_failures;
        std::fprintf(stderr, "  FAIL [%s] line %d: %s\n", g_current.c_str(),
                     line, expression);
    }
}

template <typename A, typename B>
void check_eq(const A& actual, const B& expected, const char* expression,
              int line) {
    ++g_checks;
    if (!(actual == expected)) {
        ++g_failures;
        std::fprintf(stderr, "  FAIL [%s] line %d: %s\n", g_current.c_str(),
                     line, expression);
    }
}

#define CHECK(cond) check((cond), #cond, __LINE__)
#define CHECK_EQ(actual, expected) check_eq((actual), (expected), #actual, __LINE__)

using TestFn = std::function<void()>;

struct TestCase {
    const char* name;
    TestFn fn;
};

std::vector<TestCase>& registry() {
    static std::vector<TestCase> tests;
    return tests;
}

struct Registrar {
    Registrar(const char* name, TestFn fn) { registry().push_back({name, std::move(fn)}); }
};

#define TEST(name)                                            \
    void test_##name();                                       \
    Registrar registrar_##name(#name, test_##name);            \
    void test_##name()

/// A scratch directory tree with predictable contents.
class Tree {
  public:
    explicit Tree(const std::string& name) {
        path_ = fs::temp_directory_path() / ("cscandir_cpp_" + name);
        fs::remove_all(path_);
        fs::create_directories(path_ / "sub" / "deep");
        write("a.txt");
        write("b.txt");
        write("sub/c.txt");
        write("sub/deep/d.txt");
    }

    ~Tree() {
        std::error_code ignored;
        fs::remove_all(path_, ignored);
    }

    Tree(const Tree&) = delete;
    Tree& operator=(const Tree&) = delete;

    const fs::path& path() const { return path_; }
    std::string str() const { return path_.string(); }

  private:
    void write(const std::string& rel) {
        std::ofstream out(path_ / rel);
        out << "content";
    }

    fs::path path_;
};

std::size_t count_files(const cscandir::CollectResult& result) {
    return static_cast<std::size_t>(
        std::count_if(result.entries.begin(), result.entries.end(),
                      [](const cscandir::Entry& e) { return e.is_file; }));
}

std::size_t count_dirs(const cscandir::CollectResult& result) {
    return static_cast<std::size_t>(
        std::count_if(result.entries.begin(), result.entries.end(),
                      [](const cscandir::Entry& e) { return e.is_dir; }));
}

// ---------------------------------------------------------------------------

TEST(collect_returns_entries_and_statistics) {
    Tree tree("collect");
    cscandir::CollectResult result = cscandir::collect(tree.path());

    // 4 files + 2 dirs; the scan root itself is not reported.
    CHECK_EQ(result.entries.size(), std::size_t{6});
    CHECK_EQ(count_files(result), std::size_t{4});
    CHECK_EQ(count_dirs(result), std::size_t{2});
    CHECK_EQ(result.stats.files, 4);
    CHECK_EQ(result.stats.dirs, 2);
    CHECK(result.stats.duration > 0.0);
    CHECK(result.errors.empty());
}

TEST(entries_carry_paths_types_and_times) {
    Tree tree("entries");
    cscandir::CollectResult result = cscandir::collect(tree.path());

    for (const cscandir::Entry& entry : result.entries) {
        CHECK(!entry.path.empty());
        CHECK(entry.ctime > 0.0);
        CHECK(entry.mtime > 0.0);
        CHECK(entry.atime > 0.0);
        if (entry.is_file) {
            CHECK(entry.size > 0);
        }
    }
}

TEST(base_return_type_leaves_ext_fields_unset) {
    Tree tree("base");
    cscandir::CollectResult result = cscandir::collect(tree.path());

    for (const cscandir::Entry& entry : result.entries) {
        CHECK(!entry.has_ext);
        CHECK_EQ(entry.mode, 0u);
        CHECK_EQ(entry.ino, std::uint64_t{0});
    }
}

TEST(extended_return_type_populates_ext_fields) {
    Tree tree("ext");
    cscandir::Options options;
    options.return_type(cscandir::ReturnType::ext);
    cscandir::CollectResult result = cscandir::collect(tree.path(), options);

    CHECK(!result.entries.empty());
    for (const cscandir::Entry& entry : result.entries) {
        CHECK(entry.has_ext);
        CHECK(entry.ino != 0);
    }
}

TEST(count_returns_statistics_without_entries) {
    Tree tree("count");
    cscandir::Statistics stats = cscandir::count(tree.path());

    CHECK_EQ(stats.files, 4);
    CHECK_EQ(stats.dirs, 2);
    CHECK(stats.duration > 0.0);
}

TEST(walk_groups_entries_per_directory) {
    Tree tree("walk");
    cscandir::WalkResult result = cscandir::walk(tree.path());

    CHECK_EQ(result.toc.files.size(), std::size_t{4});
    CHECK_EQ(result.toc.dirs.size(), std::size_t{2});
    CHECK(!result.toc.empty());
    CHECK(result.roots.size() >= 2);

    // Relative directory names: "" for the root, plus "sub" and "sub/deep".
    // The crate emits OS-native separators, so normalise before comparing.
    const std::vector<std::string> expected_dirs = {"", "sub", "sub/deep"};
    std::size_t per_root_files = 0;
    for (const cscandir::WalkEntry& entry : result.roots) {
        std::string normalized = entry.dir;
        std::replace(normalized.begin(), normalized.end(), '\\', '/');
        CHECK(std::find(expected_dirs.begin(), expected_dirs.end(), normalized) !=
              expected_dirs.end());
        per_root_files += entry.toc.files.size();
    }
    CHECK_EQ(per_root_files, std::size_t{4});
}

TEST(walk_toc_lists_are_populated) {
    Tree tree("walk_toc");
    cscandir::WalkResult result = cscandir::walk(tree.path());

    for (const std::string& file : result.toc.files) {
        CHECK(file.size() > 4);
        CHECK(file.substr(file.size() - 4) == ".txt");
    }
}

TEST(max_depth_limits_results) {
    Tree tree("depth");
    cscandir::Options options;
    options.max_depth(1);
    cscandir::CollectResult result = cscandir::collect(tree.path(), options);

    // Only the root's direct children: 2 files + 1 directory.
    CHECK_EQ(result.entries.size(), std::size_t{3});
}

TEST(max_depth_zero_means_unlimited) {
    Tree tree("depth_zero");
    cscandir::Options options;
    options.max_depth(0);
    cscandir::CollectResult result = cscandir::collect(tree.path(), options);

    CHECK_EQ(result.entries.size(), std::size_t{6});
}

TEST(file_include_filter_applies) {
    Tree tree("include");
    cscandir::Options options;
    options.file_include({"*.txt"});
    cscandir::CollectResult result = cscandir::collect(tree.path(), options);

    // Directories are still traversed; only the file list is filtered.
    CHECK_EQ(count_files(result), std::size_t{4});
    for (const cscandir::Entry& entry : result.entries) {
        if (entry.is_file) {
            CHECK(entry.path.size() > 4);
            CHECK(entry.path.substr(entry.path.size() - 4) == ".txt");
        }
    }
}

TEST(dir_exclude_filter_applies) {
    Tree tree("exclude");
    cscandir::Options options;
    options.dir_exclude({"sub"});
    cscandir::CollectResult result = cscandir::collect(tree.path(), options);

    // `sub` and everything below it is skipped.
    CHECK_EQ(result.entries.size(), std::size_t{2});
    CHECK_EQ(count_files(result), std::size_t{2});
}

TEST(skip_hidden_filter_applies) {
    Tree tree("hidden");
    {
        std::ofstream out(tree.path() / ".hidden.txt");
        out << "x";
    }

    cscandir::Options options;
    options.skip_hidden(true);
    cscandir::CollectResult result = cscandir::collect(tree.path(), options);

    CHECK_EQ(result.entries.size(), std::size_t{6});
    for (const cscandir::Entry& entry : result.entries) {
        CHECK(entry.path.find(".hidden") == std::string::npos);
    }
}

TEST(options_are_reusable_across_calls) {
    Tree tree("reuse");
    cscandir::Options options;
    options.file_include({"*.txt"}).sorted(true).case_sensitive(true);

    for (int i = 0; i < 20; ++i) {
        cscandir::CollectResult result = cscandir::collect(tree.path(), options);
        CHECK_EQ(count_files(result), std::size_t{4});
    }

    // The same options must still work for the other entry points.
    cscandir::Statistics stats = cscandir::count(tree.path(), options);
    CHECK_EQ(stats.files, 4);
    cscandir::WalkResult walked = cscandir::walk(tree.path(), options);
    CHECK_EQ(walked.toc.files.size(), std::size_t{4});
}

TEST(options_are_copyable) {
    Tree tree("copy");
    cscandir::Options original;
    original.file_include({"*.txt"});
    cscandir::Options copy = original;

    // The copy carries its own patterns; the original is unaffected.
    CHECK_EQ(cscandir::collect(tree.path(), copy).entries.size(),
             cscandir::collect(tree.path(), original).entries.size());
}

TEST(default_options_match_scandir_defaults) {
    cscandir::Options options;
    CHECK_EQ(options.native().sorted, std::uint8_t{0});
    CHECK_EQ(options.native().skip_hidden, std::uint8_t{0});
    CHECK_EQ(options.native().max_depth, SIZE_MAX);
    CHECK_EQ(options.native().max_file_cnt, SIZE_MAX);
    CHECK_EQ(options.native().return_type, std::uint32_t{0});
}

TEST(missing_root_throws) {
    bool threw = false;
    try {
        cscandir::CollectResult result =
            cscandir::collect("/nonexistent/cscandir/definitely/not/here");
        (void)result;
    } catch (const cscandir::Error& error) {
        threw = true;
        CHECK(error.code() == cscandir::ErrorCode::scan);
        CHECK(std::string(error.what()).find("No such file") !=
              std::string::npos);
    }
    CHECK(threw);
}

#ifndef _WIN32
TEST(invalid_utf8_root_throws) {
    // A path that cannot round-trip through UTF-8 is rejected by the C layer.
    // On Windows, std::filesystem::path would reject such a string first, so
    // this case only applies where paths are passed through as bytes.
    std::string invalid = "/tmp/\xFF\xFE";
    bool threw = false;
    try {
        cscandir::CollectResult result = cscandir::collect(invalid);
        (void)result;
    } catch (const cscandir::Error& error) {
        threw = true;
        CHECK(error.code() == cscandir::ErrorCode::invalid_utf8);
    }
    CHECK(threw);
}
#endif

TEST(error_is_a_std_exception) {
    bool caught_as_runtime_error = false;
    try {
        cscandir::CollectResult result = cscandir::collect("/nonexistent/zzz");
        (void)result;
    } catch (const std::runtime_error& error) {
        caught_as_runtime_error = true;
        CHECK(std::string(error.what()).size() > 0);
    }
    CHECK(caught_as_runtime_error);
}

TEST(collect_accepts_string_and_path) {
    Tree tree("types");
    // Both a std::string and a std::filesystem::path are accepted.
    CHECK_EQ(cscandir::collect(tree.str()).entries.size(), std::size_t{6});
    CHECK_EQ(cscandir::collect(tree.path()).entries.size(), std::size_t{6});
    CHECK_EQ(cscandir::count(tree.str()).files, 4);
    CHECK_EQ(cscandir::walk(tree.str()).toc.files.size(), std::size_t{4});
}

TEST(results_survive_source_destruction) {
    // The C buffers are copied into std::string during the call, so results
    // stay valid after the source tree is gone.
    std::size_t entries = 0;
    {
        Tree tree("lifetime");
        entries = cscandir::collect(tree.path()).entries.size();
    }
    CHECK_EQ(entries, std::size_t{6});
}

}  // namespace

int main() {
    for (const TestCase& test : registry()) {
        g_current = test.name;
        const int before = g_failures;
        try {
            test.fn();
        } catch (const std::exception& error) {
            ++g_failures;
            std::fprintf(stderr, "  FAIL [%s] threw: %s\n", test.name,
                         error.what());
        }
        if (g_failures == before) {
            std::printf("ok   %s\n", test.name);
        }
    }

    std::printf("\n%d checks, %d failures\n", g_checks, g_failures);
    return g_failures == 0 ? 0 : 1;
}
