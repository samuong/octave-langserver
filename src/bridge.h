#if ! defined(bridge_h)
#define bridge_h 1

#include "rust/cxx.h"

void init ();
void analyse (rust::Str text);
rust::String symbol_at (uint32_t line, uint32_t character);
std::array<uint32_t, 2> definition (rust::Str symbol);
void clear_indexes ();

#endif
