.code64

.macro IPI_PROLOGUE
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    pushfq
/* IPIs are always level triggered so as not to be missed. Thus when the ISR runs we must signal
   end of interrupt to the local interrupt controller to ensure the ISR isn't called repeatedly
   ad infinitum
*/
    call signal_eoi
.endm

.macro IPI_EPILOGUE
    popfq
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax
.endm

.section .data
.global sync_ipi_barrier
sync_ipi_barrier:
    .8byte 0

.section .text
.global isr_ipi_doorbell
isr_ipi_doorbell:
    IPI_PROLOGUE
    call ih_asynchronous_ipi
    IPI_EPILOGUE
    iretq
