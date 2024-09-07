.equ STACK_SIZE, 4096
.equ MAX_CPU, 8

.section .text
.globl _entry
_entry:
  # Setup rust stack
  la sp, STACK
  li a0, STACK_SIZE
  csrr a1, mhartid
  addi a1, a1, 1
  mul a0, a0, a1

  # Call main function
  call main

.section .data

# TODO: implement stack in Rust
.align 4
STACK:
.space STACK_SIZE * MAX_CPU # 4096 is the stack size

