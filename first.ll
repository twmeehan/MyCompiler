define i64 @foo(i64 %a, i64 %b) {
    %t1 = mul i64 %a, %b
    %t2 = add i64 %t1, %t1
    ret i64 %t2
}
