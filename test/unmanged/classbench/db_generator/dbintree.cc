// File: dbintree.cc
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for dbintree

#include "stdinc.h"
#include "dlist.h"
#include "dbintree.h"

dbintree::dbintree() {
// Initialize to graph with N vertices and no edges.
  skew = new float[33];
  corr = new float[33];
  p1child = new float[33];
  p2child = new float[33];
  num_tnodes = 0;
  root = NULL;
  for (int u = 0; u < 33; u++) {
    skew[u] = 0;
    corr[u] = 0;
    p1child[u] = 0;
    p2child[u] = 0;
  }
}

dbintree::~dbintree() {
  delete(skew);
  delete(corr);
  delete(p1child);
  delete(p2child);
  // call recursive node destructor
  if (root != NULL) delete_node(root);
}

void dbintree::delete_node(struct tnode *me){
  if (me->child0 != NULL) delete_node(me->child0);
  if (me->child1 != NULL) delete_node(me->child1);
  delete(me);
  return;
}

int dbintree::nodes(){
  return num_tnodes;
}

void dbintree::read_nest(FILE* fp_in){
  int done = 0;
  int matches = MAXFILTERS;
  char comm[6];
  char da_comm[]="-dnest";

  // read in destination address nest
  // printf("read in destination address nest\n");
  while (matches != EOF && done == 0) {
    matches = fscanf(fp_in,"%s",comm);
    // printf("comm = %s\n",comm);
    // printf("matches = %d\n",matches);
    if (strcmp(comm,da_comm) == 0) done = 1;
  }
  if (matches == EOF) {
    fprintf(stderr,"No destination address nest specified for custom distribution.\n");
    exit(1);
  }
  matches = fscanf(fp_in,"%d",&Nest);
  // printf("matches = %d\n",matches);
  // printf("Nest = %d\n",Nest);
  return;
}

void dbintree::read_skew(FILE* fp_in){
  int done = 0;
  int matches = MAXFILTERS;
  int level;
  float p1_t; 
  float p2_t;
  float f_skew;
  char comm[6];
  char sa_comm[]="-dskew";

  // read in destination address skew
  // printf("read in destination address skew\n");
  while (matches != EOF && done == 0) {
    matches = fscanf(fp_in,"%s",comm);
    // printf("comm = %s\n",comm);
    // printf("matches = %d\n",matches);
    if (strcmp(comm,sa_comm) == 0) done = 1;
  }
  if (matches == EOF) {
    fprintf(stderr,"No destination address skew specified for custom distribution.\n");
    exit(1);
  }
  done = 0;
  while(done == 0){
    matches = fscanf(fp_in,"%d\t%f\t%f\t%f",&level,&p1_t,&p2_t,&f_skew);
    // printf("matches = %d\n",matches);
    // printf("level = %d, skew = %.4f\n",level,skew);
    if (matches == 4) {
      if (level <= 32) {
	p1child[level] = p1_t;
	p2child[level] = p2_t;
	skew[level] = f_skew;
      }
      else {
	fprintf(stderr,"Level for destination address skew is greater than 32.\n");
	exit(1);
      }
      // printf("Read line: %d\t%.4f\t%.4f\t%.4f\n",level,p1_t,p2_t,f_skew);
    }
    else {
      done = 1;
    }
  }
  return;
}

void dbintree::read_corr(FILE* fp_in){
  int done = 0;
  int matches = 0;
  char comm[6];
  char p_comm[]="-pcorr";
  int index;
  float icorr;

  // read in address correlation
  while (matches != EOF && done == 0) {
    matches = fscanf(fp_in,"%s",comm);
    if (strcmp(comm,p_comm) == 0) done = 1;
  }
  if (matches == EOF) {
    fprintf(stderr,"No prefix correlation specified for custom distribution.\n");
    return;
  }
  done = 0;
  for (int i = 0; i < MAXFILTERS && done == 0; i++){
    matches = fscanf(fp_in,"%d %f",&index,&icorr);
    corr[index] = icorr;
    if (matches != 2) done = 1;
  }

  return;
}

void dbintree::print_skew(FILE *fp) {
  
  fprintf(fp,"Level\tp1\tp2\tSkew\n");
  for (int i = 0; i < 33; i++) {
    fprintf(fp,"%d\t%.4f\t%.4f\t%.4f\n",
	    i,p1child[i],p2child[i],skew[i]);
  }
  return;
}

void dbintree::print_corr(FILE *fp) {
  
  fprintf(fp,"Level\tCorr\n");
  for (int i = 0; i < 33; i++) {
    fprintf(fp,"%d\t%.4f\n",i,corr[i]);
  }
  return;
}

void dbintree::build_tree(dlist* Flist, struct filter filters[]){
  unsigned int addr = 0;
  // Create copy of list
  dlist *temp_list = new dlist();
  (*temp_list)=(Flist);
  
  // Build stubs
  // Pass filter list and filters to root node
  add_node(0,0,0);
  // printf("build_tree: adding stubs\n");
  add_stub(root,addr,temp_list,filters,0);
  // printf("build_tree: done adding stubs\n");
  // Pass filter list and filters to root node
  // printf("build_tree: finishing nodes\n");
  finish_node(root,addr,temp_list,filters,0);
  // printf("build_tree: done finishing nodes\n");
  delete(temp_list);
  return;
}

void dbintree::add_node(struct tnode *prnt, int lev, int dir){
  // Increment total number of nodes
  num_tnodes++;
  struct tnode *me;
  me = new struct tnode;
  me->lvl = lev;
  me->valid = 0;
  me->child0 = NULL;
  me->child1 = NULL;
  me->wt_child0 = 0;
  me->wt_child1 = 0;
  if (lev == 0) {
    me->parent = NULL;
    root = me;
  }
  else {
    me->parent = prnt;
    // Set parent's child pointer
    if (dir == 0) prnt->child0 = me;      
    else prnt->child1 = me;
  }
  me->stubList = new dlist();
  return;
}

void dbintree::add2child_stublist(struct tnode *node, int dir, int filt){
  struct tnode *child;
  if (dir == 0) {
    if (node->child0 == 0) add_node(node, (node->lvl)+1, 0);
    child = node->child0;
  } else {
    if (node->child1 == 0) add_node(node, (node->lvl)+1, 1);
    child = node->child1;
  }
  *(child->stubList)&=filt;
  return;
}

void dbintree::add_stub(struct tnode *node, unsigned int addr, dlist* Flist, struct filter filters[], int CurrNest){
  int Flist_size = 0;
  int lev = node->lvl;
  
  // printf("add_stub:\n");
  // printf("level = %d\n",lev);
  // printf("address = %u\n",addr);
  // printf("Flist = "); Flist->print(stdout); printf("\n");
  // printf("stublist = "); node->stubList->print(stdout); printf("\n");
  // printf("CurrNest = %d\n",CurrNest);

  // Find the number of items in the list
  Flist_size = (*Flist).size();
  // printf("Flist_size = %d\n",Flist_size);
  
  double temp;
  unsigned int sa;

  unsigned int addr0, addr1;
  // Adjust addresses
  if (lev == 0) {
    addr0 = 0;
    addr1 = 1;
    addr1 = addr1 << 31;
  } else {
    addr0 = addr >> (32 - lev);
    addr0 = addr0 << (32 - lev);
    addr1 = addr >> (32 - lev);
    addr1 = addr1 << 1;
    addr1 += 1;
    addr1 = addr1 << (31 - lev);
  }
 
  // Allocate temp_list's
  dlist *tempList = new dlist();

  // List counters
  int cnt1, cnt0;
  cnt0 = cnt1 = 0;

  int q;
  int MyNest = CurrNest;
  // Flag set if prefix added to this node
  int lev1_flag = 0;
  // Examine filters to "add" to this node
  struct dlist_item *filt;
  filt = (*Flist)(1);
  while (filt != NULL && Flist_size--) {
    q = filt->key;
    filt = filt->next;
    // Remove q (first item) from list
    // printf("Removing %d from Flist\n",q);
    (*Flist)<<=1;
    // printf("filters[%d].sa_len = %d, filters[%d].da_len = %d\n",q,filters[q].sa_len,q,filters[q].da_len);
    if (filters[q].da_len == lev) {
      // Destination address complete
      filters[q].da = addr;
      //printf("add_stub: destination address complete for filter %d\n",q);
      //printf("\tlevel = %d\n",lev);
      //printf("\tfilters[%d].sa_len = %d, filters[%d].da_len = %d\n",q,filters[q].sa_len,q,filters[q].da_len);
      // Set valid flag in node
      node->valid = 1;
    } else {
      // da_len > lvl
      (*tempList) &= q;
      // Check for termination at next level
      if (filters[q].da_len == lev+1) lev1_flag = 1;
    }
  }
  // Increment MyNest, if necessary
  if (node->valid == 1) MyNest++;

  Flist_size = (*tempList).size();
  // printf("Flist_size = %d, tempList: ",Flist_size); tempList->print(stdout); printf("\n");
  // if lev1_flag == 1, dump all lev1 filters to one side
  // printf("lev = %d, MyNest = %d, Nest = %d, lev1_flag = %d\n",lev,MyNest,Nest,lev1_flag);
  if ((Flist_size > 1) && (MyNest >= Nest - 1) && (lev1_flag == 1) && (lev < 31)){
    // printf("add_stub: Enforcing nesting limit\n");
    // Add all filters to stublist and let the finish_node process distribute them
    int fptr;
    for (int i = Flist_size; i > 0; i--){
      fptr = (*tempList).frst();
      *(node->stubList)&=fptr;
      (*tempList)<<=1;
    }
  }
  else {
    // No danger of violating nesting threshold, proceed normally
    // printf("Flist_size = %d, tempList: ",Flist_size); tempList->print(stdout); printf("\n");
    // else, distribute stuff normally
    dlist *tempList0 = new dlist();
    dlist *tempList1 = new dlist();
    int fptr;
    for (int i = Flist_size; i > 0; i--){
      fptr = (*tempList).frst();
      sa = filters[fptr].sa;
      
      // Continue building destination address
      if (filters[fptr].sa_len <= lev) {
	// Source prefix exhausted
	// Add filter to stublist of this node (level)
	// printf("Source prefix exhausted, Adding filter %d to tempstubList\n",fptr);
	// (*tempstubList)&=fptr;
	// stub_cnt++;
	*(node->stubList)&=fptr;
      } else {
	// printf("Branching based on correlation\n");
	// sa_len > lvl
	// Prevent prefix nesting
	sa = sa << lev;
	sa = sa >> 31;
	// sa now equals next "bit" of source address
	if (filters[fptr].sa_len == (lev+1)) {
	  // Source prefix will be exhausted at next level
	  // Take opposite "branch"
	  if (sa == 0) {
	    // take 1 branch
	    // printf("Preventing nesting, adding filter %d to child1 stublist\n",fptr);
	    if (node->child1 == NULL) add_node(node,lev+1,1);
	    add2child_stublist(node,1,fptr);
	    node->wt_child1++;
	  }
	  else {
	    // take 0 branch
	    // printf("Preventing nesting, adding filter %d to child0 stublist\n",fptr);
	    if (node->child0 == NULL) add_node(node,lev+1,0);
	    add2child_stublist(node,0,fptr);
	    node->wt_child0++;
	  }
	} else {
	  // Check for continued correlation
	  temp = drand48();
	  if (temp <= corr[lev+1]) {
	    // Correlation continues
	    if (sa == 1) {
	      // take 1 branch
	      // printf("Correlation continues, adding filter %d to tempList1\n",fptr);
	      (*tempList1)&=fptr;
	      cnt1++;
	    } 
	    else {
	      // take 0 branch
	      // printf("Correlation continues, adding filter %d to tempList0\n",fptr);
	      (*tempList0)&=fptr;
	      cnt0++;
	    } 
	  } else {
	    // Break correlation
	    if (sa == 0) {
	      // take 1 branch
	      // printf("Breaking correlation, adding filter %d to child1 stublist\n",fptr);
	      if (node->child1 == NULL) add_node(node,lev+1,1);
	      add2child_stublist(node,1,fptr);
	      node->wt_child1++;
	    } 
	    else {
	      // take 0 branch
	      // printf("Breaking correlation, adding filter %d to child0 stublist\n",fptr);
	      if (node->child0 == NULL) add_node(node,lev+1,0);
	      add2child_stublist(node,0,fptr);
	      node->wt_child0++;
	    } 
	  }
	}
      }
      (*tempList)<<=1;
    }
    
    // Pass lists onto children
    if (cnt0 > 0) {
      node->wt_child0 += cnt0;
      if (node->child0 == NULL) add_node(node,lev+1,0);
      add_stub(node->child0, addr0, tempList0, filters, MyNest);
    }
    if (cnt1 > 0) {
      node->wt_child1 += cnt1;
      if (node->child1 == NULL) add_node(node,lev+1,1);
      add_stub(node->child1, addr1, tempList1, filters, MyNest);
    }
    delete(tempList0);
    delete(tempList1);
  }
  delete(tempList);
  return;
}

void dbintree::finish_node(struct tnode *node, unsigned int addr, dlist* Flist, struct filter filters[], int CurrNest){
  int Flist_size = 0;
  int stubList_size = 0;
  int lev = node->lvl;
  int MyNest = CurrNest;

  /*
  printf("finish_node:\n");
  printf("level = %d\n",lev);
  printf("address = %u\n",addr);
  printf("Flist = "); (*Flist).print(stdout); printf("\n");
  printf("stublist = "); node->stubList->print(stdout); printf("\n");
  */

  // Find the number of items in the lists
  // for (int i = 1; (*Flist)(i) != NULL; i++) Flist_size = i;
  Flist_size = (*Flist).size();
  // printf("Flist_size = %d\n",Flist_size);
  // for (int i = 1; (*tnodes[node].stubList)(i) != NULL; i++) stubList_size = i;
  stubList_size = node->stubList->size();
  // printf("stubList_size = %d\n",stubList_size);

  int templist_size = stubList_size+Flist_size;
  dlist *templist = new dlist();
  struct dlist_item *temp_item;

  // Copy items from Flist
  for (temp_item = (*Flist)(1); temp_item != NULL; temp_item = temp_item->next) (*templist)&=(temp_item->key);
  // Copy items from stubList
  for (temp_item = (*node->stubList)(1); temp_item != NULL; temp_item = temp_item->next) (*templist)&=(temp_item->key);

  // Examine filters to "add" to this node

  // Flag set if prefix added to this node
  int lev1_flag = 0;
  struct dlist_item *filt;
  filt = (*templist)(1);
  while (filt != NULL && templist_size--) {
    int q = filt->key;
    filt = filt->next;
    // Remove q (first item) from list
    // printf("Removing %d from templist\n",q);
    (*templist)<<=1;
    // printf("filters[%d].da_len = %d\n",q,filters[q].da_len);
    if (filters[q].da_len == lev) {
      // Assign filter to this node (level)
      filters[q].da = addr;
      // Remove filter from templist (do not append it)
      //printf("finish_node: destination address complete for filter %d\n",q);
      //printf("\tlevel = %d\n",lev);
      //printf("\tfilters[%d].sa_len = %d, filters[%d].da_len = %d\n",q,filters[q].sa_len,q,filters[q].da_len);
      // Set valid flag in node
      node->valid = 1;
    } else {
      // Put q back on the list
      // printf("Appending %d back to templist\n",q);
      (*templist)&=q;
    }
    if (filters[q].da_len == lev+1) lev1_flag = 1;
    // printf("filt =  %d\n",filt);
  }
  // Increment MyNest, if necessary
  if (node->valid == 1) MyNest++;

  templist_size = 0;
  // Find the number of items in the list
  // for (int i = 1; (*templist)(i) != NULL; i++) templist_size = i;
  templist_size = (*templist).size();
  // printf("templist_size = %d\n\n",templist_size);

  double temp;
  int path;
  unsigned int addr0, addr1;

  // Adjust addresses
  if (lev == 0) {
    addr0 = 0;
    addr1 = 1;
    addr1 = addr1 << 31;
  } else {
    addr0 = addr >> (32 - lev);
    addr0 = addr0 << (32 - lev);
    addr1 = addr >> (32 - lev);
    addr1 = addr1 << 1;
    addr1 += 1;
    addr1 = addr1 << (31 - lev);
  }
  
  // Create an empty list
  dlist* emptylist = new dlist();//1);

  // If list is empty...
  if (templist_size == 0) {
    // printf("tnodes[%d].wt_child0 = %d, tnodes[%d].wt_child1 = %d\n",node,tnodes[node].wt_child0,node,tnodes[node].wt_child1);
    // Complete child nodes
    if (node->wt_child0 > 0) finish_node(node->child0,addr0,emptylist,filters,MyNest);
    if (node->wt_child1 > 0) finish_node(node->child1,addr1,emptylist,filters,MyNest);
  } else {
    // Choose heavy path
    if (node->wt_child0 > node->wt_child1) path = 0;
    else if (node->wt_child0 < node->wt_child1) path = 1;
    else { // wts are equal
      temp = drand48();
      if (temp < 0.5) path = 0;
      else path = 1;
    }
    
    // List has at least one filter...

    // If at the nesting threshold and list has more than one child,
    //   then split list (allocate all nodes with level == lev1 to one path)
    // printf("lev = %d, MyNest = %d, Nest = %d, lev1_flag = %d\n",lev,MyNest,Nest,lev1_flag);
    if ((templist_size > 1) && (MyNest >= Nest - 1) && (lev1_flag == 1) && (lev < 31)){
      // Allocate nest_list
      dlist *nest_list = new dlist();
      dlist *other_list = new dlist();
      
      int fptr;
      //      for (dlist_item* index = (*templist)(1); index != NULL; index = index->next){
      for (int i = templist_size; i > 0; i--){
	fptr = (*templist).frst();
	if (filters[fptr].da_len == lev+1) {
	  // printf("Adding %d to nest_list\n",fptr);
	  (*nest_list)&=fptr;
	} else {
	  // printf("Adding %d to other_list\n",fptr);
	  (*other_list)&=fptr;
	}
	(*templist)<<=1;
      }
      // printf("nest_list->size() = %d, other_list->size() = %d, path = %d\n",nest_list->size(),other_list->size(),path == 0);
      // Pass lists onto children
      if (node->child0 == NULL) add_node(node, lev+1, 0);
      if (node->child1 == NULL) add_node(node, lev+1, 1);      
      if (nest_list->size() > other_list->size()){
	if (path == 0){
	  finish_node(node->child0, addr0, nest_list, filters, MyNest);
	  finish_node(node->child1, addr1, other_list, filters, MyNest);
	  node->wt_child0 += nest_list->size();
	  node->wt_child1 += other_list->size();
	} else {
	  // path == 1
	  finish_node(node->child1, addr1, nest_list, filters, MyNest);
	  finish_node(node->child0, addr0, other_list, filters, MyNest);
	  node->wt_child1 += nest_list->size();
	  node->wt_child0 += other_list->size();
	}
      } else {
	// nest_list->size() <= other_list->size()
	if (path == 0){
	  finish_node(node->child1, addr1, nest_list, filters, MyNest);
	  finish_node(node->child0, addr0, other_list, filters, MyNest);
	  node->wt_child1 += nest_list->size();
	  node->wt_child0 += other_list->size();
	} else {
	  // path == 1
	  finish_node(node->child0, addr0, nest_list, filters, MyNest);
	  finish_node(node->child1, addr1, other_list, filters, MyNest);
	  node->wt_child0 += nest_list->size();
	  node->wt_child1 += other_list->size();
	}
      }
      delete(other_list);
      delete(nest_list);
    }
    else {
      // Branch according to branching probability,
      // if there are not already child nodes defined
      temp = drand48();
      if (temp < p1child[lev] && (node->child0 == NULL || node->child1 == NULL)){
	// Node will have one child
	if (node->child0 != NULL) {
	  // Pass list on to zero child
	  finish_node(node->child0, addr0, templist, filters, MyNest);
	}
	else if (node->child1 != NULL) {
	  // Pass list on to one child
	  finish_node(node->child1, addr1, templist, filters, MyNest);
	}
	else {
	  // Neither child is valid
	  // Pass list on to heavy path
	  if (path == 0) {
	    add_node(node, lev+1, 0);
	    finish_node(node->child0, addr0, templist, filters, MyNest);
	  } else {
	    add_node(node, lev+1, 1);
	    finish_node(node->child1, addr1, templist, filters, MyNest);
	  }
	} 
      } else {
	// Node will have two children
	// Split list according skew
	int total_wt = templist_size + node->wt_child0 + node->wt_child1;
	float hvy, lite, thvy, tlite;
	int delta_hvy, delta_lite;
	thvy = total_wt / ((float)2 - skew[lev]);
	thvy = floor(thvy);
	tlite = total_wt - thvy;
	// printf("thvy = %.1f, tlite = %.1f\n",thvy,tlite);
	if (path == 1) {
	  hvy = node->wt_child1;
	  lite = node->wt_child0;
	} else {
	  lite = node->wt_child1;
	  hvy = node->wt_child0;
	}
	// printf("hvy = %.1f, lite = %.1f\n",hvy,lite);
	delta_hvy = (int)thvy - (int)hvy;
	delta_lite = (int)tlite - (int)lite;
	// printf("delta_hvy = %d, delta_lite = %d\n",delta_hvy,delta_lite);
	if (delta_lite <= 0) {
	  // add all filters to hvy path
	  if (path == 1) {
	    // printf("Adding all filters to heavy path = 1\n");
	    node->wt_child1 += templist_size;
	    if (node->child1 == NULL) add_node(node, lev+1, 1);
	    finish_node(node->child1, addr1, templist, filters, MyNest);
	    // Finish child0 if it has stubs
	    if (node->wt_child0 > 0) finish_node(node->child0,addr0,emptylist,filters, MyNest);
	  } else { // path == 0
	    // printf("Adding all filters to heavy path = 0\n");
	    node->wt_child0 += templist_size;
	    if (node->child0 == NULL) add_node(node, lev+1, 0);
	    finish_node(node->child0, addr0, templist, filters, MyNest);
	    // Finish child1 if it has stubs
	    if (node->wt_child1 > 0) finish_node(node->child1,addr1,emptylist,filters, MyNest);
	  }
	}
	else if (delta_hvy <= 0){
	  // add all filters to lite path
	  if (path == 0) {
	    // printf("Adding all filters to lite path = 1\n");
	    node->wt_child1 += templist_size;
	    if (node->child1 == NULL) add_node(node, lev+1, 1);
	    finish_node(node->child1, addr1, templist, filters, MyNest);
	    // Finish child0 if it has stubs
	    if (node->wt_child0 > 0) finish_node(node->child0,addr0,emptylist,filters, MyNest);
	  } else { // path == 1
	    // printf("Adding all filters to lite path 1\n");
	    node->wt_child0 += templist_size;
	    if (node->child0 == NULL) add_node(node, lev+1, 0);
	    finish_node(node->child0, addr0, templist, filters, MyNest);
	    // Finish child1 if it has stubs
	    if (node->wt_child1 > 0) finish_node(node->child1,addr1,emptylist,filters, MyNest);
	  }
	}
	else {
	  // delta_lite > 0 and delta_hvy > 0
	  // Allocate temp_list
	  dlist *temp_list_lite = new dlist();//delta_lite);
	
	  int k;
	  for (int i = delta_lite; i > 0; i--){
	    k = (*templist).frst();
	    // printf("Adding %d to temp_list_lite\n",k);
	    (*temp_list_lite)&=k;
	    // printf("Removing %d from templist\n",k);
	    (*templist)<<=1;
	  }
	  // Pass lists onto children
	  if (node->child0 == NULL) add_node(node, lev+1, 0);
	  if (node->child1 == NULL) add_node(node, lev+1, 1);
	  if (path == 0){
	    node->wt_child0 += delta_hvy;
	    finish_node(node->child0, addr0, templist, filters, MyNest);
	    node->wt_child1 += delta_lite;
	    finish_node(node->child1, addr1, temp_list_lite, filters, MyNest);
	  } else {
	    // path == 1
	    node->wt_child1 += delta_hvy;
	    finish_node(node->child1, addr1, templist, filters, MyNest);
	    node->wt_child0 += delta_lite;
	    finish_node(node->child0, addr0, temp_list_lite, filters, MyNest);
	  }
	  delete(temp_list_lite);
	}
      }
    }
  }
  delete(emptylist);
  delete(templist);
  return;
}

void dbintree::scale_skew(float scale_factor){
  float targetDskew = scale_factor;
  float nodeDskew, p2D, new_nodeDskew, newp2child;
  for (int i = 0; (i <= 31 && targetDskew > 0); i++){
    // printf("Level = %d\n",i);
    nodeDskew = 2*((1-p2child[i]) + (skew[i]*p2child[i]));
    // printf("nodeDskew = %.6f, targetDskew = %.6f\n",nodeDskew,targetDskew);
    if (nodeDskew <= targetDskew) {
      // Completely balance this level
      targetDskew -= nodeDskew;
      skew[i] = 0;
      p2child[i] = 1;
      p1child[i] = 1 - p2child[i];
    } else {
      // First, find D achievable by shifting to all 2-child nodes
      p2D = nodeDskew - 2*skew[i];
      // printf("p2D = %.6f\n",p2D);
      if (p2D > targetDskew) {
	// Adjust p2child in order to achieve target
	new_nodeDskew = nodeDskew - targetDskew;
	newp2child = ((new_nodeDskew/2) - 1)/(skew[i] - 1);
	// printf("new_nodeDskew = %.6f, newp2child = %.6f\n",new_nodeDskew,newp2child);
	p2child[i] = newp2child;
	p1child[i] = 1 - p2child[i];
	return;
      } else {
	// Make all nodes at this level have 2 children
	p2child[i] = 1;
	p1child[i] = 0;
	// Adjust skew to hit target
	skew[i] = ((2*skew[i]) - targetDskew)/2;
	// printf("skew = %.6f\n",skew[i]);
	return;
      }
    }
  }
  return;
}
