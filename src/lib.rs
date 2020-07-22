mod interpreter;
pub use interpreter::{BreakData, GekkoInterpreter};

mod register;
pub use register::{GekkoRegister, Spr};

mod instruction;
pub use instruction::Instruction;

mod util;
pub use util::{get_bit_section, get_bit_value};

#[allow(clippy::inconsistent_digit_grouping)]
pub const OPCODE_BREAK: u32 = 0b111011_00_00000000_00000000_00000000;

pub const BASE_RW_ADRESS: u32 = 0x80000000;

#[cfg(test)]
mod tests {
    #[test]
    fn test_break() {
        use crate::GekkoInterpreter;
        use crate::BreakData;
        use crate::OPCODE_BREAK;
        use crate::BASE_RW_ADRESS;
        let mut cpu = GekkoInterpreter::new(12);
        cpu.write_u32(BASE_RW_ADRESS, 0b11111_00010_00011_00100_0_100001010_0); // r2 = r3 + r4
        cpu.write_u32(BASE_RW_ADRESS+4, OPCODE_BREAK); // custom break
        cpu.register.gpr[3] = 10;
        cpu.register.gpr[4] = 15;
        assert_eq!(cpu.run_until_event(), BreakData::Break);
        assert_eq!(cpu.register.gpr[2], 10 + 15);

    }
}
