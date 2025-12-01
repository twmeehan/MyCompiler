define i64 @foo(i64 %a) {
entry:
%a.alloc = alloca i64
%b.alloc = alloca i64
%c.alloc = alloca i64
store i64 %a, ptr %a.alloc
store i64 0, ptr %c.alloc
%t1 = load i64, ptr %a.alloc
%t2 = icmp ult i64 %t1, 4
br i1 %t2, label %if.then.0, label %if.else.1
if.then.0:
store i64 4, ptr %b.alloc
br label %if.end.2
if.else.1:
store i64 6, ptr %b.alloc
br label %if.end.2
if.end.2:
br label %while.cond.3
while.cond.3:
%t3 = load i64, ptr %b.alloc
%t4 = icmp ult i64 %t3, 10
br i1 %t4, label %while.body.4, label %while.end.5
while.body.4:
%t5 = load i64, ptr %c.alloc
%t6 = load i64, ptr %a.alloc
%t7 = add i64 %t5, %t6
store i64 %t7, ptr %c.alloc
%t8 = load i64, ptr %b.alloc
%t9 = add i64 %t8, 1
store i64 %t9, ptr %b.alloc
br label %while.cond.3
while.end.5:
%t10 = load i64, ptr %c.alloc
ret i64 %t10
}
