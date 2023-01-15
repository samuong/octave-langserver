#include "main.h"

#include <cassert>
#include <iostream>
#include <stdexcept>

#include <octave/interpreter.h>
#include <octave/parse.h>
#include <octave/pt-arg-list.h>
#include <octave/pt-const.h>
#include <octave/pt-exp.h>
#include <octave/pt-idx.h>
#include <octave/pt-stmt.h>

#define todo()                                                                 \
  do {                                                                         \
    std::ostringstream msg;                                                    \
    msg << "error: " << __FILE__ << ":" << __LINE__                            \
        << ": not yet implemented";                                            \
    throw std::runtime_error(msg.str());                                       \
  } while (false)

class tree_walker : public octave::tree_walker {
public:
  void visit_constant(octave::tree_constant &expr) override;
  void visit_index_expression(octave::tree_index_expression &) override;

private:
  std::vector<octave_value> args;
};

void tree_walker::visit_constant(octave::tree_constant &expr) {
  this->args.push_back(expr.value());
}

void tree_walker::visit_index_expression(
    octave::tree_index_expression &expr) {
  if (expr.name() != "disp") {
    std::ostringstream msg;
    msg << "'" << expr.name() << "' undefined near line " << expr.line()
        << ", column " << expr.column();
    throw std::runtime_error(msg.str());
  }
  assert(this->args.empty());
  for (octave::tree_argument_list *arg_list : expr.arg_lists()) {
    arg_list->accept(*this);
  }
  for (octave_value arg : this->args) {
    std::cout << arg.string_value(true) << "\n";
  }
  this->args.clear();
}

static octave::interpreter interp;

void init() {
  interp.initialize();
  if (!interp.initialized()) {
    throw std::runtime_error("Octave interpreter initialization failed!");
  }
}

void eval(rust::Str eval_str) {
  octave::parser parse(std::string(eval_str.data()), interp);
  int status = parse.run();
  if (status != 0) {
    throw std::runtime_error("parse error");
  }
  std::shared_ptr<octave::tree_statement_list> stmt_list =
      parse.statement_list();
  tree_walker tw;
  stmt_list->accept(tw);
  return;
}
