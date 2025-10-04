use core::{
    arch::{asm, naked_asm},
    mem::MaybeUninit,
    ptr::addr_of_mut,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{irq, println, timer, utils::sync::RWCell};

#[repr(C)]
#[derive(Debug)]
pub struct Context {
    ra: u64,
    sp: u64,
    gp: u64,
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
            t: [0; 7],
            s: [0; 12],
            a: [0; 8],
        }
    }

    pub const fn zeroed() -> Context {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

unsafe extern "C" fn switch(from: *mut Context, to: *const Context) {
    assert_eq!(size_of::<Context>(), 8 * 30);
    println!("CTX SWITCH ({:?} -> {:?})", from, to);
    unsafe { _switch(from, to) };
}

#[unsafe(naked)]
unsafe extern "C" fn _switch(from: *mut Context, to: *const Context) {
    naked_asm!(
        // Save callee-saved registers to current
        "sd  ra,  0*8(a0)",
        "sd  sp,  1*8(a0)",
        "sd  gp,  2*8(a0)",
        "sd  t0,  3*8(a0)",
        "sd  t1,  4*8(a0)",
        "sd  t2,  5*8(a0)",
        "sd  t3,  6*8(a0)",
        "sd  t4,  7*8(a0)",
        "sd  t5,  8*8(a0)",
        "sd  t6,  9*8(a0)",
        "sd  s0, 10*8(a0)",
        "sd  s1, 11*8(a0)",
        "sd  s2, 12*8(a0)",
        "sd  s3, 13*8(a0)",
        "sd  s4, 14*8(a0)",
        "sd  s5, 15*8(a0)",
        "sd  s6, 16*8(a0)",
        "sd  s7, 17*8(a0)",
        "sd  s8, 18*8(a0)",
        "sd  s9, 19*8(a0)",
        "sd s10, 20*8(a0)",
        "sd s11, 21*8(a0)",
        "sd  a0, 22*8(a0)",
        "sd  a1, 23*8(a0)",
        "sd  a2, 24*8(a0)",
        "sd  a3, 25*8(a0)",
        "sd  a4, 26*8(a0)",
        "sd  a5, 27*8(a0)",
        "sd  a6, 28*8(a0)",
        "sd  a7, 29*8(a0)",
        // Load from next
        "ld  ra,  0*8(a1)",
        "ld  sp,  1*8(a1)",
        "ld  gp,  2*8(a1)",
        "ld  t0,  3*8(a1)",
        "ld  t1,  4*8(a1)",
        "ld  t2,  5*8(a1)",
        "ld  t3,  6*8(a1)",
        "ld  t4,  7*8(a1)",
        "ld  t5,  8*8(a1)",
        "ld  t6,  9*8(a1)",
        "ld  s0, 10*8(a1)",
        "ld  s1, 11*8(a1)",
        "ld  s2, 12*8(a1)",
        "ld  s3, 13*8(a1)",
        "ld  s4, 14*8(a1)",
        "ld  s5, 15*8(a1)",
        "ld  s6, 16*8(a1)",
        "ld  s7, 17*8(a1)",
        "ld  s8, 18*8(a1)",
        "ld  s9, 19*8(a1)",
        "ld s10, 20*8(a1)",
        "ld s11, 21*8(a1)",
        "ld  a0, 22*8(a1)",
        "ld  a2, 24*8(a1)",
        "ld  a3, 25*8(a1)",
        "ld  a4, 26*8(a1)",
        "ld  a5, 27*8(a1)",
        "ld  a6, 28*8(a1)",
        "ld  a7, 29*8(a1)",
        "ld  a1, 24*8(a1)",
        "ret",
    );
}

pub type PID = u64;

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessState {
    Free,
    Idle,
    Running,
    Sleeping { start: u64, duration: u64 },
}

const MAX_PROCESSES: usize = 2;
const PROCESS_STACK_SIZE: usize = 8 * 1024;

#[unsafe(link_section = ".stack.processes")]
static mut STACKS: [[u8; PROCESS_STACK_SIZE]; MAX_PROCESSES] =
    [[0; PROCESS_STACK_SIZE]; MAX_PROCESSES];
static SCHEDULER_CONTEXT: RWCell<Context> = RWCell::new("SCHED_CTX", Context::zeroed());
static CURRENT_PID: RWCell<PID> = RWCell::new("CPID", 0);
pub static PROCESSES: Processes = Processes::new();

#[derive(Debug)]
pub struct Process {
    state: RWCell<ProcessState>,
    context: RWCell<Context>,
}

impl Process {
    pub const fn uninit() -> Self {
        Self {
            state: RWCell::new("PROC_STATE", ProcessState::Free),
            context: RWCell::new("PROC_CTX", Context::zeroed()),
        }
    }

    pub fn init(&self, stack: &mut [u8; PROCESS_STACK_SIZE], entry: fn()) {
        let mut state = self.state.write();
        assert_eq!(
            *state,
            ProcessState::Free,
            "tried to initialize an already initialized process"
        );
        let sp = addr_of_mut!(stack[PROCESS_STACK_SIZE - 1]);
        self.context.set(Context::new(sp, entry));
        *state = ProcessState::Idle;
    }

    pub fn is_free(&self) -> bool {
        matches!(self.state.try_read(), Ok(state) if *state == ProcessState::Free)
    }

    pub fn can_run(&self) -> bool {
        match *self.state.read() {
            ProcessState::Free => false,
            ProcessState::Idle => true,
            ProcessState::Running => false,
            ProcessState::Sleeping { start, duration } => timer::current_time() > start + duration,
        }
    }
}

pub struct Processes {
    buffer: [Process; MAX_PROCESSES],
    len: AtomicUsize,
}

impl Processes {
    pub const fn new() -> Self {
        Self {
            buffer: [const { Process::uninit() }; MAX_PROCESSES],
            len: AtomicUsize::new(0),
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Acquire)
    }

    pub fn create(&self, entry: fn()) -> PID {
        let (pid, process) = self
            .buffer
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_free())
            .next()
            .expect("process limit unreached");
        self.len.fetch_add(1, core::sync::atomic::Ordering::Release);
        let stack = unsafe { &mut STACKS[pid] };
        process.init(stack, entry);
        return pid as PID;
    }

    pub fn get(&self, pid: PID) -> &Process {
        &self.buffer[pid as usize]
    }

    pub fn current(&self) -> &Process {
        self.get(current_pid())
    }
}

pub fn run_scheduler(quanta: u64) -> ! {
    loop {
        let pid = current_pid();
        let process = PROCESSES.get(pid);
        println!("PROC TEST PID {} ({:?})", pid, process.state);
        if process.can_run() {
            process.state.set(ProcessState::Running);
            println!("PROC START PID {}", pid);
            timer::schedule(quanta);
            let from = SCHEDULER_CONTEXT.get_mut_ptr();
            let to = process.context.get_ptr();
            irq::enable();
            unsafe { switch(from, to) };
            irq::disable();
            println!("PROC END PID {} ({:?})", pid, process.state);
            let mut state = process.state.write();
            if let ProcessState::Running = *state {
                *state = ProcessState::Idle;
            }
        } else {
            println!("PROC SKIP PID {}", pid);
        }
        let process_count = PROCESSES.len() as u64;
        CURRENT_PID.set((pid + 1) % process_count);
    }
}

pub fn current_pid() -> PID {
    CURRENT_PID.get()
}

pub fn sleep(duration: u64) {
    PROCESSES.current().state.set(ProcessState::Sleeping {
        start: timer::current_time(),
        duration,
    });
    yield_self();
}

pub fn yield_self() {
    let process = PROCESSES.current();
    let from = process.context.get_mut_ptr();
    let to = SCHEDULER_CONTEXT.get_ptr();
    unsafe { switch(from, to) };
}

pub fn wait_irq() {
    unsafe { asm!("wfi") }
}
