#if ! defined (main_h)
#define main_h 1

#include <memory>
#include <string>

#include <octave/interpreter.h>
#include <octave/pt-walk.h>

#include "rust/cxx.h"

void init ();
void eval (rust::Str eval_str);

#endif
