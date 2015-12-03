// File: dlist.cc
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for data structure representing a list of integers
// List is a dynamically allocated, doubly-linked list
//
#include "stdinc.h"
#include "dlist.h"

dlist::dlist() {
  first = last = NULL;
  num = 0;
}

dlist::~dlist() {
  struct dlist_item *temp;
  while (first != NULL) {
    temp = first->next;
    delete(first);
    first = temp;
  }
}

// Remove all elements from dlist.
void dlist::clear() {
  struct dlist_item *temp;
  while (first != NULL) {
    temp = first->next;
    delete(first);
    first = temp;
  }
  return;
}

// Return the i-th element, where the first is 1.
struct dlist_item* dlist::operator()(int i) {
  if (i <= 0 || (i > num && num > 0)) {
    fprintf(stderr,"dlist::operator(): item %d out of range, num = %d\n",i,num);
    exit(1);      
  }
  // Items are maintained in-order
  // Index i items to the "right"
  struct dlist_item *temp;
  temp = first;
  while (--i) {
    temp = temp->next;
  }
  return temp;
}

// Add i to the end of the dlist.
void dlist::operator&=(int i) {
  struct dlist_item *temp;
  temp = new struct dlist_item;
  temp->key = i;
  temp->prev = last;
  temp->next = NULL;
  if (num == 0){
    first = temp;
  } else {
    last->next = temp;
  }
  last = temp;
  num++;
  return;
}

// Remove the first i items.
void dlist::operator<<=(int i) {
  struct dlist_item *temp;
  while (first != NULL && i--) {
    temp = first->next;
    delete(first);
    first = temp;
    num--;
  }
  return;
}

// Assign value of right operand to left operand.
void dlist::operator=(dlist *L) {
  clear();
  struct dlist_item *temp;
  temp = (*L)(1);
  for (temp = (*L)(1); temp != NULL; temp=temp->next)
    (*this) &= temp->key;
  return;
}

// Add i to the front of the dlist.
void dlist::push(int i) {
  struct dlist_item *temp;
  temp = new struct dlist_item;
  temp->key = i;
  temp->next = first;
  temp->prev = NULL;
  if (num == 0){
    last = temp;
  } else {
    first->prev = temp;
  }
  first = temp;
  num++;
  return;
}  

// Return true if i in dlist, else false.
bit dlist::mbr(int i) {
  if (num == 0) return 0;
  struct dlist_item *j;
  j = first;
  while (j != NULL){
    if (j->key == i) return 1;
    j = j->next;
  }
  return 0;
}

// Return the successor of i.
int dlist::suc(int i) {
  struct dlist_item *j;
  int k = 0;
  j = first;
  while (j != NULL && k == 0){
    if (j->key == i) k = 1;
    else j = j->next;
  }
  if (k == 0) fatal("dlist::suc: item not on dlist");
  if (j == last) return 0;
  else return j->next->key;
}

// Print the contents of the dlist.
void dlist::print(FILE* fp) {
  struct dlist_item *i;
  for (i = first; i != NULL; i = i->next)
    fprintf(fp,"%d ",i->key);
}

// Return the last element of dlist.
int dlist::lst() {
  if (last != NULL) return last->key;
  else return 0;
}

// Return the first element of dlist.
int dlist::frst() {
  if (first != NULL) return first->key;
  else return 0;
}

// Return the current number of elements in the dlist.
int dlist::size() {
  return num;
}

// insert item at given position
void dlist::insert(struct dlist_item *item, int i){
  struct dlist_item *newitem;
  newitem = new struct dlist_item;
  newitem->key = i;
  newitem->prev = item->prev;
  newitem->next = item;
  if (first == item) first = newitem;
  else item->prev->next = newitem;
  item->prev = newitem;
  num++;
  return;
}

void dlist::qsort(int i, int j) {
  // Sort subarray between item i and item j
  int x, p, q, temp;
  dlist_item *item, *pitem, *qitem;

  if (i >= j) return;
  
  // partition the subarray around a random pivot
  item = (*this)(randint(i,j));
  x = item->key;

  for (p = i, q = j; true ; p++, q--) {
    pitem = (*this)(p);
    qitem = (*this)(q);
    while (pitem->key < x) {p++; pitem = (*this)(p);}
    while (qitem->key > x) {q--; qitem = (*this)(q);}
    if (p >= q) break;
    pitem = (*this)(p);
    qitem = (*this)(q);
    temp = pitem->key; pitem->key = qitem->key; qitem->key = temp;
  }
  // recursively sort the two "halves"
  qsort(i,p-1); qsort(p,j);
}

void dlist::sort() {
  qsort(1,num);
}
