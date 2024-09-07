.section .text
.globl timervec
timervec:

# Temporarily store a1..a3 into the timer scratch's register buffer
csrrw a0, mscratch, a0
sd a1, 0(a0)
sd a2, 8(a0)
sd a3, 16(a0)

# Load mtimercmp's address and value into a1 and a2, respectively
ld a1, 24(a0)
ld a2, 0(a1)

# Load the interval into a3
ld a3, 32(a0)

# Schedule the next interrupt
add a2, a2, a3
sd a2, 0(a1)

# Arrange an supervisor-mode interrupt
li a1, 2
csrw sip, a1

# Restore registers
ld a3, 16(a0)
ld a2, 8(a0)
ld a1, 0(a0)
csrrw a0, mscratch, a0

# Return to supervisor-mode and trigger the arranged interrupt
mret