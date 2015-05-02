// File: hlist.h
// David E. Taylor 
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Header file for data structure representing a list of headers
// List is a dynamically allocated, doubly-linked list
//
#ifndef __HLIST_H_ 
#define __HLIST_H_

struct hlist_item {
  unsigned *hdr;
  int filt;
  struct hlist_item *prev;
  struct hlist_item *next;
};

class hlist {
  int df;  // number of fields in headers
  struct hlist_item *first;	// pointer to first hlist_item of hlist
  struct hlist_item *last;	// pointer to last hlist_item of hlist
  int	num;			// number of hlist_items currently on the hlist
  void	qsort(int,int);	        // quicksort subroutine
public:
  hlist(int);
  ~hlist();
  struct hlist_item* operator()(int);	// access hlist_item
  void	print(FILE* fp);	// print the items on hlist
  int	size();			// return the number of items currently stored in the hlist
  void	add(unsigned *hdr, int filt);  // add header to filter list with bestmatching filter filt
  bit   mbr(int);               // return true if a header matching filter is member of hlist
  void  sort();                 // sort list items in ascending order by bestmatching filter filt
};

#endif
