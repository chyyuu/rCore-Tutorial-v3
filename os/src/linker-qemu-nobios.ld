/* Linker script for -bios none mode */
/* M-Mode code starts at 0x80000000, S-Mode at 0x80200000 */

OUTPUT_ARCH(riscv)
ENTRY(_m_start)
BASE_ADDRESS = 0x80000000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    /* M-Mode entry code - must be first */
    .text.m_entry : {
        *(.text.m_entry)
    }

    /* M-Mode trap handler */
    .text.m_trap : {
        *(.text.m_trap)
    }

    /* Pad to 0x80200000 for S-Mode entry */
    . = ALIGN(4K);

    /* S-Mode code */
    stext = .;
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        strampoline = .;
        *(.text.trampoline)
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

    /* M-Mode stack (4KB) */
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


