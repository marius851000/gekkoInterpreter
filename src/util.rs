#[inline]
pub fn get_bit_section(data: u32, start: usize, len: usize) -> u32 {
    (data << start) >> (32 - len)
}

#[inline]
pub fn get_bit_value(data: u32, position: usize) -> bool {
    (data >> (31-position)) & 1== 1
}

#[test]
fn test_get_bit() {
    assert_eq!(
        get_bit_section(0b00001111_10000000_00000000_00000000, 4, 5),
        0x0000001F
    );
    assert_eq!(get_bit_value(0xFF7FFFFF, 8), false);
    assert_eq!(get_bit_value(0x00010000, 15), true);
}
