.altmacro
.macro SAVE_GP n
    sw x\n, \n*4(sp)
.endm
.macro LOAD_GP n
    lw x\n, \n*4(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # now sp->kernel stack, sscratch->user stack
    # allocate a TrapContext on kernel stack
    addi sp, sp, -34*4
    # save general-purpose registers
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
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus
    csrr t1, sepc
    sw t0, 32*4(sp)
    sw t1, 33*4(sp)
    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch
    sw t2, 2*4(sp)
    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp
    call trap_handler

__restore:
    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore sstatus/sepc
    lw t0, 32*4(sp)
    lw t1, 33*4(sp)
    lw t2, 2*4(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # restore general-purpuse registers except sp/tp
    lw x1, 1*4(sp)
    lw x3, 3*4(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*4
    # now sp->kernel stack, sscratch->user stack
    csrrw sp, sscratch, sp
    sret
