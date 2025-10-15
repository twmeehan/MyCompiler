define i64 @foo(i64 %a, i64 %b, i64 %c) {
    %t1 = mul i64 67, 67
    %t2 = add i64 67, %t1
    %t3 = mul i64 1, %a
    %t4 = add i64 %t2, %t3
    %t5 = mul i64 67, %c
    %t6 = mul i64 %b, %t5
    %t7 = add i64 %t4, %t6
    %t8 = add i64 %t7, 6767
    ret i64 %t8
}
