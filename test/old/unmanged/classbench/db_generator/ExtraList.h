// File: ExtraList.h
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for ExtraList
//   - Maintains distribution of "extra" field values
//   - Reads in distribution from seed file
//   - Selects extra fields based on selected protocol
//
// A seed file defines the number of extra fields, the number of non-wildcard
// values an extra field may assume, and the probability of choosing a non-wildcard
// extra field value.
// The seed file format is as follows:
// -extra
// <# of extra fields>
// <Protocol number>   <# of non-wildcard values for extra field 1>,<prob. of non-wildcard for extra field 1>  <# of non-wildcard values for extra field 2>,<prob. of non-wildcard for extra field 2> ...
// <Protocol number>   <# of non-wildcard values for extra field 1>,<prob. of non-wildcard for extra field 1>  <# of non-wildcard values for extra field 2>,<prob. of non-wildcard for extra field 2> ...
//  
// The values of non-wildcard values are randomly chosen.
// The distribution of non-wildcard values is also randomly chosen.
// *** If a protocol does not specify extra fields, or specifies fewer extra fields, then it must
//     include proper entries in the seed file (0,1.0).  For example:
//
// -extra
// 3
// 17   4,0.24   5,0.22   2,0.11
// 47   2,0.5    0,1.0    0,1.0
// 99   0,1.0    0,1.0    0,1.0

#ifndef __EXTRALIST_H_ 
#define __EXTRALIST_H_

// We maintain a 2-D list (a list of lists)
// This is the list header for a list of extra fields associated with a protocol
struct ExtraListHeader{
  int prot_num;
  struct ExtraListItem **field;
  struct ExtraListHeader *prev;
  struct ExtraListHeader *next;
};

// This is a list item for an extra field
struct ExtraListItem{
  int num;
  int *value;
  float *prob;
};

class ExtraList {
  int N; // Number of extra fields
  int P; // Number of protocols
  struct ExtraListHeader *first; // pointer to first item in list
  struct ExtraListHeader *last;  // pointer to last item in list
 public:
  ExtraList(int P); // constructor
  ~ExtraList(); // destructor
  void choose(int prot, int *extras); // choose extras based on probability
  void read(FILE *fp, float scale_factor); // read distributions from input file
  void print(FILE *fp); // print distribution to output file
  struct ExtraListHeader* operator()(int prot); // return pointer to header with protocol number
  int size(); // return the number of extra fields
};

#endif
