use crate::GekkoInterpreter;
use crate::BASE_RW_ADRESS;
use crate::OPCODE_BREAK;

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
    gekko.write_u32(BASE_RW_ADRESS, 0b001010_101_0_0_00100_00000000_00000101);
    gekko.register.gpr[4] = 35;
    gekko.register.setxer_ov_so(true);
    gekko.register.setxer_ov_so(false);
    gekko.step().unwrap();
    assert_eq!(gekko.register.cr[5], 0b0101);
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

#[test]
fn test_bcx() {
    let mut gekko = GekkoInterpreter::new(100);
    //test "bc 0b00100 0b00000 8"
    gekko.write_u32(BASE_RW_ADRESS, 0b010000_00100_00000_00000000_001000_0_0);
    gekko.step().unwrap();
    assert_eq!(gekko.register.pc, BASE_RW_ADRESS + 8);
    gekko.register.pc = BASE_RW_ADRESS;
    gekko.register.cr[0] = 0b00001000;
    gekko.step().unwrap();
    assert_eq!(gekko.register.pc, BASE_RW_ADRESS + 4);
    assert_eq!(gekko.register.lr, 0);
    //test "bcla 0b00010 0b00100 12"
    gekko.reboot();
    gekko.write_u32(BASE_RW_ADRESS, 0b010000_00010_00100_00000000_001100_1_1);
    gekko.register.ctr = 2;
    gekko.step().unwrap();
    assert_eq!(gekko.register.pc, BASE_RW_ADRESS + 4);
    assert_eq!(gekko.register.ctr, 1);
    assert_eq!(gekko.register.lr, 0);
    gekko.register.pc = BASE_RW_ADRESS;
    gekko.step().unwrap();
    assert_eq!(gekko.register.pc, 12);
    assert_eq!(gekko.register.ctr, 0);
    assert_eq!(gekko.register.lr, BASE_RW_ADRESS + 4);
    //test "bca 0b10000 0 12"
    gekko.reboot();
    gekko.write_u32(BASE_RW_ADRESS, 0b010000_10000_00000_00000000_001100_1_0);
    gekko.register.ctr = 5;
    gekko.step().unwrap();
    assert_eq!(gekko.register.ctr, 4);
    assert_eq!(gekko.register.pc, 12);
    assert_eq!(gekko.register.lr, 0);

    gekko.reboot();
    //test:
    //with r3 = 10
    //with r2 = 0
    //cmpli 5, 0, r3, 10
    gekko.write_u32(BASE_RW_ADRESS, 0b001010_101_0_0_00011_00000000_00001010);
    //bc 0b01100 (5*4+2) 8 ;5*4+2=22
    gekko.write_u32(BASE_RW_ADRESS + 4, 0b010000_01100_10110_00000000_001000_0_0);
    //break
    gekko.write_u32(BASE_RW_ADRESS + 8, OPCODE_BREAK);
    //or r2, r3, r3
    gekko.write_u32(BASE_RW_ADRESS + 12, 0b011111_00011_00010_00011_0110111100_0);
    //break
    gekko.write_u32(BASE_RW_ADRESS + 16, OPCODE_BREAK);
    gekko.register.gpr[3] = 10;
    gekko.register.gpr[2] = 0;
    gekko.run_until_event();
    assert_eq!(gekko.register.gpr[2], 10);
    gekko.register.pc = BASE_RW_ADRESS;
    gekko.register.gpr[3] = 100;
    gekko.register.gpr[2] = 3;
    gekko.run_until_event();
    assert_eq!(gekko.register.gpr[2], 3);
}

#[test]
fn test_rlwinmx() {
    let mut gekko = GekkoInterpreter::new(4);
    //test "rlwinm r4, r3, 2, 10, 20"
    //tested with dolphin
    gekko.write_u32(BASE_RW_ADRESS, 0b010101_00011_00100_00010_01010_10100_0);
    gekko.register.gpr[3] = 0xabcdef12;
    gekko.step().unwrap();
    assert_eq!(gekko.register.gpr[4], 0x0037b800)
    //TODO: test with Rc
}

#[test]
fn test_lwz() {
    let mut gekko = GekkoInterpreter::new(12);
    //test "lwz r3, 4(r5)"
    gekko.write_u32(BASE_RW_ADRESS, 0b100000_00011_00101_00000000_00000100);
    gekko.register.gpr[5] = BASE_RW_ADRESS + 4;
    gekko.write_u32(BASE_RW_ADRESS + 8, 5434);
    gekko.step().unwrap();
    assert_eq!(gekko.register.gpr[3], 5434);
    gekko.reboot();
    //test lwz r31, -4(r16)
    gekko.write_u32(BASE_RW_ADRESS, 0b100000_11111_10000_11111111_11111100);
    gekko.register.gpr[16] = BASE_RW_ADRESS + 8;
    gekko.write_u32(BASE_RW_ADRESS + 4, 0xDEAD_BEEF);
    gekko.step().unwrap();
    assert_eq!(gekko.register.gpr[31], 0xDEAD_BEEF);
}

#[test]
fn test_stb() {
    let mut gekko = GekkoInterpreter::new(12);
    //test "stb r3, 0(r5)"
    gekko.write_u32(BASE_RW_ADRESS, 0b100110_00011_00101_00000000_00000000);
    gekko.register.gpr[5] = BASE_RW_ADRESS + 6;
    gekko.register.gpr[3] = 0x12345678;
    gekko.step().unwrap();
    assert_eq!(gekko.read_u8(BASE_RW_ADRESS + 6), 0x78);
}

#[test]
fn test_addis() {
    let mut gekko = GekkoInterpreter::new(4);
    //test "addis r4, r3, 10"
    gekko.write_u32(BASE_RW_ADRESS, 0b001111_00100_00011_00000000_00001010);
    gekko.register.gpr[3] = 25;
    gekko.step().unwrap();
    assert_eq!(gekko.register.gpr[4], 35);
    gekko.reboot();
    //test "addis r20, 0, -8"
    gekko.write_u32(BASE_RW_ADRESS, 0b001111_10100_00000_11111111_11111000);
    gekko.step().unwrap();
    assert_eq!(gekko.register.gpr[20], (-8i32) as u32)
}

#[test]
fn test_addi() {
    let mut gekko = GekkoInterpreter::new(4);
    //test "addi r4, r3, 10"
    gekko.write_u32(BASE_RW_ADRESS, 0b001110_00100_00011_00000000_00001010);
    gekko.register.gpr[3] = 25;
    gekko.step().unwrap();
    assert_eq!(gekko.register.gpr[4], 35);
    gekko.reboot();
    //test "addi r20, 0, 0xFFF0"
    gekko.write_u32(BASE_RW_ADRESS, 0b001110_10100_00000_11111111_11110000);
    gekko.step().unwrap();
    assert_eq!(gekko.register.gpr[20], 0xFFF0);
}
