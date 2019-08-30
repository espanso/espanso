#ifndef ESPANSO_BRIDGE_H
#define ESPANSO_BRIDGE_H

#include <stdio.h>

extern "C" void testcall(float value)
{
    printf("Hello, world from C! Value passed: %f\n",value);
}

#endif //ESPANSO_BRIDGE_H
