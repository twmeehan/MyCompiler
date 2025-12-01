.text
.global foo
foo:
    pushq %rbp
    movq %rsp, %rbp
    pushq %rbx
    movq %r9, %rax
    pushq %rax
    movq %r8, %rax
    pushq %rax
    movq %rcx, %rax
    pushq %rax
    movq $8, %rax
    pushq %rax
    movq %rdx, %rax
    pushq %rax
    movq %rdx, %rax
    pushq %rax
    movq %rsi, %rax
    popq %rbx
    addq %rbx, %rax
    pushq %rax
    movq $5, %rax
    pushq %rax
    movq %rdi, %rax
    pushq %rax
    movq $900, %rax
    popq %rbx
    addq %rbx, %rax
    popq %rbx
    imulq %rbx, %rax
    popq %rbx
    imulq %rbx, %rax
    popq %rbx
    addq %rbx, %rax
    popq %rbx
    imulq %rbx, %rax
    popq %rbx
    imulq %rbx, %rax
    popq %rbx
    imulq %rbx, %rax
    pushq %rax
    movq %rdi, %rax
    pushq %rax
    movq $67, %rax
    pushq %rax
    movq %rdi, %rax
    pushq %rax
    movq $900, %rax
    popq %rbx
    addq %rbx, %rax
    popq %rbx
    imulq %rbx, %rax
    popq %rbx
    addq %rbx, %rax
    popq %rbx
    addq %rbx, %rax
    popq %rbx
    addq %rbx, %rax
    popq %rbx
    popq %rbp
    ret
.section .note.GNU-stack,"",@progbits
