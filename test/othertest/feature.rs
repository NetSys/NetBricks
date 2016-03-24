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
    //test_comp();
    //let f = box |x| { x + 5 };
    //println!("Value {}", f(22));
    let x = vec![0, 1, 2, 3, 4];
    let mut y = x.iter_mut().cycle();
    for c in 1..20 {
        println!("c {} iter {}", c, y.next().expect("Inf"));
    }
}
