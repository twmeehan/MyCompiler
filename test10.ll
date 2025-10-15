define i64 @foo(i64 %a, i64 %b, i64 %c) {
    %t1 = add i64 900, %c
    %t2 = mul i64 %t1, 67
    %t3 = add i64 %t2, %c
    %t4 = mul i64 %t1, 5
    %t5 = add i64 %a, %b
    %t6 = mul i64 %t4, %t5
    %t7 = add i64 %t6, %b
    %t8 = mul i64 %t7, 8
    %t9 = add i64 %t3, %t8
    ret i64 %t9
}
