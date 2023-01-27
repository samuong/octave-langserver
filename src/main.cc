#include "main.h"

#include <cassert>
#include <iostream>

#include "octave/interpreter.h"
#include "octave/parse.h"
#include "octave/pt-pr-code.h"
#include "octave/pt-stmt.h"

#include "tree_walker.h"

static octave::interpreter interp;

void
init ()
{
  std::cerr << "initializing...\n";
  interp.initialize ();
  if (! interp.initialized ())
    throw std::runtime_error ("Octave interpreter initialization failed!");
}

void
analyse (rust::Str text)
{
  std::string s (text.data (), text.size ());
  octave::parser parse (s, interp);

  do
    {
      try
        {
          std::cerr << "parse.run()\n";
          int status = parse.run ();
          std::cerr << "parse status = " << status << "\n";
          std::shared_ptr<octave::tree_statement_list> stmt_list
              = parse.statement_list ();
          if (stmt_list == nullptr)
            continue;
          std::cerr << "stmt list length is " << stmt_list->size () << "\n";
        }
      catch (const octave::execution_exception& e)
        {
          // TODO: capture parse errors and return to the client via
          // textDocument.publishDiagnostics and textDocument.diagnostic.
          e.display (std::cerr);
          continue;
        }
    }
  while (! parse.at_end_of_input ());

  std::shared_ptr<octave::tree_statement_list> stmt_list
      = parse.statement_list ();

  std::cerr << "stmt list length is " << stmt_list->size () << "\n";

  if (stmt_list)
    {
      octave::tree_print_code print_code (std::cerr, "> ");
      tree_walker print_symbols;
      stmt_list->accept (print_code);
      stmt_list->accept (print_symbols);
    }
}

rust::String
symbol_at (uint32_t line, uint32_t character)
{
  std::string symbol;
  if (! find_symbol (line, character, symbol))
    throw std::runtime_error ("symbol not found");
  return rust::String (symbol);
}

std::array<uint32_t, 2>
definition (rust::Str symbol)
{
  uint32_t line, character;
  if (! find_definition (std::string (symbol), line, character))
    throw std::runtime_error ("definition not found");
  return { line, character };
}

void
clear_indexes ()
{
  clear ();
}
