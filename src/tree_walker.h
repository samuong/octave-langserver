#if ! defined(tree_walker_h)
#define tree_walker_h 1

#include <set>
#include <vector>

#include <octave/ov.h>
#include <octave/pt-walk.h>

class tree_walker : public octave::tree_walker
{
public:
  void visit_anon_fcn_handle (octave::tree_anon_fcn_handle&) override;
  void visit_constant (octave::tree_constant&) override;
  void visit_decl_command (octave::tree_decl_command&) override;
  void visit_fcn_handle (octave::tree_fcn_handle&) override;
  void visit_function_def (octave::tree_function_def&) override;
  void visit_identifier (octave::tree_identifier&) override;
  void visit_index_expression (octave::tree_index_expression&) override;

private:
  std::vector<octave_value> m_args;
};

extern bool find_symbol (uint32_t, uint32_t, std::string&);
extern bool find_definition (const std::string&, uint32_t&, uint32_t&);
extern void clear ();

#endif
