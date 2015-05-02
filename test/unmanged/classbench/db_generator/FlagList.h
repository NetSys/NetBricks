// File: FlagList.h
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for FlagList
//   - Maintains distribution of flag values for each protocol
//   - Reads in distribution from seed file
//   - Selects flag fields based on selected protocol

#ifndef __FLAGLIST_H_ 
#define __FLAGLIST_H_

struct FlagListItem{
  unsigned flags;
  unsigned flags_mask;
  float prob;
  struct FlagListItem *prev;
  struct FlagListItem *next;
};

class FlagList {
  struct FlagListItem **first;       // array of pointers to first item in list
  struct FlagListItem **last;       // array of pointers to last item in list
 public:
  FlagList(); // constructor
  ~FlagList(); // destructor
  void choose(float p, int prot, unsigned *flags, unsigned *flags_mask); // choose flags based on probability
  void read(FILE *fp); // read distributions from input file
  void print(FILE *fp); // print distribution to output file
};

#endif
