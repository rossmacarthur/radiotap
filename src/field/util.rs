#[inline]
pub fn flag_is_set(number: u64, flag: u64) -> bool {
    number & flag > 0
}

#[inline]
pub fn bit_is_set(number: u64, bit: u8) -> bool {
    flag_is_set(number, 1 << bit)
}

#[inline]
pub fn bits_as_int(number: u64, bit: u64, count: u64) -> u64 {
    (number >> bit) & ((1 << count) - 1)
}
