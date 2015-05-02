// File: dlist.h
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Header file for data structure representing a list of integers
// List is a dynamically allocated, doubly-linked list
#ifndef __DLIST_H_ 
#define __DLIST_H_

struct dlist_item {
  int key;
  struct dlist_item *prev;
  struct dlist_item *next;
};

class dlist {
	struct dlist_item *first;	// pointer to first dlist_item of dlist
      	struct dlist_item *last;	// pointer to last dlist_item of dlist
      	int	num;			// number of dlist_items currently on the dlist
	void	qsort(int,int);	        // quicksort subroutine
public:		dlist();
		~dlist();
	struct dlist_item* operator()(int);	// access dlist_item
	void	operator&=(int);	// append item
	void	operator<<=(int);	// remove initial items
	void	operator=(dlist*);	// dlist assignment
	void	push(int);		// push item onto front of dlist
	bit	mbr(int);		// return true if member of dlist
	int	suc(int);		// return successor
	void	clear();		// remove everything
	void	print(FILE* fp);	// print the items on dlist
	int	frst();			// return first element of the dlist
	int	lst();			// return last element of the dlist
	int	size();			// return the number of items currently stored in the dlist
	void	insert(struct dlist_item*, int);       // insert item prior to given dlist_item
	void    sort();                 // sort list items in ascending order by key
};

#endif
