# M-Mode entry point and trap handler for RV32
# This is used when booting with -bios none

    .section .text.m_entry
    .globl _m_start
_m_start:
    # Set up M-mode stack
    la sp, m_stack_top
    csrw mscratch, sp

    # Set mstatus: MPP=S-mode (01), MPIE=1
    li t0, (1 << 11) | (1 << 7)
    csrw mstatus, t0

    # Set mepc to S-mode entry point
    la t0, _start
    csrw mepc, t0

    # Set up M-mode trap vector
    la t0, m_trap_vector
    csrw mtvec, t0

    # Delegate interrupts to S-mode (except timer which we handle)
    li t0, 0xffff
    csrw mideleg, t0

    # Delegate exceptions to S-mode (except ecall from S-mode)
    li t0, 0xffff
    li t1, (1 << 9)        # Don't delegate ecall from S-mode
    not t1, t1
    and t0, t0, t1
    csrw medeleg, t0

    # Set up PMP to allow S-mode full access to memory
    li t0, -1
    csrw pmpaddr0, t0
    li t0, 0x0f            # TOR, RWX
    csrw pmpcfg0, t0

    # Enable counters for S-mode
    li t0, -1
    csrw mcounteren, t0

    # Clear M-mode BSS
    la t0, m_sbss
    la t1, m_ebss
1:
    bgeu t0, t1, 2f
    sw zero, 0(t0)
    addi t0, t0, 4
    j 1b
2:
    # Jump to S-mode
    mret

    .section .text.m_trap
    .globl m_trap_vector
    .align 4
m_trap_vector:
    # Swap sp with mscratch (get M-mode stack)
    csrrw sp, mscratch, sp

    # Save context (18 registers * 4 bytes = 72 bytes, round up to 80)
    addi sp, sp, -80
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

    # Save S-mode sp
    csrr t0, mscratch
    sw t0, 64(sp)

    # Prepare arguments for m_trap_handler
    # Rust function signature: fn(a0, a1, a2, a3, a4, a5, eid, fid) -> SbiRet
    # Hardware state: a0-a5 are args, a7 is EID, a6 is FID
    # We need to pass: original a0-a5, then a7 (as eid), then a6 (as fid)

    # Load original a0-a5 from stack
    lw a0, 32(sp)
    lw a1, 36(sp)
    lw a2, 40(sp)
    lw a3, 44(sp)
    lw a4, 48(sp)
    lw a5, 52(sp)

    # Load original a7 (EID) into a6 position for Rust call
    lw a6, 60(sp)
    # Load original a6 (FID) into a7 position for Rust call
    lw a7, 56(sp)

    # Call the Rust trap handler
    call m_trap_handler

    # Return value is in a0 (error) and a1 (value)
    # These are already set correctly by the function return

    # Advance mepc past ecall instruction
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0

    # Restore S-mode sp to mscratch
    lw t0, 64(sp)
    csrw mscratch, t0

    # Restore caller-saved registers (except a0, a1 which have return values)
    lw ra, 0(sp)
    lw t0, 4(sp)
    lw t1, 8(sp)
    lw t2, 12(sp)
    lw t3, 16(sp)
    lw t4, 20(sp)
    lw t5, 24(sp)
    lw t6, 28(sp)
    # Skip a0, a1 - they contain return values
    lw a2, 40(sp)
    lw a3, 44(sp)
    lw a4, 48(sp)
    lw a5, 52(sp)
    lw a6, 56(sp)
    lw a7, 60(sp)

    # Restore stack pointer
    addi sp, sp, 80

    # Swap back to S-mode stack
    csrrw sp, mscratch, sp

    # Return to S-mode
    mret


