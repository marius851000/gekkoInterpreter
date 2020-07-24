use crate::BASE_RW_ADRESS;

pub struct GekkoRegister {
    // general purpose register
    pub gpr: [u32; 32],

    // program counter (position of the cursor in the code)
    pub pc: u32,

    // link register
    pub lr: u32,

    pub xer: u32,

    // field of a condition register. The data are the first four bit to the left, like 0, 0, 0, 0, LT, GT, EQ, SO
    pub cr: [u8; 8],

    pub ctr: u32,
}

impl GekkoRegister {
    #[inline]
    pub fn setxer_ov_so(&mut self, value: bool) {
        self.xer = (self.xer & 0xBFFFFFFF) | ((value as u32) << 30);
        self.xer |= (value as u32) << 31;
    }

    #[inline]
    pub fn get_xer_so(&mut self) -> bool {
        (self.xer >> 31) != 0
    }

    #[inline]
    pub fn update_cr0(&mut self, value: u32) {
        let value = value as i32;
        self.cr[0] = if value < 0 {
            0x8
        } else if value > 0 {
            0x4
        } else {
            0x2
        } | (self.get_xer_so() as u8);
    }

    #[inline]
    pub fn increment_pc(&mut self) {
        self.pc += 4;
    }

    #[inline]
    pub fn get_bit_cr(&self, cr_bit: usize) -> bool {
        (self.cr[cr_bit / 4] >> (3 - (cr_bit % 4))) & 1 == 1
    }

    #[inline]
    pub fn decrement_ctr(&mut self) {
        self.ctr = self.ctr.wrapping_sub(1);
    }

    #[inline]
    pub fn compute_address_based_on_register(&self, gpr_a: u8, d: i16) -> u32 {
        ((if gpr_a == 0 {
            0
        } else {
            self.gpr[gpr_a as usize] as i64
        }) + (d as i64)) as u32
    }
}

impl Default for GekkoRegister {
    fn default() -> Self {
        Self {
            gpr: [0; 32],
            pc: BASE_RW_ADRESS,
            lr: 0,
            xer: 0,
            cr: [0; 8],
            ctr: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Spr {
    XER,
    LR,
    CTR,
}

impl Spr {
    #[inline]
    pub fn decode_from_mfspr(data: u16) -> Spr {
        debug_assert_eq!(data << (15 - 5), 0);
        match data >> 5 {
            0b00001 => Spr::XER,
            0b01000 => Spr::LR,
            0b01001 => Spr::CTR,
            _ => panic!("unknown SPR for mfspr"),
        }
    }
}
