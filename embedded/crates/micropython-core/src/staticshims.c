#include "staticshims.h"

void mprs_cstack_init_with_top(void *top, size_t stack_size) {
    mp_cstack_init_with_top(top, stack_size);
}
