#include <stdio.h>
#include <stdarg.h>

int testprintf(const char *format, ...)
{
    va_list argp;

    va_start(argp, format);
    vprintf(format, argp);
    va_end(argp);

}

int main(int argc, char *argv)
{
    testprintf("vprintf: Hello World\n");
    printf("printf: Hello World\n");

}