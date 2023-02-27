#if ! defined(bridge_h)
#define bridge_h 1

#include "rust/cxx.h"

#include "octave-langserver/src/bridge.rs.h"

void init (rust::Fn<void (rust::Str)> logger);
void analyse (rust::Str text, Index& index);

#endif
