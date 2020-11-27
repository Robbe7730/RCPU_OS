pub unsafe fn memcpy(dest: *mut u8, src: *const u8, n: usize) {
    let mut i = 0;
    while i < n {
        *dest.offset(i as isize) = *src.offset(i as isize);
        i += 1;
    }
}

pub fn swap_endianness(value: u16) -> u16 {
    let b0 = value & 0x00ff;
    let b1 = (value & 0xff00) >> 8;
    b0 << 8 | b1
}
