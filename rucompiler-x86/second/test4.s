.text
.global foo
foo:
    pushq %rbp
    movq %rsp, %rbp
    pushq %rbx
    subq $24, %rsp
    movq %rdi, -8(%rbp)
    movq $0, -16(%rbp)
    movq $0, -24(%rbp)
    jmp while.cond.0
while.cond.0:
    movq -8(%rbp), %rax
    pushq %rax
    movq -16(%rbp), %rax
    popq %rbx
    cmpq %rbx, %rax
    jb while.body.1
    jmp while.end.2
while.body.1:
    movq -16(%rbp), %rbx
    cmpq $5, %rbx
    jb if.then.3
    jmp if.else.4
if.then.3:
    movq -16(%rbp), %rax
    pushq %rax
    movq -24(%rbp), %rax
    popq %rbx
    addq %rbx, %rax
    movq %rax, -24(%rbp)
    jmp if.end.5
if.else.4:
    movq -8(%rbp), %rax
    pushq %rax
    movq -24(%rbp), %rax
    popq %rbx
    addq %rbx, %rax
    movq %rax, -24(%rbp)
    jmp if.end.5
if.end.5:
    movq $1, %rax
    pushq %rax
    movq -16(%rbp), %rax
    popq %rbx
    addq %rbx, %rax
    movq %rax, -16(%rbp)
    jmp while.cond.0
while.end.2:
    movq -24(%rbp), %rax
    addq $24, %rsp
    popq %rbx
    popq %rbp
    ret
.section .note.GNU-stack,"",@progbits
