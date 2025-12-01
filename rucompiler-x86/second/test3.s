.text
.global foo
foo:
    pushq %rbp
    movq %rsp, %rbp
    pushq %rbx
    subq $24, %rsp
    movq %rdi, -8(%rbp)
    movq %rsi, -16(%rbp)
    movq -16(%rbp), %rax
    pushq %rax
    movq -8(%rbp), %rax
    popq %rbx
    cmpq %rbx, %rax
    ja if.then.0
    jmp if.else.1
if.then.0:
    movq -16(%rbp), %rax
    pushq %rax
    movq -8(%rbp), %rax
    popq %rbx
    subq %rbx, %rax
    movq %rax, -24(%rbp)
    jmp if.end.2
if.else.1:
    movq -8(%rbp), %rax
    pushq %rax
    movq -16(%rbp), %rax
    popq %rbx
    subq %rbx, %rax
    movq %rax, -24(%rbp)
    jmp if.end.2
if.end.2:
    movq -24(%rbp), %rax
    addq $24, %rsp
    popq %rbx
    popq %rbp
    ret
.section .note.GNU-stack,"",@progbits
