use core::{
    arch::{asm, naked_asm},
    mem::MaybeUninit,
};

use crate::{
    irq, println,
    timer::{self, current_time},
    utils::{cells::Idc, collections::ArrayVec},
};

#[repr(C)]
#[derive(Debug)]
pub struct Context {
    ra: u64,
    sp: u64,
    gp: u64,
    tp: u64,
    t: [u64; 7],
    s: [u64; 12],
    a: [u64; 8],
}

impl Context {
    pub fn new(stack_end: *mut u8, entry: fn()) -> Self {
        Self {
            ra: entry as u64,
            sp: stack_end as u64,
            gp: 0,
            tp: 0,
            t: [0; 7],
            s: [0; 12],
            a: [0; 8],
        }
    }

    pub const fn zeroed() -> Context {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[unsafe(naked)]
unsafe extern "C" fn switch(from: *mut Context, to: *const Context) {
    naked_asm!(
        // Save callee-saved registers to current
        "sd ra, 0(a0)",
        "sd sp, 8(a0)",
        "sd gp, 16(a0)",
        "sd tp, 24(a0)",
        "sd t0, 32(a0)",
        "sd t1, 40(a0)",
        "sd t2, 48(a0)",
        "sd t3, 56(a0)",
        "sd t4, 64(a0)",
        "sd t5, 72(a0)",
        "sd t6, 80(a0)",
        "sd s0, 88(a0)",
        "sd s1, 96(a0)",
        "sd s2, 104(a0)",
        "sd s3, 112(a0)",
        "sd s4, 120(a0)",
        "sd s5, 128(a0)",
        "sd s6, 136(a0)",
        "sd s7, 144(a0)",
        "sd s8, 152(a0)",
        "sd s9, 160(a0)",
        "sd s10, 168(a0)",
        "sd s11, 176(a0)",
        "sd a0, 184(a0)",
        "sd a1, 192(a0)",
        "sd a2, 200(a0)",
        "sd a3, 208(a0)",
        "sd a4, 216(a0)",
        "sd a5, 224(a0)",
        "sd a6, 232(a0)",
        "sd a7, 240(a0)",
        // Load from next
        "ld ra, 0(a1)",
        "ld sp, 8(a1)",
        "ld gp, 16(a1)",
        "ld tp, 24(a1)",
        "ld t0, 32(a1)",
        "ld t1, 40(a1)",
        "ld t2, 48(a1)",
        "ld t3, 56(a1)",
        "ld t4, 64(a1)",
        "ld t5, 72(a1)",
        "ld t6, 80(a1)",
        "ld s0, 88(a1)",
        "ld s1, 96(a1)",
        "ld s2, 104(a1)",
        "ld s3, 112(a1)",
        "ld s4, 120(a1)",
        "ld s5, 128(a1)",
        "ld s6, 136(a1)",
        "ld s7, 144(a1)",
        "ld s8, 152(a1)",
        "ld s9, 160(a1)",
        "ld s10, 168(a1)",
        "ld s11, 176(a1)",
        "ld a0, 184(a1)",
        "ld a2, 200(a1)",
        "ld a3, 208(a1)",
        "ld a4, 216(a1)",
        "ld a5, 224(a1)",
        "ld a6, 232(a1)",
        "ld a7, 240(a1)",
        "ld a1, 192(a1)",
        "ret",
    );
}

pub type PID = u64;

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessState {
    Idle,
    Running,
    Sleeping { start: u64, duration: u64 },
}

impl ProcessState {
    pub fn can_run(&self) -> bool {
        match self {
            ProcessState::Idle => true,
            ProcessState::Running => false,
            ProcessState::Sleeping { start, duration } => current_time() > start + duration,
        }
    }
}

#[derive(Debug)]
pub struct Process {
    state: ProcessState,
    context: Context,
}

impl Process {
    pub fn can_run(&self) -> bool {
        self.state.can_run()
    }
}

const MAX_PROCESSES: usize = 2;
const PROCESS_STACK_SIZE: usize = 8 * 1024;

static mut STACKS: [[u8; PROCESS_STACK_SIZE]; MAX_PROCESSES] =
    [[0; PROCESS_STACK_SIZE]; MAX_PROCESSES];
static PROCESSES: Idc<ArrayVec<Process, MAX_PROCESSES>> = Idc::new(ArrayVec::new());
static mut CURRENT_PID: PID = 0;

pub fn create_process(entry: fn()) -> PID {
    let pid = PROCESSES.get().len();
    let stack = unsafe { &raw mut STACKS[pid] as *mut u8 };
    let sp = stack.wrapping_offset((PROCESS_STACK_SIZE - 1) as isize);
    PROCESSES.get().push(Process {
        state: ProcessState::Idle,
        context: Context::new(sp, entry),
    });
    pid as u64
}

static SCHEDULER_CONTEXT: Idc<Context> = Idc::new(Context::zeroed());

pub fn run_scheduler(quanta: u64) -> ! {
    loop {
        let pid = current_pid() as usize;
        let process = &mut PROCESSES.get()[pid];
        println!("PROC TEST PID {}({:?})", pid, process.state);
        if process.can_run() {
            process.state = ProcessState::Running;
            println!("PROC START PID {}", pid);
            timer::schedule(quanta);
            irq::enable();
            unsafe {
                switch(SCHEDULER_CONTEXT.get(), &process.context);
            }
            irq::disable();
            println!("PROC END PID {}({:?})", pid, process.state);
            if let ProcessState::Running = process.state {
                process.state = ProcessState::Idle;
            }
        } else {
            println!("PROC SKIP PID {}", pid);
        }
        let process_count = PROCESSES.get().len();
        unsafe {
            CURRENT_PID = ((pid + 1) % process_count) as u64;
        }
    }
}

pub fn current_pid() -> u64 {
    unsafe { CURRENT_PID }
}

pub fn sleep(duration: u64) {
    PROCESSES.get()[current_pid() as usize].state = ProcessState::Sleeping {
        start: timer::current_time(),
        duration,
    };
    yield_self();
}

pub fn yield_self() {
    unsafe {
        let process = &mut PROCESSES.get()[CURRENT_PID as usize];
        switch(&raw mut process.context, SCHEDULER_CONTEXT.get());
    }
}

pub fn wait_irq() {
    unsafe { asm!("wfi") }
}
