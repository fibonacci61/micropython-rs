#include "function.h"

#include "py/runtime.h"

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
