/*
    // © 2023 Lilly & GeNT Developers archaic.archea@gmail.com
    // License: [CNPL](https://git.pixie.town/thufie/npl-builder/raw/branch/main/cnpl.md)
*/

ENTRY(start)

SECTIONS
{
    . = 0xffffffff80000000;
    
    PROVIDE(__exec_start = .);

    .text : ALIGN(4k) {
        PROVIDE(__text_start = .);
        *(.initext)
        *(.text .text.*)
        PROVIDE(__text_end = .);
    }

    .rodata                 : ALIGN(4k) { *(.rodata .rodata.*) }
    .eh_frame_hdr           : ALIGN(4k) { KEEP(*(.eh_frame_hdr)) }
    PROVIDE(__eh_frame = .);
    .eh_frame               : ALIGN(4k) { KEEP(*(.eh_frame)) }
    .dynsym                 : ALIGN(4k) { *(.dynsym) }
    .dynstr                 : ALIGN(4k) { *(.dynstr) }
    .rela                   : ALIGN(4k) { *(.rela*) }
    .dynamic                : ALIGN(4k) { *(.dynamic) }
    .got                    : ALIGN(4K) { *(.got) }

    .tdata                   : ALIGN(4K) { 
        PROVIDE(__tdata_start = .);
        *(.tdata .tdata.*) 
    }
    .tbss                    : ALIGN(4K) { 
        *(.tbss .tbss.*) 
        PROVIDE(__tdata_end = .);
    }

    .data.rel.ro            : ALIGN(4K) { *(.data.rel.ro*) }
    .data                   : ALIGN(4K) { *(.data .data.*) }
    PROVIDE(__global_pointer = .);
    .bss                    : ALIGN(4K) {
        *(.dynbss)
        *(.bss .bss.*)
        *(COMMON)
        . += 0x100000;
        . = ALIGN(16);
        PROVIDE(__stack_top = .);
    }

    PROVIDE(__exec_end = .);
}
