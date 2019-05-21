extern crate bindgen;

use std::env;
use std::path::Path;

/// Cargo runs main in this file to get some additional settings (e.g.,
/// LD_LIBRARY_PATH). It reads the printed output looking for certain variables,
/// see [here](http://doc.crates.io/build-script.html) for documentation.
fn main() {
    // Get the directory where we are building.
    let cargo_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dpdk_dir = env::var("RTE_SDK").unwrap();
    let dpdk_build = Path::new(&dpdk_dir).join("build");

    let dpdk_libs = dpdk_build.clone().join("lib");
    let native_path = Path::new(&cargo_dir)
        .parent()
        .unwrap()
        .join("target")
        .join("native");
    //println!("DPDK {:?}", dpdk_libs.to_str());
    // Use DPDK directory as -L
    println!(
        "cargo:rustc-link-search=native={}",
        dpdk_libs.to_str().unwrap()
    );
    if dpdk_libs.join("libdpdk.so").exists() {
        println!("cargo:rustc-link-lib=dpdk");
    }
    println!(
        "cargo:rustc-link-search=native={}",
        native_path.to_str().unwrap()
    );
    let header_path = Path::new(&cargo_dir)
        .join("src")
        .join("native_include")
        .join("dpdk-headers.h");
    let dpdk_include_path = dpdk_build.clone().join("include");
    println!("Header path {:?}", header_path.to_str());

    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .rust_target(bindgen::RustTarget::Nightly)
        .clang_args(vec!["-I", dpdk_include_path.to_str().unwrap()].iter())
        .generate()
        .expect("Unable to generate DPDK bindings");
    let out_dir = env::var("OUT_DIR").unwrap();
    let dpdk_bindings = Path::new(&out_dir).join("dpdk_bindings.rs");

    bindings
        .write_to_file(dpdk_bindings)
        .expect("Could not write bindings");
}
