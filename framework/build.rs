use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

#[allow(dead_code)]
fn parse_ld_archive(ar: &Path) -> Vec<String> {
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

#[allow(dead_code)]
fn write_external_link(libs: &Vec<String>) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("linkage.rs");
    let mut f = File::create(&dest).unwrap();
    for l in libs {
        let link_str = format!("#[link(name=\"{}\", kind=\"static\")]", l);
        let overall = link_str + "\nextern \"C\" {}\n";
        f.write_all(&overall.into_bytes()).unwrap();
    }
}

/// Cargo runs main in this file to get some additional settings (e.g., LD_LIBRARY_PATH). It reads the printed output
/// looking for certain variables, see [here](http://doc.crates.io/build-script.html) for documentation.
fn main() {
    // Get the directory where we are building.
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dpdk_path = Path::new(&dir).parent().unwrap()
                                   .join("3rdparty")
                                   .join("dpdk")
                                   .join("build")
                                   .join("lib");
    let native_path = Path::new(&dir).parent().unwrap()
                                     .join("target")
                                     .join("native");
    //println!("DPDK {:?}", dpdk_path.to_str());
    // Send current directory as -L
    println!("cargo:rustc-link-search=native={}", dpdk_path.to_str().unwrap());
    if dpdk_path.join("libdpdk.so").exists() {
        println!("cargo:rustc-link-lib=dpdk");
    }
    println!("cargo:rustc-link-search=native={}", native_path.to_str().unwrap());
}
