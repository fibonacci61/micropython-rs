#ifndef MICROPYTHON_RS_NLRSHIMS_H
#define MICROPYTHON_RS_NLRSHIMS_H

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
 *   NLRSHIM_DECLARE_MP_OBJ(my_call, (mp_obj_t fun, mp_obj_t arg));
 *
 * In exactly one C file:
 *
 *   NLRSHIM_DEFINE_MP_OBJ(my_call, (mp_obj_t fun, mp_obj_t arg),
 *       mp_call_function_1(fun, arg))
 *
 * The parameter list must be parenthesised.  `expression` is evaluated inside
 * the NLR handler, so argument evaluation is protected too.
 */

#include <stdbool.h>
#include <stddef.h>

#include "py/nlr.h"
#include "py/obj.h"

#define NLRSHIM_RESULT_TYPE(name, value_type) \
    typedef struct {                         \
        bool ok;                             \
        value_type value;                    \
        mp_obj_t exception;                  \
    } nlrshim_result_##name##_t

NLRSHIM_RESULT_TYPE(mp_obj, mp_obj_t);
NLRSHIM_RESULT_TYPE(bool, bool);
NLRSHIM_RESULT_TYPE(mp_int, mp_int_t);
NLRSHIM_RESULT_TYPE(mp_uint, mp_uint_t);
NLRSHIM_RESULT_TYPE(size, size_t);
NLRSHIM_RESULT_TYPE(ptr, void *);

typedef struct {
    bool ok;
    mp_obj_t exception;
} nlrshim_result_void_t;

/* Generic declaration macros, also usable with application-defined results. */
#define NLRSHIM_DECLARE(result_type, name, parameters) \
    result_type name parameters

#define NLRSHIM_DECLARE_VOID(name, parameters) \
    NLRSHIM_DECLARE(nlrshim_result_void_t, name, parameters)
#define NLRSHIM_DECLARE_MP_OBJ(name, parameters) \
    NLRSHIM_DECLARE(nlrshim_result_mp_obj_t, name, parameters)
#define NLRSHIM_DECLARE_BOOL(name, parameters) \
    NLRSHIM_DECLARE(nlrshim_result_bool_t, name, parameters)
#define NLRSHIM_DECLARE_MP_INT(name, parameters) \
    NLRSHIM_DECLARE(nlrshim_result_mp_int_t, name, parameters)
#define NLRSHIM_DECLARE_MP_UINT(name, parameters) \
    NLRSHIM_DECLARE(nlrshim_result_mp_uint_t, name, parameters)
#define NLRSHIM_DECLARE_SIZE(name, parameters) \
    NLRSHIM_DECLARE(nlrshim_result_size_t, name, parameters)
#define NLRSHIM_DECLARE_PTR(name, parameters) \
    NLRSHIM_DECLARE(nlrshim_result_ptr_t, name, parameters)

/*
 * Keep the push, potentially raising operation, and pop in this C function.
 * The error value is initialized to zero so every byte exposed through the
 * C ABI has a defined value.  nlr.ret_val is the caught exception object.
 */
#define NLRSHIM_DEFINE_VALUE_IMPL(result_type, value_type, name, parameters, expression) \
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

#define NLRSHIM_DEFINE_VOID_IMPL(name, parameters, statement)                    \
    nlrshim_result_void_t name parameters {                                      \
        nlr_buf_t nlr;                                                           \
        if (nlr_push(&nlr) == 0) {                                               \
            statement;                                                          \
            nlr_pop();                                                           \
            return (nlrshim_result_void_t){ .ok = true, .exception = MP_OBJ_NULL }; \
        }                                                                        \
        return (nlrshim_result_void_t){                                          \
            .ok = false, .exception = MP_OBJ_FROM_PTR(nlr.ret_val)               \
        };                                                                       \
    }

#define NLRSHIM_DEFINE_VOID(name, parameters, statement) \
    NLRSHIM_DEFINE_VOID_IMPL(name, parameters, statement)
#define NLRSHIM_DEFINE_MP_OBJ(name, parameters, expression) \
    NLRSHIM_DEFINE_VALUE_IMPL(nlrshim_result_mp_obj_t, mp_obj_t, name, parameters, expression)
#define NLRSHIM_DEFINE_BOOL(name, parameters, expression) \
    NLRSHIM_DEFINE_VALUE_IMPL(nlrshim_result_bool_t, bool, name, parameters, expression)
#define NLRSHIM_DEFINE_MP_INT(name, parameters, expression) \
    NLRSHIM_DEFINE_VALUE_IMPL(nlrshim_result_mp_int_t, mp_int_t, name, parameters, expression)
#define NLRSHIM_DEFINE_MP_UINT(name, parameters, expression) \
    NLRSHIM_DEFINE_VALUE_IMPL(nlrshim_result_mp_uint_t, mp_uint_t, name, parameters, expression)
#define NLRSHIM_DEFINE_SIZE(name, parameters, expression) \
    NLRSHIM_DEFINE_VALUE_IMPL(nlrshim_result_size_t, size_t, name, parameters, expression)
#define NLRSHIM_DEFINE_PTR(name, parameters, expression) \
    NLRSHIM_DEFINE_VALUE_IMPL(nlrshim_result_ptr_t, void *, name, parameters, expression)

NLRSHIM_DECLARE_PTR(nlrshim_m_malloc, (size_t num_bytes));

#endif /* MICROPYTHON_RS_NLRSHIMS_H */
