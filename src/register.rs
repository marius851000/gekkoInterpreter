use crate::util::u16_get_section;
use crate::BASE_RW_ADRESS;

pub struct GekkoRegister {
    // general purpose register
    gpr: [u32; 32],

    // fpr, can be accessed as a pair of f32
    // in this case, ps0 is first and ps1 is second
    // (didn't really understood how this really work. Let just copy dolphin on how this is stored)
    fpr: [[f64; 2]; 32],

    // program counter (position of the cursor in the code)
    pub pc: u32,

    // link register
    pub lr: u32,

    pub xer: u32,

    // field of a condition register. The data are the first four bit to the left, like 0, 0, 0, 0, LT, GT, EQ, SO
    pub cr: [u8; 8],

    pub ctr: u32,

    // graphical quantification register
    pub qr: [u32; 8],
}

impl Default for GekkoRegister {
    fn default() -> Self {
        Self {
            gpr: [0; 32],
            fpr: [[0.0; 2]; 32],
            pc: BASE_RW_ADRESS,
            lr: 0,
            xer: 0,
            cr: [0; 8],
            ctr: 0,
            qr: [0; 8],
        }
    }
}

impl GekkoRegister {
    #[inline]
    pub fn get_gpr(&self, nb: u8) -> u32 {
        //println!("read 0x{:x} from gpr {}", self.gpr[nb as usize], nb);
        self.gpr[nb as usize]
    }

    #[inline]
    pub fn set_gpr(&mut self, nb: u8, data: u32) {
        //DEBUG:
        //println!("wrote 0x{:x} to gpr {}", data, nb);
        self.gpr[nb as usize] = data;
    }

    #[inline]
    pub fn set_fpr_ps0(&mut self, nb: u8, value: f64) {
        println!("set ps0 of fpr {} to {}", nb, value);
        self.fpr[nb as usize][0] = value
    }

    #[inline]
    pub fn set_fpr_ps1(&mut self, nb: u8, value: f64) {
        println!("set ps1 of fpr {} to {}", nb, value);
        self.fpr[nb as usize][1] = value
    }

    #[inline]
    pub fn set_fpr_both(&mut self, nb: u8, value: f64) {
        println!("set ps0 and ps1 of fpr {} to {}", nb, value);
        self.fpr[nb as usize][0] = value;
        self.fpr[nb as usize][1] = value;
    }

    #[inline]
    pub fn get_fpr_ps0(&mut self, nb: u8) -> f64 {
        self.fpr[nb as usize][0]
    }

    #[inline]
    pub fn get_fpr_ps1(&mut self, nb: u8) -> f64 {
        self.fpr[nb as usize][1]
    }

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
    pub fn update_cr1_f64(&mut self, _value: f64) {
        todo!("update_cr1 is not yet implemented");
    }

    #[inline]
    pub fn update_cr1_f32(&mut self, _value: f32) {
        todo!("update_cr1 is not yet implemented");
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
    pub fn set_bit_cr(&mut self, cr_bit: usize, value: bool) {
        let cr_value = &mut self.cr[cr_bit / 4];
        let bit_number = cr_bit % 4;
        *cr_value &= 0b1110111 >> bit_number;
        *cr_value |= (if value { 1 } else { 0 }) << (3 - bit_number);
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
            self.get_gpr(gpr_a) as i64
        }) + (d as i64)) as u32
    }

    #[inline]
    pub fn get_spr(&self, spr: Spr) -> u32 {
        match spr {
            Spr::LR => self.lr,
            spr => todo!("getting the spr {:?}", spr),
        }
    }

    #[inline]
    pub fn set_spr(&mut self, spr: Spr, value: u32) {
        match spr {
            Spr::LR => self.lr = value,
            Spr::CTR => self.ctr = value,
            spr => todo!("setting the spr {:?}", spr),
        }
    }

    #[inline]
    pub fn compute_address_based_on_pair_of_register(&self, gpr_a: u8, gpr_b: u8) -> u32 {
        (if gpr_a == 0 { 0 } else { self.get_gpr(gpr_a) }) + self.get_gpr(gpr_b)
    }

    #[inline]
    pub fn set_carry(&mut self, value: bool) {
        self.xer &= 0b11011111_11111111_11111111_11111111;
        self.xer |= (value as u32) << 29;
    }

    #[inline]
    pub fn get_carry(&self) -> bool {
        (self.xer & (1 << 29)) != 0
    }

    #[inline]
    pub fn get_qr(&self, id: u8) -> u32 {
        self.qr[id as usize]
    }
}

#[derive(Debug, PartialEq)]
pub enum Tbr {
    Tbl,
    Tbu,
}

impl Tbr {
    #[inline]
    pub fn decode_from_mftb(data: u16) -> Tbr {
        debug_assert_eq!(u16_get_section(data, 16 - 5, 5), 0b01000);
        match data >> 5 {
            0b01100 => Tbr::Tbl,
            0b01101 => Tbr::Tbu,
            _ => panic!(),
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
        debug_assert_eq!(data << (16 - 5), 0);
        match data >> 5 {
            0b00001 => Spr::XER,
            0b01000 => Spr::LR,
            0b01001 => Spr::CTR,
            _ => panic!("unknown SPR for mfspr"),
        }
    }
}
