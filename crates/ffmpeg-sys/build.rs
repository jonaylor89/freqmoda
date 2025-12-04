use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=FFMPEG_DIR");
    println!("cargo:rerun-if-env-changed=FFMPEG_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=FFMPEG_LIB_DIR");
    println!("cargo:rerun-if-env-changed=FFMPEG_STATIC");
    println!("cargo:rerun-if-changed=include/wrapper.h");

    let ffmpeg_dir = env::var("FFMPEG_DIR").ok();
    let static_link = env::var("FFMPEG_STATIC").map(|v| v == "1").unwrap_or(false);

    // Determine include and lib directories
    let (include_dir, lib_dir) = if let Some(ref dir) = ffmpeg_dir {
        (format!("{}/include", dir), format!("{}/lib", dir))
    } else {
        // Try pkg-config for system-installed FFmpeg
        match try_pkg_config(static_link) {
            Ok((inc, lib)) => (inc, lib),
            Err(e) => {
                eprintln!("pkg-config failed: {}", e);
                eprintln!("Set FFMPEG_DIR to point to your FFmpeg installation");
                panic!("Cannot find FFmpeg libraries");
            }
        }
    };

    let include_dir = env::var("FFMPEG_INCLUDE_DIR").unwrap_or_else(|_| include_dir);
    let lib_dir = env::var("FFMPEG_LIB_DIR").unwrap_or_else(|_| lib_dir);

    // Link FFmpeg libraries
    println!("cargo:rustc-link-search=native={}", lib_dir);

    let link_type = if static_link { "static" } else { "dylib" };

    // Core libraries in dependency order (most dependent first)
    let libs = ["avformat", "avcodec", "avfilter", "swresample", "avutil"];

    for lib in &libs {
        println!("cargo:rustc-link-lib={}={}", link_type, lib);
    }

    // For static linking, we may need additional system libraries
    if static_link {
        // Common dependencies for static FFmpeg builds
        #[cfg(target_os = "linux")]
        {
            println!("cargo:rustc-link-lib=z");
            println!("cargo:rustc-link-lib=bz2");
            println!("cargo:rustc-link-lib=lzma");
            println!("cargo:rustc-link-lib=m");
            println!("cargo:rustc-link-lib=pthread");
        }
        #[cfg(target_os = "macos")]
        {
            println!("cargo:rustc-link-lib=z");
            println!("cargo:rustc-link-lib=bz2");
            println!("cargo:rustc-link-lib=lzma");
            println!("cargo:rustc-link-lib=iconv");
            println!("cargo:rustc-link-lib=framework=AudioToolbox");
            println!("cargo:rustc-link-lib=framework=CoreAudio");
            println!("cargo:rustc-link-lib=framework=CoreMedia");
            println!("cargo:rustc-link-lib=framework=CoreVideo");
            println!("cargo:rustc-link-lib=framework=VideoToolbox");
            println!("cargo:rustc-link-lib=framework=Security");
        }
    }

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("include/wrapper.h")
        .clang_arg(format!("-I{}", include_dir))
        // Allow all FFmpeg types and functions
        .allowlist_type("AV.*")
        .allowlist_type("Swr.*")
        .allowlist_function("av_.*")
        .allowlist_function("avformat_.*")
        .allowlist_function("avcodec_.*")
        .allowlist_function("avfilter_.*")
        .allowlist_function("avio_.*")
        .allowlist_function("swr_.*")
        .allowlist_var("AV.*")
        .allowlist_var("FF_.*")
        .allowlist_var("AVERROR.*")
        .allowlist_var("LIBAV.*")
        // Blocklist problematic types that cause issues
        .blocklist_type("max_align_t")
        // Use core types instead of std for no_std compatibility
        .use_core()
        .ctypes_prefix("libc")
        // Generate proper enum bindings
        .rustified_enum("AV.*")
        .rustified_enum("Swr.*")
        // Layout tests can be noisy, disable for cleaner output
        .layout_tests(false)
        .generate()
        .expect("Unable to generate FFmpeg bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}

fn try_pkg_config(static_link: bool) -> Result<(String, String), String> {
    let mut include_dirs = Vec::new();
    let mut lib_dirs = Vec::new();

    let libs = [
        "libavformat",
        "libavcodec",
        "libavfilter",
        "libswresample",
        "libavutil",
    ];

    for lib_name in &libs {
        let mut config = pkg_config::Config::new();
        config.statik(static_link);

        match config.probe(lib_name) {
            Ok(lib) => {
                for path in lib.include_paths {
                    let path_str = path.to_string_lossy().to_string();
                    if !include_dirs.contains(&path_str) {
                        include_dirs.push(path_str);
                    }
                }
                for path in lib.link_paths {
                    let path_str = path.to_string_lossy().to_string();
                    if !lib_dirs.contains(&path_str) {
                        lib_dirs.push(path_str);
                    }
                }
            }
            Err(e) => return Err(format!("{}: {}", lib_name, e)),
        }
    }

    let include_dir = include_dirs.first().cloned().unwrap_or_default();
    let lib_dir = lib_dirs.first().cloned().unwrap_or_default();

    Ok((include_dir, lib_dir))
}
