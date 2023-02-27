#include "tree_walker.h"

#include <iostream>
#include <map>
#include <string>

#include <octave/pt-arg-list.h>
#include <octave/pt-const.h>
#include <octave/pt-fcn-handle.h>
#include <octave/pt-idx.h>

#include "octave-langserver/src/bridge.rs.h"

void
tree_walker::visit_anon_fcn_handle (octave::tree_anon_fcn_handle& afh)
{
  std::cerr << "==> encountered anon-fcn-handle: at " << afh.line () << ":"
            << afh.column () << "\n";
  octave::tree_parameter_list *params = afh.parameter_list ();
  if (params != nullptr)
    params->accept (*this);
}

void
tree_walker::visit_constant (octave::tree_constant& expr)
{
  this->m_args.push_back (expr.value ());
}

void
tree_walker::visit_decl_command (octave::tree_decl_command& decl)
{
  std::cerr << "==> encountered decl-command: " << decl.name () << " at "
            << decl.line () << ":" << decl.column () << "\n";
  octave::tree_decl_init_list *init_list = decl.initializer_list ();
  if (init_list != nullptr)
    init_list->accept (*this);
}

void
tree_walker::visit_fcn_handle (octave::tree_fcn_handle& fh)
{
  std::cerr << "==> encountered fcn-handle: " << fh.name () << " at "
            << fh.line () << ":" << fh.column () << "\n";
  this->m_index->add_symbol (fh.line () - 1, fh.column () - 1, fh.name ());
}

void
tree_walker::visit_function_def (octave::tree_function_def& def)
{
  octave_value fcn = def.function ();

  octave_user_function *user_fcn = fcn.user_function_value ();
  assert (user_fcn != nullptr);

  std::cerr << "==> encountered function-def: " << user_fcn->name () << " at "
            << def.line () << ":" << def.column () << " until "
            << user_fcn->ending_line () << ":" << user_fcn->ending_column ()
            << "\n";

  this->m_index->add_symbol (def.line () - 1, def.column () - 1, user_fcn->name ());
  this->m_index->add_definition (user_fcn->name (), def.line () - 1, def.column () - 1);

  octave::tree_parameter_list *outputs = user_fcn->return_list ();
  if (outputs != nullptr)
    outputs->accept (*this);

  octave::tree_parameter_list *params = user_fcn->parameter_list ();
  if (params != nullptr)
    params->accept (*this);

  // TODO: what about the nested function bar()?
  /*
  std::cerr << "====> has subfunctions = " << user_fcn->has_subfunctions ()
            << "\n";
  for (auto name : user_fcn->subfunction_names ())
    std::cerr << "======> " << name << "\n";
    */

  octave_function *f = fcn.function_value ();

  if (f)
    f->accept (*this);
}

void
tree_walker::visit_identifier (octave::tree_identifier& id)
{
  std::cerr << "==> encountered identifier: " << id.name () << " at "
            << id.line () << ":" << id.column () << "\n";

  this->m_index->add_symbol (id.line () - 1, id.column () - 1, id.name ());
}

void
tree_walker::visit_index_expression (octave::tree_index_expression& expr)
{
  std::cerr << "==> encountered index-expression: " << expr.name () << " at "
            << expr.line () << ":" << expr.column () << "\n";

  this->m_index->add_symbol (expr.line () - 1, expr.column () - 1, expr.name ());

  for (octave::tree_argument_list *arg_list : expr.arg_lists ())
    if (arg_list != nullptr)
      arg_list->accept (*this);
}
