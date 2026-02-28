// pippa.c
#include <stddef.h>
#include "pippa.h"

const char * Cosi_as_string(Cosi val) {
    switch (val) {
        case coUno: return "Uno";
        case coTre: return "Tre";
        default: return "<invalid Cosi value>";
    }
}

