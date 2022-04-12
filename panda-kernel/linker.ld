ENTRY(_start)

SECTIONS {
  . = 0xC0000000;

  .text : ALIGN(2097152)
  {
    KEEP(*(.text.start))
    *(.text.*)
  }

  .rodata : ALIGN(2097152)
  {
    *(.rodata.*)
  }

  .data : ALIGN(2097152)
  {
    *(.data.*)
  }

  .bss : ALIGN(2097152)
  {
    *(.bss.*)
  }

  .got : ALIGN(2097152)
  {
    *(.got.*)
  }

  /DISCARD/ :
  {
    *(.note.gnu.*)
    *(.eh_frame_hdr)
    *(.eh_frame) 
  }
}
