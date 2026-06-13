#include <stdio.h>

int sum_to(int limit)
{
    int total = 0;

    for (int i = 0; i < limit; i++)
        total += i;

    if (total > 10)
        printf("%d\n", total);

    return total;
}

int main(void)
{
    return sum_to(5);
}
