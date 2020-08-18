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

int testprintf(const char *str)
{
    printf(str);
}


int main()
{
    testreadlink("/tmp/wisk_testlink");
    testvprintf("Hello World! from vprintf");
    testprintf("Hello World! from printf");

    return 0;
}
