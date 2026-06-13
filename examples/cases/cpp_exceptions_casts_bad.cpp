#include <iostream>

int main()
{
    void* raw = NULL;
    int value = (int)3.14;

    try {
        if (value > 0) {
            while (value > 0) {
                if (value > 0) {
                    if (value > 0) {
                        if (value > 0) {
                            std::cout << value << std::endl;
                        }
                    }
                }
            }
        }
        throw value;
    } catch (...) {
        std::cout << raw << std::endl;
    }

    return value;
}
