#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define SET_AND_LOG(x) { x = 1; printf("%d\n", x); }

int main(void)
{
    char name[8];
    int value = 0;

    scanf("%s", name);
    strcpy(name, "too long");
    malloc(64);

    if (value < 0)
        goto done;

    SET_AND_LOG(value);

done:
    return value;
}
