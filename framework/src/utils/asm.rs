#[inline]
pub fn cpuid() {
    unsafe {
        asm!("movl $$0x2, %eax":::"eax");
        asm!("movl $$0x0, %ecx":::"ecx");
        asm!("cpuid"
             :
             :
             : "rax rbx rcx rdx");
    }
}

#[inline]
pub fn rdtsc_unsafe() -> u64 {
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
pub fn rdtscp_unsafe() -> u64 {
    // Doing the equivalent of a rdtscp manually, for some reason rdtscp is causing a
    let high: u32;
    let low: u32;
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
pub fn pause() {
    unsafe {
        asm!("pause"::::"volatile");
    }
}
