#include "py/misc.h"

#include "nlrshims.h"

NLRSHIM_DEFINE_PTR(nlrshim_m_malloc, (size_t num_bytes), m_malloc(num_bytes));
