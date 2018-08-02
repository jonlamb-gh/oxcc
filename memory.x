MEMORY
{
    /* NOTE K = KiBi = 1024 bytes */

    /* STM32F767ZI 2 MByte FLASH, 512 KByte RAM */
    FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 2048K
    RAM (xrw)  : ORIGIN = 0x20000000, LENGTH = 512K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* You may want to use this variable to locate the call stack and static
   variables in different memory regions. Below is shown the default value */
/* _stack_start = ORIGIN(RAM) + LENGTH(RAM); */

/* You can use this symbol to customize the location of the .text section */
/* If omitted the .text section will be placed right after the .vector_table
   section */
/* This is required only on microcontrollers that store some configuration right
   after the vector table */
/* _stext = ORIGIN(FLASH) + 0x400; */

/* Size of the heap (in bytes) */
/* _heap_size = 1024; */
