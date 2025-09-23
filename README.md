# Obligatory Rewrite-it-in-Rust for xv6

I like operating systems and low-level system development. I felt like rewriting [xv6](https://github.com/mit-pdos/xv6-public) in Rust would be fun. That's how this came to be. What else would I need?

## Features

- [ ] Process management;
- [ ] Scheduling;
- [ ] Virtual memory;
- [ ] Memory allocation;
- [ ] File system;
- [ ] Disk I/O;
- [ ] Console I/O;

- Prototype 1:
    - [x] Serial IO through UART;
    - [x] Interrupts;
    - [ ] Threads;
    - [x] Timer;
- Prototype 2:
    - [ ] Multitasking;
    - [ ] Memory allocator;
    - [ ] Userspace;
    - [ ] Basic Syscalls (read, write, sleep, fork, exit);
- Prototype 3:
    - [ ] Filesystem;
    - [ ] Basic shell;
    - [ ] Multicore;
    - [ ] FS Syscalls (open, close, fstat, opendir, readdir, closedir, exec);
