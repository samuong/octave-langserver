#ifndef MAIN_H
#define MAIN_H

#include <memory>
#include <string>

#include <octave/interpreter.h>
#include <octave/pt-walk.h>

#include "rust/cxx.h"

void init();
void eval(rust::Str eval_str);

#endif
