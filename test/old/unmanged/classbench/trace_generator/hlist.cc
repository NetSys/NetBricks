// File: hlist.cc
// David E. Taylor 
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for data structure representing a list of headers
// List is a dynamically allocated, doubly-linked list
//
#include "stdinc.h"
#include "hlist.h"

hlist::hlist(int d) {
  df = d;
  first = last = NULL;
  num = 0;
}

hlist::~hlist() {
  struct hlist_item *temp;
  while (first != NULL) {
    temp = first->next;
    delete(first->hdr);
    delete(first);
    first = temp;
  }
}

// Return the i-th element, where the first is 1.
struct hlist_item* hlist::operator()(int i) {
  if (i <= 0 || (i > num && num > 0)) {
    fprintf(stderr,"hlist::operator(): item %d out of range, num = %d\n",i,num);
    exit(1);      
  }
  // Items are maintained in-order
  // Index i items to the "right"
  struct hlist_item *temp;
  temp = first;
  while (--i) {
    temp = temp->next;
  }
  return temp;
}


// Print the contents of the hlist.
void hlist::print(FILE* fp) {
  struct hlist_item *i;
  for (i = first; i != NULL; i = i->next){
    for (int j = df-1; j >= 0; j--){
      fprintf(fp,"%u\t",i->hdr[j]);
    }
    fprintf(fp,"%d\n",i->filt);
  }
  return;
}

// Return the current number of elements in the hlist.
int hlist::size() {
  return num;
}

// add item to list
void hlist::add(unsigned *hdr, int filt){
  struct hlist_item *newitem;
  newitem = new struct hlist_item;
  newitem->hdr = new unsigned[df];
  for (int i = 0; i < df; i++) newitem->hdr[i] = hdr[i];
  newitem->filt = filt;
  newitem->prev = last;
  newitem->next = NULL;
  if (first == NULL) first = newitem;
  else last->next = newitem;
  last = newitem;
  num++;
  return;
}

void hlist::qsort(int i, int j) {
  // Sort subarray between item i and item j
  int x, p, q, temp;
  hlist_item *item, *pitem, *qitem;

  if (i >= j) return;
  
  // partition the subarray around a random pivot
  item = (*this)(randint(i,j));
  x = item->filt;

  for (p = i, q = j; true ; p++, q--) {
    pitem = (*this)(p);
    qitem = (*this)(q);
    while (pitem->filt < x) {p++; pitem = (*this)(p);}
    while (qitem->filt > x) {q--; qitem = (*this)(q);}
    if (p >= q) break;
    pitem = (*this)(p);
    qitem = (*this)(q);
    temp = pitem->filt; pitem->filt = qitem->filt; qitem->filt = temp;
  }
  // recursively sort the two "halves"
  qsort(i,p-1); qsort(p,j);
}

void hlist::sort() {
  qsort(1,num);
}

// Return true if i in hlist, else false.
bit hlist::mbr(int i) {
  if (num == 0) return 0;
  struct hlist_item *j;
  j = first;
  while (j != NULL){
    if (j->filt == i) return 1;
    j = j->next;
  }
  return 0;
}
