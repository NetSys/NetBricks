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
        (u64::from(high) << 32) | u64::from(low)
    }
}

#[inline]
pub fn rdtscp_unsafe() -> u64 {
    let high: u32;
    let low: u32;
    unsafe {
        asm!("rdtscp"
             : "={eax}" (low), "={edx}" (high)
             :
             : "ecx"
             : "volatile");
        (u64::from(high) << 32) | u64::from(low)
    }
}

#[inline]
pub fn pause() {
    unsafe {
        asm!("pause"::::"volatile");
    }
}
