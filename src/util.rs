#[inline]
pub fn get_bit_section(data: u32, start: usize, len: usize) -> u32 {
    (data << start) >> (32 - len)
}

#[inline]
pub fn u16_get_section(data: u16, start: usize, len: usize) -> u16 {
    (data << start) >> (16 - len)
}

#[inline]
pub fn get_bit_value(data: u32, position: usize) -> bool {
    (data >> (31 - position)) & 1 == 1
}

#[inline]
pub fn u8_get_bit(data: u8, position: usize) -> bool {
    (data >> (7 - position)) & 1 == 1
}

#[inline]
pub fn extend_sign_16(mut data: u16, position: usize) -> i16 {
    let is_negative = (data >> (position - 1)) == 1;
    if is_negative {
        for nb in position..16 {
            data |= 1 << nb
        }
    };
    data as i16
}

#[inline]
pub fn extend_sign_32(mut data: u32, position: usize) -> i32 {
    let is_negative = (data >> (position - 1)) == 1;
    if is_negative {
        for nb in position..32 {
            data |= 1 << nb
        }
    };
    data as i32
}

#[inline]
pub fn make_rotation_mask(mb: u32, me: u32) -> u32 {
    let begin = 0xFFFFFFFF >> mb;
    let end = 0x7FFFFFFF >> me;
    let mask = begin ^ end;

    if me < mb {
        !mask
    } else {
        mask
    }
}

#[inline]
pub fn raw_u64_to_f64(value: u64) -> f64 {
    f64::from_ne_bytes(value.to_ne_bytes())
}

#[inline]
pub fn get_size_for_quantized_type(size: u8) -> u32 {
    match size {
        4 | 6 => 10,
        5 | 7 => 20,
        _ => 4,
    }
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

#[test]
fn test_extend_sign() {
    assert_eq!(
        extend_sign_16(0x4000, 15),
        i16::from_be_bytes((0xC000u16).to_be_bytes())
    );
    assert_eq!(
        extend_sign_16(0x0F0F, 12),
        i16::from_be_bytes((0xFF0Fu16).to_be_bytes())
    );
    assert_eq!(extend_sign_16(0x1F0F, 15), 0x1F0F);
}
