use std::env;
/// Cargo runs main in this file to get some additional settings (e.g., LD_LIBRARY_PATH). It reads the printed output
/// looking for certain variables, see [here](http://doc.crates.io/build-script.html) for documentation.
fn main() {
    // Get the directory where we are building.
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // Send current directory as -L
    println!("cargo:rustc-link-search=native={}", dir.clone() + "/../3rdparty/dpdk/build/lib");
    println!("cargo:rustc-link-search=native={}", dir + "/../native");
    // Add -ldpdk
    println!("cargo:rustc-link-lib=dylib=dpdk");
}
