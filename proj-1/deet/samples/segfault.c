#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>

void func2(int a) {
    printf("About to segfault... a=%d\n", a);
    *(int*)0 = a;
    printf("Did segfault!\n");
}

void func1(int a) {
    sleep(10);
    printf("Calling func2\n");
    func2(a % 5);
}

int main() {
    func1(42);
}
