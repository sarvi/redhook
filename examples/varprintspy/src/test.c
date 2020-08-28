#include <stdio.h>
#include <stdarg.h>
#include <unistd.h>

int testreadlink(const char *link)
{
    char buf[500];
    readlink(link, buf, 500);
}

int testvprintf(const char *format, ...)
{
    va_list argp;

    va_start(argp, format);
    vprintf(format, argp);
    va_end(argp);

}

int testprintf(const char *str, int i, float f, char *s)
{
    printf(str, i, f, s);
}


int main()
{
    testreadlink("/tmp/wisk_testlink");
    testvprintf("Hello World! from vprintf: %d %f %s \n", 100, 1.23456, "something");
    testprintf("Hello World! from printf: %d %f %s \n", 100, 1.23456, "something");

    return 0;
}
