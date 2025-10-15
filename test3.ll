define i64 @foo(i64 %a, i64 %b) {
    %t1 = add i64 %a, 3
    %t2 = mul i64 %t1, 3
    %t3 = add i64 %t2, 3
    %t4 = mul i64 %b, 8
    %t5 = add i64 %t3, %t4
    ret i64 %t5
}
