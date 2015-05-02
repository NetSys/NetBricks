// File: ProtList.h
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for ProtList
//   - Maintains distribution of filter protocols and port pair types
//       * Each item is a (protocol, probability) pair with an associated distribution
//         of port range pair types (wc/wc, wc/em, etc.)
//   - Reads in distribution from seed file
//   - Selects protocol and port pair type based on random inputs

struct protocol {
  int prot_num;
  float prob;
  float* pt_prob;
};

class ProtList {
  int N; // ProtList of N protocols
  int first; // beginning of list
  int last;  // end of list
  struct protocol *protocols; // array of protocol structs
 
 public: ProtList();
  ~ProtList();
  void read(FILE *fp); // Read protocol information from input file *fp
  int choose_prot(float r); // Choose protocol from distribution given random number r [0:1]
  int choose_ports(float r, int prot); // Choose port range type given random number r and protocol
  void print(FILE*); // Print protocol distribution
  int size(); // Return the number of unique protocol specs in list
  int operator()(int i); // Return protocol number for protocol i
};
  
