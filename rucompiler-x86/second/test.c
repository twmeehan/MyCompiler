#include<stdio.h>
#include<stdlib.h>
extern unsigned long int foo (unsigned long int);

int main(int argc, char** argv){
    unsigned long a = 5;
    printf("the result is %ld\n", foo(a));
    return 0;
}