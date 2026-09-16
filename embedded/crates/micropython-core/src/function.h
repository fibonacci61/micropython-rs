#ifndef MPRS_FUNCTION_H
#define MPRS_FUNCTION_H

#include "py/obj.h"

typedef struct {
    bool ok;
    // ok=true => return value
    // ok=false => exception
    mp_obj_t value;
} mprs_function_ret_t;

typedef mprs_function_ret_t (*mprs_function_t)(size_t n_args, size_t n_kw, const mp_obj_t *args);

typedef struct {
    mp_obj_base_t base;
    mprs_function_t function;
} mprs_obj_function_t;

extern const mp_obj_type_t mprs_type_function;

#endif
