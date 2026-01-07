# M-Mode entry point for RV64 (-bios none boot)
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

    # Delegate all interrupts to S-Mode
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
    sd zero, 0(t0)
    addi t0, t0, 8
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

    # Save original sp
    csrr t0, mscratch
    sd t0, 128(sp)

    # Call Rust handler
    # Arguments are already in a0-a7
    call m_trap_handler

    # Return values in a0, a1 (SbiRet)

    # Advance mepc past ecall instruction
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0

    # Restore original sp to mscratch
    ld t0, 128(sp)
    csrw mscratch, t0

    # Restore context (except a0, a1 which have return values)
    ld ra, 0(sp)
    ld t0, 8(sp)
    ld t1, 16(sp)
    ld t2, 24(sp)
    ld t3, 32(sp)
    ld t4, 40(sp)
    ld t5, 48(sp)
    ld t6, 56(sp)
    # Skip a0, a1 - they contain return values
    ld a2, 80(sp)
    ld a3, 88(sp)
    ld a4, 96(sp)
    ld a5, 104(sp)
    ld a6, 112(sp)
    ld a7, 120(sp)

    addi sp, sp, 272

    # Restore sp
    csrrw sp, mscratch, sp

    mret
