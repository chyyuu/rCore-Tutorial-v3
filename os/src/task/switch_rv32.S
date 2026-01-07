# Task switch assembly for RV32

.altmacro
.macro SAVE_SN n
    sw s\n, (\n+2)*4(a0)
.endm
.macro LOAD_SN n
    lw s\n, (\n+2)*4(a1)
.endm

    .section .text
    .globl __switch
__switch:
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )
    # save kernel stack of current task
    sw sp, 4(a0)
    # save ra & s0~s11 of current execution
    sw ra, 0(a0)
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    # restore ra & s0~s11 of next execution
    lw ra, 0(a1)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # restore kernel stack of next task
    lw sp, 4(a1)
    ret

