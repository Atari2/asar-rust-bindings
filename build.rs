use cmake::Config;
use std::env;
use std::path::PathBuf;

fn make_lib_name(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.lib", name)
    } else {
        format!("lib{}.a", name)
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();

    let expected_lib_path = out_dir.join("lib").join(make_lib_name("asar-static"));
    // build asar with cmake
    if !expected_lib_path.exists() {
        let _dst = Config::new("src/asar/src")
            .out_dir(out_dir.clone())
            .define("ASAR_GEN_LIB", "ON")
            .define("ASAR_GEN_EXE", "OFF")
            .define("ASAR_GEN_DLL", "OFF")
            .define("ASAR_GEN_EXE_TEST", "OFF")
            .define("ASAR_GEN_DLL_TEST", "OFF")
            .profile("Release")
            .build();
    }

    println!("cargo:rerun-if-changed=src/asar/src/asar-dll-bindings/c/asar.h");

    if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/asar/src/asar-dll-bindings/c/asar.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("errordata")
        .allowlist_type("labeldata")
        .allowlist_type("definedata")
        .allowlist_type("writtenblockdata")
        .allowlist_type("mappertype")
        .allowlist_type("warnsetting")
        .allowlist_type("memoryfile")
        .allowlist_type("patchparams")
        .allowlist_function("asar_.*")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    println!("cargo:rustc-link-search={}", out_dir.join("lib").display());
    println!("cargo:rustc-link-lib=static=asar-static");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
