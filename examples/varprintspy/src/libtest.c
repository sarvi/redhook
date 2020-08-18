#include <stdio.h>
#include <stdarg.h>
#include <unistd.h>

int puts(const char *);

int printf(const char *__restrict __fmt, ...)
{
    puts("\nC: printf --> intercept");
    return puts(__fmt);
}
