# M-Mode entry point for -bios none boot (RV64 version)
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
    sd zero, 0(t0)
    addi t0, t0, 8
    j 1b
2:
    mret

    .section .text.m_trap
    .globl m_trap_vector
    .align 4
m_trap_vector:
    csrrw sp, mscratch, sp
    addi sp, sp, -272
    sd ra, 0(sp)
    sd t0, 8(sp)
    sd t1, 16(sp)
    sd t2, 24(sp)
    sd t3, 32(sp)
    sd t4, 40(sp)
    sd t5, 48(sp)
    sd t6, 56(sp)
    sd a0, 64(sp)
    sd a1, 72(sp)
    sd a2, 80(sp)
    sd a3, 88(sp)
    sd a4, 96(sp)
    sd a5, 104(sp)
    sd a6, 112(sp)
    sd a7, 120(sp)
    csrr t0, mscratch
    sd t0, 128(sp)

    call m_trap_handler

    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0

    ld t0, 128(sp)
    csrw mscratch, t0
    ld ra, 0(sp)
    ld t0, 8(sp)
    ld t1, 16(sp)
    ld t2, 24(sp)
    ld t3, 32(sp)
    ld t4, 40(sp)
    ld t5, 48(sp)
    ld t6, 56(sp)
    ld a2, 80(sp)
    ld a3, 88(sp)
    ld a4, 96(sp)
    ld a5, 104(sp)
    ld a6, 112(sp)
    ld a7, 120(sp)
    addi sp, sp, 272
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

