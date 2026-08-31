/*
PHYSICAL_FLASH_SIZE = the entire 2 MiB physical flash.
LSA_LOG_SIZE = the 128 KiB you've reserved for your log.
LSA_LOG_START = the address where that reserved region begins.

0x1000_0000 ┌─────────────────────┐
            │                     │
            │     Application     │
            │                     │
0x101E_0000 ├─────────────────────┤
            │                     │
            │      LSA log        │ 128 KiB
            │                     │
0x1020_0000 └─────────────────────┘
*/
PROVIDE(__physical_flash_start = 0x10000000);
PROVIDE(__physical_flash_end   = 0x10200000);

PROVIDE(__lsa_log_start = 0x101E0000);
PROVIDE(__lsa_log_end   = 0x10200000);

MEMORY {
    /* RP2350 standard 2MB bootable Flash region */
    FLASH : ORIGIN = 0x10000000, LENGTH = 1920K

    /* Primary striped working memory block required by cortex-m-rt */
    RAM   : ORIGIN = 0x20000000, LENGTH = 512K

    /* Auxiliary unstriped execution blocks */
    SRAM8 : ORIGIN = 0x20080000, LENGTH = 4K
    SRAM9 : ORIGIN = 0x20081000, LENGTH = 4K
}

SECTIONS {
    /* ### Boot ROM info
     * Goes after .vector_table, to keep it in the first 4K of flash
     * where the Boot ROM (and picotool) can find it.
     */
    .start_block : ALIGN(4)
    {
        __start_block_addr = .;
        KEEP(*(.start_block));
        KEEP(*(.boot_info));
    } > FLASH

} INSERT AFTER .vector_table;

/* move .text to start /after/ the boot info block */
_stext = ADDR(.start_block) + SIZEOF(.start_block);

SECTIONS {
    /* ### Picotool 'Binary Info' Entries
     * Picotool looks through this block to locate specific metadata headers.
     */
    .bi_entries : ALIGN(4)
    {
        __bi_entries_start = .;
        KEEP(*(.bi_entries));
        . = ALIGN(4);
        __bi_entries_end = .;
    } > FLASH
} INSERT AFTER .text;

SECTIONS {
    /* ### Boot ROM extra info
     * Goes after everything in our program, so it can contain a signature block.
     */
    .end_block : ALIGN(4)
    {
        __end_block_addr = .;
        KEEP(*(.end_block));
        __flash_binary_end = .;
    } > FLASH

} INSERT AFTER .uninit;

/* Provide the required memory tracker offsets */
PROVIDE(start_to_end = __end_block_addr - __start_block_addr);
PROVIDE(end_to_start = __start_block_addr - __end_block_addr);
