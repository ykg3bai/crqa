#include <iostream>
#include <memory>

int main()
{
    auto value = std::make_unique<int>(1);

    for (int i = 0; i < 3; ++i)
        *value += i;

    if (*value > 0)
        std::cout << *value << '\n';

    return 0;
}
