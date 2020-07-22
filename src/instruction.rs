use crate::get_bit_section;
use crate::get_bit_value;
use crate::Spr;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Addx(u8, u8, u8, bool, bool), // D, A, B, OE, Rc
    Stwu(u8, u8, i16),            // S, A, d
    Mfspr(u8, Spr),               // D, spr
    Cmpli(u8, bool, u8, u16),     //crfD, L, rA, UIMM
    Stw(u8, u8, i16),             //rS, rA, d
    CustomBreak,
}

impl Instruction {
    pub fn decode_instruction(opcode: u32) -> Option<Instruction> {
        let primary_opcode = opcode >> (31 - 5);
        Some(match primary_opcode {
            10 => {
                debug_assert_eq!(get_bit_value(opcode, 9), false);
                Instruction::Cmpli(
                    get_bit_section(opcode, 6, 3) as u8,
                    get_bit_value(opcode, 10),
                    get_bit_section(opcode, 11, 5) as u8,
                    get_bit_section(opcode, 16, 16) as u16,
                )
            }
            31 => {
                let extended_opcode = get_bit_section(opcode, 22, 9);
                match extended_opcode {
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
                    _ => return None,
                }
            }
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
            0b111011 => {
                let extended_opcode = get_bit_section(opcode, 26, 5);
                match extended_opcode {
                    // custom instruction
                    0b00000 => Instruction::CustomBreak,
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
