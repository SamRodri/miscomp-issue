use std::{env, fs, path::PathBuf};

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "android" {
        android();
    }
}

// copy and link "c++_shared"
fn android() {
    println!("cargo:rustc-link-lib=c++_shared");

    if let Ok(output_path) = env::var("CARGO_NDK_OUTPUT_PATH") {
        let output_path = PathBuf::from(output_path);

        let sysroot_libs_path = PathBuf::from(env::var_os("CARGO_NDK_SYSROOT_LIBS_PATH").unwrap());
        let lib_path = sysroot_libs_path.join("libc++_shared.so");

        let output_path = output_path.join(env::var("CARGO_NDK_ANDROID_TARGET").unwrap());
        let _ = fs::create_dir(&output_path);

        let output_path = output_path.join("libc++_shared.so");
        std::fs::copy(lib_path, &output_path).unwrap();
        println!("cargo:rerun-if-changed={}", output_path.display());
    }
    println!("cargo:rerun-if-env-changed=CARGO_NDK_OUTPUT_PATH");
    println!("cargo:rerun-if-env-changed=CARGO_NDK_ANDROID_TARGET");
}