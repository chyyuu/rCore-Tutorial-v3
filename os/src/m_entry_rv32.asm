# M-Mode entry point for -bios none boot (RV32 version)
# This code runs at 0x80000000 in M-Mode when QEMU starts with -bios none

    .section .text.m_entry
    .globl _m_start
_m_start:
    # Set up M-Mode stack
    la sp, m_stack_top
    # Save M-Mode sp to mscratch for trap handler
    csrw mscratch, sp

    # Set mstatus: MPP=01 (S-Mode), MPIE=1
    li t0, (1 << 11) | (1 << 7)
    csrw mstatus, t0

    # Set mepc to S-Mode entry point
    la t0, _start
    csrw mepc, t0

    # Set mtvec to M-Mode trap handler
    la t0, m_trap_vector
    csrw mtvec, t0

    # Delegate interrupts and exceptions to S-Mode except ecall from S-Mode
    li t0, 0xffff
    csrw mideleg, t0
    li t0, 0xffff
    li t1, (1 << 9)
    not t1, t1
    and t0, t0, t1
    csrw medeleg, t0

    # Set up PMP to allow S-Mode full access
    li t0, -1
    csrw pmpaddr0, t0
    li t0, 0x0f
    csrw pmpcfg0, t0

    # Enable S-Mode to access counters
    li t0, -1
    csrw mcounteren, t0

    # Clear M-Mode bss
    la t0, m_sbss
    la t1, m_ebss
1:
    bgeu t0, t1, 2f
    sw zero, 0(t0)
    addi t0, t0, 4
    j 1b
2:
    mret

    .section .text.m_trap
    .globl m_trap_vector
    .align 4
m_trap_vector:
    csrrw sp, mscratch, sp
    addi sp, sp, -136
    sw ra, 0(sp)
    sw t0, 4(sp)
    sw t1, 8(sp)
    sw t2, 12(sp)
    sw t3, 16(sp)
    sw t4, 20(sp)
    sw t5, 24(sp)
    sw t6, 28(sp)
    sw a0, 32(sp)
    sw a1, 36(sp)
    sw a2, 40(sp)
    sw a3, 44(sp)
    sw a4, 48(sp)
    sw a5, 52(sp)
    sw a6, 56(sp)
    sw a7, 60(sp)
    csrr t0, mscratch
    sw t0, 64(sp)

    call m_trap_handler

    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0

    lw t0, 64(sp)
    csrw mscratch, t0
    lw ra, 0(sp)
    lw t0, 4(sp)
    lw t1, 8(sp)
    lw t2, 12(sp)
    lw t3, 16(sp)
    lw t4, 20(sp)
    lw t5, 24(sp)
    lw t6, 28(sp)
    lw a2, 40(sp)
    lw a3, 44(sp)
    lw a4, 48(sp)
    lw a5, 52(sp)
    lw a6, 56(sp)
    lw a7, 60(sp)
    addi sp, sp, 136
    csrrw sp, mscratch, sp
    mret

    .section .bss.m_stack
    .globl m_stack_lower_bound
m_stack_lower_bound:
    .space 4096 * 4
    .globl m_stack_top
m_stack_top:

    .section .bss.m_data
    .globl m_sbss
m_sbss:
    .space 64
    .globl m_ebss
m_ebss:

