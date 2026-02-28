// toyforth.h
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef struct TFObject TFObject;
typedef TFObject *TFObjectPtr;

typedef struct TFParser TFParser;
typedef TFParser *TFParserPtr;

typedef enum {
    eint = 1,
    estr,
    ebool,
    elist,
    esymbol,
    EObjType__SUP
} EObjType;

const char *EObjType_as_string(EObjType val);


struct TFObject {
    int refcount;
    EObjType type;
    union {
        int i;
        struct {
            char *ptr;
            size_t len;
        } str;
        struct {
            TFObject **ele;
            size_t len;
        } list;
    };
};



struct TFParser {
    charPtr prg;
    charPtr p;
};



#ifdef __cplusplus
} // extern "C"
#endif
