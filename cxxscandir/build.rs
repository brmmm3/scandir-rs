fn main() {
    // Build the C++ bridge
    cxx_build::bridge("src/bridge.rs")
        .file("src/cxxscandir.cc")
        .file("src/impl.cc")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wextra")
        .flag_if_supported("-Werror")
        .include("../cscandir/include")
        .compile("cxxscandir");

    // Link against the cscandir library
    println!("cargo:rustc-link-search=native=../target/release");
    println!("cargo:rustc-link-lib=cscandir");

    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=src/cxxscandir.cc");
    println!("cargo:rerun-if-changed=src/impl.cc");
    println!("cargo:rerun-if-changed=../cscandir/include/cscandir.h");
}