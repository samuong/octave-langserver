# Identifiers in Octave

Identifiers in Octave can refer to functions or variables.

When the [lexer][lex-rule] encounters an identifier, it will emit a
[NAME][lex-handler] token.

The [parser][yacc-rule] will then create a [`tree_identifier`][tree-identifier]
node, which could appear in a number of places in the AST depending on how it
was used.

[lex-regex]: https://hg.octave.org/octave/file/release-7-3-0/libinterp/parse-tree/lex.ll#l369
[lex-rule]: https://hg.octave.org/octave/file/release-7-3-0/libinterp/parse-tree/lex.ll#l1336
[lex-handler]: https://hg.octave.org/octave/file/release-7-3-0/libinterp/parse-tree/lex.ll#l3574
[yacc-rule]: https://hg.octave.org/octave/file/release-7-3-0/libinterp/parse-tree/oct-parse.yy#l540
[yacc-handler]: https://hg.octave.org/octave/file/release-7-3-0/libinterp/parse-tree/oct-parse.yy#l5303
[tree-identifier]: https://docs.octave.org/doxygen/dev/d4/d6b/classoctave_1_1tree__identifier.html

## Functions

1. Functions are defined using the [function][fcn-def] keyword. The parser will
   emit a [`tree_function_def`][tree-function-def] node in the tree for these.
2. [Functions handles][fcn-handle] can be passed around as values. The parser
   will emit a [`tree_fcn_handle`][tree-fcn-handle] node for these.
3. [Anonymous functions][fcn-anonymous] are also supported, although these
   don't seem to be able to create/modify variables inside the function body.
   The parser will create a [`tree_anon_fcn_handle`][tree-anon-fcn-handle]
   object for these.
4. All other identifiers that refer to functions names are probably just
   [function calls][fcn-call].

Assumption: there don't seem to be any namespaces in Octave, so all functions
share a global scope level.

[fcn-def]: https://docs.octave.org/latest/Defining-Functions.html
[fcn-call]: https://docs.octave.org/latest/Calling-Functions.html
[fcn-handle]: https://docs.octave.org/latest/Function-Handles.html
[fcn-anonymous]: https://docs.octave.org/latest/Anonymous-Functions.html
[tree-function-def]: https://docs.octave.org/doxygen/dev/df/dee/classoctave_1_1tree__function__def.html
[tree-fcn-handle]: https://docs.octave.org/doxygen/dev/d4/d1e/classoctave_1_1tree__fcn__handle.html
[tree-anon-fcn-handle]: https://docs.octave.org/doxygen/dev/d5/dbc/classoctave_1_1tree__anon__fcn__handle.html

## Variables

1. Variables can be [global][var-global] (declared with the `global` keyword)
   or local (to a function).
2. Local variables may or may not be [persistent][var-persistent].
3. Script files do *not* create a new scope.

Assumption: it's not possible to know at "compile-time" (i.e. when walking the
AST) whether an identifier refers to an undefined function or variable. For
example, a script file might expect another script file (that calls it) to
define the functions and variables that it references. This is only knowable at
run-time, so the language server should never references as undefined.

Assumption: I don't think there are any nested scopes. Code inside conditional
statements and loops seem to use the same variables as code outside of those
blocks. So there's 1) a global scope, 2) a top-level scope, and 3) a scope for
every function call.

[var-global]: https://docs.octave.org/latest/Global-Variables.html
[var-persistent]: https://docs.octave.org/latest/Persistent-Variables.html
