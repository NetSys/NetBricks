This needs a nightly rust and a modified Cargo (available 
[here](https://github.com/apanda/cargo)). The Cargo modifications mostly enable 
the use of SIMD in Rust.

To build:

-	First run build.sh from the root directory. This builds and sets up
	native support libraries.
-	Go to framework and run "cargo build --release"
