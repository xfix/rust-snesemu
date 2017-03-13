use bitwidth::BitWidth;
use cpu::{CPU, Flags, FLAG_NO_IRQ, FLAG_A16};
use mapper::Mapper;

fn fetch<M: Mapper>(cpu: &mut CPU<M>) -> u8 {
    let byte = cpu.read(cpu.registers.pb, cpu.registers.pc);
    cpu.registers.pc = cpu.registers.pc.wrapping_add(1);
    byte
}

fn set_flag<M: Mapper>(cpu: &mut CPU<M>, flags: Flags) {
    cpu.registers.flags |= flags;
}

// Addressing types
//
// This is actually fairly crazy. Many addressing modes work differently
// depending on 16-bit mode. To avoid writing the same code twice, opcode
// implementations are generic over BitWidth which implements generic
// functions to handle any bit width mode.
//
// Rust doesn't currently support higher-kinded types, and function literal
// can be only resolved to a single type. To resolve this issue, a function
// is passed twice, so Rust has to resolve types twice.

fn absolute<M, F, G>(cpu: &mut CPU<M>, sixteen_bits: bool, f: F, g: G)
    where M: Mapper,
          F: FnOnce(&mut CPU<M>, u8),
          G: FnOnce(&mut CPU<M>, u16)
{
    absolute_address(cpu,
                     sixteen_bits,
                     |cpu, address| {
                         let value = cpu.read(cpu.registers.db, address);
                         g(cpu, value);
                     },
                     |cpu, address| {
                         let value = cpu.read(cpu.registers.db, address);
                         f(cpu, value);
                     });
}

fn absolute_address<M, F, G>(cpu: &mut CPU<M>, sixteen_bits: bool, f: F, g: G)
    where M: Mapper,
          F: FnOnce(&mut CPU<M>, u16),
          G: FnOnce(&mut CPU<M>, u16)
{
    let a = fetch(cpu);
    let b = fetch(cpu);
    let address = a as u16 | ((b as u16) << 8);

    if sixteen_bits {
        g(cpu, address);
    } else {
        f(cpu, address);
    }
}

fn immediate<M, F, G>(cpu: &mut CPU<M>, sixteen_bits: bool, f: F, g: G)
    where M: Mapper,
          F: FnOnce(&mut CPU<M>, u8),
          G: FnOnce(&mut CPU<M>, u16)
{
    let a = fetch(cpu);

    if sixteen_bits {
        let b = (fetch(cpu) as u16) << 8;
        g(cpu, a as u16 | b);
    } else {
        f(cpu, a);
    }
}

fn a16<M, F, G, H>(cpu: &mut CPU<M>, f: F, g: G, h: H)
    where M: Mapper,
          F: FnOnce(&mut CPU<M>, bool, G, H)
{
    let sixteen_bits = cpu.registers.flags.contains(FLAG_A16);
    f(cpu, sixteen_bits, g, h);
}

fn lda<M: Mapper, T: BitWidth>(cpu: &mut CPU<M>, value: T) {
    T::set(&mut cpu.registers.a, value);
}

fn stz<M: Mapper, T: BitWidth + Default>(cpu: &mut CPU<M>, address: u16) {
    let db = cpu.registers.db;
    cpu.write(db, address, T::default());
}

pub fn run_instruction<M: Mapper>(cpu: &mut CPU<M>) {
    match fetch(cpu) {
        // LDA (Load Accumulator from Memory)
        // immediate
        0xA9 => a16(cpu, immediate, lda, lda),
        // absolute
        0xAD => a16(cpu, absolute, lda, lda),

        // SEI (Set Interrupt Disable Flag)
        // implied
        0x78 => set_flag(cpu, FLAG_NO_IRQ),

        // STZ (Store Zero to Memory)
        // absolute
        0x9C => a16(cpu, absolute_address, stz::<M, u8>, stz::<M, u16>),

        code => {
            println!("{:x}", code);
            unimplemented!();
        }
    }
}