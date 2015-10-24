use super::*;

#[test]
fn bitmanip_high() {
    assert_eq!(0xABCD.high_byte(), 0xAB);
}

#[test]
fn bitmanip_low() {
    assert_eq!(0xABCD.low_byte(), 0xCD);
}

#[test]
fn bitmanip_nibble() {
    let n = 0xABCD;
    assert_eq!(n.nibble(1), 0xA);
    assert_eq!(n.nibble(2), 0xB);
    assert_eq!(n.nibble(3), 0xC);
    assert_eq!(n.nibble(4), 0xD);
}

fn memset(c: &mut Chip8, location: usize, values: &[u16]) {
    for (ind, mem) in values.iter().enumerate() {
        c.memory[location + ind * 2] = (mem >> 8) as u8;
        c.memory[location + (ind * 2) + 1] = (mem & 0x00FF) as u8;
    }
}

#[test]
fn chip8test() {
    let mut c = Chip8::init();
    assert_eq!(c.pc, 0x200);
    memset(&mut c, 0x200, &[0x1204]); //jump to 0x204
    c.reginfo();
    c.step(); //0x200
    c.reginfo();
    assert_eq!(c.pc, 0x204);
    memset(&mut c, 0x190, &[0x6C0F, 0x3C0F, 0x0000, 0x00EE]); //load 0xF into V0, skip next instruction (0x0000) if V0 == 0xF, return from function.
    memset(&mut c, 0x204, &[0x2190]); //call function at 0x190
    c.step(); //0x204
    c.reginfo();
    assert_eq!(c.pc, 0x190);
    assert_eq!(c.sp, 15);
    assert_eq!(c.stack[15], 0x204);
    c.step(); //0x190
    c.reginfo();
    assert_eq!(c.V[0xC], 0xF);
    c.step(); //0x192: will panic with invalid instruction 0x0000 if this instruction fails
    c.reginfo();
    c.step(); //0x196
    assert_eq!(c.sp, 16);
    assert_eq!(c.pc, 0x206);
}

#[test]
fn chip8drawtest() {
    let mut c = Chip8::init();
    memset(&mut c,
           0x200,
           &[0x6003, 0xF029, 0xD125, 0x00E0, 0x600A, 0xF029, 0xD125]);
    c.step(); //0x6003
    assert_eq!(c.V[0], 0x3);
    c.step(); //0xF029
    assert_eq!(c.I, 0xF);
    c.step(); //0xD125
    c.dumpgfx();
    c.step(); //0x00E0
    c.step(); //0x600A
    c.step(); //0xF029
    c.step(); //0xD125
    c.dumpgfx();
    assert!(c.draw_flag);
}