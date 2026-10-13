#![allow(clippy::uninlined_format_args)]

// Patched build script for whisper-rs-sys 0.15.0 (voice-to-text, ADR-0011).
// Differences from upstream, each marked "patch:" below:
//   1. Builds the whisper.cpp tree pinned as the third_party/whisper.cpp
//      submodule (or WHISPER_CPP_DIR) instead of a copy bundled in the crate,
//      and stops with the fix when it is missing.
//   2. Reads the version from whisper.cpp >= 1.9's WHISPER_VERSION_* variables
//      and builds with WHISPER_BUILD_IS_DEV=OFF, so whisper_version() reports
//      the release it is.
//   3. Links libparakeet and binds include/parakeet.h (see wrapper.h).
//   4. Under `vulkan`, builds the pinned third_party/SPIRV-Headers submodule
//      (or SPIRV_HEADERS_DIR) into OUT_DIR for ggml-vulkan's
//      find_package(SPIRV-Headers CONFIG REQUIRED).
//   5. A bindgen failure is an error; there is no stale bundled bindings.rs
//      to fall back on.
//   6. x86-64 builds target a fixed x86-64-v3 baseline instead of the build
//      machine's CPU (GGML_NATIVE=OFF), so the binary runs where it ships.

extern crate bindgen;

use cmake::Config;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// patch: the parts of the whisper.cpp tree the CMake build reads, mirroring
/// upstream's crate `include` list. Copied into OUT_DIR because configuring
/// writes into the source tree (bindings/javascript/package.json).
const WHISPER_SOURCES: &[&str] = &[
    "CMakeLists.txt",
    "LICENSE",
    "cmake",
    "include",
    "src",
    "ggml",
    "bindings/CMakeLists.txt",
    "bindings/javascript/package-tmpl.json",
];

fn main() {
    let target = env::var("TARGET").unwrap();
    // Link C++ standard library
    if let Some(cpp_stdlib) = get_cpp_link_stdlib(&target) {
        println!("cargo:rustc-link-lib=dylib={}", cpp_stdlib);
    }
    // Link macOS Accelerate framework for matrix calculations
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=framework=Accelerate");
        #[cfg(feature = "coreml")]
        {
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=CoreML");
        }
        #[cfg(feature = "metal")]
        {
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=Metal");
            println!("cargo:rustc-link-lib=framework=MetalKit");
        }
    }

    #[cfg(feature = "coreml")]
    println!("cargo:rustc-link-lib=static=whisper.coreml");

    #[cfg(feature = "openblas")]
    {
        if let Ok(openblas_path) = env::var("OPENBLAS_PATH") {
            println!(
                "cargo::rustc-link-search={}",
                PathBuf::from(openblas_path).join("lib").display()
            );
        }
        if cfg!(windows) {
            println!("cargo:rustc-link-lib=libopenblas");
        } else {
            println!("cargo:rustc-link-lib=openblas");
        }
    }
    #[cfg(feature = "cuda")]
    {
        println!("cargo:rustc-link-lib=cublas");
        println!("cargo:rustc-link-lib=cudart");
        println!("cargo:rustc-link-lib=cublasLt");
        println!("cargo:rustc-link-lib=cuda");
        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                let cuda_path = PathBuf::from(env::var("CUDA_PATH").unwrap()).join("lib/x64");
                println!("cargo:rustc-link-search={}", cuda_path.display());
            } else {
                println!("cargo:rustc-link-lib=culibos");
                println!("cargo:rustc-link-search=/usr/local/cuda/lib64");
                println!("cargo:rustc-link-search=/usr/local/cuda/lib64/stubs");
                println!("cargo:rustc-link-search=/opt/cuda/lib64");
                println!("cargo:rustc-link-search=/opt/cuda/lib64/stubs");
            }
        }
    }
    #[cfg(feature = "hipblas")]
    {
        println!("cargo:rustc-link-lib=hipblas");
        println!("cargo:rustc-link-lib=rocblas");
        println!("cargo:rustc-link-lib=amdhip64");

        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                panic!("Due to a problem with the last revision of the ROCm 5.7 library, it is not possible to compile the library for the windows environment.\nSee https://github.com/ggerganov/whisper.cpp/issues/2202 for more details.")
            } else {
                println!("cargo:rerun-if-env-changed=HIP_PATH");

                let hip_path = match env::var("HIP_PATH") {
                    Ok(path) =>PathBuf::from(path),
                    Err(_) => PathBuf::from("/opt/rocm"),
                };
                let hip_lib_path = hip_path.join("lib");

                println!("cargo:rustc-link-search={}",hip_lib_path.display());
            }
        }
    }

    #[cfg(feature = "openmp")]
    {
        if target.contains("gnu") {
            println!("cargo:rustc-link-lib=gomp");
        } else if target.contains("apple") {
            println!("cargo:rustc-link-lib=omp");
            println!("cargo:rustc-link-search=/opt/homebrew/opt/libomp/lib");
        }
    }

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let third_party = manifest_dir
        .parent()
        .expect("whisper-rs-sys lives under third_party/")
        .to_path_buf();

    // patch: build the pinned submodule, not a bundled copy.
    println!("cargo:rerun-if-env-changed=WHISPER_CPP_DIR");
    let whisper_src = env::var_os("WHISPER_CPP_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| third_party.join("whisper.cpp"));
    if !whisper_src.join("CMakeLists.txt").is_file() {
        panic!(
            "whisper.cpp source not found at {}.\n\
             Fetch the pinned engine with:\n    git submodule update --init\n\
             (or set WHISPER_CPP_DIR to a whisper.cpp checkout)",
            whisper_src.display()
        );
    }
    for watched in ["CMakeLists.txt", "cmake", "include", "src", "ggml"] {
        println!(
            "cargo:rerun-if-changed={}",
            whisper_src.join(watched).display()
        );
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let whisper_root = out.join("whisper.cpp");
    copy_whisper_sources(&whisper_src, &whisper_root);

    // patch: bindgen reads the headers from the submodule itself, not the
    // OUT_DIR copy, so the per-header rerun-if-changed lines bindgen emits
    // point at files whose mtimes only move when the submodule does.
    // https://github.com/rust-lang/rust-bindgen/issues/2691
    // https://github.com/rust-lang/rust-bindgen/issues/3264
    // https://gitlab.freedesktop.org/gstreamer/gst-plugins-rs/-/issues/779
    let mut bindings = bindgen::Builder::default()
        .rust_edition(bindgen::RustEdition::Edition2021)
        .rust_target(bindgen::RustTarget::stable(88, 0).map_err(|v| v.to_string()).unwrap())
        .header("wrapper.h");

    #[cfg(feature = "metal")]
    {
        bindings = bindings.header(
            whisper_src
                .join("ggml/include/ggml-metal.h")
                .display()
                .to_string(),
        );
    }
    #[cfg(feature = "vulkan")]
    {
        bindings = bindings
            .header(
                whisper_src
                    .join("ggml/include/ggml-vulkan.h")
                    .display()
                    .to_string(),
            )
            .clang_arg("-DGGML_USE_VULKAN=1");
    }

    let bindings = bindings
        .clang_arg(format!("-I{}", whisper_src.display()))
        .clang_arg(format!("-I{}", whisper_src.join("include").display()))
        .clang_arg(format!("-I{}", whisper_src.join("ggml/include").display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap_or_else(|e| {
            panic!(
                "Unable to generate whisper.cpp bindings ({e}). bindgen needs libclang: \
                 install libclang-dev (Linux), LLVM (Windows, LIBCLANG_PATH) or Xcode \
                 command-line tools (macOS)."
            )
        });
    bindings
        .write_to_file(out.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // stop if we're on docs.rs
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    let mut config = Config::new(&whisper_root);

    config
        .profile("Release")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("WHISPER_ALL_WARNINGS", "OFF")
        .define("WHISPER_ALL_WARNINGS_3RD_PARTY", "OFF")
        .define("WHISPER_BUILD_TESTS", "OFF")
        .define("WHISPER_BUILD_EXAMPLES", "OFF")
        // patch: the submodule sits exactly on a release tag.
        .define("WHISPER_BUILD_IS_DEV", "OFF")
        .very_verbose(true)
        .pic(true);

    if cfg!(target_os = "windows") {
        config.cxxflag("/utf-8");
        println!("cargo:rustc-link-lib=advapi32");
    }

    if cfg!(feature = "coreml") {
        config.define("WHISPER_COREML", "ON");
        config.define("WHISPER_COREML_ALLOW_FALLBACK", "1");
    }

    if cfg!(feature = "cuda") {
        config.define("GGML_CUDA", "ON");
        config.define("CMAKE_POSITION_INDEPENDENT_CODE", "ON");
        config.define("CMAKE_CUDA_FLAGS", "-Xcompiler=-fPIC");
    }

    if cfg!(feature = "hipblas") {
        config.define("GGML_HIP", "ON");
        config.define("CMAKE_C_COMPILER", "hipcc");
        config.define("CMAKE_CXX_COMPILER", "hipcc");
        println!("cargo:rerun-if-env-changed=AMDGPU_TARGETS");
        if let Ok(gpu_targets) = env::var("AMDGPU_TARGETS") {
            config.define("AMDGPU_TARGETS", gpu_targets);
        }
    }

    if cfg!(feature = "vulkan") {
        config.define("GGML_VULKAN", "ON");
        // patch: ggml-vulkan needs SPIRV-Headers from ggml 0.26 on.
        if let Some(prefix) = build_spirv_headers(&third_party, &out) {
            config.define("CMAKE_PREFIX_PATH", &prefix);
            // ggml-vulkan includes <spirv/unified1/spirv.hpp> without adding
            // the package's include directory to its target.
            config.cxxflag(format!("-I{}", prefix.join("include").display()));
        }
        if cfg!(windows) {
            println!("cargo:rerun-if-env-changed=VULKAN_SDK");
            println!("cargo:rustc-link-lib=vulkan-1");
            let vulkan_path = match env::var("VULKAN_SDK") {
                Ok(path) => PathBuf::from(path),
                Err(_) => panic!(
                    "Please install Vulkan SDK and ensure that VULKAN_SDK env variable is set"
                ),
            };
            let vulkan_lib_path = vulkan_path.join("Lib");
            println!("cargo:rustc-link-search={}", vulkan_lib_path.display());
        } else if cfg!(target_os = "macos") {
            println!("cargo:rerun-if-env-changed=VULKAN_SDK");
            println!("cargo:rustc-link-lib=vulkan");
            let vulkan_path = match env::var("VULKAN_SDK") {
                Ok(path) => PathBuf::from(path),
                Err(_) => panic!(
                    "Please install Vulkan SDK and ensure that VULKAN_SDK env variable is set"
                ),
            };
            let vulkan_lib_path = vulkan_path.join("lib");
            println!("cargo:rustc-link-search={}", vulkan_lib_path.display());
        } else {
            println!("cargo:rustc-link-lib=vulkan");
        }
    }

    if cfg!(feature = "openblas") {
        config.define("GGML_BLAS", "ON");
        config.define("GGML_BLAS_VENDOR", "OpenBLAS");
        if env::var("BLAS_INCLUDE_DIRS").is_err() {
            panic!("BLAS_INCLUDE_DIRS environment variable must be set when using OpenBLAS");
        }
        config.define("BLAS_INCLUDE_DIRS", env::var("BLAS_INCLUDE_DIRS").unwrap());
        println!("cargo:rerun-if-env-changed=BLAS_INCLUDE_DIRS");
    }

    if cfg!(feature = "metal") {
        config.define("GGML_METAL", "ON");
        config.define("GGML_METAL_NDEBUG", "ON");
        config.define("GGML_METAL_EMBED_LIBRARY", "ON");
    } else {
        // Metal is enabled by default, so we need to explicitly disable it
        config.define("GGML_METAL", "OFF");
    }

    if cfg!(debug_assertions) || cfg!(feature = "force-debug") {
        // debug builds are too slow to even remotely be usable,
        // so we build with optimizations even in debug mode
        config.define("CMAKE_BUILD_TYPE", "RelWithDebInfo");
        config.cxxflag("-DWHISPER_DEBUG");
    } else {
        // we're in release mode, explicitly set to release mode
        // see also https://codeberg.org/tazz4843/whisper-rs/issues/226
        config.define("CMAKE_BUILD_TYPE", "Release");
    }

    // patch: a portable CPU baseline (TASK-VTT184). ggml's default,
    // GGML_NATIVE=ON, compiles for the build machine (-march=native), so a
    // binary built on one machine (a CI runner with AVX-512, say) can stop
    // with an illegal instruction on a user's older CPU. x86-64 builds target
    // x86-64-v3 instead: AVX2, FMA, F16C and BMI2, never AVX-512. A GGML_*
    // variable in the environment still wins (the loop below), e.g.
    // GGML_NATIVE=ON for a build that never leaves its machine.
    if env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("x86_64") {
        config.define("GGML_NATIVE", "OFF");
        for isa in [
            "GGML_SSE42",
            "GGML_AVX",
            "GGML_AVX2",
            "GGML_BMI2",
            "GGML_FMA",
            "GGML_F16C",
        ] {
            config.define(isa, "ON");
        }
        config.define("GGML_AVX512", "OFF");
    }

    // Allow passing any WHISPER or CMAKE compile flags
    for (key, value) in env::vars() {
        let is_whisper_flag = key.starts_with("WHISPER_") && key != "WHISPER_CPP_DIR";
        let is_ggml_flag = key.starts_with("GGML_");
        let is_cmake_flag = key.starts_with("CMAKE_");
        if is_whisper_flag || is_ggml_flag || is_cmake_flag {
            config.define(&key, &value);
        }
    }

    if cfg!(not(feature = "openmp")) {
        config.define("GGML_OPENMP", "OFF");
    }

    if cfg!(feature = "intel-sycl") {
        config.define("BUILD_SHARED_LIBS", "ON");
        config.define("GGML_SYCL", "ON");
        config.define("GGML_SYCL_TARGET", "INTEL");
        config.define("CMAKE_C_COMPILER", "icx");
        config.define("CMAKE_CXX_COMPILER", "icpx");
    }

    let destination = config.build();

    add_link_search_path(&out.join("build")).unwrap();

    println!("cargo:rustc-link-search=native={}", destination.display());
    if cfg!(feature = "intel-sycl") {
        println!("cargo:rustc-link-lib=whisper");
        // patch: the Parakeet engine (ADR-0011).
        println!("cargo:rustc-link-lib=parakeet");
        println!("cargo:rustc-link-lib=ggml");
        println!("cargo:rustc-link-lib=ggml-base");
        println!("cargo:rustc-link-lib=ggml-cpu");
    } else {
        println!("cargo:rustc-link-lib=static=whisper");
        // patch: the Parakeet engine (ADR-0011).
        println!("cargo:rustc-link-lib=static=parakeet");
        println!("cargo:rustc-link-lib=static=ggml");
        println!("cargo:rustc-link-lib=static=ggml-base");
        println!("cargo:rustc-link-lib=static=ggml-cpu");
    }
    if cfg!(target_os = "macos") || cfg!(feature = "openblas") {
        println!("cargo:rustc-link-lib=static=ggml-blas");
    }
    if cfg!(feature = "vulkan") {
        if cfg!(feature = "intel-sycl") {
            println!("cargo:rustc-link-lib=ggml-vulkan");
        } else {
            println!("cargo:rustc-link-lib=static=ggml-vulkan");
        }
    }

    if cfg!(feature = "hipblas") {
        println!("cargo:rustc-link-lib=static=ggml-hip");
    }

    if cfg!(feature = "metal") {
        println!("cargo:rustc-link-lib=static=ggml-metal");
    }

    if cfg!(feature = "cuda") {
        println!("cargo:rustc-link-lib=static=ggml-cuda");
    }

    if cfg!(feature = "openblas") {
        println!("cargo:rustc-link-lib=static=ggml-blas");
    }

    if cfg!(feature = "intel-sycl") {
        println!("cargo:rustc-link-lib=ggml-sycl");
    }

    println!(
        "cargo:WHISPER_CPP_VERSION={}",
        get_whisper_cpp_version(&whisper_root)
            .expect("Failed to read whisper.cpp CMake config")
            .expect("Could not find whisper.cpp version declaration"),
    );
}

/// patch: replace OUT_DIR's copy of the whisper.cpp sources with a fresh one,
/// so moving the submodule always rebuilds what it points at.
fn copy_whisper_sources(src: &Path, dst: &Path) {
    if dst.exists() {
        std::fs::remove_dir_all(dst)
            .unwrap_or_else(|e| panic!("Failed to clear {}: {}", dst.display(), e));
    }
    for rel in WHISPER_SOURCES {
        let from = src.join(rel);
        let to = dst.join(rel);
        let parent = to.parent().expect("source paths are relative");
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|e| panic!("Failed to create {}: {}", parent.display(), e));
        let copied = if from.is_dir() {
            fs_extra::dir::copy(&from, parent, &fs_extra::dir::CopyOptions::new())
                .map(|_| ())
                .map_err(|e| e.to_string())
        } else {
            std::fs::copy(&from, &to)
                .map(|_| ())
                .map_err(|e| e.to_string())
        };
        if let Err(e) = copied {
            panic!(
                "Failed to copy {} into {}: {}",
                from.display(),
                to.display(),
                e
            );
        }
    }
}

/// patch: install the pinned SPIRV-Headers into OUT_DIR and return the
/// prefix. None when the submodule is absent, leaving CMake to find a
/// system or Vulkan SDK copy (the SDK ships one).
fn build_spirv_headers(third_party: &Path, out: &Path) -> Option<PathBuf> {
    println!("cargo:rerun-if-env-changed=SPIRV_HEADERS_DIR");
    let src = env::var_os("SPIRV_HEADERS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| third_party.join("SPIRV-Headers"));
    if !src.join("CMakeLists.txt").is_file() {
        println!(
            "cargo:warning=SPIRV-Headers not found at {}; relying on a system or Vulkan SDK copy \
             (git submodule update --init fetches the pinned one)",
            src.display()
        );
        return None;
    }
    println!("cargo:rerun-if-changed={}", src.join("include").display());
    let prefix = Config::new(&src)
        .profile("Release")
        .define("SPIRV_HEADERS_ENABLE_TESTS", "OFF")
        .define("SPIRV_HEADERS_ENABLE_INSTALL", "ON")
        .out_dir(out.join("spirv-headers"))
        .build();
    Some(prefix)
}

// From https://github.com/alexcrichton/cc-rs/blob/fba7feded71ee4f63cfe885673ead6d7b4f2f454/src/lib.rs#L2462
fn get_cpp_link_stdlib(target: &str) -> Option<&'static str> {
    if target.contains("msvc") {
        None
    } else if target.contains("apple") || target.contains("freebsd") || target.contains("openbsd") {
        Some("c++")
    } else if target.contains("android") {
        Some("c++_shared")
    } else {
        Some("stdc++")
    }
}

fn add_link_search_path(dir: &std::path::Path) -> std::io::Result<()> {
    if dir.is_dir() {
        println!("cargo:rustc-link-search={}", dir.display());
        for entry in std::fs::read_dir(dir)? {
            add_link_search_path(&entry?.path())?;
        }
    }
    Ok(())
}

/// patch: whisper.cpp >= 1.9 sets WHISPER_VERSION_MAJOR/MINOR/PATCH; older
/// trees declare `project("whisper.cpp" VERSION x.y.z)`. Both are read.
fn get_whisper_cpp_version(whisper_root: &std::path::Path) -> std::io::Result<Option<String>> {
    let cmake_lists = BufReader::new(File::open(whisper_root.join("CMakeLists.txt"))?);

    let (mut major, mut minor, mut patch) = (None, None, None);
    for line in cmake_lists.lines() {
        let line = line?;
        let line = line.trim();

        if let Some(suffix) = line.strip_prefix(r#"project("whisper.cpp" VERSION "#) {
            let whisper_cpp_version = suffix.trim_end_matches(')');
            return Ok(Some(whisper_cpp_version.into()));
        }
        let part = |name: &str| {
            line.strip_prefix(&format!("set({name} "))
                .map(|v| v.trim_end_matches(')').trim().to_string())
        };
        major = major.or_else(|| part("WHISPER_VERSION_MAJOR"));
        minor = minor.or_else(|| part("WHISPER_VERSION_MINOR"));
        patch = patch.or_else(|| part("WHISPER_VERSION_PATCH"));
    }

    Ok(match (major, minor, patch) {
        (Some(a), Some(b), Some(c)) => Some(format!("{a}.{b}.{c}")),
        _ => None,
    })
}
