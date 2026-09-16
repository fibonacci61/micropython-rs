#include "function.h"

#include "py/runtime.h"

mprs_function_ret_t mprs_function_argument_error(unsigned int kind) {
    nlr_buf_t nlr;
    if (nlr_push(&nlr) == 0) {
        switch (kind) {
            case 0: mp_raise_TypeError(MP_ERROR_TEXT("argument has incorrect type"));
            case 1: mp_raise_ValueError(MP_ERROR_TEXT("invalid argument value"));
            case 2: mp_raise_TypeError(MP_ERROR_TEXT("missing required argument"));
            case 3: mp_raise_TypeError(MP_ERROR_TEXT("multiple values for argument"));
            case 4: mp_raise_TypeError(MP_ERROR_TEXT("too many positional arguments"));
            default: mp_raise_TypeError(MP_ERROR_TEXT("unexpected keyword argument"));
        }
        nlr_pop();
    }
    return (mprs_function_ret_t){ .ok = false, .value = MP_OBJ_FROM_PTR(nlr.ret_val) };
}

static mp_obj_t mprs_function_call(mp_obj_t self_in, size_t n_args, size_t n_kw,
    const mp_obj_t *args) {
    mprs_obj_function_t *self = MP_OBJ_TO_PTR(self_in);
    mprs_function_ret_t ret = self->function(n_args, n_kw, args);

    if (!ret.ok) {
        nlr_raise(ret.value);
    }

    return ret.value;
}

MP_DEFINE_CONST_OBJ_TYPE(
    mprs_type_function,
    MP_QSTR_function,
    MP_TYPE_FLAG_BINDS_SELF,
    call, mprs_function_call
    );
