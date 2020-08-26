use crate::util::{make_rotation_mask, raw_u64_to_f64, u8_get_bit, get_bit_section, get_size_for_quantized_type};
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
    pub log: bool,
}

impl GekkoInterpreter {
    pub fn new(ram_amount: usize) -> GekkoInterpreter {
        GekkoInterpreter {
            ram: vec![0; ram_amount],
            register: GekkoRegister::default(),
            counter: 0,
            log: false,
        }
    }

    pub fn get_timebase(&self) -> u64 {
        self.counter >> 3
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
        if self.register.pc == 0x80003264 {
            self.log = true;
        }
        if self.log {
            println!("----");
            println!("pc: 0x{:x}", self.register.pc);
            println!("opcode: 0x{:x}", self.read_u32(self.register.pc));
        };
        let instruction = Instruction::decode_instruction(self.read_u32(self.register.pc)).unwrap();
        if self.log {
            println!("{:?}", instruction);
        }
        // second, run it
        let mut break_data = BreakData::None;
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
            //TODO: test
            Instruction::Cmp(crf_d, gpr_a, gpr_b) => {
                let a = self.register.get_gpr(gpr_a) as i32;
                let b = self.register.get_gpr(gpr_b) as i32;
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
                    let old_pc = self.register.pc;
                    self.register.pc = (self.register.lr >> 2) << 2;
                    if lk {
                        self.register.lr = old_pc + 4;
                    }
                } else {
                    self.register.increment_pc();
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
            Instruction::Lwzu(gpr_d, gpr_a, d) => {
                let address = (self.register.get_gpr(gpr_a) as i64 + (d as i64)) as u32;
                let new_value = self.read_u32(address);
                self.register.set_gpr(gpr_d, new_value);
                self.register.set_gpr(gpr_a, address);
                self.register.increment_pc();
            }
            Instruction::Stb(gpr_s, gpr_a, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                self.write_u8(address, self.register.get_gpr(gpr_s) as u8);
                self.register.increment_pc();
            }
            Instruction::Stbu(gpr_s, gpr_a, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                self.write_u8(address, self.register.get_gpr(gpr_s) as u8);
                self.register.set_gpr(gpr_a, address);
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
            Instruction::Addcx(gpr_d, gpr_a, gpr_b, oe, rc) => {
                let (result, overflow) = self
                    .register
                    .get_gpr(gpr_a)
                    .overflowing_add(self.register.get_gpr(gpr_b));
                self.register.set_gpr(gpr_d, result);

                if oe {
                    self.register.update_cr0(result);
                }
                if rc {
                    self.register.setxer_ov_so(overflow);
                    self.register.set_carry(overflow);
                }
                self.register.increment_pc();
            }
            Instruction::Addex(gpr_d, gpr_a, gpr_b, oe, rc) => {
                let (sum, overflow_1) = self
                    .register
                    .get_gpr(gpr_a)
                    .overflowing_add(self.register.get_gpr(gpr_b));
                let (sum_with_carry, overflow_2) =
                    sum.overflowing_add(if self.register.get_carry() { 1 } else { 0 });
                self.register.set_gpr(gpr_d, sum_with_carry);
                if oe {
                    self.register.update_cr0(sum_with_carry);
                }
                if rc {
                    let overflow = overflow_1 && overflow_2;
                    self.register.setxer_ov_so(overflow);
                    self.register.set_carry(overflow);
                }
                self.register.increment_pc();
            }
            Instruction::Lbz(gpr_d, gpr_a, d) => {
                //TODO: some unit test for it
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                let new_value = self.read_u8(address) as u32;
                self.register.set_gpr(gpr_d, new_value);
                self.register.increment_pc();
            }
            Instruction::Lbzu(gpr_d, gpr_a, d) => {
                //TODO: some unit test for it
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                let new_value = self.read_u8(address) as u32;
                self.register.set_gpr(gpr_d, new_value);
                self.register.set_gpr(gpr_a, address);
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
            Instruction::Lhz(gpr_d, gpr_a, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                let value = self.read_u16(address) as u32;
                self.register.set_gpr(gpr_d, value);
                self.register.increment_pc();
            }
            Instruction::Andidot(gpr_s, gpr_a, d) => {
                let value = self.register.get_gpr(gpr_a) & (d as u32);
                self.register.set_gpr(gpr_s, value);
                self.register.update_cr0(value);
                self.register.increment_pc();
            }
            Instruction::Subfx(gpr_d, gpr_a, gpr_b, oe, rc) => {
                let (result, overflow) = self
                    .register
                    .get_gpr(gpr_b)
                    .overflowing_sub(self.register.get_gpr(gpr_a));
                self.register.set_gpr(gpr_d, result);
                if oe {
                    self.register.setxer_ov_so(overflow);
                }
                if rc {
                    self.register.update_cr0(result);
                }
                self.register.increment_pc();
            }
            Instruction::Crxor(crb_d, crb_a, crb_b) => {
                self.register.set_bit_cr(
                    crb_d as usize,
                    self.register.get_bit_cr(crb_a as usize)
                        ^ self.register.get_bit_cr(crb_b as usize),
                );
                self.register.increment_pc();
            }
            Instruction::Lfd(fr_d, gpr_a, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                let value = raw_u64_to_f64(self.read_u64(address));
                self.register.set_fpr_ps0(fr_d, value);
                self.register.increment_pc();
            }
            Instruction::Frsqrtex(fr_d, fr_b, rc) => {
                let input_b = self.register.get_fpr_ps0(fr_b);
                let new_value = 1.0 / input_b.sqrt();
                self.register.set_fpr_ps0(fr_d, new_value);

                //remember: no exception handling will be implemented

                if rc {
                    self.register.update_cr1_f64(new_value);
                }
                self.register.increment_pc();
            }
            Instruction::Fmulx(fr_d, fr_a, fr_c, rc) => {
                let result = self.register.get_fpr_ps0(fr_a) * self.register.get_fpr_ps0(fr_c);
                self.register.set_fpr_ps0(fr_d, result);

                if rc {
                    self.register.update_cr1_f64(result);
                }
                self.register.increment_pc();
            }
            Instruction::Fnmsubx(fr_d, fr_a, fr_b, fr_c, rc) => {
                let value_a = self.register.get_fpr_ps0(fr_a);
                let value_b = self.register.get_fpr_ps0(fr_b);
                let value_c = self.register.get_fpr_ps0(fr_c);
                let result = -((value_a * value_c) - value_b);
                self.register.set_fpr_ps0(fr_d, result);

                if rc {
                    self.register.update_cr1_f64(result);
                }
                self.register.increment_pc();
            }
            Instruction::Frspx(fr_d, fr_b, rc) => {
                let value_source = self.register.get_fpr_ps0(fr_b) as f32;
                self.register.set_fpr_ps0(fr_d, value_source as f64);
                if rc {
                    self.register.update_cr1_f32(value_source);
                };
                self.register.increment_pc();
            }
            Instruction::Stfs(fr_s, gpr_a, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                let value_to_write = (self.register.get_fpr_ps0(fr_s) as f32).to_bits();
                self.write_u32(address, value_to_write);
                self.register.increment_pc();
            }
            Instruction::Lfs(fr_d, gpr_a, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                let new_value = f32::from_bits(self.read_u32(address)) as f64;
                self.register.set_fpr_both(fr_d, new_value);
                self.register.increment_pc();
            }
            Instruction::Stfdu(fr_s, gpr_a, d) => {
                let address = (self.register.get_gpr(gpr_a) as i64 + (d as i64)) as u32;
                let value_to_store = self.register.get_fpr_ps0(fr_s).to_bits();
                self.write_u64(address, value_to_store);
                self.register.set_gpr(gpr_a, address);
                self.register.increment_pc();
            }
            Instruction::Stfd(fr_s, gpr_a, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                let value_to_store = self.register.get_fpr_ps0(fr_s).to_bits();
                self.write_u64(address, value_to_store);
                self.register.increment_pc();
            }
            Instruction::Psq_st(fr_s, gpr_a, w, i, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                let qr = self.register.get_qr(i);
                let stt = get_bit_section(qr, 29, 3) as u8;
                let sts = get_bit_section(qr, 18, 6) as u8;
                let c = get_size_for_quantized_type(stt);
                if !w { // w == 0, to keep the order in the documentation
                    let fpr_0 = self.register.get_fpr_ps0(fr_s);
                    self.quantize_and_store(fpr_0, stt, sts, address);
                    let fpr_1 = self.register.get_fpr_ps1(fr_s);
                    self.quantize_and_store(fpr_1, stt, sts, address + c);
                } else {
                    todo!("psq_st for just one float");
                }
                self.register.increment_pc();
            }
            Instruction::Psq_l(fr_s, gpr_a, w, i, d) => {
                let address = self.register.compute_address_based_on_register(gpr_a, d);
                let qr = self.register.get_qr(i);
                let lt = get_bit_section(qr, 13, 3) as u8;
                let ls = get_bit_section(qr, 2, 6) as u8;
                let c = get_size_for_quantized_type(lt);
                let fpr_0 = self.dequantize(address, lt, ls);
                self.register.set_fpr_ps0(fr_s, fpr_0);
                if !w {
                    let fpr_1 = self.dequantize(address+c, lt, ls);
                    self.register.set_fpr_ps1(fr_s, fpr_1);
                } else {
                    self.register.set_fpr_ps1(fr_s, 1.0);
                }
                self.register.increment_pc();
            }
            Instruction::CustomBreak => {
                break_data = BreakData::Break;
                self.register.increment_pc();
            }
        };
        Ok(break_data)
    }

    fn quantize_and_store(&mut self, fpr: f64, st_type: u8, _st_scale: u8, address: u32) {
        match st_type {
            0 => {
                // no scaling
                self.write_u32(
                    address,
                    u32::from_ne_bytes((fpr as f32).to_ne_bytes())
                );
            }
            4 => todo!("quantize_and_store for type 4"),
            5 => todo!("quantize_and_store for type 5"),
            6 => todo!("quantize_and_store for type 6"),
            7 => todo!("quantize_and_store for type 7"),
            _ => panic!("invalid value for st_type in quantize_and_store: {}", st_type),
        }
    }

    fn dequantize(&self, address: u32, l_type: u8, _l_scale: u8) -> f64 {
        match l_type {
            0 => {
                let encoded_value = self.read_u32(address);
                return f32::from_ne_bytes((encoded_value).to_ne_bytes()) as f64
            }
            4 => todo!("dequantize type 4"),
            5 => todo!("dequantize type 5"),
            6 => todo!("dequantize type 6"),
            7 => todo!("dequantize type 7"),
            _ => panic!("invalide value for l_type in dequantize: {}", l_type)
        }
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
        offset -= BASE_RW_ADRESS;
        for d in &data.to_be_bytes() {
            self.ram[offset as usize] = *d;
            offset += 1;
        }
    }

    #[inline]
    pub fn write_u64(&mut self, mut offset: u32, data: u64) {
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
    pub fn read_u32(&self, mut offset: u32) -> u32 {
        offset -= BASE_RW_ADRESS;
        let mut buffer = [0; 4];
        for d in &mut buffer {
            *d = self.ram[offset as usize];
            offset += 1;
        }
        u32::from_be_bytes(buffer)
    }

    #[inline]
    pub fn read_u64(&self, mut offset: u32) -> u64 {
        offset -= BASE_RW_ADRESS;
        let mut buffer = [0; 8];
        for d in &mut buffer {
            *d = self.ram[offset as usize];
            offset += 1;
        }
        u64::from_be_bytes(buffer)
    }

    #[inline]
    pub fn read_u16(&self, mut offset: u32) -> u16 {
        offset -= BASE_RW_ADRESS;
        let v1 = self.ram[offset as usize];
        let v2 = self.ram[(offset as usize) + 1];
        u16::from_be_bytes([v1, v2])
    }

    #[inline]
    pub fn read_u8(&self, offset: u32) -> u8 {
        self.ram[(offset - BASE_RW_ADRESS) as usize]
    }
}
