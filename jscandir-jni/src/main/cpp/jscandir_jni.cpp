#include <jni.h>
#include <cscandir.h>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <memory>

// Helper to convert Java string to C string
static const char* to_cstr(JNIEnv* env, jstring jstr) {
    if (!jstr) return nullptr;
    return env->GetStringUTFChars(jstr, nullptr);
}

static void release_cstr(JNIEnv* env, jstring jstr, const char* cstr) {
    if (jstr && cstr) {
        env->ReleaseStringUTFChars(jstr, cstr);
    }
}

// Helper to convert C string to Java string
static jstring to_jstring(JNIEnv* env, const char* cstr) {
    return cstr ? env->NewStringUTF(cstr) : nullptr;
}

// Helper to convert Java string array to C array
static char** to_c_array(JNIEnv* env, jobjectArray jarray, size_t& out_len) {
    if (!jarray) {
        return nullptr;
    }
    jsize len = env->GetArrayLength(jarray);
    char** arr = static_cast<char**>(std::malloc(len * sizeof(char*)));
    for (jsize i = 0; i < len; ++i) {
        jstring jstr = (jstring)env->GetObjectArrayElement(jarray, i);
        const char* cstr = env->GetStringUTFChars((jstring)env->GetObjectArrayElement(jarray, i), nullptr);
        arr[i] = cstr ? strdup(cstr) : nullptr;
        env->ReleaseStringUTFChars((jstring)env->GetObjectArrayElement(jarray, i), cstr);
        env->DeleteLocalRef(jstr);
    }
    return arr;
}

static void free_c_array(char** arr, size_t len) {
    if (!arr) return;
    for (size_t i = 0; i < len; ++i) {
        if (arr[i]) std::free(arr[i]);
    }
    std::free(arr);
}

// Convert Java Options to C struct
static void populate_options(JNIEnv* env, jobject joptions, cscandir_options* native) {
    if (!joptions) return;

    jclass cls = env->GetObjectClass(joptions);
    
    // Get field IDs
    jfieldID fid_sorted = env->GetFieldID(env->GetObjectClass(joptions), "sorted", "Z");
    jfieldID fid_skip_hidden = env->GetFieldID(env->GetObjectClass(joptions), "skipHidden", "Z");
    jfieldID fid_max_depth = env->GetFieldID(env->GetObjectClass(joptions), "maxDepth", "J");
    jfieldID fid_max_file_cnt = env->GetFieldID(env->GetObjectClass(joptions), "maxFileCnt", "J");
    jfieldID fid_case_sensitive = env->GetFieldID(env->GetObjectClass(joptions), "caseSensitive", "Z");
    jfieldID fid_follow_links = env->GetFieldID(env->GetObjectClass(joptions), "followLinks", "Z");
    jfieldID fid_return_type = env->GetFieldID(env->GetObjectClass(joptions), "returnType", "Lcom/scandir/options/ReturnType;");

    // Get filter arrays
    jfieldID fid_dir_include = env->GetFieldID(env->GetObjectClass(joptions), "dirInclude", "Ljava/util/List;");
    jfieldID fid_dir_exclude = env->GetFieldID(env->GetObjectClass(joptions), "dirExclude", "Ljava/util/List;");
    jfieldID fid_file_include = env->GetFieldID(env->GetObjectClass(joptions), "fileInclude", "Ljava/util/List;");
    jfieldID fid_file_exclude = env->GetFieldID(env->GetObjectClass(joptions), "fileExclude", "Ljava/util/List;");

    // Get boolean fields
    native->sorted = env->GetBooleanField(joptions, fid_sorted) ? 1 : 0;
    native->skip_hidden = env->GetBooleanField(joptions, fid_skip_hidden) ? 1 : 0;
    native->case_sensitive = env->GetBooleanField(joptions, fid_case_sensitive) ? 1 : 0;
    native->follow_links = env->GetBooleanField(joptions, fid_follow_links) ? 1 : 0;

    // Get long fields
    native->max_depth = env->GetLongField(joptions, fid_max_depth);
    native->max_file_cnt = env->GetLongField(joptions, fid_max_file_cnt);

    // Get return type
    jobject jreturn_type = env->GetObjectField(joptions, fid_return_type);
    if (jreturn_type) {
        jclass return_type_cls = env->GetObjectClass(jreturn_type);
        jmethodID mid_get_value = env->GetMethodID(return_type_cls, "getValue", "()I");
        native->return_type = env->CallIntMethod(jreturn_type, mid_get_value);
    } else {
        native->return_type = 0;
    }

    // Get filter lists and convert to C arrays
    auto get_list_array = [&](jobject list_obj, char*** out_ptr, size_t* out_len) -> void {
        if (!list_obj) {
            *out_ptr = nullptr;
            return;
        }
        jclass list_cls = env->GetObjectClass(list_obj);
        jmethodID mid_size = env->GetMethodID(list_cls, "size", "()I");
        jmethodID mid_get = env->GetMethodID(list_cls, "get", "(I)Ljava/lang/Object;");
        int len = env->CallIntMethod(list_obj, mid_size);
        char** arr = static_cast<char**>(std::malloc(len * sizeof(char*)));
        for (int i = 0; i < len; ++i) {
            jstring jstr = (jstring)env->CallObjectMethod(list_obj, env->GetMethodID(env->GetObjectClass(list_obj), "get", "(I)Ljava/lang/Object;"), i);
            const char* cstr = env->GetStringUTFChars(jstr, nullptr);
            arr[i] = cstr ? strdup(cstr) : nullptr;
            env->ReleaseStringUTFChars((jstring)env->CallObjectMethod(list_obj, env->GetMethodID(env->GetObjectClass(list_obj), "get", "(I)Ljava/lang/Object;"), i), cstr);
            env->DeleteLocalRef(jstr);
        }
        // Store in output
        // This is simplified - we'd need to return the array properly
    };
}

// JNIEXPORT for collect
extern "C" JNIEXPORT jobject JNICALL
Java_com_scandir_jni_JScandir_collect(JNIEnv* env, jclass, jstring jroot, jobject joptions) {
    // This is a simplified stub - the actual implementation would need
    // proper JNI handling of all the types and memory management
    return nullptr;
}

extern "C" JNIEXPORT jobject JNICALL
Java_com_scandir_jni_JScandir_count(JNIEnv* env, jclass, jstring jroot, jobject joptions) {
    return nullptr;
}

extern "C" JNIEXPORT jobject JNICALL
Java_com_scandir_jni_JScandir_walk(JNIEnv* env, jclass, jstring jroot, jobject joptions) {
    return nullptr;
}