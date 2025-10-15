define i64 @foo(i64 %a, i64 %b, i64 %c) {
    %t1 = mul i64 %b, %c
    %t2 = mul i64 %t1, 4
    %t3 = add i64 %a, %b
    %t4 = mul i64 %t2, %t3
    %t5 = add i64 %a, %t4
    ret i64 %t5
}
