use crate::GekkoRegister;
use crate::Instruction;
use crate::Spr;
use crate::BASE_RW_ADRESS;

#[derive(Debug, PartialEq)]
pub enum BreakData {
    None,
    Break,
}

pub struct GekkoInterpreter {
    pub ram: Vec<u8>,
    pub register: GekkoRegister,
}

impl GekkoInterpreter {
    pub fn new(ram_amount: usize) -> GekkoInterpreter {
        GekkoInterpreter {
            ram: vec![0; ram_amount],
            register: GekkoRegister::default(),
        }
    }

    pub fn reboot(&mut self) {
        self.ram = vec![0; self.ram.len()];
        self.register = GekkoRegister::default();
    }

    pub fn step(&mut self) -> Result<BreakData, String> {
        // first, get the instruction
        let instruction = Instruction::decode_instruction(self.read_u32(self.register.pc)).unwrap();
        // second, run it
        let mut break_data = BreakData::None;
        //println!("{:?}", &instruction);
        match instruction {
            Instruction::Addx(gpr_dest, gpr_1, gpr_2, oe, rc) => {
                let (result, overflow) = self.register.gpr[gpr_1 as usize]
                    .overflowing_add(self.register.gpr[gpr_2 as usize]);
                self.register.gpr[gpr_dest as usize] = result;
                if oe {
                    self.register.setxer_ov_so(overflow);
                };
                if rc {
                    self.register.update_cr0(result);
                };
                self.register.increment_pc();
            }
            Instruction::Stwu(gpr_s, gpr_a, d) => {
                let address = ((self.register.gpr[gpr_a as usize] as i64) + (d as i64)) as u32;
                self.write_u32(address, self.register.gpr[gpr_s as usize]);
                self.register.gpr[gpr_a as usize] = address;
                self.register.increment_pc();
            }
            Instruction::Mfspr(gpr_d, spr) => {
                self.register.gpr[gpr_d as usize] = match spr {
                    Spr::LR => self.register.lr,
                    x => panic!("mfspr: unimplemented for the LR {:?}", x),
                };
                self.register.increment_pc();
            }
            Instruction::Cmpli(crf_d, l, gpr_a, uimm) => {
                assert_eq!(l, false);
                let a = self.register.gpr[gpr_a as usize];
                let b = uimm as u32;
                let f = if a < b {
                    0x8
                } else if a > b {
                    0x4
                } else {
                    0x2
                } | (self.register.get_xer_so() as u8);

                self.register.cr[crf_d as usize] = f;

                self.register.increment_pc();
            }
            Instruction::Stw(r_s, r_a, d) => {
                let address = ((if r_a == 0 {
                    0
                } else {
                    self.register.gpr[r_a as usize]
                } as i64)
                    + (d as i64)) as u32;
                self.write_u32(address, self.register.gpr[r_s as usize]);

                self.register.increment_pc();
            }
            Instruction::Stmw(mut gpr_s, gpr_a, d) => {
                let mut address = (if gpr_a == 0 {
                    0
                } else {
                    self.register.gpr[gpr_a as usize] as i64
                } + d as i64) as u32;
                while gpr_s < 32 {
                    self.write_u32(address, self.register.gpr[gpr_s as usize]);
                    gpr_s += 1;
                    address += 4;
                }
                self.register.increment_pc();
            }
            Instruction::Orx(gpr_s, gpr_a, gpr_b, rc) => {
                self.register.gpr[gpr_a as usize] =
                    self.register.gpr[gpr_s as usize] | self.register.gpr[gpr_b as usize];
                if rc {
                    panic!("orx: rc not implemented");
                };
                self.register.increment_pc();
            }
            Instruction::CustomBreak => {
                break_data = BreakData::Break;
                self.register.increment_pc();
            }
        };
        Ok(break_data)
    }

    pub fn run_until_event(&mut self) -> BreakData {
        loop {
            match self.step().unwrap() {
                BreakData::None => continue,
                x => return x,
            }
        }
    }

    pub fn get_ram(&self) -> &Vec<u8> {
        &self.ram
    }

    pub fn write_u32(&mut self, mut offset: u32, data: u32) {
        offset -= BASE_RW_ADRESS;
        for d in &data.to_be_bytes() {
            self.ram[offset as usize] = *d;
            offset += 1;
        }
    }

    pub fn read_u32(&mut self, mut offset: u32) -> u32 {
        offset -= BASE_RW_ADRESS;
        let mut buffer = [0; 4];
        for d in &mut buffer {
            *d = self.ram[offset as usize];
            offset += 1;
        }
        u32::from_be_bytes(buffer)
    }
}

#[test]
fn test_read_write_ram() {
    let mut gekko = GekkoInterpreter::new(10);
    assert_eq!(gekko.read_u32(BASE_RW_ADRESS + 0), 0);
    gekko.write_u32(BASE_RW_ADRESS + 4, 0x0000FFFF);
    assert_eq!(gekko.read_u32(BASE_RW_ADRESS + 4), 0x0000FFFF);
    assert_eq!(gekko.read_u32(BASE_RW_ADRESS + 6), 0xFFFF0000);
}

#[test]
fn test_reboot() {
    let mut gekko = GekkoInterpreter::new(4);
    gekko.write_u32(BASE_RW_ADRESS + 0, 1);
    gekko.register.gpr[10] = 10;
    gekko.register.pc = 4;
    gekko.reboot();
    let mut gekko_base = GekkoInterpreter::new(4);
    assert_eq!(
        gekko.read_u32(BASE_RW_ADRESS + 0),
        gekko_base.read_u32(BASE_RW_ADRESS)
    );
    assert_eq!(gekko.register.gpr[10], gekko_base.register.gpr[10]);
    assert_eq!(gekko.register.pc, gekko_base.register.pc);
}
#[test]
fn test_addx() {
    let mut gekko = GekkoInterpreter::new(4);
    // test "add r0, r1, r2"
    gekko.register.gpr[1] = 100;
    gekko.register.gpr[2] = 2510;
    gekko.write_u32(BASE_RW_ADRESS + 0, 0b011111_00000_00001_00010_0_100001010_0);
    gekko.step().unwrap();
    assert_eq!(gekko.register.gpr[0], 100 + 2510);
    gekko.reboot();

    // test "addo r0, r1, r2"
    gekko.register.gpr[1] = u32::MAX - 10;
    gekko.register.gpr[2] = 100;
    gekko.write_u32(BASE_RW_ADRESS + 0, 0b011111_00000_00001_00010_1_100001010_0);
    gekko.step().unwrap();
    assert_eq!(gekko.register.gpr[0], (u32::MAX - 10).wrapping_add(100));
    gekko.reboot();

    //TODO: test cr0
}

#[test]
fn test_stwu() {
    let mut gekko = GekkoInterpreter::new(30);
    // test "stwu r1, -8(r2)"
    gekko.write_u32(BASE_RW_ADRESS + 0, 0b100101_00001_00010_1111_1111_1111_1000);
    gekko.register.gpr[1] = 35;
    gekko.register.gpr[2] = BASE_RW_ADRESS + 10 + 8;
    gekko.step().unwrap();
    assert_eq!(gekko.read_u32(BASE_RW_ADRESS + 10), 35);
    assert_eq!(gekko.register.gpr[2], BASE_RW_ADRESS + 10);
}

#[test]
fn test_mfspr() {
    let mut gekko = GekkoInterpreter::new(4);
    // test "mfspr r0, LR"
    gekko.write_u32(BASE_RW_ADRESS, 0x7C0802A6);
    gekko.register.lr = 123;
    gekko.step().unwrap();
    assert_eq!(gekko.register.gpr[0], 123);
}

#[test]
fn test_cmpli() {
    let mut gekko = GekkoInterpreter::new(4);
    //test "cmpli crf5, 0, r4, 5"
    gekko.write_u32(BASE_RW_ADRESS, 0b001010_101_0_0_00101_00000000_00000101);
    gekko.register.gpr[4] = 35;
    gekko.register.setxer_ov_so(true);
    gekko.register.setxer_ov_so(false);
    gekko.step().unwrap();
    assert_eq!(gekko.register.cr[5], 0b1001);
}

#[test]
fn test_stw() {
    let mut gekko = GekkoInterpreter::new(12);
    //test "stw r1, -4(r2)"
    gekko.write_u32(BASE_RW_ADRESS, 0b100100_00001_00010_1111_1111_1111_1100);
    gekko.register.gpr[2] = BASE_RW_ADRESS + 12;
    gekko.register.gpr[1] = 300;
    gekko.step().unwrap();
    assert_eq!(gekko.read_u32(BASE_RW_ADRESS + 8), 300);
}

#[test]
fn test_stmw() {
    let mut gekko = GekkoInterpreter::new(30);
    //test "stmw r29, -4(r3)"
    gekko.write_u32(BASE_RW_ADRESS, 0b101111_11101_00011_1111_1111_1111_1100);
    gekko.register.gpr[3] = BASE_RW_ADRESS + 12;
    gekko.register.gpr[29] = 20;
    gekko.register.gpr[30] = 30;
    gekko.register.gpr[31] = 50;
    gekko.step().unwrap();
    assert_eq!(gekko.read_u32(BASE_RW_ADRESS + 8), 20);
    assert_eq!(gekko.read_u32(BASE_RW_ADRESS + 12), 30);
    assert_eq!(gekko.read_u32(BASE_RW_ADRESS + 16), 50);
}

#[test]
fn test_orx() {
    let mut gekko = GekkoInterpreter::new(4);
    //test "or r1, r2, r3"
    gekko.write_u32(BASE_RW_ADRESS, 0b011111_00010_00001_00011_0110111100_0);
    gekko.register.gpr[3] = 0x000000FC;
    gekko.register.gpr[2] = 0x0000000F;
    gekko.step().unwrap();
    assert_eq!(gekko.register.gpr[1], 0x000000FF);
    //TODO: or. (Rc = 1)
}
