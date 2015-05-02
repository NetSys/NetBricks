// File: dbintree.h
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for dbintree
//   - Maintains a binary tree for generating destination addresses given a list of
//     destination address prefix lengths
//   - Reads in nesting, skew, and correlation parameters from seed file
//   - Guarantees nesting will not be exceeded, attempts to maintain skew and correlation
//     statistics

typedef int level;

class dbintree {

  struct tnode {
    level lvl;
    struct tnode *parent;
    struct tnode *child0;
    struct tnode *child1;
    int wt_child0;
    int wt_child1;
    dlist *stubList;
    int valid;
  } *tnodes;

  struct tnode *root; // pointer to root node
  float *skew; // array of target skews for each level
  float *corr; // array of target correlations for each level 
  float *p1child; // probability that a node at a given level has one child
  float *p2child; // probability that a node at a given level has two children
  int num_tnodes; // number of tree nodes 
  void add_stub(struct tnode *node, unsigned int addr, dlist* Flist, struct filter filters[],int CurrNest);
  void add2child_stublist(struct tnode *node, int dir, int filt);
  void add_node(struct tnode *prnt, int lev, int dir);
  void finish_node(struct tnode *node, unsigned int addr, dlist* Flist, struct filter filters[],int CurrNest);
  int Nest; // Maximum allowed nesting

 public: dbintree();
  ~dbintree();
  int nodes(); // return number of nodes
  void delete_node(struct tnode *me);
  void read_skew(FILE*); // read in destination address tree statistics
  void read_nest(FILE*); // read in source address tree statistics
  void read_corr(FILE*); // read in address correlation statistics
  void scale_skew(float scale_factor); // scale branching and skew according to scaling factor
  void print_skew(FILE*); // print average skew per level
  void print_corr(FILE*); // print correlation per level
  void build_tree(dlist* Flist, struct filter filters[]);
  void lsort();    // sort nodes by level 
};
