define i64 @foo(i64 %a, i64 %b, i64 %c) {
    %t1 = add i64 8, %b
    %t2 = mul i64 %t1, %a
    %t3 = mul i64 3, 5
    %t4 = add i64 %c, %t3
    %t5 = mul i64 %b, %t4
    %t6 = add i64 %t2, %t5
    ret i64 %t6
}
