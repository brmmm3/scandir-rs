// cppscandir_test.cc - Tests for the cppscandir wrapper
#include "cppscandir.h"
#include <gtest/gtest.h>
#include <filesystem>
#include <fstream>
#include <algorithm>

namespace fs = std::filesystem;

class ScandirTest : public ::testing::Test {
protected:
    void SetUp() override {
        root = fs::temp_directory_path() / "cppscandir_test";
        fs::remove_all(root);
        fs::create_directories(root / "sub" / "deep");
        write(root / "a.txt");
        write(root / "b.txt");
        write(root / "sub" / "c.txt");
        write(root / "sub" / "deep" / "d.txt");
    }

    void TearDown() override {
        fs::remove_all(root);
    }

    static void write(const fs::path& path) {
        std::ofstream out(path);
        out << "content";
    }

    fs::path root;

    static std::size_t count_files(const cppscandir::CollectResult& result) {
        return std::count_if(result.entries.begin(), result.entries.end(),
                             [](const cppscandir::Entry& e) { return e.is_file; });
    }

    static std::size_t count_dirs(const cppscandir::CollectResult& result) {
        return std::count_if(result.entries.begin(), result.entries.end(),
                             [](const cppscandir::Entry& e) { return e.is_dir; });
    }
};

TEST_F(ScandirTest, CollectReturnsEntriesAndStatistics) {
    auto result = cppscandir::collect(root.string(), cppscandir::Options());
    ASSERT_EQ(result.entries.size(), 6u);  // 4 files + 2 dirs
    ASSERT_EQ(count_files(result), 4u);
    ASSERT_EQ(count_dirs(result), 2u);
    EXPECT_EQ(result.statistics.files, 4);
    EXPECT_EQ(result.statistics.dirs, 2);
    EXPECT_GT(result.statistics.duration, 0.0);
    EXPECT_TRUE(result.errors.empty());
}

TEST_F(ScandirTest, EntriesCarryPathsAndTimes) {
    auto result = cppscandir::collect(root.string(), cppscandir::Options());
    for (const auto& entry : result.entries) {
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
    cppscandir::Options opts;
    opts.return_type = cppscandir::ReturnType::Base;
    auto result = cppscandir::collect(root.string(), opts);
    
    for (const auto& entry : result.entries) {
        EXPECT_FALSE(entry.has_ext);
        EXPECT_EQ(entry.mode, 0u);
        EXPECT_EQ(entry.ino, 0u);
    }
}

TEST_F(ScandirTest, ExtendedReturnTypePopulatesExt) {
    cppscandir::Options opts;
    opts.return_type = cppscandir::ReturnType::Ext;
    auto result = cppscandir::collect(root.string(), opts);
    
    for (const auto& entry : result.entries) {
        EXPECT_TRUE(entry.has_ext);
        EXPECT_NE(entry.ino, 0u);
    }
}

TEST_F(ScandirTest, CountReturnsStatistics) {
    auto stats = cppscandir::count(root.string(), cppscandir::Options());
    EXPECT_EQ(stats.files, 4);
    EXPECT_EQ(stats.dirs, 2);
    EXPECT_GT(stats.duration, 0.0);
}

TEST_F(ScandirTest, WalkGroupsEntriesPerDirectory) {
    auto result = cppscandir::walk(root.string(), cppscandir::Options());
    ASSERT_EQ(result.toc.files.size(), 4u);
    ASSERT_EQ(result.toc.dirs.size(), 2u);
    EXPECT_GE(result.roots.size(), 2u);

    const std::set<std::string> expected = {"", "sub", "sub/deep"};
    std::size_t total_files = 0;
    for (const auto& entry : result.roots) {
        std::string normalized = entry.dir;
        std::replace(normalized.begin(), normalized.end(), '\\', '/');
        EXPECT_TRUE(expected.count(normalized)) << "unexpected root dir: " << entry.dir;
        total_files += entry.toc.files.size();
    }
    EXPECT_EQ(total_files, 4u);
}

TEST_F(ScandirTest, WalkTocFilesEndInTxt) {
    auto result = cppscandir::walk(root.string(), cppscandir::Options());
    for (const auto& file : result.toc.files) {
        EXPECT_TRUE(file.ends_with(".txt")) << "unexpected file: " << file;
    }
}

TEST_F(ScandirTest, MaxDepthLimitsResults) {
    cppscandir::Options opts;
    opts.max_depth = 1;
    auto result = cppscandir::collect(root.string(), opts);
    ASSERT_EQ(result.entries.size(), 3u);  // 2 files + 1 dir
}

TEST_F(ScandirTest, MaxDepthZeroMeansUnlimited) {
    cppscandir::Options opts;
    opts.max_depth = 0;
    auto result = cppscandir::collect(root.string(), opts);
    ASSERT_EQ(result.entries.size(), 6u);
}

TEST_F(ScandirTest, FileIncludeFilter) {
    cppscandir::Options opts;
    opts.file_include = {"*.txt"};
    auto result = cppscandir::collect(root.string(), opts);
    
    int files = 0;
    for (const auto& entry : result.entries) {
        if (entry.is_file) {
            ++files;
            EXPECT_TRUE(entry.path.ends_with(".txt"));
        }
    }
    EXPECT_EQ(files, 4);
}

TEST_F(ScandirTest, DirExcludeFilter) {
    cppscandir::Options opts;
    opts.dir_exclude = {"sub"};
    auto result = cppscandir::collect(root.string(), opts);
    ASSERT_EQ(result.entries.size(), 2u);  // only root files
}

TEST_F(ScandirTest, SkipHiddenFilter) {
    std::ofstream(root / ".hidden.txt") << "x";
    cppscandir::Options opts;
    opts.skip_hidden = true;
    auto result = cppscandir::collect(root.string(), opts);
    ASSERT_EQ(result.entries.size(), 6u);
    for (const auto& entry : result.entries) {
        EXPECT_TRUE(entry.path.find(".hidden") == std::string::npos);
    }
}

TEST_F(ScandirTest, OptionsAreReusable) {
    cppscandir::Options opts;
    opts.file_include = {"*.txt"};
    opts.sorted = true;
    opts.case_sensitive = true;

    for (int i = 0; i < 20; ++i) {
        auto result = cppscandir::collect(root.string(), opts);
        ASSERT_EQ(count_files(result), 4u);
    }
}

TEST_F(ScandirTest, MissingRootReturnsError) {
    EXPECT_THROW({
        cppscandir::collect("/nonexistent/path/here", cppscandir::Options());
    }, cppscandir::CppScandirError);
    
    try {
        cppscandir::collect("/nonexistent/path/here", cppscandir::Options());
    } catch (const cppscandir::CppScandirError& e) {
        EXPECT_EQ(e.code(), static_cast<int32_t>(cppscandir::ErrorCode::Scan));
        EXPECT_FALSE(e.what() == nullptr);
    }
}

TEST_F(ScandirTest, InvalidUTF8RootIsRejected) {
#ifndef _WIN32
    EXPECT_THROW({
        cppscandir::collect("/tmp/\xff\xfe", cppscandir::Options());
    }, cppscandir::CppScandirError);
    
    try {
        cppscandir::collect("/tmp/\xff\xfe", cppscandir::Options());
    } catch (const cppscandir::CppScandirError& e) {
        EXPECT_EQ(e.code(), static_cast<int32_t>(cppscandir::ErrorCode::InvalidUtf8));
    }
#endif
}

TEST_F(ScandirTest, CountPropagatesErrors) {
    EXPECT_THROW({
        cppscandir::count("/nonexistent/path", cppscandir::Options());
    }, cppscandir::CppScandirError);
    
    EXPECT_THROW({
        cppscandir::walk("/nonexistent/path", cppscandir::Options());
    }, cppscandir::CppScandirError);
}

TEST_F(ScandirTest, ResultsSurviveSourceDeletion) {
    fs::path root2 = fs::temp_directory_path() / "cppscandir_lifetime";
    fs::remove_all(root2);
    fs::create_directories(root2 / "sub");
    std::ofstream(root2 / "a.txt") << "a";
    std::ofstream(root2 / "sub" / "b.txt") << "b";

    auto result = cppscandir::collect(root2.string(), cppscandir::Options());
    fs::remove_all(root2);

    EXPECT_EQ(result.entries.size(), 3u);
    for (const auto& entry : result.entries) {
        EXPECT_FALSE(entry.path.empty());
    }
}

int main(int argc, char** argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}