#![feature(cfg_target_feature)]
#![feature(box_syntax)]

#[cfg(any(target_feature="avx"))]
fn test_comp() {
    println!("Found avx")
}

#[cfg(not(any(target_feature="avx")))]
fn test_comp() {
    println!("Did not find avx")
}

fn main() {
    test_comp();
    let f = box |x| { x + 5 };
    println!("Value {}", f(22));
}
