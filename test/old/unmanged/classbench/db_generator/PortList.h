// File: PortList.h
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for PortList
//   - Maintains distribution of port range values
//   - Reads in distribution from seed file
//   - Selects port range based on random input

struct port {
  int high, low;
  float prob;
  int next;
};

class PortList {
  int N; // PortList of N port ranges
  int first; // beginning of list
  int last;  // end of list
  struct port *ports; // array of port structs
 
 public: PortList(int=200);
  ~PortList();
  void read(int t, FILE *fp); // Read port information from input file *fp
  struct range choose_port(double r); // Choose port from distribution given random number r [0:1]
  void print(FILE*); // Print portocol distribution
};
  
