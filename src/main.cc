#include "main.h"

#include <iostream>

#include <octave/interpreter.h>

void cpp_main() {
  octave::interpreter interp;
  interp.initialize();
  if (!interp.initialized()) {
    std::cerr << "Octave interpreter initialization failed!" << std::endl;
    return;
  }
  int status = interp.execute();
  if (status != 0) {
    std::cerr << "Creating embedded Octave interpreter failed!" << std::endl;
    return;
  }
  int parse_status = 0;
  std::string input = "disp('Hello, world!')";
  octave_value output = interp.eval_string(input, true, parse_status);
  if (parse_status != 0) {
    std::cerr << "Parsing embedded Octave sources failed!" << std::endl;
    return;
  }
  output.print(std::cout);
}
