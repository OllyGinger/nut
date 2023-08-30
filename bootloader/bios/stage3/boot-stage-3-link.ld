ENTRY(_start)

SECTIONS {
    . = 0x00130000;

    .start : {
        *(.start)
    }
    .text : {
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
    }
    .data : {
        *(.data .data.*)
    }
    .bss : {
        *(.bss .bss.*)
    }
}
