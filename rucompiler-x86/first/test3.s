.text
.global foo
foo:
    pushq %rbp
    movq %rsp, %rbp
    pushq %rbx
    movq $8, %rax
    pushq %rax
    movq %rsi, %rax
    popq %rbx
    imulq %rbx, %rax
    pushq %rax
    movq $3, %rax
    pushq %rax
    movq $3, %rax
    pushq %rax
    movq $3, %rax
    pushq %rax
    movq %rdi, %rax
    popq %rbx
    addq %rbx, %rax
    popq %rbx
    imulq %rbx, %rax
    popq %rbx
    addq %rbx, %rax
    popq %rbx
    addq %rbx, %rax
    popq %rbx
    popq %rbp
    ret
.section .note.GNU-stack,"",@progbits
