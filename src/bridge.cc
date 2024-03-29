#include "bridge.h"

#include <cassert>
#include <iostream>

#include <octave/interpreter.h>
#include <octave/parse.h>
#include <octave/pt-pr-code.h>
#include <octave/pt-stmt.h>

#include "tree_walker.h"

static octave::interpreter interp;

void
init (rust::Fn<void (rust::Str)> logger)
{
  logger("initializing (from c++)...");
  interp.initialize ();
  if (! interp.initialized ())
    throw std::runtime_error ("Octave interpreter initialization failed!");
}

void
analyse (rust::Str text, Index& index)
{
  std::cerr << "analysing text of length " << text.size () << "\n";
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
      tree_walker print_symbols(&index);
      stmt_list->accept (print_code);
      stmt_list->accept (print_symbols);
    }
}
