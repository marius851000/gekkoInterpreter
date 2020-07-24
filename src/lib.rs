mod interpreter;
pub use interpreter::{BreakData, GekkoInterpreter};

mod register;
pub use register::{GekkoRegister, Spr};

mod instruction;
pub use instruction::Instruction;

pub mod util;

#[allow(clippy::inconsistent_digit_grouping)]
pub const OPCODE_BREAK: u32 = 0b111011_00_00000000_00000000_00000000;

pub const BASE_RW_ADRESS: u32 = 0x80003100;

#[cfg(test)]
mod tests {
    mod test_interpreter;

    #[test]
    fn test_break() {
        use crate::BreakData;
        use crate::GekkoInterpreter;
        use crate::BASE_RW_ADRESS;
        use crate::OPCODE_BREAK;
        let mut cpu = GekkoInterpreter::new(12);
        cpu.write_u32(BASE_RW_ADRESS, 0b11111_00010_00011_00100_0_100001010_0); // r2 = r3 + r4
        cpu.write_u32(BASE_RW_ADRESS + 4, OPCODE_BREAK); // custom break
        cpu.register.set_gpr(3, 10);
        cpu.register.set_gpr(4, 15);
        assert_eq!(cpu.run_until_event(), BreakData::Break);
        assert_eq!(cpu.register.get_gpr(2), 10 + 15);
    }
}
