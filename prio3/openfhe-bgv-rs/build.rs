//! build.rs for openfhe-bgv-rs
//!
//! Compiles the C++ shim and tells Cargo how to link against the
//! OpenFHE shared libraries.
//!
//! Environment variables (override with OPENFHE_DIR or individually):
//!   OPENFHE_INCLUDE_DIR   — path to OpenFHE headers (default: /usr/local/include/openfhe)
//!   OPENFHE_LIB_DIR       — path to OpenFHE .so/.a files (default: /usr/local/lib)
//!   OPENFHE_STATIC        — if set to "1", link statically

use std::env;

fn main() {
    let base = env::var("OPENFHE_DIR").unwrap_or_else(|_| "/usr/local".into());

    let include_dir = env::var("OPENFHE_INCLUDE_DIR")
        .unwrap_or_else(|_| format!("{base}/include/openfhe"));

    let lib_dir = env::var("OPENFHE_LIB_DIR")
        .unwrap_or_else(|_| format!("{base}/lib"));

    let static_link = env::var("OPENFHE_STATIC")
        .map(|v| v == "1")
        .unwrap_or(false);

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .opt_level(2)
        .file("wrapper/openfhe_wrapper.cpp")
        .include(&include_dir)
        .include(format!("{include_dir}/core"))
        .include(format!("{include_dir}/core/lattice"))
        .include(format!("{include_dir}/pke"))
        .include(format!("{include_dir}/binfhe"))
        .define("MATHBACKEND", "4")
        .warnings(false);

    if cfg!(target_os = "macos") {
        build.flag("-stdlib=libc++");
    }

    build.compile("openfhe_wrapper");

    println!("cargo:rustc-link-search=native={lib_dir}");

    let link_type = if static_link { "static" } else { "dylib" };
    for lib in &["OPENFHEpke", "OPENFHEcore", "OPENFHEbinfhe"] {
        println!("cargo:rustc-link-lib={link_type}={lib}");
    }

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=dylib=c++");
    }

    #[cfg(feature = "regenerate-bindings")]
    {
        let bindings = bindgen::Builder::default()
            .header("wrapper/openfhe_wrapper.h")
            .allowlist_function("ofhe_bgv_.*")
            .allowlist_type("OFHE.*")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("bindgen failed");

        let out_path = std::path::PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("could not write bindings.rs");
    }

    println!("cargo:rerun-if-changed=wrapper/openfhe_wrapper.h");
    println!("cargo:rerun-if-changed=wrapper/openfhe_wrapper.cpp");
    println!("cargo:rerun-if-env-changed=OPENFHE_DIR");
    println!("cargo:rerun-if-env-changed=OPENFHE_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=OPENFHE_LIB_DIR");
    println!("cargo:rerun-if-env-changed=OPENFHE_STATIC");
}
