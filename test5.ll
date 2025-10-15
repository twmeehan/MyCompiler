define i64 @foo(i64 %first, i64 %second, i64 %test) {
    %t1 = mul i64 %second, 43
    %t2 = add i64 %first, %t1
    %t3 = mul i64 167, %test
    %t4 = add i64 %t2, %t3
    ret i64 %t4
}
