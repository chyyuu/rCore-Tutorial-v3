OUTPUT_ARCH(riscv)
ENTRY(_m_start)
BASE_ADDRESS = 0x80000000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    /* M-Mode code at the very beginning */
    .text.m_entry : {
        *(.text.m_entry)
    }

    .text.m_trap : {
        *(.text.m_trap)
    }

    /* S-Mode code starts after M-Mode code */
    . = ALIGN(4K);
    stext = .;
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        strampoline = .;
        *(.text.trampoline);
        . = ALIGN(4K);
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    sbss_with_stack = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /* M-Mode stack */
    . = ALIGN(4K);
    m_stack = .;
    .bss.m_stack : {
        . += 4K;
        m_stack_top = .;
    }

    /* M-Mode BSS */
    . = ALIGN(4K);
    m_sbss = .;
    .bss.m_bss : {
        . += 4K;
        m_ebss = .;
    }

    /DISCARD/ : {
        *(.eh_frame)
    }
}
