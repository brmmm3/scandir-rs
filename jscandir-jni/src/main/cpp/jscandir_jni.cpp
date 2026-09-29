#include <jni.h>
#include <cscandir.h>
#include <cstdlib>
#include <cstring>

// ---------------------------------------------------------------------------
// JNI bridge for jscandir-jni
//
// Converts between the Java API (JScandir / Options / result classes) and the
// C API exposed by libcscandir (cscandir.h).
//
// NOTE: the underlying cscandir C library currently exports only:
//   - cscandir_collect
//   - cscandir_count
// There is no walk/TOC API yet, so JScandir.walk() throws a descriptive
// JScandirException instead of silently returning null.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// String helpers
// ---------------------------------------------------------------------------

static const char *to_cstr(JNIEnv *env, jstring jstr)
{
    if (!jstr)
        return nullptr;
    return env->GetStringUTFChars(jstr, nullptr);
}

static void release_cstr(JNIEnv *env, jstring jstr, const char *cstr)
{
    if (jstr && cstr)
    {
        env->ReleaseStringUTFChars(jstr, cstr);
    }
}

static jstring to_jstring(JNIEnv *env, const char *cstr)
{
    return cstr ? env->NewStringUTF(cstr) : nullptr;
}

static void free_c_array(char **arr, size_t len)
{
    if (!arr)
        return;
    for (size_t i = 0; i < len; ++i)
    {
        if (arr[i])
            std::free(arr[i]);
    }
    std::free(arr);
}

// ---------------------------------------------------------------------------
// Java String list -> C char** array
// ---------------------------------------------------------------------------

static char **java_list_to_c_array(JNIEnv *env, jobject jlist, size_t &out_len)
{
    out_len = 0;
    if (!jlist)
        return nullptr;

    // Convert the List to an Object[] once, then process each element.
    jclass list_cls = env->GetObjectClass(jlist);
    jmethodID mid_to_array = env->GetMethodID(list_cls, "toArray", "()[Ljava/lang/Object;");
    if (!mid_to_array)
        return nullptr;

    jobjectArray jarr = (jobjectArray)env->CallObjectMethod(jlist, mid_to_array);
    if (!jarr)
        return nullptr;

    jsize len = env->GetArrayLength(jarr);
    if (len == 0)
    {
        env->DeleteLocalRef(jarr);
        return nullptr;
    }

    char **arr = static_cast<char **>(std::malloc(static_cast<size_t>(len) * sizeof(char *)));
    if (!arr)
    {
        env->DeleteLocalRef(jarr);
        return nullptr;
    }

    for (jsize i = 0; i < len; ++i)
    {
        jstring jstr = (jstring)env->GetObjectArrayElement(jarr, i);
        if (!jstr)
        {
            arr[i] = nullptr;
            continue;
        }
        const char *cstr = env->GetStringUTFChars(jstr, nullptr);
        arr[i] = cstr ? strdup(cstr) : nullptr;
        if (cstr)
            env->ReleaseStringUTFChars(jstr, cstr);
        env->DeleteLocalRef(jstr);
    }

    env->DeleteLocalRef(jarr);
    out_len = static_cast<size_t>(len);
    return arr;
}

// ---------------------------------------------------------------------------
// Options: Java Options -> C cscandir_options
// ---------------------------------------------------------------------------

static bool populate_options(JNIEnv *env, jobject joptions, cscandir_options *native)
{
    if (!joptions)
    {
        cscandir_options_init(native);
        return true;
    }

    jclass cls = env->GetObjectClass(joptions);

    jfieldID fid_sorted = env->GetFieldID(cls, "sorted", "Z");
    jfieldID fid_skip_hidden = env->GetFieldID(cls, "skipHidden", "Z");
    jfieldID fid_max_depth = env->GetFieldID(cls, "maxDepth", "J");
    jfieldID fid_max_file_cnt = env->GetFieldID(cls, "maxFileCnt", "J");
    jfieldID fid_case_sensitive = env->GetFieldID(cls, "caseSensitive", "Z");
    jfieldID fid_follow_links = env->GetFieldID(cls, "followLinks", "Z");
    jfieldID fid_return_type = env->GetFieldID(cls, "returnType", "Lcom/scandir/options/ReturnType;");

    jfieldID fid_dir_include = env->GetFieldID(cls, "dirInclude", "Ljava/util/List;");
    jfieldID fid_dir_exclude = env->GetFieldID(cls, "dirExclude", "Ljava/util/List;");
    jfieldID fid_file_include = env->GetFieldID(cls, "fileInclude", "Ljava/util/List;");
    jfieldID fid_file_exclude = env->GetFieldID(cls, "fileExclude", "Ljava/util/List;");

    if (env->ExceptionCheck())
        return false;

    cscandir_options_init(native);

    native->sorted = env->GetBooleanField(joptions, fid_sorted) ? 1 : 0;
    native->skip_hidden = env->GetBooleanField(joptions, fid_skip_hidden) ? 1 : 0;
    native->max_depth = static_cast<size_t>(env->GetLongField(joptions, fid_max_depth));
    native->max_file_cnt = static_cast<size_t>(env->GetLongField(joptions, fid_max_file_cnt));
    native->case_sensitive = env->GetBooleanField(joptions, fid_case_sensitive) ? 1 : 0;
    native->follow_links = env->GetBooleanField(joptions, fid_follow_links) ? 1 : 0;

    jobject jreturn_type = env->GetObjectField(joptions, fid_return_type);
    if (jreturn_type)
    {
        jclass rt_cls = env->GetObjectClass(jreturn_type);
        jmethodID mid_get_value = env->GetMethodID(rt_cls, "getValue", "()I");
        native->return_type = static_cast<uint32_t>(env->CallIntMethod(jreturn_type, mid_get_value));
        env->DeleteLocalRef(jreturn_type);
    }
    else
    {
        native->return_type = CSCANDIR_RETURN_BASE;
    }

    jobject jdir_include = env->GetObjectField(joptions, fid_dir_include);
    jobject jdir_exclude = env->GetObjectField(joptions, fid_dir_exclude);
    jobject jfile_include = env->GetObjectField(joptions, fid_file_include);
    jobject jfile_exclude = env->GetObjectField(joptions, fid_file_exclude);

    native->dir_include = java_list_to_c_array(env, jdir_include, native->dir_include_len);
    native->dir_exclude = java_list_to_c_array(env, jdir_exclude, native->dir_exclude_len);
    native->file_include = java_list_to_c_array(env, jfile_include, native->file_include_len);
    native->file_exclude = java_list_to_c_array(env, jfile_exclude, native->file_exclude_len);

    env->DeleteLocalRef(jdir_include);
    env->DeleteLocalRef(jdir_exclude);
    env->DeleteLocalRef(jfile_include);
    env->DeleteLocalRef(jfile_exclude);

    if (env->ExceptionCheck())
        return false;
    return true;
}

static void free_options_arrays(cscandir_options *native)
{
    free_c_array(const_cast<char **>(native->dir_include), native->dir_include_len);
    free_c_array(const_cast<char **>(native->dir_exclude), native->dir_exclude_len);
    free_c_array(const_cast<char **>(native->file_include), native->file_include_len);
    free_c_array(const_cast<char **>(native->file_exclude), native->file_exclude_len);
    native->dir_include = nullptr;
    native->dir_exclude = nullptr;
    native->file_include = nullptr;
    native->file_exclude = nullptr;
    native->dir_include_len = 0;
    native->dir_exclude_len = 0;
    native->file_include_len = 0;
    native->file_exclude_len = 0;
}

// ---------------------------------------------------------------------------
// Exception helper
// ---------------------------------------------------------------------------

static void throw_jscandir_exception(JNIEnv *env, int code, const char *message)
{
    jclass ex_cls = env->FindClass("com/scandir/error/JScandirException");
    if (!ex_cls)
        return;
    jmethodID mid = env->GetMethodID(ex_cls, "<init>", "(ILjava/lang/String;)V");
    if (!mid)
        return;
    jstring jmsg = to_jstring(env, message ? message : "unknown error");
    jobject ex = env->NewObject(ex_cls, mid, static_cast<jint>(code), jmsg);
    if (jmsg)
        env->DeleteLocalRef(jmsg);
    if (ex)
        env->Throw((jthrowable)ex);
    if (ex)
        env->DeleteLocalRef(ex);
}

// ---------------------------------------------------------------------------
// Result conversion helpers
// ---------------------------------------------------------------------------

static jobject make_statistics(JNIEnv *env, const cscandir_statistics *stats)
{
    jclass cls = env->FindClass("com/scandir/result/Statistics");
    if (!cls)
        return nullptr;
    jmethodID mid = env->GetMethodID(cls, "<init>", "(IIIIIIJJD)V");
    if (!mid)
        return nullptr;
    return env->NewObject(
        cls, mid,
        static_cast<jint>(stats->dirs),
        static_cast<jint>(stats->files),
        static_cast<jint>(stats->slinks),
        static_cast<jint>(stats->hlinks),
        static_cast<jint>(stats->devices),
        static_cast<jint>(stats->pipes),
        static_cast<jlong>(stats->size),
        static_cast<jlong>(stats->usage),
        static_cast<jdouble>(stats->duration));
}

static jobject make_entry(JNIEnv *env, const cscandir_entry *entry)
{
    jclass cls = env->FindClass("com/scandir/result/Entry");
    if (!cls)
        return nullptr;
    jmethodID mid = env->GetMethodID(
        cls, "<init>",
        "(Ljava/lang/String;ZZZDDDJZIJJJJJJIIJ)V");
    if (!mid)
        return nullptr;
    jstring jpath = to_jstring(env, entry->path);
    jobject obj = env->NewObject(
        cls, mid,
        jpath,
        (jboolean)(entry->is_symlink ? JNI_TRUE : JNI_FALSE),
        (jboolean)(entry->is_dir ? JNI_TRUE : JNI_FALSE),
        (jboolean)(entry->is_file ? JNI_TRUE : JNI_FALSE),
        static_cast<jdouble>(entry->ctime),
        static_cast<jdouble>(entry->mtime),
        static_cast<jdouble>(entry->atime),
        static_cast<jlong>(entry->size),
        (jboolean)(entry->has_ext ? JNI_TRUE : JNI_FALSE),
        static_cast<jint>(entry->mode),
        static_cast<jlong>(entry->ino),
        static_cast<jlong>(entry->dev),
        static_cast<jlong>(entry->nlink),
        static_cast<jlong>(entry->blksize),
        static_cast<jlong>(entry->blocks),
        static_cast<jint>(entry->uid),
        static_cast<jint>(entry->gid),
        static_cast<jlong>(entry->rdev));
    if (jpath)
        env->DeleteLocalRef(jpath);
    return obj;
}

static jobject make_string_list(JNIEnv *env, const cscandir_string_list *list)
{
    jclass arr_cls = env->FindClass("java/util/ArrayList");
    jmethodID arr_init = env->GetMethodID(arr_cls, "<init>", "()V");
    jmethodID arr_add = env->GetMethodID(arr_cls, "add", "(Ljava/lang/Object;)Z");
    if (!arr_init || !arr_add)
        return nullptr;
    jobject result = env->NewObject(arr_cls, arr_init);
    if (!result)
        return nullptr;
    for (size_t i = 0; i < list->len; ++i)
    {
        jstring js = to_jstring(env, list->items[i]);
        env->CallBooleanMethod(result, arr_add, js);
        if (js)
            env->DeleteLocalRef(js);
    }
    return result;
}

static jobject make_entry_list(JNIEnv *env, const cscandir_entry_list *list)
{
    jclass arr_cls = env->FindClass("java/util/ArrayList");
    jmethodID arr_init = env->GetMethodID(arr_cls, "<init>", "()V");
    jmethodID arr_add = env->GetMethodID(arr_cls, "add", "(Ljava/lang/Object;)Z");
    if (!arr_init || !arr_add)
        return nullptr;
    jobject result = env->NewObject(arr_cls, arr_init);
    if (!result)
        return nullptr;
    for (size_t i = 0; i < list->len; ++i)
    {
        jobject je = make_entry(env, &list->entries[i]);
        if (!je)
            continue;
        env->CallBooleanMethod(result, arr_add, je);
        env->DeleteLocalRef(je);
    }
    return result;
}

// ---------------------------------------------------------------------------
// Native entry points
// ---------------------------------------------------------------------------

extern "C" JNIEXPORT jobject JNICALL
Java_com_scandir_jni_JScandir_collect(JNIEnv *env, jclass, jstring jroot, jobject joptions)
{
    const char *root = to_cstr(env, jroot);
    if (!root)
    {
        throw_jscandir_exception(env, CSCANDIR_ERR_INVALID_ARGUMENT, "root path is null");
        return nullptr;
    }

    cscandir_options opts;
    if (!populate_options(env, joptions, &opts))
    {
        release_cstr(env, jroot, root);
        throw_jscandir_exception(env, CSCANDIR_ERR_INVALID_ARGUMENT, "invalid options");
        return nullptr;
    }

    cscandir_entry_list entries{};
    cscandir_string_list errors{};
    cscandir_error error{};
    cscandir_statistics stats{};

    int32_t code = cscandir_collect(root, &opts, &entries, &errors, &error);

    free_options_arrays(&opts);
    release_cstr(env, jroot, root);

    if (code != CSCANDIR_OK)
    {
        throw_jscandir_exception(
            env, (int)code,
            error.message ? error.message : "collect failed");
        cscandir_free_error(&error);
        cscandir_free_string_list(&errors);
        return nullptr;
    }

    jobject j_stats = make_statistics(env, &stats);
    jobject j_entries = make_entry_list(env, &entries);
    jobject j_errors = make_string_list(env, &errors);

    cscandir_free_entry_list(&entries);
    cscandir_free_string_list(&errors);
    cscandir_free_error(&error);

    if (env->ExceptionCheck())
    {
        if (j_stats)
            env->DeleteLocalRef(j_stats);
        if (j_entries)
            env->DeleteLocalRef(j_entries);
        if (j_errors)
            env->DeleteLocalRef(j_errors);
        return nullptr;
    }

    jclass res_cls = env->FindClass("com/scandir/result/CollectResult");
    jmethodID res_init = env->GetMethodID(
        res_cls, "<init>",
        "(Ljava/util/List;Ljava/util/List;Lcom/scandir/result/Statistics;)V");
    if (!res_init)
    {
        if (j_stats)
            env->DeleteLocalRef(j_stats);
        if (j_entries)
            env->DeleteLocalRef(j_entries);
        if (j_errors)
            env->DeleteLocalRef(j_errors);
        return nullptr;
    }

    jobject result = env->NewObject(res_cls, res_init, j_entries, j_errors, j_stats);

    if (j_stats)
        env->DeleteLocalRef(j_stats);
    if (j_entries)
        env->DeleteLocalRef(j_entries);
    if (j_errors)
        env->DeleteLocalRef(j_errors);

    return result;
}

extern "C" JNIEXPORT jobject JNICALL
Java_com_scandir_jni_JScandir_count(JNIEnv *env, jclass, jstring jroot, jobject joptions)
{
    const char *root = to_cstr(env, jroot);
    if (!root)
    {
        throw_jscandir_exception(env, CSCANDIR_ERR_INVALID_ARGUMENT, "root path is null");
        return nullptr;
    }

    cscandir_options opts;
    if (!populate_options(env, joptions, &opts))
    {
        release_cstr(env, jroot, root);
        throw_jscandir_exception(env, CSCANDIR_ERR_INVALID_ARGUMENT, "invalid options");
        return nullptr;
    }

    cscandir_string_list errors{};
    cscandir_error error{};
    cscandir_statistics stats{};

    int32_t code = cscandir_count(root, &opts, &stats, &errors, &error);

    free_options_arrays(&opts);
    release_cstr(env, jroot, root);

    if (code != CSCANDIR_OK)
    {
        throw_jscandir_exception(
            env, (int)code,
            error.message ? error.message : "count failed");
        cscandir_free_error(&error);
        cscandir_free_string_list(&errors);
        return nullptr;
    }

    jobject j_stats = make_statistics(env, &stats);

    cscandir_free_string_list(&errors);
    cscandir_free_error(&error);

    if (env->ExceptionCheck())
    {
        if (j_stats)
            env->DeleteLocalRef(j_stats);
        return nullptr;
    }

    return j_stats;
}

extern "C" JNIEXPORT jobject JNICALL
Java_com_scandir_jni_JScandir_walk(JNIEnv *env, jclass, jstring, jobject)
{
    // The cscandir C library does not expose a walk/TOC API yet, so this
    // operation cannot be backed by the native layer. Throw a descriptive
    // exception instead of silently returning null.
    throw_jscandir_exception(
        env, CSCANDIR_ERR_INVALID_ARGUMENT,
        "walk is not supported: the cscandir C library does not expose a walk/TOC API");
    return nullptr;
}