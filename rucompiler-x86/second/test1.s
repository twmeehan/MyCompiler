.text
.global foo
foo:
    pushq %rbp
    movq %rsp, %rbp
    pushq %rbx
    subq $24, %rsp
    movq %rdi, -8(%rbp)
    movq $0, -24(%rbp)
    movq -8(%rbp), %rbx
    cmpq $4, %rbx
    jb if.then.0
    jmp if.else.1
if.then.0:
    movq $4, -16(%rbp)
    jmp if.end.2
if.else.1:
    movq $6, -16(%rbp)
    jmp if.end.2
if.end.2:
    jmp while.cond.3
while.cond.3:
    movq -16(%rbp), %rbx
    cmpq $10, %rbx
    jb while.body.4
    jmp while.end.5
while.body.4:
    movq -8(%rbp), %rax
    pushq %rax
    movq -24(%rbp), %rax
    popq %rbx
    addq %rbx, %rax
    movq %rax, -24(%rbp)
    movq $1, %rax
    pushq %rax
    movq -16(%rbp), %rax
    popq %rbx
    addq %rbx, %rax
    movq %rax, -16(%rbp)
    jmp while.cond.3
while.end.5:
    movq -24(%rbp), %rax
    addq $24, %rsp
    popq %rbx
    popq %rbp
    ret
.section .note.GNU-stack,"",@progbits
