.text
.global foo
foo:
    pushq %rbp
    movq %rsp, %rbp
    pushq %rbx
    movq $2, %rax
    pushq %rax
    movq $4, %rax
    popq %rbx
    addq %rbx, %rax
    pushq %rax
    movq $1, %rax
    popq %rbx
    imulq %rbx, %rax
    pushq %rax
    movq $38, %rax
    pushq %rax
    movq $78, %rax
    pushq %rax
    movq $1, %rax
    pushq %rax
    movq $3, %rax
    popq %rbx
    imulq %rbx, %rax
    pushq %rax
    movq $4, %rax
    pushq %rax
    movq $2, %rax
    pushq %rax
    movq $3, %rax
    popq %rbx
    addq %rbx, %rax
    popq %rbx
    addq %rbx, %rax
    popq %rbx
    imulq %rbx, %rax
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
