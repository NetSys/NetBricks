// File: sbintree.h
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for sbintree
//   - Maintains a binary tree for generating source addresses given a list of
//     source address prefix lengths
//   - Reads in nesting and skew parameters from seed file
//   - Guarantees nesting will not be exceeded, attempts to maintain skew statistics

#ifndef __SBINTREE_H_ 
#define __SBINTREE_H_

typedef int level;

struct stnode {
  level lvl;
  struct stnode *parent;
  struct stnode *child0;
  struct stnode *child1;
  int wt_child0;
  int wt_child1;
};
  
class sbintree {
  struct stnode *root; // pointer to root node
  float *skew; // array of target skews for each level;
  float *p1child; // probability that a node at a given level has one child
  float *p2child; // probability that a node at a given level has two children
  int num_stnodes; // number of tree nodes 
  void add_node(struct stnode *prnt, int lev, int dir, unsigned int addr, dlist* Flist, struct filter filters[], int CurrNest);
  int Nest; // Maximum allowed nesting
 public:
  sbintree();
  ~sbintree();
  void delete_node(struct stnode *me);
  int nodes(); // return number of nodes
  void read_skew(FILE*); // read in source address tree statistics
  void read_nest(FILE*); // read in source address tree statistics
  void scale_skew(float scale_factor); // scale branching and skew according to scaling factor
  void print_skew(FILE*); // print average skew per level
  void build_tree(dlist* Flist, struct filter filters[]);
};

#endif
