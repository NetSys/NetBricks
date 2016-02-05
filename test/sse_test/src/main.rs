#![feature(cfg_target_feature)]
extern crate simd;
extern crate rand;
use rand::Rng;

//#[cfg(any(target_feature="avx"))]
//fn test_comp() {
    //println!("Found avx")
//}

//#[cfg(not(any(target_feature="avx")))]
//fn test_comp() {
    //println!("Did not find avx")
//}

fn main() {
   let mut x = simd::u32x4::new(1, 1, 1, 1); 
   let mut y = simd::u32x4::new(1, 1, 1, 1);
   let mut i = 0;
   let iter = rand::thread_rng().gen();
   while i < iter {
       let z = y + x;
       x = y;
       y = z;
       i = i + 1;
   }
   print!("Sum = {}", y.extract(2))
}
