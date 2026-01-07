# Trap handling assembly for RV32

.altmacro
.macro SAVE_GP n
    sw x\n, \n*4(sp)
.endm
.macro LOAD_GP n
    lw x\n, \n*4(sp)
.endm

    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # now sp->*TrapContext in user space, sscratch->user stack
    # save other general purpose registers
    sw x1, 1*4(sp)
    # skip sp(x2), we will save it later
    sw x3, 3*4(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sw t0, 32*4(sp)
    sw t1, 33*4(sp)
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch
    sw t2, 2*4(sp)
    # load kernel_satp into t0
    lw t0, 34*4(sp)
    # load trap_handler into t1
    lw t1, 36*4(sp)
    # move to kernel_sp
    lw sp, 35*4(sp)
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    lw t0, 32*4(sp)
    lw t1, 33*4(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general purpose registers except x0/sp/tp
    lw x1, 1*4(sp)
    lw x3, 3*4(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    lw sp, 2*4(sp)
    sret


