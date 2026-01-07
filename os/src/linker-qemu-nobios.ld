OUTPUT_ARCH(riscv)
ENTRY(_m_start)

/* M-Mode starts at 0x80000000 (QEMU virt default with -bios none) */
M_BASE_ADDRESS = 0x80000000;
/* S-Mode kernel starts at 0x80200000 (same as before) */
S_BASE_ADDRESS = 0x80200000;

SECTIONS
{
    /* M-Mode code and data, starting at 0x80000000 */
    . = M_BASE_ADDRESS;
    m_skernel = .;

    .text.m_entry : {
        *(.text.m_entry)
    }

    .text.m_trap : {
        *(.text.m_trap)
    }

    . = ALIGN(4K);
    .bss.m_stack : {
        *(.bss.m_stack)
    }

    . = ALIGN(4K);
    .bss.m_data : {
        *(.bss.m_data)
    }

    . = ALIGN(4K);
    m_ekernel = .;

    /* S-Mode kernel, starting at 0x80200000 */
    . = S_BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
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
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}

