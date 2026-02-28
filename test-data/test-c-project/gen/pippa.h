// pippa.h
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    coUno = 1,
    coTre,
    Cosi__SUP
} Cosi;

const char *Cosi_as_string(Cosi val);


#ifdef __cplusplus
} // extern "C"
#endif
