#![no_std]
#![no_main]

use core::{arch::naked_asm, panic::PanicInfo};

use poc_rxv6::{irq, println, proc};

const CPU_FREQ_HZ: u64 = 10_000_000;

unsafe extern "C" {
    static mut __stack_size: u8;
    static mut __stack_start: u8;
    static mut __stack_end: u8;
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _entry() -> ! {
    naked_asm!("la sp, __stack_end", "call start", "1: j 1b")
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _trapvec() {
    naked_asm!(
        // Save user registers on current stack, except sp & tp (which is hart-local)
        "addi sp, sp, -30*8",
        "sd  ra,  0*8(sp)",
        "sd  gp,  1*8(sp)",
        "sd  t0,  2*8(sp)",
        "sd  t1,  3*8(sp)",
        "sd  t2,  4*8(sp)",
        "sd  t3,  5*8(sp)",
        "sd  t4,  6*8(sp)",
        "sd  t5,  7*8(sp)",
        "sd  t6,  8*8(sp)",
        "sd  a0,  9*8(sp)",
        "sd  a1, 10*8(sp)",
        "sd  a2, 11*8(sp)",
        "sd  a3, 12*8(sp)",
        "sd  a4, 13*8(sp)",
        "sd  a5, 14*8(sp)",
        "sd  a6, 15*8(sp)",
        "sd  a7, 16*8(sp)",
        "sd  s0, 17*8(sp)",
        "sd  s1, 18*8(sp)",
        "sd  s2, 19*8(sp)",
        "sd  s3, 20*8(sp)",
        "sd  s4, 21*8(sp)",
        "sd  s5, 22*8(sp)",
        "sd  s6, 23*8(sp)",
        "sd  s7, 24*8(sp)",
        "sd  s8, 25*8(sp)",
        "sd  s9, 26*8(sp)",
        "sd s10, 27*8(sp)",
        "sd s11, 28*8(sp)",
        // Save mepc
        "csrr a0, mepc",
        "sd a0, 29*8(sp)",
        // Call trapvec
        // a0 is already loaded with mepc
        "csrr a1, mcause",
        "csrr a2, mtval",
        "call trapvec",
        // Restore mepc
        "ld t0, 29*8(sp)",
        "csrw mepc, t0",
        // Restore registers from current stack, except sp & tp
        "ld  ra,  0*8(sp)",
        "ld  gp,  1*8(sp)",
        "ld  t0,  2*8(sp)",
        "ld  t1,  3*8(sp)",
        "ld  t2,  4*8(sp)",
        "ld  t3,  5*8(sp)",
        "ld  t4,  6*8(sp)",
        "ld  t5,  7*8(sp)",
        "ld  t6,  8*8(sp)",
        "ld  a0,  9*8(sp)",
        "ld  a1, 10*8(sp)",
        "ld  a2, 11*8(sp)",
        "ld  a3, 12*8(sp)",
        "ld  a4, 13*8(sp)",
        "ld  a5, 14*8(sp)",
        "ld  a6, 15*8(sp)",
        "ld  a7, 16*8(sp)",
        "ld  s0, 17*8(sp)",
        "ld  s1, 18*8(sp)",
        "ld  s2, 19*8(sp)",
        "ld  s3, 20*8(sp)",
        "ld  s4, 21*8(sp)",
        "ld  s5, 22*8(sp)",
        "ld  s6, 23*8(sp)",
        "ld  s7, 24*8(sp)",
        "ld  s8, 25*8(sp)",
        "ld  s9, 26*8(sp)",
        "ld s10, 27*8(sp)",
        "ld s11, 28*8(sp)",
        "addi sp, sp, 30*8",
        // Return from irq
        "mret"
    )
}

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() -> ! {
    println!("rxv6 start");
    println!(
        "stack: [{:?}-{:?}]({})",
        &raw const __stack_start, &raw const __stack_end, &raw const __stack_size as usize
    );

    println!("Setting up irq...");
    irq::setup(_trapvec);

    println!("Creating process 1...");
    proc::PROCESSES.create(process1);
    println!("Creating process 2...");
    proc::PROCESSES.create(process2);
    println!("Starting scheduler...");
    io
    proc::run_scheduler(CPU_FREQ_HZ * 2);
}

pub fn process1() {
    loop {
        println!("I'm process 1!");
        proc::wait_irq();
    }
}

pub fn process2() {
    loop {
        println!("I'm process 2!");
        proc::sleep(CPU_FREQ_HZ * 3);
    }
}

const MCAUSE_MTI: u64 = 7 | 1 << 63;

#[unsafe(no_mangle)]
pub unsafe fn trapvec(mepc: u64, mcause: u64, mtval: u64) {
    println!("TRAP");
    match mcause {
        MCAUSE_MTI => proc::yield_self(),
        _ => panic!(
            "unhandled irq (mepc: {:#016X}; mcause: {:#016X}; mtval: {:#016X})",
            mepc, mcause, mtval
        ),
    }
}
