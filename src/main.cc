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
  assert(stmt_list->length() == 1);
  octave::tree_statement *stmt = stmt_list->front();
  assert(stmt != nullptr);
  octave::tree_expression *expr = stmt->expression();
  assert(expr != nullptr);
  octave::tree_index_expression &ie =
      dynamic_cast<octave::tree_index_expression &>(*expr);
  assert(ie.name() == "disp");
  assert(ie.arg_lists().size() == 1);
  octave::tree_argument_list *arg_list = ie.arg_lists().front();
  assert(arg_list != nullptr);
  assert(arg_list->size() == 1);
  octave::tree_expression *arg_expr = arg_list->front();
  assert(arg_expr != nullptr);
  assert(arg_expr->is_constant());
  octave::tree_constant *const_expr =
      dynamic_cast<octave::tree_constant *>(arg_expr);
  octave_value value = const_expr->value();
  std::cout << value.string_value() << "\n";
}
