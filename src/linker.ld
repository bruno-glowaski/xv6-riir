MEMORY
{
  RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 8M
}

OUTPUT_ARCH("riscv")
ENTRY(_entry)

__stack_size = 16K;

SECTIONS
{
  . = ORIGIN(RAM);

  .text : ALIGN(4K) {
    *(.text._entry)
    *(.text*)
  } > RAM

  .rodata : ALIGN(4K) {
    *(.rodata*)
  } > RAM

  . = ALIGN(0x1000);
  PROVIDE(etext = .);

  .data : ALIGN(4K) {
    *(.data*)
  } > RAM

  .bss : ALIGN(4K) {
    __bss_start = .;
    *(.bss*)
    *(COMMON)
    __bss_end = .;
  } > RAM

  .stack (NOLOAD) : ALIGN(16) {
    __stack_start = .;
    . = . + __stack_size;
    __stack_end = .;
  } > RAM
}
