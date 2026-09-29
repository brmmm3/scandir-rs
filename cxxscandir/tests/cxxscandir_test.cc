// cxxscandir_test.cc - Tests for the C++ wrapper.
#include "cxxscandir.h"
#include <gtest/gtest.h>
#include <filesystem>
#include <fstream>
#include <algorithm>

namespace fs = std::filesystem;

class ScandirTest : public ::testing::Test {
protected:
    void SetUp() override {
        root = fs::temp_directory_path() / "cxxscandir_test";
        fs::remove_all(root);
        fs::create_directories(root / "sub" / "deep");
        std::ofstream(root / "a.txt") << "a";
        std::ofstream(root / "b.txt") << "bb";
        std::ofstream(root / "sub" / "c.txt") << "ccc";
        std::ofstream(root / "sub" / "deep" / "d.txt") << "dddd";
    }

    void TearDown() override {
        fs::remove_all(root);
    }

    fs::path root;

    static void mustWrite(const fs::path& path, const std::string& content) {
        std::ofstream out(path);
        out << content;
    }
};

TEST_F(ScandirTest, CollectReturnsEntriesAndStatistics) {
    auto result = cxxscandir::collect(root.string(), cxxscandir::Options());
    ASSERT_TRUE(result.is_ok()) << result.error().message;

    const auto& r = result.ok();
    EXPECT_EQ(r.entries.size(), 6u);  // 4 files + 2 dirs
    EXPECT_EQ(r.statistics.files, 4);
    EXPECT_EQ(r.statistics.dirs, 2);
    EXPECT_GT(r.statistics.duration, 0.0);
    EXPECT_TRUE(r.errors.empty());
}

TEST_F(ScandirTest, EntriesCarryPathsAndTimes) {
    auto result = cxxscandir::collect(root.string(), cxxscandir::Options());
    ASSERT_TRUE(result.is_ok());

    for (const auto& entry : result.ok().entries) {
        EXPECT_FALSE(entry.path.empty());
        EXPECT_GT(entry.ctime, 0.0);
        EXPECT_GT(entry.mtime, 0.0);
        EXPECT_GT(entry.atime, 0.0);
        if (entry.is_file) {
            EXPECT_GT(entry.size, 0u);
        }
    }
}

TEST_F(ScandirTest, BaseReturnTypeLeavesExtUnset) {
    cxxscandir::Options opts;
    opts.return_type = cxxscandir::ReturnType::Base;
    auto result = cxxscandir::collect(root.string(), opts);
    ASSERT_TRUE(result.is_ok());

    for (const auto& entry : result.ok().entries) {
        EXPECT_FALSE(entry.has_ext);
        EXPECT_EQ(entry.mode, 0u);
        EXPECT_EQ(entry.ino, 0u);
    }
}

TEST_F(ScandirTest, ExtendedReturnTypePopulatesExt) {
    cxxscandir::Options opts;
    opts.return_type = cxxscandir::ReturnType::Ext;
    auto result = cxxscandir::collect(root.string(), opts);
    ASSERT_TRUE(result.is_ok());

    for (const auto& entry : result.ok().entries) {
        EXPECT_TRUE(entry.has_ext);
        EXPECT_NE(entry.ino, 0u);
    }
}

TEST_F(ScandirTest, CountReturnsStatistics) {
    auto result = cxxscandir::count(root.string(), cxxscandir::Options());
    ASSERT_TRUE(result.is_ok());
    const auto& stats = result.ok();
    EXPECT_EQ(stats.files, 4);
    EXPECT_EQ(stats.dirs, 2);
    EXPECT_GT(stats.duration, 0.0);
}

TEST_F(ScandirTest, WalkGroupsEntriesPerDirectory) {
    auto result = cxxscandir::walk(root.string(), cxxscandir::Options());
    ASSERT_TRUE(result.is_ok());
    const auto& r = result.ok();

    EXPECT_EQ(r.toc.files.size(), 4u);
    EXPECT_EQ(r.toc.dirs.size(), 2u);
    EXPECT_GE(r.roots.size(), 2u);

    // Normalize paths for cross-platform comparison
    const std::set<std::string> expected = {"", "sub", "sub/deep"};
    size_t total_files = 0;
    for (const auto& entry : r.roots) {
        std::string normalized = entry.path;
        std::replace(normalized.begin(), normalized.end(), '\\', '/');
        EXPECT_TRUE(expected.count(normalized)) << "unexpected root dir: " << entry.path;
        total_files += entry.toc.files.size();
    }
    EXPECT_EQ(total_files, 4u);
}

TEST_F(ScandirTest, WalkTocFilesEndInTxt) {
    auto result = cxxscandir::walk(root.string(), cxxscandir::Options());
    ASSERT_TRUE(result.is_ok());
    for (const auto& file : result.ok().toc.files) {
        EXPECT_TRUE(file.ends_with(".txt")) << "unexpected file: " << file;
    }
}

TEST_F(ScandirTest, MaxDepthLimitsResults) {
    cxxscandir::Options opts;
    opts.max_depth = 1;
    auto result = cxxscandir::collect(root.string(), opts);
    ASSERT_TRUE(result.is_ok());
    // Only root's direct children: 2 files + 1 dir
    EXPECT_EQ(result.ok().entries.size(), 3u);
}

TEST_F(ScandirTest, MaxDepthZeroMeansUnlimited) {
    cxxscandir::Options opts;
    opts.max_depth = 0;
    auto result = cxxscandir::collect(root.string(), opts);
    ASSERT_TRUE(result.is_ok());
    EXPECT_EQ(result.ok().entries.size(), 6u);
}

TEST_F(ScandirTest, FileIncludeFilter) {
    cxxscandir::Options opts;
    opts.file_include = {"*.txt"};
    auto result = cxxscandir::collect(root.string(), opts);
    ASSERT_TRUE(result.is_ok());

    int files = 0, dirs = 0;
    for (const auto& e : result.ok().entries) {
        if (e.is_file) {
            ++files;
            EXPECT_TRUE(e.path.ends_with(".txt"));
        }
        if (e.is_dir) ++dirs;
    }
    EXPECT_EQ(files, 4);
}

TEST_F(ScandirTest, DirExcludeFilter) {
    cxxscandir::Options opts;
    opts.dir_exclude = {"sub"};
    auto result = cxxscandir::collect(root.string(), opts);
    ASSERT_TRUE(result.is_ok());
    EXPECT_EQ(result.ok().entries.size(), 2u);  // only root files
}

TEST_F(ScandirTest, SkipHiddenFilter) {
    std::ofstream(root / ".hidden.txt") << "x";
    cxxscandir::Options opts;
    opts.skip_hidden = true;
    auto result = cxxscandir::collect(root.string(), opts);
    ASSERT_TRUE(result.is_ok());
    EXPECT_EQ(result.ok().entries.size(), 6u);
    for (const auto& entry : result.ok().entries) {
        EXPECT_FALSE(entry.path.find(".hidden") != std::string::npos);
    }
}

TEST_F(ScandirTest, OptionsAreReusable) {
    cxxscandir::Options opts;
    opts.file_include = {"*.txt"};
    opts.sorted = true;
    opts.case_sensitive = true;

    for (int i = 0; i < 20; ++i) {
        auto result = cxxscandir::collect(root.string(), opts);
        ASSERT_TRUE(result.is_ok());
        int files = 0;
        for (const auto& e : result.ok().entries) if (e.is_file) ++files;
        EXPECT_EQ(files, 4) << "iteration " << i;
    }
}

TEST_F(ScandirTest, MissingRootReturnsError) {
    auto result = cxxscandir::collect("/nonexistent/path/here", cxxscandir::Options());
    ASSERT_FALSE(result.is_ok());
    const auto& err = result.error();
    EXPECT_EQ(err.code, cxxscandir::ErrorCode::Scan);
    EXPECT_FALSE(err.message.empty());
}

TEST_F(ScandirTest, ErrorIsComparable) {
    auto result = cxxscandir::collect("/nonexistent/path", cxxscandir::Options());
    ASSERT_FALSE(result.is_ok());
    // The error should be comparable
    EXPECT_EQ(result.error().code, cxxscandir::ErrorCode::Scan);
}

TEST_F(ScandirTest, InvalidUTF8RootIsRejected) {
#if !defined(_WIN32)
    auto result = cxxscandir::collect("/tmp/\xff\xfe", cxxscandir::Options());
    ASSERT_FALSE(result.is_ok());
    EXPECT_EQ(result.error().code, cxxscandir::ErrorCode::InvalidUtf8);
#endif
}

TEST_F(ScandirTest, CountPropagatesErrors) {
    auto result = cxxscandir::count("/nonexistent/path", cxxscandir::Options());
    ASSERT_FALSE(result.is_ok());
    result = cxxscandir::walk("/nonexistent/path", cxxscandir::Options());
    ASSERT_FALSE(result.is_ok());
}

TEST_F(ScandirTest, ResultsSurviveSourceDeletion) {
    fs::path root2 = fs::temp_directory_path() / "cxxscandir_lifetime";
    fs::remove_all(root2);
    fs::create_directories(root2 / "sub");
    std::ofstream(root2 / "a.txt") << "a";
    std::ofstream(root2 / "sub" / "b.txt") << "b";

    auto result = cxxscandir::collect(root2.string(), cxxscandir::Options());
    ASSERT_TRUE(result.is_ok());
    fs::remove_all(root2);

    EXPECT_EQ(result.ok().entries.size(), 3u);
    for (const auto& entry : result.ok().entries) {
        EXPECT_FALSE(entry.path.empty());
    }
    fs::remove_all(root2);
}

int main(int argc, char** argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}