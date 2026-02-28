// toyforth.c
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include "commondefs.h"
#include "toyforth.h"

const char * EObjType_as_string(EObjType val) {
    switch (val) {
        case eint: return "int";
        case estr: return "str";
        case ebool: return "bool";
        case elist: return "list";
        case esymbol: return "symbol";
        default: return "<invalid EObjType value>";
    }
}

void *xmalloc(size_t size) {
    void *result = malloc(size);
    if (NULL == result) {
        fprintf(stderr, "Out of memory allocating %zu bytes", size);
        exit(1);
    }
    return result;
}


TFObjectPtr create_object(EObjType type) {
    TFObjectPtr result = malloc(sizeof(TFObject));
    result->type = type;
    result->refcount = 1;
    return result;
}

TFObjectPtr create_string_object(char *s, size_t len) {
    TFObjectPtr result = create_object(estr);
    result->str.ptr = s;
    result->str.len = len;
    return result;
}

TFObjectPtr create_symbol_object(char *s, size_t len) {
    TFObjectPtr result = create_string_object(s, len);
    result->type = esymbol;
    return result;
}

TFObjectPtr create_int_object(int i) {
    TFObjectPtr result = create_object(eint);
    result->i = i;
    return result;
}

TFObjectPtr create_bool_object(bool b) {
    TFObjectPtr result = create_object(ebool);
    result->i = b;
    return result;
}

TFObjectPtr create_list_object(bool b) {
    TFObjectPtr result = create_object(elist);
    result->list.ele = NULL;
    result->list.len = 0;
    return result;
}


int main(int argc, char **argv) {
    return 0;
}
