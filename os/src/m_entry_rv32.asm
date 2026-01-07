# M-Mode entry point for RV32 (-bios none boot)
.section .text.m_entry
.globl _m_start
_m_start:
    # Set up M-Mode stack
    la sp, m_stack_top
    csrw mscratch, sp

    # Set mstatus: MPP=S (01), MPIE=1
    li t0, (1 << 11) | (1 << 7)
    csrw mstatus, t0

    # Set mepc to S-Mode entry point
    la t0, _start
    csrw mepc, t0

    # Set mtvec to M-Mode trap handler
    la t0, m_trap_vector
    csrw mtvec, t0

    # Delegate interrupts to S-Mode (except machine interrupts)
    li t0, 0xffff
    csrw mideleg, t0

    # Delegate exceptions to S-Mode (except ecall from S-Mode)
    li t0, 0xffff
    li t1, (1 << 9)  # Environment call from S-mode
    not t1, t1
    and t0, t0, t1
    csrw medeleg, t0

    # Set PMP to allow all access
    li t0, -1
    csrw pmpaddr0, t0
    li t0, 0x0f
    csrw pmpcfg0, t0

    # Enable access to all counters from S-Mode
    li t0, -1
    csrw mcounteren, t0

    # Clear M-Mode BSS
    la t0, m_sbss
    la t1, m_ebss
1:
    bgeu t0, t1, 2f
    sw zero, 0(t0)
    addi t0, t0, 4
    j 1b
2:
    # Return to S-Mode
    mret

# M-Mode trap handler
.section .text.m_trap
.globl m_trap_vector
.align 4
m_trap_vector:
    # Swap sp with mscratch
    csrrw sp, mscratch, sp

    # Save context (136 bytes for RV32)
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

    # Save original sp
    csrr t0, mscratch
    sw t0, 64(sp)

    # Prepare arguments for m_trap_handler
    # Rust function signature: m_trap_handler(a0, a1, a2, a3, a4, a5, eid, fid)
    # SBI convention: a7=eid, a6=fid, a0-a5=args
    # We need to pass: a0, a1, a2, a3, a4, a5, a7 (eid), a6 (fid)
    
    # a0-a5 are already in place
    # Move a7 (eid) to position 6 and a6 (fid) to position 7
    mv t0, a6          # Save fid temporarily
    mv a6, a7          # eid goes to arg6
    mv a7, t0          # fid goes to arg7

    # Call Rust handler
    call m_trap_handler

    # Advance mepc past ecall
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0

    # Restore original sp to mscratch
    lw t0, 64(sp)
    csrw mscratch, t0

    # Restore context
    lw ra, 0(sp)
    lw t0, 4(sp)
    lw t1, 8(sp)
    lw t2, 12(sp)
    lw t3, 16(sp)
    lw t4, 20(sp)
    lw t5, 24(sp)
    lw t6, 28(sp)
    # a0, a1 contain return values from handler
    lw a2, 40(sp)
    lw a3, 44(sp)
    lw a4, 48(sp)
    lw a5, 52(sp)
    lw a6, 56(sp)
    lw a7, 60(sp)

    addi sp, sp, 136

    # Swap sp back
    csrrw sp, mscratch, sp

    mret
