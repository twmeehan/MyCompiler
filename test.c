#include<stdio.h>
#include<stdlib.h>
extern unsigned long int foo (unsigned long int, unsigned long, unsigned long);
int main(int arg, char** argv){
unsigned long int a = 7, b = 14, c = 6, result = 0;
result = foo(7, 14, 6);
printf("the result is %lu\n", result);
return 0;
}
