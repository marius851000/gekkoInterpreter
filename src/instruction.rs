use crate::util::{extend_sign_16, extend_sign_32, get_bit_section, get_bit_value};
use crate::{Spr, Tbr};

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Addx(u8, u8, u8, bool, bool),      //rD, rA, rB, OE, Rc
    Stwu(u8, u8, i16),                 //rS, rA, d
    Mfspr(u8, Spr),                    //rD, spr
    Cmpli(u8, u8, u16),                //crfD, L, rA, UIMM
    Cmpi(u8, u8, i16),                 //crfD, L, rA, UIMM
    Cmp(u8, u8, u8),                   //crfD, L, rA, UIMM
    Stw(u8, u8, i16),                  //rS, rA, d
    Stmw(u8, u8, i16),                 //rS, rA, d
    Orx(u8, u8, u8, bool),             //rS, rA, rB, Rc
    Bcx(u8, u8, i16, bool, bool),      //BO, BI, BD, AA, LK
    Rlwinmx(u8, u8, u8, u8, u8, bool), //rS, rA, SH, MB, ME, Rc
    Lwz(u8, u8, i16),                  //rD, rA, d
    Stb(u8, u8, i16),                  //rS, rA, d
    Stbu(u8, u8, i16),                 //rS, rA, d
    Addis(u8, u8, u16),                //rD, rA, SIMM
    Addi(u8, u8, i16),                 //rD, rA, SIMM
    Bx(i32, bool, bool),               //LI, AA, LK
    Lbz(u8, u8, i16),                  //rD, rA, d
    Extsbx(u8, u8, bool),              //rS, rA, Rc
    Lwzx(u8, u8, u8),                  //rD, rA, rB
    Lmw(u8, u8, i16),                  //rD, rA, d
    Mtspr(u8, Spr),                    //rS, Spr
    Bclrx(u8, u8, bool),               //BO, BI, LK
    Stwx(u8, u8, u8),                  //rS, rA, rB
    Ori(u8, u8, u16),                  //rA, rS, UUIM
    Cmpl(u8, u8, u8),                  //crfD, rA, rB
    Nor(u8, u8, u8, bool),             //rS, rA, rB, Rc
    Addicdot(u8, u8, i16),             //rD, rA, simm
    Mftb(u8, Tbr),                     //rD, tbr
    Addcx(u8, u8, u8, bool, bool),     //rD, rA, rB, OE, Rc
    Addex(u8, u8, u8, bool, bool),     //rD, rA, rB, OE, Rc
    Lhz(u8, u8, i16),                  //rD, rA, d
    Andidot(u8, u8, u16),              //rD, rA, d
    Subfx(u8, u8, u8, bool, bool),     //rD, rA, rB, OE, Rc
    Crxor(u8, u8, u8),                 //crbD, crbA, crbB
    Lbzu(u8, u8, i16),                 //rD, rA, d
    Lfd(u8, u8, i16),                  //frD, rA, d
    Frsqrtex(u8, u8, bool),            //frD, frB, Rc
    Fmulx(u8, u8, u8, bool),           //frD, frA, frC, Rc
    Fnmsubx(u8, u8, u8, u8, bool),     //frD, frA, frB, frC, Rc
    Frspx(u8, u8, bool),               //frD, frB, Rc
    Stfs(u8, u8, i16),                 //frD, rA, d
    Lfs(u8, u8, i16),
    CustomBreak,
}

impl Instruction {
    pub fn decode_instruction(opcode: u32) -> Option<Instruction> {
        let primary_opcode = opcode >> (31 - 5);
        Some(match primary_opcode {
            10 => {
                debug_assert_eq!(get_bit_value(opcode, 9), false);
                debug_assert_eq!(get_bit_value(opcode, 10), false);
                Instruction::Cmpli(
                    get_bit_section(opcode, 6, 3) as u8,
                    get_bit_section(opcode, 11, 5) as u8,
                    get_bit_section(opcode, 16, 16) as u16,
                )
            }
            11 => {
                debug_assert_eq!(get_bit_value(opcode, 9), false);
                debug_assert_eq!(get_bit_value(opcode, 10), false);
                Instruction::Cmpi(
                    get_bit_section(opcode, 6, 3) as u8,
                    get_bit_section(opcode, 11, 5) as u8,
                    get_bit_section(opcode, 16, 16) as i16,
                )
            }
            13 => Instruction::Addicdot(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            14 => Instruction::Addi(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            15 => Instruction::Addis(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as u16,
            ),
            16 => Instruction::Bcx(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                extend_sign_16(get_bit_section(opcode, 16, 14) as u16, 14),
                get_bit_value(opcode, 30),
                get_bit_value(opcode, 31),
            ),
            19 => {
                let secondary_opcode = get_bit_section(opcode, 21, 10);
                match secondary_opcode {
                    16 => {
                        debug_assert_eq!(get_bit_section(opcode, 16, 5), 0);
                        Instruction::Bclrx(
                            get_bit_section(opcode, 6, 5) as u8,
                            get_bit_section(opcode, 11, 5) as u8,
                            get_bit_value(opcode, 31),
                        )
                    }
                    193 => {
                        debug_assert_eq!(get_bit_value(opcode, 31), false);
                        Instruction::Crxor(
                            get_bit_section(opcode, 6, 5) as u8,
                            get_bit_section(opcode, 11, 5) as u8,
                            get_bit_section(opcode, 16, 5) as u8,
                        )
                    }
                    _ => return None,
                }
            }
            18 => Instruction::Bx(
                extend_sign_32(get_bit_section(opcode, 6, 24), 24),
                get_bit_value(opcode, 30),
                get_bit_value(opcode, 31),
            ),
            21 => Instruction::Rlwinmx(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 5) as u8,
                get_bit_section(opcode, 21, 5) as u8,
                get_bit_section(opcode, 26, 5) as u8,
                get_bit_value(opcode, 31),
            ),
            24 => Instruction::Ori(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as u16,
            ),
            28 => Instruction::Andidot(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as u16,
            ),
            31 => {
                let extended_opcode = get_bit_section(opcode, 22, 9);
                match extended_opcode {
                    0 => {
                        debug_assert_eq!(get_bit_value(opcode, 21), false);
                        debug_assert_eq!(get_bit_value(opcode, 31), false);
                        debug_assert_eq!(get_bit_value(opcode, 9), false);
                        debug_assert_eq!(get_bit_value(opcode, 10), false);
                        Instruction::Cmp(
                            get_bit_section(opcode, 6, 3) as u8,
                            get_bit_section(opcode, 11, 5) as u8,
                            get_bit_section(opcode, 16, 5) as u8,
                        )
                    }
                    10 => Instruction::Addcx(
                        get_bit_section(opcode, 6, 5) as u8,
                        get_bit_section(opcode, 11, 5) as u8,
                        get_bit_section(opcode, 16, 5) as u8,
                        get_bit_value(opcode, 21),
                        get_bit_value(opcode, 31),
                    ),
                    23 => {
                        debug_assert_eq!(get_bit_value(opcode, 21), false);
                        debug_assert_eq!(get_bit_value(opcode, 31), false);
                        Instruction::Lwzx(
                            get_bit_section(opcode, 6, 5) as u8,
                            get_bit_section(opcode, 11, 5) as u8,
                            get_bit_section(opcode, 16, 5) as u8,
                        )
                    }
                    32 => {
                        debug_assert_eq!(get_bit_value(opcode, 21), false);
                        debug_assert_eq!(get_bit_value(opcode, 31), false);
                        debug_assert_eq!(get_bit_value(opcode, 10), false);
                        Instruction::Cmpl(
                            get_bit_section(opcode, 6, 3) as u8,
                            get_bit_section(opcode, 11, 5) as u8,
                            get_bit_section(opcode, 16, 5) as u8,
                        )
                    }
                    40 => Instruction::Subfx(
                        get_bit_section(opcode, 6, 5) as u8,
                        get_bit_section(opcode, 11, 5) as u8,
                        get_bit_section(opcode, 16, 5) as u8,
                        get_bit_value(opcode, 21),
                        get_bit_value(opcode, 31),
                    ),
                    124 => {
                        debug_assert_eq!(get_bit_value(opcode, 21), false);
                        Instruction::Nor(
                            get_bit_section(opcode, 6, 5) as u8,
                            get_bit_section(opcode, 11, 5) as u8,
                            get_bit_section(opcode, 16, 5) as u8,
                            get_bit_value(opcode, 31),
                        )
                    }
                    138 => Instruction::Addex(
                        get_bit_section(opcode, 6, 5) as u8,
                        get_bit_section(opcode, 11, 5) as u8,
                        get_bit_section(opcode, 16, 5) as u8,
                        get_bit_value(opcode, 21),
                        get_bit_value(opcode, 31),
                    ),
                    151 => {
                        debug_assert_eq!(get_bit_value(opcode, 21), false);
                        debug_assert_eq!(get_bit_value(opcode, 31), false);
                        Instruction::Stwx(
                            get_bit_section(opcode, 6, 5) as u8,
                            get_bit_section(opcode, 11, 5) as u8,
                            get_bit_section(opcode, 16, 5) as u8,
                        )
                    }
                    266 => Instruction::Addx(
                        get_bit_section(opcode, 6, 5) as u8,
                        get_bit_section(opcode, 11, 5) as u8,
                        get_bit_section(opcode, 16, 5) as u8,
                        get_bit_value(opcode, 21),
                        get_bit_value(opcode, 31),
                    ),
                    339 => {
                        debug_assert_eq!(get_bit_value(opcode, 31), false);
                        Instruction::Mfspr(
                            get_bit_section(opcode, 6, 5) as u8,
                            Spr::decode_from_mfspr(get_bit_section(opcode, 11, 10) as u16),
                        )
                    }
                    371 => {
                        debug_assert_eq!(get_bit_value(opcode, 31), false);
                        debug_assert_eq!(get_bit_value(opcode, 21), false);
                        Instruction::Mftb(
                            get_bit_section(opcode, 6, 5) as u8,
                            Tbr::decode_from_mftb(get_bit_section(opcode, 11, 10) as u16),
                        )
                    }
                    442 => {
                        debug_assert_eq!(get_bit_section(opcode, 16, 6), 0b000001);
                        Instruction::Extsbx(
                            get_bit_section(opcode, 6, 5) as u8,
                            get_bit_section(opcode, 11, 5) as u8,
                            get_bit_value(opcode, 31),
                        )
                    }
                    444 => {
                        debug_assert_eq!(get_bit_value(opcode, 21), false);
                        Instruction::Orx(
                            get_bit_section(opcode, 6, 5) as u8,
                            get_bit_section(opcode, 11, 5) as u8,
                            get_bit_section(opcode, 16, 5) as u8,
                            get_bit_value(opcode, 31),
                        )
                    }
                    467 => {
                        debug_assert_eq!(get_bit_value(opcode, 31), false);
                        Instruction::Mtspr(
                            get_bit_section(opcode, 6, 5) as u8,
                            Spr::decode_from_mfspr(get_bit_section(opcode, 11, 10) as u16),
                        )
                    }
                    _ => return None,
                }
            }
            32 => Instruction::Lwz(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            34 => Instruction::Lbz(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            35 => Instruction::Lbzu(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            36 => Instruction::Stw(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            37 => Instruction::Stwu(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            38 => Instruction::Stb(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            39 => Instruction::Stbu(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            40 => Instruction::Lhz(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            46 => Instruction::Lmw(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            47 => Instruction::Stmw(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            48 => Instruction::Lfs(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            50 => Instruction::Lfd(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            52 => Instruction::Stfs(
                get_bit_section(opcode, 6, 5) as u8,
                get_bit_section(opcode, 11, 5) as u8,
                get_bit_section(opcode, 16, 16) as i16,
            ),
            59 => {
                let extended_opcode = get_bit_section(opcode, 26, 5);
                match extended_opcode {
                    // custom instruction
                    0b00000 => Instruction::CustomBreak,
                    _ => return None,
                }
            }
            63 => {
                let upper_extended_opcode = get_bit_section(opcode, 26, 5);
                match upper_extended_opcode {
                    12 => {
                        debug_assert_eq!(get_bit_section(opcode, 11, 5), 0);
                        Instruction::Frspx(
                            get_bit_section(opcode, 6, 5) as u8,
                            get_bit_section(opcode, 16, 5) as u8,
                            get_bit_value(opcode, 31),
                        )
                    }
                    25 => {
                        debug_assert_eq!(get_bit_section(opcode, 16, 5), 0);
                        Instruction::Fmulx(
                            get_bit_section(opcode, 6, 5) as u8,
                            get_bit_section(opcode, 11, 5) as u8,
                            get_bit_section(opcode, 21, 5) as u8,
                            get_bit_value(opcode, 31),
                        )
                    }
                    30 => Instruction::Fnmsubx(
                        get_bit_section(opcode, 6, 5) as u8,
                        get_bit_section(opcode, 11, 5) as u8,
                        get_bit_section(opcode, 16, 5) as u8,
                        get_bit_section(opcode, 21, 5) as u8,
                        get_bit_value(opcode, 31),
                    ),
                    26 => {
                        debug_assert_eq!(get_bit_section(opcode, 11, 5), 0);
                        debug_assert_eq!(get_bit_section(opcode, 21, 5), 0);
                        Instruction::Frsqrtex(
                            get_bit_section(opcode, 6, 5) as u8,
                            get_bit_section(opcode, 16, 5) as u8,
                            get_bit_value(opcode, 31),
                        )
                    }
                    _ => return None,
                }
            }
            _ => return None,
        })
    }
}

#[test]
fn test_decode() {
    assert_eq!(
        Instruction::decode_instruction(0b011111_00010_00011_00100_0_100001010_1),
        Some(Instruction::Addx(2, 3, 4, false, true))
    );
}
