#![feature(asm)]
#[inline]
fn rdtscp_unsafe() -> u64 {
    let high: u32;
    let low: u32;
    let aux: u32;
    unsafe {
        asm!("rdtscp"
             : "={eax}" (low), "={edx}" (high)
             :
             : "ecx"
             : "volatile");
        let ret = ((high as u64) << 32) | (low as u64);
        ret
    }
}

#[inline]
fn rdtsc_unsafe() -> u64 {
    unsafe {
        let low: u32;
        let high: u32;
        asm!("rdtsc"
             : "={eax}" (low), "={edx}" (high)
             :
             : "rdx rax"
             : "volatile");
        ((high as u64) << 32) | (low as u64)
    }
}

#[inline]
fn cpuid() {
    unsafe {
        asm!("movl $$0x2, %eax":::"eax":"volatile");
        asm!("movl $$0x0, %ecx":::"ecx":"volatile");
        asm!("cpuid"
             :
             :
             : "rax", "rbx", "rcx", "rdx"
             : "volatile");
    }
}

fn main() {
    let mut a = 0;
    let mut b = 0;
    let mut c = 0;
    let mut d = 0;
    let mut e = 0;
    let mut e = 0;
    let mut f = 0;
    let mut g = 0;
    let mut h = 0;
    let mut i = 0;
    loop {
        a = rdtscp_unsafe();
        while b < a + 750 {
            b = rdtscp_unsafe();
        }
        while c < b + (b - a) {
            c = rdtscp_unsafe();
        }
        d += 1;
        e += b * 2;
        cpuid();
        g = rdtscp_unsafe();
        cpuid();
        h = g + 2;
        i = a + g;
        println!("a {} b {} c {} d {} e {} f {} g {} h {} i {}", a, b, c, d, e, f, g, h, i);
    }
}
