#ifndef MPRS_STATICSHIMS_H
#define MPRS_STATICSHIMS_H

#include "py/cstack.h"

void mprs_cstack_init_with_top(void *top, size_t stack_size);

#endif
