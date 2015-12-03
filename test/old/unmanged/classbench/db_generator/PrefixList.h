// File: PrefixList.h
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for PrefixList
//   - Maintains distribution of address prefix lengths
//       * Each item is a (total length, probability) pair with an associated distribution
//         of source address prefix lengths (destination length defined by difference)
//   - Reads in distribution from seed file
//   - Applies smoothing adjustment to distributions
//   - Selects length based on two random inputs

struct prefix {
  float prob;
  float sprob[33];
};

class PrefixList {
  int N; // PrefixList of N prefixes
  struct prefix** prefixes; // array of prefix structs
  int cdist; // flag signaling if cummulative distribution has been computed
  void build_cdist();
  void binomial(int k, double p[]);
  void read_type(int type, FILE *fp); // Read prefix information from input file *fp, type = t
  void smooth_type(int type, int s); // adjust for smoothness parameter s
 public: PrefixList();
  ~PrefixList();
  void read(FILE *fp); // Read prefix information from input file *fp
  void smooth(int s); // adjust for smoothness parameter s
  struct ppair choose_prefix(int type, float rs, float rt); // Choose prefix pair from distribution given random number r [0:1]
  void print(int type, FILE*); // Print prefix distribution
};
  
