#include <jni.h>
#include <cscandir.h>

#include <cstdint>
#include <atomic>
#include <chrono>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <limits>
#include <functional>
#include <string>
#include <vector>
#include <new>
#include <unordered_set>
#include <initializer_list>

namespace {

jclass entry_class = nullptr;
jclass statistics_class = nullptr;
jclass collect_result_class = nullptr;
jclass array_list_class = nullptr;
jclass exception_class = nullptr;
jclass options_class = nullptr;
jclass return_type_class = nullptr;
jmethodID entry_constructor = nullptr;
jmethodID statistics_constructor = nullptr;
jmethodID statistics_with_errors_constructor = nullptr;
jmethodID collect_result_constructor = nullptr;
jmethodID array_list_constructor = nullptr;
jmethodID array_list_add = nullptr;
jmethodID exception_constructor = nullptr;
jmethodID return_type_get_value = nullptr;
jfieldID options_sorted = nullptr;
jfieldID options_skip_hidden = nullptr;
jfieldID options_max_depth = nullptr;
jfieldID options_max_file_count = nullptr;
jfieldID options_case_sensitive = nullptr;
jfieldID options_follow_links = nullptr;
jfieldID options_return_type = nullptr;
jfieldID options_string_lists[4]{};
std::atomic<bool> jni_ready{false};

struct InodeKey {
    uint64_t device;
    uint64_t inode;
    bool operator==(const InodeKey &other) const {
        return device == other.device && inode == other.inode;
    }
};

struct InodeKeyHash {
    size_t operator()(const InodeKey &key) const {
        const size_t device_hash = std::hash<uint64_t>{}(key.device);
        const size_t inode_hash = std::hash<uint64_t>{}(key.inode);
        return device_hash ^ (inode_hash + 0x9e3779b9U + (device_hash << 6) + (device_hash >> 2));
    }
};

void throw_scandir(JNIEnv *env, int code, const char *message);

bool java_to_utf8(JNIEnv *env, jstring value, std::string &output) {
    if (!value) return false;
    const jsize length = env->GetStringLength(value);
    const jchar *chars = env->GetStringChars(value, nullptr);
    if (!chars) return false;
    for (jsize i = 0; i < length; ++i) {
        uint32_t codepoint = chars[i];
        if (codepoint == 0) {
            env->ReleaseStringChars(value, chars);
            throw_scandir(env, CSCANDIR_ERR_NUL_BYTE, "string contains NUL");
            return false;
        }
        if (codepoint >= 0xd800 && codepoint <= 0xdbff) {
            if (i + 1 >= length || chars[i + 1] < 0xdc00 || chars[i + 1] > 0xdfff) {
                env->ReleaseStringChars(value, chars);
                throw_scandir(env, CSCANDIR_ERR_INVALID_ARGUMENT, "string contains an invalid surrogate");
                return false;
            }
            codepoint = 0x10000 + ((codepoint - 0xd800) << 10) + (chars[++i] - 0xdc00);
        } else if (codepoint >= 0xdc00 && codepoint <= 0xdfff) {
            env->ReleaseStringChars(value, chars);
            throw_scandir(env, CSCANDIR_ERR_INVALID_ARGUMENT, "string contains an invalid surrogate");
            return false;
        }
        if (codepoint <= 0x7f) output.push_back(static_cast<char>(codepoint));
        else if (codepoint <= 0x7ff) {
            output.push_back(static_cast<char>(0xc0 | (codepoint >> 6)));
            output.push_back(static_cast<char>(0x80 | (codepoint & 0x3f)));
        } else if (codepoint <= 0xffff) {
            output.push_back(static_cast<char>(0xe0 | (codepoint >> 12)));
            output.push_back(static_cast<char>(0x80 | ((codepoint >> 6) & 0x3f)));
            output.push_back(static_cast<char>(0x80 | (codepoint & 0x3f)));
        } else {
            output.push_back(static_cast<char>(0xf0 | (codepoint >> 18)));
            output.push_back(static_cast<char>(0x80 | ((codepoint >> 12) & 0x3f)));
            output.push_back(static_cast<char>(0x80 | ((codepoint >> 6) & 0x3f)));
            output.push_back(static_cast<char>(0x80 | (codepoint & 0x3f)));
        }
    }
    env->ReleaseStringChars(value, chars);
    return true;
}

void throw_scandir(JNIEnv *env, int code, const char *message) {
    if (env->ExceptionCheck() || !jni_ready.load(std::memory_order_acquire) ||
        !exception_class || !exception_constructor) return;
    jstring text = env->NewStringUTF(message ? message : "unknown cscandir error");
    if (!text) return;
    jobject exception = env->NewObject(exception_class, exception_constructor, static_cast<jint>(code), text);
    if (exception) env->Throw(static_cast<jthrowable>(exception));
    if (exception) env->DeleteLocalRef(exception);
    env->DeleteLocalRef(text);
}

jstring new_string(JNIEnv *env, const char *value) {
    if (!value) return nullptr;
    std::vector<jchar> utf16;
    const auto *bytes = reinterpret_cast<const unsigned char *>(value);
    while (*bytes != 0) {
        uint32_t cp;
        size_t count;
        if (bytes[0] < 0x80) { cp = bytes[0]; count = 1; }
        else if ((bytes[0] & 0xe0) == 0xc0) { cp = bytes[0] & 0x1f; count = 2; }
        else if ((bytes[0] & 0xf0) == 0xe0) { cp = bytes[0] & 0x0f; count = 3; }
        else if ((bytes[0] & 0xf8) == 0xf0) { cp = bytes[0] & 0x07; count = 4; }
        else { throw_scandir(env, CSCANDIR_ERR_INVALID_UTF8, "native string is not valid UTF-8"); return nullptr; }
        for (size_t i = 1; i < count; ++i) {
            if ((bytes[i] & 0xc0) != 0x80) {
                throw_scandir(env, CSCANDIR_ERR_INVALID_UTF8, "native string is not valid UTF-8");
                return nullptr;
            }
            cp = (cp << 6) | (bytes[i] & 0x3f);
        }
        if ((count == 2 && cp < 0x80) || (count == 3 && cp < 0x800) ||
            (count == 4 && cp < 0x10000) || cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff)) {
            throw_scandir(env, CSCANDIR_ERR_INVALID_UTF8, "native string is not valid UTF-8");
            return nullptr;
        }
        if (cp <= 0xffff) utf16.push_back(static_cast<jchar>(cp));
        else {
            cp -= 0x10000;
            utf16.push_back(static_cast<jchar>(0xd800 + (cp >> 10)));
            utf16.push_back(static_cast<jchar>(0xdc00 + (cp & 0x3ff)));
        }
        bytes += count;
    }
    static const jchar empty = 0;
    return env->NewString(utf16.empty() ? &empty : utf16.data(), static_cast<jsize>(utf16.size()));
}

jobject new_statistics(JNIEnv *env, const cscandir_statistics &stats) {
    if (!statistics_class || !statistics_constructor) return nullptr;
    return env->NewObject(statistics_class, statistics_constructor,
            static_cast<jint>(stats.dirs), static_cast<jint>(stats.files),
            static_cast<jint>(stats.slinks), static_cast<jint>(stats.hlinks),
            static_cast<jint>(stats.devices), static_cast<jint>(stats.pipes),
            static_cast<jlong>(stats.size), static_cast<jlong>(stats.usage),
            static_cast<jdouble>(stats.duration));
}

jobject new_statistics(JNIEnv *env, const cscandir_statistics &stats, jobject errors) {
    if (!statistics_class || !statistics_with_errors_constructor) return nullptr;
    return env->NewObject(statistics_class, statistics_with_errors_constructor,
            static_cast<jint>(stats.dirs), static_cast<jint>(stats.files),
            static_cast<jint>(stats.slinks), static_cast<jint>(stats.hlinks),
            static_cast<jint>(stats.devices), static_cast<jint>(stats.pipes),
            static_cast<jlong>(stats.size), static_cast<jlong>(stats.usage),
            static_cast<jdouble>(stats.duration), errors);
}

jobject new_entry(JNIEnv *env, const cscandir_entry &entry) {
    jstring path = new_string(env, entry.path);
    if (!path || !entry_class || !entry_constructor) {
        if (path) env->DeleteLocalRef(path);
        return nullptr;
    }
    jobject result = env->NewObject(entry_class, entry_constructor, path,
            static_cast<jboolean>(entry.is_symlink != 0),
            static_cast<jboolean>(entry.is_dir != 0),
            static_cast<jboolean>(entry.is_file != 0),
            static_cast<jdouble>(entry.ctime), static_cast<jdouble>(entry.mtime),
            static_cast<jdouble>(entry.atime), static_cast<jlong>(entry.size),
            static_cast<jboolean>(entry.has_ext != 0), static_cast<jint>(entry.mode),
            static_cast<jlong>(entry.ino), static_cast<jlong>(entry.dev),
            static_cast<jlong>(entry.nlink), static_cast<jlong>(entry.blksize),
            static_cast<jlong>(entry.blocks), static_cast<jint>(entry.uid),
            static_cast<jint>(entry.gid), static_cast<jlong>(entry.rdev));
    env->DeleteLocalRef(path);
    return result;
}

jobject new_list(JNIEnv *env, jsize capacity) {
    if (!array_list_class || !array_list_constructor) return nullptr;
    return env->NewObject(array_list_class, array_list_constructor, capacity);
}

bool list_add(JNIEnv *env, jobject list, jobject value) {
    if (!value || env->ExceptionCheck()) return false;
    env->CallBooleanMethod(list, array_list_add, value);
    env->DeleteLocalRef(value);
    return !env->ExceptionCheck();
}

jobject new_entries(JNIEnv *env, const cscandir_entry_list &entries) {
    if (entries.len > static_cast<size_t>(std::numeric_limits<jsize>::max())) {
        throw_scandir(env, CSCANDIR_ERR_INVALID_ARGUMENT, "entry count exceeds Java list capacity");
        return nullptr;
    }
    jobject list = new_list(env, static_cast<jsize>(entries.len));
    if (!list) return nullptr;
    if (!entry_class || !entry_constructor || !array_list_add) {
        env->DeleteLocalRef(list);
        return nullptr;
    }
    for (size_t i = 0; i < entries.len; ++i) {
        if (!list_add(env, list, new_entry(env, entries.entries[i]))) {
            env->DeleteLocalRef(list);
            return nullptr;
        }
    }
    return list;
}

jobject new_errors(JNIEnv *env, const cscandir_string_list &errors) {
    if (errors.len > static_cast<size_t>(std::numeric_limits<jsize>::max())) {
        throw_scandir(env, CSCANDIR_ERR_INVALID_ARGUMENT, "error count exceeds Java list capacity");
        return nullptr;
    }
    jobject list = new_list(env, static_cast<jsize>(errors.len));
    if (!list) return nullptr;
    if (!array_list_add) { env->DeleteLocalRef(list); return nullptr; }
    for (size_t i = 0; i < errors.len; ++i) {
        if (!list_add(env, list, new_string(env, errors.items[i]))) {
            env->DeleteLocalRef(list);
            return nullptr;
        }
    }
    return list;
}

bool java_strings(JNIEnv *env, jobject list, const char *field_name,
                  char ***field, size_t *length, size_t *owned_length) {
    const char *names[] = {"dirInclude", "dirExclude", "fileInclude", "fileExclude"};
    size_t field_index = 0;
    while (field_index < 4 && std::strcmp(field_name, names[field_index]) != 0) ++field_index;
    if (field_index == 4) return false;
    jfieldID id = options_string_lists[field_index];
    if (!id) return false;
    jobject value = env->GetObjectField(list, id);
    if (env->ExceptionCheck()) return false;
    if (!value) { *field = nullptr; *length = 0; return true; }

    jclass list_cls = env->GetObjectClass(value);
    jmethodID to_array = list_cls ? env->GetMethodID(list_cls, "toArray", "()[Ljava/lang/Object;") : nullptr;
    if (list_cls) env->DeleteLocalRef(list_cls);
    if (!to_array) { env->DeleteLocalRef(value); return false; }
    jobjectArray array = static_cast<jobjectArray>(env->CallObjectMethod(value, to_array));
    env->DeleteLocalRef(value);
    if (!array) return false;
    jsize count = env->GetArrayLength(array);
    char **items = count ? static_cast<char **>(std::calloc(static_cast<size_t>(count), sizeof(char *))) : nullptr;
    if (count && !items) {
        env->DeleteLocalRef(array);
        throw_scandir(env, CSCANDIR_ERR_PANIC, "out of memory converting filter list");
        return false;
    }
    size_t used = 0;
    auto cleanup = [&]() {
        for (size_t j = 0; j < used; ++j) std::free(items[j]);
        std::free(items);
        env->DeleteLocalRef(array);
    };
    for (jsize i = 0; i < count; ++i) {
        jstring string = static_cast<jstring>(env->GetObjectArrayElement(array, i));
        if (!string) {
            cleanup();
            throw_scandir(env, CSCANDIR_ERR_INVALID_ARGUMENT, "filter pattern cannot be null");
            return false;
        }
        std::string encoded;
        if (!java_to_utf8(env, string, encoded)) {
            env->DeleteLocalRef(string); cleanup();
            return false;
        }
        char *copy = static_cast<char *>(std::malloc(encoded.size() + 1));
        if (!copy) {
            env->DeleteLocalRef(string); cleanup();
            throw_scandir(env, CSCANDIR_ERR_PANIC, "out of memory converting filter pattern");
            return false;
        }
        std::memcpy(copy, encoded.c_str(), encoded.size() + 1);
        env->DeleteLocalRef(string);
        items[used++] = copy;
        *owned_length = used;
    }
    env->DeleteLocalRef(array);
    *field = items;
    *length = used;
    *owned_length = used;
    return true;
}

struct NativeOptions {
    cscandir_options value{};
    char **arrays[4]{};
    size_t lengths[4]{};
    ~NativeOptions() {
        for (size_t i = 0; i < 4; ++i) {
            if (!arrays[i]) continue;
            for (size_t j = 0; j < lengths[i]; ++j) std::free(arrays[i][j]);
            std::free(arrays[i]);
        }
    }
};

bool populate_options(JNIEnv *env, jobject source, NativeOptions &native) {
    cscandir_options_init(&native.value);
    if (!source) return true;
    if (!options_class || !options_sorted || !options_skip_hidden || !options_max_depth || !options_max_file_count ||
        !options_case_sensitive || !options_follow_links || !options_return_type) return false;
    native.value.sorted = static_cast<uint8_t>(env->GetBooleanField(source, options_sorted));
    native.value.skip_hidden = static_cast<uint8_t>(env->GetBooleanField(source, options_skip_hidden));
    jlong max_depth = env->GetLongField(source, options_max_depth);
    jlong max_file_count = env->GetLongField(source, options_max_file_count);
    if (max_depth < 0 || max_file_count < 0) {
        throw_scandir(env, CSCANDIR_ERR_INVALID_ARGUMENT, "limits must be non-negative");
        return false;
    }
    native.value.max_depth = static_cast<size_t>(max_depth);
    native.value.max_file_cnt = static_cast<size_t>(max_file_count);
    native.value.case_sensitive = static_cast<uint8_t>(env->GetBooleanField(source, options_case_sensitive));
    native.value.follow_links = static_cast<uint8_t>(env->GetBooleanField(source, options_follow_links));
    jobject enum_value = env->GetObjectField(source, options_return_type);
    if (enum_value) {
        if (!return_type_get_value) { env->DeleteLocalRef(enum_value); return false; }
        jint value = env->CallIntMethod(enum_value, return_type_get_value);
        env->DeleteLocalRef(enum_value);
        if (value != CSCANDIR_RETURN_BASE && value != CSCANDIR_RETURN_EXT) {
            throw_scandir(env, CSCANDIR_ERR_INVALID_ARGUMENT, "invalid return type");
            return false;
        }
        native.value.return_type = static_cast<uint32_t>(value);
    }

    const char *names[] = {"dirInclude", "dirExclude", "fileInclude", "fileExclude"};
    char ***fields[] = {&native.arrays[0], &native.arrays[1], &native.arrays[2], &native.arrays[3]};
    size_t *lengths[] = {&native.value.dir_include_len, &native.value.dir_exclude_len,
                         &native.value.file_include_len, &native.value.file_exclude_len};
    for (size_t i = 0; i < 4; ++i) {
        char **converted = nullptr;
        size_t count = 0;
        if (!java_strings(env, source, names[i], &converted, &count, &native.lengths[i])) return false;
        *fields[i] = converted;
        native.lengths[i] = count;
        *lengths[i] = count;
    }
    return !env->ExceptionCheck();
}

jobject make_collect_result(JNIEnv *env, jobject entries, jobject errors, jobject stats) {
    if (!entries || !errors || !stats || !collect_result_class || !collect_result_constructor ||
        env->ExceptionCheck()) return nullptr;
    return env->NewObject(collect_result_class, collect_result_constructor, entries, errors, stats);
}

} // namespace

extern "C" JNIEXPORT jint JNICALL JNI_OnLoad(JavaVM *vm, void *) {
    JNIEnv *env = nullptr;
    if (vm->GetEnv(reinterpret_cast<void **>(&env), JNI_VERSION_1_8) != JNI_OK) return JNI_ERR;
    auto cache_class = [env](const char *name) -> jclass {
        jclass local = env->FindClass(name);
        if (!local) return nullptr;
        jclass global = static_cast<jclass>(env->NewGlobalRef(local));
        env->DeleteLocalRef(local);
        return global;
    };
    entry_class = cache_class("com/scandir/result/Entry");
    statistics_class = cache_class("com/scandir/result/Statistics");
    collect_result_class = cache_class("com/scandir/result/CollectResult");
    array_list_class = cache_class("java/util/ArrayList");
    exception_class = cache_class("com/scandir/error/JScandirException");
    options_class = cache_class("com/scandir/options/Options");
    return_type_class = cache_class("com/scandir/options/ReturnType");
    if (!entry_class || !statistics_class || !collect_result_class || !array_list_class || !exception_class ||
        !options_class || !return_type_class) {
        auto clear_cached_classes = [env]() {
            jclass *classes[] = {&entry_class, &statistics_class, &collect_result_class, &array_list_class,
                                 &exception_class, &options_class, &return_type_class};
            for (jclass *cls : classes) {
                if (*cls) env->DeleteGlobalRef(*cls);
                *cls = nullptr;
            }
        };
        clear_cached_classes();
        return JNI_ERR;
    }
    entry_constructor = env->GetMethodID(entry_class, "<init>",
            "(Ljava/lang/String;ZZZDDDJZIJJJJJIIJ)V");
    statistics_constructor = env->GetMethodID(statistics_class, "<init>", "(IIIIIIJJD)V");
    statistics_with_errors_constructor = env->GetMethodID(statistics_class, "<init>",
            "(IIIIIIJJDLjava/util/List;)V");
    collect_result_constructor = env->GetMethodID(collect_result_class, "<init>",
            "(Ljava/util/List;Ljava/util/List;Lcom/scandir/result/Statistics;)V");
    array_list_constructor = env->GetMethodID(array_list_class, "<init>", "(I)V");
    array_list_add = env->GetMethodID(array_list_class, "add", "(Ljava/lang/Object;)Z");
    exception_constructor = env->GetMethodID(exception_class, "<init>", "(ILjava/lang/String;)V");
    return_type_get_value = env->GetMethodID(return_type_class, "getValue", "()I");
    options_sorted = env->GetFieldID(options_class, "sorted", "Z");
    options_skip_hidden = env->GetFieldID(options_class, "skipHidden", "Z");
    options_max_depth = env->GetFieldID(options_class, "maxDepth", "J");
    options_max_file_count = env->GetFieldID(options_class, "maxFileCnt", "J");
    options_case_sensitive = env->GetFieldID(options_class, "caseSensitive", "Z");
    options_follow_links = env->GetFieldID(options_class, "followLinks", "Z");
    options_return_type = env->GetFieldID(options_class, "returnType", "Lcom/scandir/options/ReturnType;");
    const char *option_lists[] = {"dirInclude", "dirExclude", "fileInclude", "fileExclude"};
    for (size_t i = 0; i < 4; ++i) {
        options_string_lists[i] = env->GetFieldID(options_class, option_lists[i], "Ljava/util/List;");
    }
    if (!entry_constructor || !statistics_constructor || !statistics_with_errors_constructor ||
        !collect_result_constructor || !array_list_constructor || !array_list_add || !exception_constructor ||
        !return_type_get_value || !options_sorted || !options_skip_hidden || !options_max_depth ||
        !options_max_file_count || !options_case_sensitive || !options_follow_links || !options_return_type ||
        !options_string_lists[0] || !options_string_lists[1] || !options_string_lists[2] || !options_string_lists[3]) {
        auto clear_cached_classes = [env]() {
            jclass *classes[] = {&entry_class, &statistics_class, &collect_result_class, &array_list_class,
                                 &exception_class, &options_class, &return_type_class};
            for (jclass *cls : classes) {
                if (*cls) env->DeleteGlobalRef(*cls);
                *cls = nullptr;
            }
        };
        clear_cached_classes();
        return JNI_ERR;
    }
    jni_ready.store(true, std::memory_order_release);
    return JNI_VERSION_1_8;
}

extern "C" JNIEXPORT jobject JNICALL
Java_com_scandir_jni_JScandir_collect(JNIEnv *env, jclass, jstring jroot, jobject joptions) {
    std::string root;
    if (!java_to_utf8(env, jroot, root)) {
        if (!env->ExceptionCheck()) throw_scandir(env, CSCANDIR_ERR_INVALID_ARGUMENT, "root path is null");
        return nullptr;
    }
    NativeOptions options;
    if (!populate_options(env, joptions, options)) return nullptr;
    cscandir_entry_list entries{};
    cscandir_string_list errors{};
    cscandir_error error{};
    const auto started = std::chrono::steady_clock::now();
    int32_t status = cscandir_collect(root.c_str(), &options.value, &entries, &errors, &error);
    const double duration = std::chrono::duration<double>(std::chrono::steady_clock::now() - started).count();
    if (status != CSCANDIR_OK) {
        throw_scandir(env, status, error.message ? error.message : "collect failed");
        cscandir_free_entry_list(&entries); cscandir_free_string_list(&errors); cscandir_free_error(&error);
        return nullptr;
    }
    jobject java_entries = new_entries(env, entries);
    jobject java_errors = java_entries ? new_errors(env, errors) : nullptr;
    cscandir_statistics stats_value{};
    std::unordered_set<InodeKey, InodeKeyHash> inodes;
    if (java_errors) {
        size_t first_counted = 0;
        std::filesystem::path root_path(root);
        std::error_code path_error;
        auto absolute_root = std::filesystem::canonical(root_path, path_error);
        if (path_error) absolute_root = std::filesystem::absolute(root_path, path_error);
        if (path_error) absolute_root = root_path;
        auto normalized_root = absolute_root.lexically_normal();
        auto root_name = normalized_root.filename();
        if (root_name.empty()) root_name = normalized_root.parent_path().filename();
        if (entries.len > 0 && entries.entries[0].is_dir && root_name.native() == entries.entries[0].path) {
            first_counted = 1;
        }
        for (size_t i = first_counted; i < entries.len; ++i) {
            const cscandir_entry &entry = entries.entries[i];
            if (entry.is_file) {
                ++stats_value.files;
                stats_value.size += entry.size;
                if (entry.has_ext) {
                    stats_value.usage += entry.blocks << 9;
                    if (entry.nlink > 1 && !inodes.insert({entry.dev, entry.ino}).second) {
                        ++stats_value.hlinks;
                        --stats_value.files;
                    }
                }
            } else if (entry.is_dir) {
                ++stats_value.dirs; stats_value.size += 4096; stats_value.usage += 4096;
            } else if (entry.is_symlink) {
                ++stats_value.slinks; stats_value.size += 4096; stats_value.usage += 4096;
            } else {
                if (entry.has_ext) {
                    if (entry.rdev > 0) ++stats_value.devices;
                    else if ((entry.mode & 4096U) != 0) ++stats_value.pipes;
                }
                stats_value.size += 4096; stats_value.usage += 4096;
            }
        }
        stats_value.duration = duration;
    }
    jobject stats = java_errors ? new_statistics(env, stats_value) : nullptr;
    jobject result = make_collect_result(env, java_entries, java_errors, stats);
    if (java_entries) env->DeleteLocalRef(java_entries);
    if (java_errors) env->DeleteLocalRef(java_errors);
    if (stats) env->DeleteLocalRef(stats);
    cscandir_free_entry_list(&entries); cscandir_free_string_list(&errors); cscandir_free_error(&error);
    return result;
}

extern "C" JNIEXPORT jobject JNICALL
Java_com_scandir_jni_JScandir_count(JNIEnv *env, jclass, jstring jroot, jobject joptions) {
    std::string root;
    if (!java_to_utf8(env, jroot, root)) {
        if (!env->ExceptionCheck()) throw_scandir(env, CSCANDIR_ERR_INVALID_ARGUMENT, "root path is null");
        return nullptr;
    }
    NativeOptions options;
    if (!populate_options(env, joptions, options)) return nullptr;
    cscandir_string_list errors{};
    cscandir_error error{};
    cscandir_statistics stats{};
    int32_t status = cscandir_count(root.c_str(), &options.value, &stats, &errors, &error);
    if (status != CSCANDIR_OK) {
        throw_scandir(env, status, error.message ? error.message : "count failed");
        cscandir_free_string_list(&errors); cscandir_free_error(&error);
        return nullptr;
    }
    jobject java_errors = new_errors(env, errors);
    jobject result = java_errors ? new_statistics(env, stats, java_errors) : nullptr;
    if (java_errors) env->DeleteLocalRef(java_errors);
    cscandir_free_string_list(&errors); cscandir_free_error(&error);
    return result;
}