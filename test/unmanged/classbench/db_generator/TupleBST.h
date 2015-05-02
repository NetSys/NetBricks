// File: TupleBST.h
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for TupleBST
//   - Maintains a binary search tree of tuples, keyed by scope value
//       * Each node has a tuple definition (FiveTuple) and a pointer to a list of
//         filter indexes for filters specifying the tuple
//         of port range pair types (wc/wc, wc/em, etc.)
//   - Supports insert of a tuple, returns pointer to list of filter indices
//   - Returns a list of pointers to lists of filter indices in order of increasing scope
//       * This allows us to order filters in such a manner as to prevent nesting

#ifndef __TUPLEBST_H_ 
#define __TUPLEBST_H_

#include "dlist.h"

struct FiveTuple{
  int sa_len;
  int da_len;
  int sp_wid;
  int dp_wid;
  int prot;
  int flag;
};

struct TupleBST_item{
  TupleBST_item* left;
  TupleBST_item* right;
  TupleBST_item* parent;
  // scope value for tuple
  int scope;
  // tuple definition
  struct FiveTuple tuple;
  // List of indexes to filters in temp_filters
  // that map to the tuple
  dlist* FilterIndexListPtr;
};

class TupleBST {
  int N; // Number of extra fields
  int PtrIndex; // Index into ListOfFilterIndexPtrs
  struct TupleBST_item* root; // pointer to root node
  dlist** ListOfFilterIndexPtrs; // array of dlist pointers
  void InorderTreeWalk(TupleBST_item*); // constructs ListOfFilterIndexPtrs
  void PrintNode(TupleBST_item*);
  void cleanup(TupleBST_item*);
  int scope(FiveTuple*);
 public:
  TupleBST(); // constructor
  ~TupleBST(); // destructor
  int size(); // return the number of tuples
  dlist** GetTupleLists(); // construct list of pointers to lists of filter indexes
  dlist* Insert(FiveTuple* ftuple); // add tuple to list, return pointer to list of filter indexes
  void PrintTree();
};

#endif
