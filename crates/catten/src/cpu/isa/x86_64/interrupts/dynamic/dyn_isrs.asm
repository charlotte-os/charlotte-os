.code64

.extern dispatch_dynamic_interrupt
.extern increment_interrupt_depth
.extern decrement_interrupt_depth
.extern cond_yield_lp
.extern signal_eoi
.extern DYN_VECS_PER_LP
.extern DYN_VEC_START_OFFSET

.macro m_dyn_isr vector:req
.global dyn_isr_\vector
dyn_isr_\vector:
//; Save RFLAGS and caller saved registers
    pushfq
    push rax
    push rdi
    push rsi
    push rdx
    push rcx
    push r8
    push r9
    push r10
    push r11
//; Ensure the stack is 16 byte aligned
    push rbp
    mov rbp, rsp
    and rsp, ~0xf
    cld
    call increment_interrupt_depth
    mov rdi, \vector
    call dispatch_dynamic_interrupt
//; Signal EOI since these interrupts all come through the LAPIC
    call signal_eoi
    call decrement_interrupt_depth
//; Execute context switch if pending
    call cond_yield_lp
//; Restore the stack pointer to the value before alignment correction
    mov rsp, rbp
    pop rbp
//; Restore caller saved registers and RFLAGS
    pop r11
    pop r10
    pop r9
    pop r8
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    pop rax
    popfq
//; Exit ISR
    iretq
.endm

.section .text
.altmacro
.set vector_num, 0
.rept 220
    m_dyn_isr %vector_num
    .set vector_num, vector_num+1
.endr
.noaltmacro
