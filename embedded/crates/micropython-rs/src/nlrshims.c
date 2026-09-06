#include "py/misc.h"

#include "nlrshims.h"

MPRS_NLRSHIM_DEFINE_PTR(mprs_nlrshim_m_malloc, (size_t num_bytes), m_malloc(num_bytes));
