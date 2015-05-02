// File: flist.h
// David E. Taylor 
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Header file for data structure representing a list of filters
// containing d fields
// Each field, d, specifies a range, low[d] to high[d]
//
#ifndef __FLIST_H_ 
#define __FLIST_H_

class flist {
  int df; // Number of fields in the filters
  int Nf; // Number of filters in the list
  unsigned **lowf;
  unsigned **highf;
public:
  flist(int, int); // Constructor
  ~flist(); // Destructor
  int d(); // return number of fields per filter
  int N(); // return capacity of filter list
  //   int size(); // return number of filters currently stored in filter list
  unsigned low(int filt, int field); // return low bound for filter i, field j
  unsigned high(int filt, int field); // return low bound for filter i, field j
  void add(int filt, int field, unsigned low, unsigned high); // assign low and high bounds for filter i, field j
  void print(FILE *fp); // print filter list
};

#endif
