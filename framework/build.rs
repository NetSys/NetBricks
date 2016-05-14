use std::env;
use std::io::prelude::*;
use std::fs::File;
fn parse_ld_archive(ar: &str) -> Vec<String> {
    let mut f = File::open(ar).unwrap();
    let mut content = String::new();
    f.read_to_string(&mut content).unwrap();
    if "GROUP" == &content[0..5] {
        println!("Found group");
        let open_idx = content.find("(").unwrap_or_else(|| {content.len()});
        let remove_open = content[open_idx + 1..].trim();
        let end_idx = remove_open.find(")").unwrap_or_else(|| {remove_open.len()});
        let remaining = remove_open[..end_idx].trim();
        println!("Remaining is {}", remaining);
        remaining.split_whitespace().map(|s| {
            let end = s.len() - 2;
            String::from(&s[3..end])
        }).collect()
    } else {
        panic!("Could not find a group");
    }
}

/// Cargo runs main in this file to get some additional settings (e.g., LD_LIBRARY_PATH). It reads the printed output
/// looking for certain variables, see [here](http://doc.crates.io/build-script.html) for documentation.
fn main() {
    // Get the directory where we are building.
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dpdk_path = dir.clone() + "/../3rdparty/dpdk/build/lib/libdpdk.a";
    let libs = parse_ld_archive(&dpdk_path);
    // Send current directory as -L
    println!("cargo:rustc-link-search=native={}", dir.clone() + "/../3rdparty/dpdk/build/lib");
    println!("cargo:rustc-link-search=native={}", dir + "/../native");
    // Add -ldpdk
    //println!("cargo:rustc-link-lib=dylib=dpdk");
    //for lib in libs {
        //println!("Linking with {}", lib);
        //println!("cargo:rustc-link-lib=static={}", lib);
    //}
}
