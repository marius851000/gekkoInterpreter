use gekko_interpreter::GekkoInterpreter;
use gekko_interpreter::OPCODE_BREAK;
use std::fs::File;
use std::io::Read;

pub fn main() {
    // for reallocating the data
    let realloc_data: [(u32, u32); 11] = [
        (0x80003100, 0x000024e8), //.init
        (0x80005600, 0x48),       //extab
        (0x80005660, 0x5c),       //extabindex
        (0x800056c0, 0x350798),   //.text
        (0x80355e60, 0x200),      //.ctors
        (0x80356060, 0x8),        //.dtors
        (0x80356080, 0x195dc),    //.rodata
        (0x8036f660, 0x75750),    //.data
        (0x80586dc0, 0x1888),     //.sdata
        (0x8058ba40, 0x9a40),     //.sdata2
        (0x81000000, 0x0000),
    ];

    let mut binary = File::open("spyro06US2.bin").unwrap();
    let mut vec = Vec::new();

    let mut actual_offset = gekko_interpreter::BASE_RW_ADRESS;
    for (offset, size) in realloc_data.iter() {
        let offset = *offset;
        let size = *size;
        if actual_offset != offset {
            vec.resize((offset - gekko_interpreter::BASE_RW_ADRESS) as usize, 0);
            actual_offset = offset;
        };
        let mut buffer = vec![0; size as usize];
        binary.read_exact(&mut buffer).unwrap();
        vec.append(&mut buffer);
        actual_offset += size;
    }

    let mut gekko = GekkoInterpreter::new(4);
    gekko.replace_memory(vec);

    let instruction_to_blr = [
        0x8029ee88, //OSDisableInterrupts
        0x8029eeb0, //OSRestoreInterrupts
        0x80299980, //PPCMtmmcr0
        0x80299988, //PPCMtmmcr1
        0x80299990, //PPCMtpmc1
        0x80299998, //PPCMtpmc2
        0x802999a0, //PPCMtpmc3
        0x802999a8, //PPCMtpmc4
        0x80299a10, //PPCDisableSpeculation
        0x80299a38, //PPCSetFpNonIEEEMode
                    //0x80299cb8, //OSInit, tmp
    ];

    for offset in instruction_to_blr.iter() {
        gekko.write_u32(*offset, 0x4e800020);
    }

    if false {
        gekko.register.set_gpr(1, 0x805a5420);
        gekko.register.set_gpr(13, 0x8058edc0);
        gekko.register.lr = 0x8030ef84;

        gekko.register.pc = 0x803047c4; //MKHeap::InitModule
        gekko.write_u32(0x8030482c, OPCODE_BREAK);
    } else {
        gekko.write_u32(0x80003158, 0x60000000); //nop some hardware init
        gekko.register.pc = 0x80003154; //start of the program
    }
    println!("{:?}", gekko.run_until_event());
}
