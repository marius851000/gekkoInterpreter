use crate::util::{make_rotation_mask, u8_get_bit};
use crate::GekkoRegister;
use crate::Instruction;
use crate::Tbr;
use crate::BASE_RW_ADRESS;
use std::mem::replace;

#[derive(Debug, PartialEq)]
pub enum BreakData {
    None,
    Break,
}

pub struct GekkoInterpreter {
    pub ram: Vec<u8>,
    pub register: GekkoRegister,
    pub counter: u64,
}

impl GekkoInterpreter {
    pub fn new(ram_amount: usize) -> GekkoInterpreter {
        GekkoInterpreter {
            ram: vec![0; ram_amount],
            register: GekkoRegister::default(),
            counter: 0,
        }
    }

    pub fn get_timebase(&self) -> u64 {
        self.counter
    }

    pub fn replace_memory(&mut self, new_ram: Vec<u8>) -> Vec<u8> {
        replace(&mut self.ram, new_ram)
    }

    pub fn reboot(&mut self) {
        self.ram = vec![0; self.ram.len()];
        self.register = GekkoRegister::default();
    }

    pub fn step(&mut self) -> Result<BreakData, String> {
        self.counter += 1;
        // first, get the instruction
        println!("----");
        println!("pc: 0x{:x}", self.register.pc);
        //println!("inst: 0x{:x}", self.read_u32(self.register.pc));
        let instruction = Instruction::decode_instruction(self.read_u32(self.register.pc)).unwrap();
        // second, run it
        let mut break_data = BreakData::None;
        println!("{:?}", &instruction);
        //println!("{:?}", instruction);
        match instruction {
            Instruction::Addx(gpr_dest, gpr_1, gpr_2, oe, rc) => {
                let (result, overflow) = self
                    .register
                    .get_gpr(gpr_1)
                    .overflowing_add(self.register.get_gpr(gpr_2));
                self.register.set_gpr(gpr_dest, result);
                if oe {
                    self.register.setxer_ov_so(overflow);
                };
                if rc {
                    self.register.update_cr0(result);
                };
                self.register.increment_pc();
            }
            Instruction::Stwu(gpr_s, gpr_a, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                self.write_u32(address, self.register.get_gpr(gpr_s));
                self.register.set_gpr(gpr_a, address);
                self.register.increment_pc();
            }
            Instruction::Mfspr(gpr_d, spr) => {
                self.register.set_gpr(gpr_d, self.register.get_spr(spr));
                self.register.increment_pc();
            }
            Instruction::Cmpli(crf_d, gpr_a, uimm) => {
                let a = self.register.get_gpr(gpr_a);
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
            //TODO: test
            Instruction::Cmpl(crf_d, gpr_a, gpr_b) => {
                let a = self.register.get_gpr(gpr_a);
                let b = self.register.get_gpr(gpr_b);
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
            //TODO: test
            Instruction::Cmpi(crf_d, gpr_a, uimm) => {
                let a = self.register.get_gpr(gpr_a) as i32;
                let b = uimm as i32;
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
            Instruction::Stw(gpr_s, gpr_a, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                self.write_u32(address, self.register.get_gpr(gpr_s));

                self.register.increment_pc();
            }
            Instruction::Stmw(mut gpr_s, gpr_a, d) => {
                let mut address = self.register.compute_address_based_on_register(gpr_a, d);
                while gpr_s < 32 {
                    self.write_u32(address, self.register.get_gpr(gpr_s));
                    gpr_s += 1;
                    address += 4;
                }
                self.register.increment_pc();
            }
            Instruction::Orx(gpr_s, gpr_a, gpr_b, rc) => {
                self.register.set_gpr(
                    gpr_a,
                    self.register.get_gpr(gpr_s) | self.register.get_gpr(gpr_b),
                );
                if rc {
                    panic!("orx: rc not implemented");
                };
                self.register.increment_pc();
            }
            Instruction::Bcx(bo, bi, bd, aa, lk) => {
                let (ctr_ok, cond_ok) = self.check_and_apply_conditional_jump(bo, bi);
                if ctr_ok & cond_ok {
                    if lk {
                        self.register.lr = self.register.pc + 4;
                    }
                    if aa {
                        self.register.pc = ((bd as i32) << 2) as u32;
                    } else {
                        self.register.pc = ((self.register.pc as i64) + ((bd as i64) << 2)) as u32
                    }
                } else {
                    self.register.increment_pc();
                }
            }
            Instruction::Bclrx(bo, bi, lk) => {
                let (ctr_ok, cond_ok) = self.check_and_apply_conditional_jump(bo, bi);
                if ctr_ok & cond_ok {
                    self.register.pc = (self.register.lr >> 2) << 2;
                    if lk {
                        self.register.lr = self.register.pc + 4;
                    }
                }
            }
            Instruction::Bx(li, aa, lk) => {
                if lk {
                    self.register.lr = self.register.pc + 4;
                };
                if aa {
                    self.register.pc = (li << 2) as u32;
                } else {
                    self.register.pc = (self.register.pc as i64 + ((li as i64) << 2)) as u32;
                }
            }
            Instruction::Rlwinmx(gpr_s, gpr_a, sh, mb, me, rc) => {
                let mask = make_rotation_mask(mb as u32, me as u32);
                self.register.set_gpr(
                    gpr_a,
                    self.register.get_gpr(gpr_s).rotate_left(sh as u32) & mask,
                );
                if rc {
                    self.register.update_cr0(self.register.get_gpr(gpr_a));
                };
                self.register.increment_pc();
            }
            Instruction::Lwz(gpr_d, gpr_a, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                let new_value = self.read_u32(address);
                self.register.set_gpr(gpr_d, new_value);
                self.register.increment_pc();
            }
            Instruction::Stb(gpr_s, gpr_a, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                self.write_u8(address, self.register.get_gpr(gpr_s) as u8);
                self.register.increment_pc();
            }
            Instruction::Addis(gpr_d, gpr_a, simm) => {
                self.register.set_gpr(
                    gpr_d,
                    (if gpr_a == 0 {
                        0
                    } else {
                        self.register.get_gpr(gpr_a)
                    })
                    .wrapping_add((simm as u32) << 16),
                );
                self.register.increment_pc();
            }
            Instruction::Addi(gpr_d, gpr_a, simm) => {
                self.register.set_gpr(
                    gpr_d,
                    (if gpr_a == 0 {
                        0
                    } else {
                        self.register.get_gpr(gpr_a)
                    })
                    .wrapping_add((simm as i32) as u32),
                );
                self.register.increment_pc();
            }
            Instruction::Lbz(gpr_d, gpr_a, d) => {
                //TODO: some unit test for it
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                let new_value = self.read_u8(address) as u32;
                self.register.set_gpr(gpr_d, new_value);
                self.register.increment_pc();
            }
            Instruction::Extsbx(gpr_s, gpr_a, rc) => {
                self.register
                    .set_gpr(gpr_a, ((self.register.get_gpr(gpr_s) as i8) as i32) as u32);

                if rc {
                    self.register.update_cr0(self.register.get_gpr(gpr_a));
                }

                self.register.increment_pc();
            }
            Instruction::Lwzx(gpr_d, gpr_a, gpr_b) => {
                let address = self
                    .register
                    .compute_address_based_on_pair_of_register(gpr_a, gpr_b);

                let value = self.read_u32(address);
                self.register.set_gpr(gpr_d, value);
                self.register.increment_pc();
            }
            Instruction::Stwx(gpr_s, gpr_a, gpr_b) => {
                let address = self
                    .register
                    .compute_address_based_on_pair_of_register(gpr_a, gpr_b);
                self.write_u32(address, self.register.get_gpr(gpr_s));
                self.register.increment_pc();
            }
            Instruction::Lmw(gpr_d, gpr_a, d) => {
                let mut address = self.register.compute_address_based_on_register(gpr_a, d);
                let mut r = self.register.get_gpr(gpr_d);
                while r < 32 {
                    let value = self.read_u32(address);
                    self.register.set_gpr(r as u8, value);
                    r += 1;
                    address += 4;
                }
                self.register.increment_pc();
            }
            Instruction::Mtspr(gpr_s, spr) => {
                self.register.set_spr(spr, self.register.get_gpr(gpr_s));
                self.register.increment_pc();
            }
            Instruction::Ori(gpr_s, gpr_a, uuim) => {
                self.register
                    .set_gpr(gpr_s, self.register.get_gpr(gpr_a) | (uuim as u32));
                self.register.increment_pc();
            }
            Instruction::Nor(gpr_s, gpr_a, gpr_b, rc) => {
                self.register.set_gpr(
                    gpr_a,
                    !(self.register.get_gpr(gpr_s) | self.register.get_gpr(gpr_b)),
                );
                if rc {
                    self.register.update_cr0(self.register.get_gpr(gpr_a));
                }
                self.register.increment_pc();
            }
            Instruction::Addicdot(gpr_d, gpr_a, simm) => {
                let a = self.register.get_gpr(gpr_a);
                let (d, overflow) = a.overflowing_add((simm as i32) as u32);
                self.register.set_gpr(gpr_d, d);
                self.register.set_carry(overflow);
                self.register.update_cr0(self.register.get_gpr(gpr_d));
                self.register.increment_pc();
            }
            Instruction::Mftb(gpr_d, tbr) => {
                self.register.set_gpr(
                    gpr_d,
                    match tbr {
                        Tbr::Tbl => (self.get_timebase() >> 32) as u32,
                        Tbr::Tbu => (self.get_timebase() << 32 >> 32) as u32,
                    },
                );
                self.register.increment_pc();
            }
            Instruction::CustomBreak => {
                break_data = BreakData::Break;
                self.register.increment_pc();
            }
        };
        Ok(break_data)
    }

    fn check_and_apply_conditional_jump(&mut self, bo: u8, bi: u8) -> (bool, bool) {
        let dont_use_ctr = u8_get_bit(bo, 7 - 2);
        if !dont_use_ctr {
            self.register.decrement_ctr();
        };
        let ctr_diff_0 = u8_get_bit(bo, 7 - 1);
        let ctr_ok = dont_use_ctr | ((self.register.ctr != 0) ^ ctr_diff_0);
        let dont_use_cond = u8_get_bit(bo, 7 - 4);
        let cond_ok =
            dont_use_cond | (self.register.get_bit_cr(bi as usize) == u8_get_bit(bo, 7 - 3));
        (ctr_ok, cond_ok)
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

    #[inline]
    pub fn write_u32(&mut self, mut offset: u32, data: u32) {
        if offset == 0x805a549c {
            if data == 0 {
                panic!()
            }
            println!("wrote to {} with 0x{:x}", offset, data);
        }
        offset -= BASE_RW_ADRESS;
        for d in &data.to_be_bytes() {
            self.ram[offset as usize] = *d;
            offset += 1;
        }
    }

    #[inline]
    pub fn write_u8(&mut self, offset: u32, data: u8) {
        self.ram[(offset - BASE_RW_ADRESS) as usize] = data;
    }

    #[inline]
    pub fn read_u32(&mut self, mut offset: u32) -> u32 {
        offset -= BASE_RW_ADRESS;
        let mut buffer = [0; 4];
        for d in &mut buffer {
            *d = self.ram[offset as usize];
            offset += 1;
        }
        u32::from_be_bytes(buffer)
    }

    #[inline]
    pub fn read_u8(&mut self, offset: u32) -> u8 {
        self.ram[(offset - BASE_RW_ADRESS) as usize]
    }
}
