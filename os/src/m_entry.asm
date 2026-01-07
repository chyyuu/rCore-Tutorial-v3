# M-Mode entry point for -bios none boot
# This code runs at 0x80000000 in M-Mode when QEMU starts with -bios none
# It sets up the environment and jumps to S-Mode

    .section .text.m_entry
    .globl _m_start
_m_start:
    # Set up M-Mode stack
    la sp, m_stack_top
    # Save M-Mode sp to mscratch for trap handler
    csrw mscratch, sp

    # Set mstatus: MPP=01 (S-Mode), MPIE=1
    # When mret executes, we'll be in S-Mode with interrupts enabled
    li t0, (1 << 11) | (1 << 7)  # MPP=S-Mode(01), MPIE=1
    csrw mstatus, t0

    # Set mepc to S-Mode entry point
    la t0, _start
    csrw mepc, t0

    # Set mtvec to M-Mode trap handler
    la t0, m_trap_vector
    csrw mtvec, t0

    # Delegate all interrupts and exceptions to S-Mode except ecall from S-Mode
    # We need to handle ecall from S-Mode (SBI calls) in M-Mode
    li t0, 0xffff
    csrw mideleg, t0      # Delegate all interrupts to S-Mode
    li t0, 0xffff
    li t1, (1 << 9)       # Exception 9: ecall from S-Mode
    not t1, t1
    and t0, t0, t1
    csrw medeleg, t0      # Delegate all exceptions except ecall from S-Mode

    # Set up Physical Memory Protection (PMP) to allow S-Mode full access
    # Configure PMP entry 0 to allow all access to all memory
    # Use TOR (Top of Range) mode with max address
    li t0, -1             # All 1s = max address
    csrw pmpaddr0, t0
    li t0, 0x0f           # R=1, W=1, X=1, A=TOR(01), L=0
    csrw pmpcfg0, t0

    # Enable S-Mode to access counters
    li t0, -1
    csrw mcounteren, t0

    # Clear bss for M-Mode
    la t0, m_sbss
    la t1, m_ebss
1:
    bgeu t0, t1, 2f
    sd zero, 0(t0)
    addi t0, t0, 8
    j 1b
2:

    # Jump to S-Mode
    mret

    .section .text.m_trap
    .globl m_trap_vector
    .align 4
m_trap_vector:
    # Save S-Mode sp to mscratch and switch to M-Mode stack
    csrrw sp, mscratch, sp
    
    # Save context on M-Mode stack
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
    # Save S-Mode sp (now in mscratch)
    csrr t0, mscratch
    sd t0, 128(sp)

    # Call Rust trap handler
    # a0-a7 contain the SBI call arguments
    # a7 = extension ID (EID)
    # a6 = function ID (FID)
    call m_trap_handler

    # Advance mepc past ecall instruction (4 bytes)
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0

    # Restore S-Mode sp to mscratch first (we'll swap it back at the end)
    ld t0, 128(sp)
    csrw mscratch, t0
    
    # Restore context
    ld ra, 0(sp)
    ld t0, 8(sp)
    ld t1, 16(sp)
    ld t2, 24(sp)
    ld t3, 32(sp)
    ld t4, 40(sp)
    ld t5, 48(sp)
    ld t6, 56(sp)
    # a0, a1 contain the return value, don't restore
    ld a2, 80(sp)
    ld a3, 88(sp)
    ld a4, 96(sp)
    ld a5, 104(sp)
    ld a6, 112(sp)
    ld a7, 120(sp)
    addi sp, sp, 272
    
    # Swap sp and mscratch: sp gets S-Mode sp, mscratch gets M-Mode sp
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

