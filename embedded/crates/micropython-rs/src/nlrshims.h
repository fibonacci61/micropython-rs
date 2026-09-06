#ifndef MPRS_NLRSHIMS_H
#define MPRS_NLRSHIMS_H

/*
 * C ABI helpers for calling MicroPython functions from Rust.
 *
 * An NLR-capable MicroPython function must never be called directly while a
 * Rust stack frame is below it: an exception would longjmp through that frame.
 * Define a C shim with one of the macros below instead.  The shim catches the
 * exception and returns it as an mp_obj_t.
 *
 * Example:
 *
 *   MPRS_NLRSHIM_DECLARE_MP_OBJ(my_call, (mp_obj_t fun, mp_obj_t arg));
 *
 * In exactly one C file:
 *
 *   MPRS_NLRSHIM_DEFINE_MP_OBJ(my_call, (mp_obj_t fun, mp_obj_t arg),
 *       mp_call_function_1(fun, arg))
 *
 * The parameter list must be parenthesised.  `expression` is evaluated inside
 * the NLR handler, so argument evaluation is protected too.
 */

#include <stdbool.h>
#include <stddef.h>

#include "py/nlr.h"
#include "py/obj.h"

#define MPRS_NLRSHIM_RESULT_TYPE(name, value_type) \
    typedef struct {                         \
        bool ok;                             \
        value_type value;                    \
        mp_obj_t exception;                  \
    } mprs_nlrshim_result_##name##_t

MPRS_NLRSHIM_RESULT_TYPE(mp_obj, mp_obj_t);
MPRS_NLRSHIM_RESULT_TYPE(bool, bool);
MPRS_NLRSHIM_RESULT_TYPE(mp_int, mp_int_t);
MPRS_NLRSHIM_RESULT_TYPE(mp_uint, mp_uint_t);
MPRS_NLRSHIM_RESULT_TYPE(size, size_t);
MPRS_NLRSHIM_RESULT_TYPE(ptr, void *);

typedef struct {
    bool ok;
    mp_obj_t exception;
} mprs_nlrshim_result_void_t;

/* Generic declaration macros, also usable with application-defined results. */
#define MPRS_NLRSHIM_DECLARE(result_type, name, parameters) \
    result_type name parameters

#define MPRS_NLRSHIM_DECLARE_VOID(name, parameters) \
    MPRS_NLRSHIM_DECLARE(mprs_nlrshim_result_void_t, name, parameters)
#define MPRS_NLRSHIM_DECLARE_MP_OBJ(name, parameters) \
    MPRS_NLRSHIM_DECLARE(mprs_nlrshim_result_mp_obj_t, name, parameters)
#define MPRS_NLRSHIM_DECLARE_BOOL(name, parameters) \
    MPRS_NLRSHIM_DECLARE(mprs_nlrshim_result_bool_t, name, parameters)
#define MPRS_NLRSHIM_DECLARE_MP_INT(name, parameters) \
    MPRS_NLRSHIM_DECLARE(mprs_nlrshim_result_mp_int_t, name, parameters)
#define MPRS_NLRSHIM_DECLARE_MP_UINT(name, parameters) \
    MPRS_NLRSHIM_DECLARE(mprs_nlrshim_result_mp_uint_t, name, parameters)
#define MPRS_NLRSHIM_DECLARE_SIZE(name, parameters) \
    MPRS_NLRSHIM_DECLARE(mprs_nlrshim_result_size_t, name, parameters)
#define MPRS_NLRSHIM_DECLARE_PTR(name, parameters) \
    MPRS_NLRSHIM_DECLARE(mprs_nlrshim_result_ptr_t, name, parameters)

/*
 * Keep the push, potentially raising operation, and pop in this C function.
 * The error value is initialized to zero so every byte exposed through the
 * C ABI has a defined value.  nlr.ret_val is the caught exception object.
 */
#define MPRS_NLRSHIM_DEFINE_VALUE_IMPL(result_type, value_type, name, parameters, expression) \
    result_type name parameters {                                                    \
        nlr_buf_t nlr;                                                               \
        if (nlr_push(&nlr) == 0) {                                                   \
            value_type value = (expression);                                         \
            nlr_pop();                                                               \
            return (result_type){ .ok = true, .value = value, .exception = MP_OBJ_NULL }; \
        }                                                                            \
        return (result_type){                                                        \
            .ok = false,                                                             \
            .value = (value_type)0,                                                  \
            .exception = MP_OBJ_FROM_PTR(nlr.ret_val),                               \
        };                                                                           \
    }

#define MPRS_NLRSHIM_DEFINE_VOID_IMPL(name, parameters, statement)                    \
    mprs_nlrshim_result_void_t name parameters {                                      \
        nlr_buf_t nlr;                                                           \
        if (nlr_push(&nlr) == 0) {                                               \
            statement;                                                          \
            nlr_pop();                                                           \
            return (mprs_nlrshim_result_void_t){ .ok = true, .exception = MP_OBJ_NULL }; \
        }                                                                        \
        return (mprs_nlrshim_result_void_t){                                          \
            .ok = false, .exception = MP_OBJ_FROM_PTR(nlr.ret_val)               \
        };                                                                       \
    }

#define MPRS_NLRSHIM_DEFINE_VOID(name, parameters, statement) \
    MPRS_NLRSHIM_DEFINE_VOID_IMPL(name, parameters, statement)
#define MPRS_NLRSHIM_DEFINE_MP_OBJ(name, parameters, expression) \
    MPRS_NLRSHIM_DEFINE_VALUE_IMPL(mprs_nlrshim_result_mp_obj_t, mp_obj_t, name, parameters, expression)
#define MPRS_NLRSHIM_DEFINE_BOOL(name, parameters, expression) \
    MPRS_NLRSHIM_DEFINE_VALUE_IMPL(mprs_nlrshim_result_bool_t, bool, name, parameters, expression)
#define MPRS_NLRSHIM_DEFINE_MP_INT(name, parameters, expression) \
    MPRS_NLRSHIM_DEFINE_VALUE_IMPL(mprs_nlrshim_result_mp_int_t, mp_int_t, name, parameters, expression)
#define MPRS_NLRSHIM_DEFINE_MP_UINT(name, parameters, expression) \
    MPRS_NLRSHIM_DEFINE_VALUE_IMPL(mprs_nlrshim_result_mp_uint_t, mp_uint_t, name, parameters, expression)
#define MPRS_NLRSHIM_DEFINE_SIZE(name, parameters, expression) \
    MPRS_NLRSHIM_DEFINE_VALUE_IMPL(mprs_nlrshim_result_size_t, size_t, name, parameters, expression)
#define MPRS_NLRSHIM_DEFINE_PTR(name, parameters, expression) \
    MPRS_NLRSHIM_DEFINE_VALUE_IMPL(mprs_nlrshim_result_ptr_t, void *, name, parameters, expression)

MPRS_NLRSHIM_DECLARE_PTR(mprs_nlrshim_m_malloc, (size_t num_bytes));

#endif /* MICROPYTHON_RS_NLRSHIMS_H */
