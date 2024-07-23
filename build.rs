use std::env;
use std::path::PathBuf;
use git2::Repository;
use cmake::Config;

fn make_lib_name(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.lib", name)
    } else {
        format!("lib{}.a", name)
    }
}

fn main() {

    // Clone the asar repository and checkout the v1.91 tag
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    if !out_dir.join("asar").exists() {
        let url = "https://github.com/RPGHacker/asar";
        let asar_repo = Repository::clone(url, out_dir.join("asar")).unwrap();
        let (object, reference) = asar_repo.revparse_ext("v1.91").expect("Failed to get tag");
        asar_repo.checkout_tree(&object, None).expect("Failed to checkout tree");

        match reference {
            Some(gref) => asar_repo.set_head(gref.name().unwrap()),
            None => asar_repo.set_head_detached(object.id()),
        }.expect("Failed to set head");
    }

    let expected_lib_path = out_dir.join("lib").join(make_lib_name("asar"));

    // build asar with cmake
    if !expected_lib_path.exists() {
        let _dst = Config::new(out_dir.join("asar/src"))
            .define("ASAR_GEN_LIB", "ON")
            .define("ASAR_GEN_EXE", "OFF")
            .define("ASAR_GEN_DLL", "OFF")
            .define("ASAR_GEN_EXE_TEST", "OFF")
            .define("ASAR_GEN_DLL_TEST", "OFF")
            .profile("Release")
            .build();
    }

    println!("cargo:rerun-if-changed=src/asar/asar.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/asar/asar.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    println!("cargo:rustc-link-search={}", out_dir.join("lib").display());
    println!("cargo:rustc-link-lib=static=asar-static");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}