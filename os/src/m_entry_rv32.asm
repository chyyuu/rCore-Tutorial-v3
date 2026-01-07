# M-Mode entry point for RV32 (-bios none boot)
# This code runs in M-Mode and sets up the environment for S-Mode

.section .text.m_entry
.globl _m_start
_m_start:
    # Set up M-Mode stack
    la sp, m_stack_top
    csrw mscratch, sp

    # Set mstatus: MPP=S (01), set MPIE
    li t0, (1 << 11) | (1 << 7)
    csrw mstatus, t0

    # Set mepc to S-Mode entry point
    la t0, _start
    csrw mepc, t0

    # Set mtvec to M-Mode trap handler
    la t0, m_trap_vector
    csrw mtvec, t0

    # Delegate all interrupts and exceptions to S-Mode
    # except for M-Mode ecall (needed for SBI calls)
    li t0, 0xffff
    csrw mideleg, t0
    
    # Delegate all exceptions except ecall from S-mode
    li t0, 0xffff
    li t1, (1 << 9)      # Environment call from S-mode
    not t1, t1
    and t0, t0, t1
    csrw medeleg, t0

    # Set up PMP to allow S-Mode full access to memory
    # PMP entry 0: full access to all memory
    li t0, -1            # pmpaddr0 = 0xFFFFFFFF (all memory)
    csrw pmpaddr0, t0
    li t0, 0x0f          # pmpcfg0: A=TOR, R=1, W=1, X=1
    csrw pmpcfg0, t0

    # Enable counter access for S-Mode
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

    # Jump to S-Mode
    mret

# M-Mode trap handler
.section .text.m_trap
.globl m_trap_vector
.align 4
m_trap_vector:
    # Switch to M-Mode stack
    csrrw sp, mscratch, sp

    # Save context
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

    # Call Rust handler
    # Arguments are already in a0-a7
    call m_trap_handler

    # Return values in a0, a1 (SbiRet)

    # Advance mepc past ecall instruction
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0

    # Restore original sp to mscratch
    lw t0, 64(sp)
    csrw mscratch, t0

    # Restore context (except a0, a1 which have return values)
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

    addi sp, sp, 136

    # Restore sp
    csrrw sp, mscratch, sp

    mret
