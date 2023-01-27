#include "tree_walker.h"

#include <iostream>
#include <map>
#include <string>

#include <octave/pt-arg-list.h>
#include <octave/pt-const.h>
#include <octave/pt-fcn-handle.h>
#include <octave/pt-idx.h>

// line -> (character -> symbol)
static std::map<uint32_t, std::map<uint32_t, std::string> > symbols;

// symbol -> (line, character)
static std::map<std::string, std::pair<uint32_t, uint32_t> > definitions;

void
add_symbol (uint32_t line, uint32_t character, const std::string& symbol)
{
  std::cout << "adding '" << symbol << "' at " << line << "," << character
            << "\n";

  auto result = symbols.emplace (line, std::map<uint32_t, std::string> ());
  assert (result.first->first == line);
  result.first->second[character] = symbol;
}

bool
find_symbol (uint32_t line, uint32_t character, std::string& symbol)
{
  std::cout << "finding symbol at '" << line << "," << character << "\n";

  if (symbols.find (line) == symbols.end ())
    {
      std::cout << "no line\n";
      return false;
    }
  for (const auto& entry : symbols[line])
    {
      uint32_t start = entry.first;
      uint32_t end = entry.first + entry.second.size ();
      std::cout << "start=" << start << ", end=" << end << "\n";
      if (start <= character && character < end)
        {
          std::cout << "found " << entry.second << "\n";
          symbol = entry.second;
          return true;
        }
    }
  return false;
}

void
add_definition (const std::string& symbol, uint32_t line, uint32_t character)
{
  definitions[symbol] = std::make_pair (line, character);
}

bool
find_definition (const std::string& symbol, uint32_t& line,
                 uint32_t& character)
{
  auto it = definitions.find (symbol);
  if (it == definitions.end ())
    return false;
  line = it->second.first;
  character = it->second.second;
  return true;
}

void
tree_walker::visit_anon_fcn_handle (octave::tree_anon_fcn_handle& afh)
{
  std::cout << "==> encountered anon-fcn-handle: at " << afh.line () << ":"
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
  std::cout << "==> encountered decl-command: " << decl.name () << " at "
            << decl.line () << ":" << decl.column () << "\n";
  octave::tree_decl_init_list *init_list = decl.initializer_list ();
  if (init_list != nullptr)
    init_list->accept (*this);
}

void
tree_walker::visit_fcn_handle (octave::tree_fcn_handle& fh)
{
  std::cout << "==> encountered fcn-handle: " << fh.name () << " at "
            << fh.line () << ":" << fh.column () << "\n";
  add_symbol (fh.line () - 1, fh.column () - 1, fh.name ());
}

void
tree_walker::visit_function_def (octave::tree_function_def& def)
{
  octave_value fcn = def.function ();

  octave_user_function *user_fcn = fcn.user_function_value ();
  assert (user_fcn != nullptr);

  std::cout << "==> encountered function-def: " << user_fcn->name () << " at "
            << def.line () << ":" << def.column () << " until "
            << user_fcn->ending_line () << ":" << user_fcn->ending_column ()
            << "\n";

  add_symbol (def.line () - 1, def.column () - 1, user_fcn->name ());
  add_definition (user_fcn->name (), def.line () - 1, def.column () - 1);

  octave::tree_parameter_list *outputs = user_fcn->return_list ();
  if (outputs != nullptr)
    outputs->accept (*this);

  octave::tree_parameter_list *params = user_fcn->parameter_list ();
  if (params != nullptr)
    params->accept (*this);

  // TODO: what about the nested function bar()?
  /*
  std::cout << "====> has subfunctions = " << user_fcn->has_subfunctions ()
            << "\n";
  for (auto name : user_fcn->subfunction_names ())
    std::cout << "======> " << name << "\n";
    */

  octave_function *f = fcn.function_value ();

  if (f)
    f->accept (*this);
}

void
tree_walker::visit_identifier (octave::tree_identifier& id)
{
  std::cout << "==> encountered identifier: " << id.name () << " at "
            << id.line () << ":" << id.column () << "\n";

  add_symbol (id.line () - 1, id.column () - 1, id.name ());
}

void
tree_walker::visit_index_expression (octave::tree_index_expression& expr)
{
  std::cout << "==> encountered index-expression: " << expr.name () << " at "
            << expr.line () << ":" << expr.column () << "\n";

  add_symbol (expr.line () - 1, expr.column () - 1, expr.name ());

  for (octave::tree_argument_list *arg_list : expr.arg_lists ())
    if (arg_list != nullptr)
      arg_list->accept (*this);
}

void
clear ()
{
  symbols.clear ();
  definitions.clear ();
}
