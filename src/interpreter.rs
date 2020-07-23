use crate::util::{make_rotation_mask, u8_get_bit};
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
            Instruction::Bcx(bo, bi, bd, aa, lk) => {
                let dont_use_ctr = u8_get_bit(bo, 7 - 2);
                if !dont_use_ctr {
                    self.register.decrement_ctr();
                };
                let ctr_diff_0 = u8_get_bit(bo, 7 - 1);
                let ctr_ok = dont_use_ctr | ((self.register.ctr != 0) ^ ctr_diff_0);
                let dont_use_cond = u8_get_bit(bo, 7 - 4);
                let cond_ok = dont_use_cond
                    | (self.register.get_bit_cr(bi as usize) == u8_get_bit(bo, 7 - 3));
                if ctr_ok & cond_ok {
                    if lk {
                        self.register.lr = self.register.pc + 4;
                    }
                    if aa {
                        self.register.pc = (bd as i32) as u32;
                    } else {
                        self.register.pc = ((self.register.pc as i64) + (bd as i64)) as u32
                    }
                } else {
                    self.register.increment_pc();
                }
            }
            Instruction::Rlwinmx(gpr_s, gpr_a, sh, mb, me, rc) => {
                let mask = make_rotation_mask(mb as u32, me as u32);
                self.register.gpr[gpr_a as usize] =
                    self.register.gpr[gpr_s as usize].rotate_left(sh as u32) & mask;
                if rc {
                    panic!("rlwinmx: rc not implemented");
                };
                self.register.increment_pc();
            }
            Instruction::Lwz(gpr_d, gpr_a, d) => {
                let address = ((if gpr_a == 0 { 0 } else { self.register.gpr[gpr_a as usize] as i64}) + (d as i64)) as u32;
                self.register.gpr[gpr_d as usize] = self.read_u32(address);
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
