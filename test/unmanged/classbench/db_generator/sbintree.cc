// File: sbintree.cc
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for sbintree

#include "stdinc.h"
#include "dlist.h"
#include "sbintree.h"

sbintree::sbintree() {
// Initialize to graph with N vertices and no edges.
  skew = new float[33];
  p1child = new float[33];
  p2child = new float[33];
  num_stnodes = 0;
  root = NULL;
  for (int u = 0; u < 33; u++) {
    skew[u] = 0;
    p1child[u] = 0;
    p2child[u] = 0;
  }
}

sbintree::~sbintree() {
  delete(skew);
  delete(p1child);
  delete(p2child);
  // call recursive node destructor
  if (root != NULL) delete_node(root);
}

void sbintree::delete_node(struct stnode *me){
  if (me->child0 != NULL) delete_node(me->child0);
  if (me->child1 != NULL) delete_node(me->child1);
  delete(me);
  return;
}

int sbintree::nodes(){
  return num_stnodes;
}

void sbintree::read_nest(FILE* fp_in){
  int done = 0;
  int matches = MAXFILTERS;
  char comm[6];
  char sa_comm[]="-snest";

  // read in source address nest
  // printf("read in source address nest\n");
  while (matches != EOF && done == 0) {
    matches = fscanf(fp_in,"%s",comm);
    // printf("comm = %s\n",comm);
    // printf("matches = %d\n",matches);
    if (strcmp(comm,sa_comm) == 0) done = 1;
  }
  if (matches == EOF) {
    fprintf(stderr,"No source address nest specified for custom distribution.\n");
    exit(1);
  }
  matches = fscanf(fp_in,"%d",&Nest);
  // printf("matches = %d\n",matches);
  // printf("Nest = %d\n",Nest);
  return;
}

void sbintree::read_skew(FILE* fp_in){
  int done = 0;
  int matches = MAXFILTERS;
  int level;
  float p1_t; 
  float p2_t;
  float f_skew;
  char comm[6];
  char sa_comm[]="-sskew";

  // read in source address skew
  // printf("read in source address skew\n");
  while (matches != EOF && done == 0) {
    matches = fscanf(fp_in,"%s",comm);
    // printf("comm = %s\n",comm);
    // printf("matches = %d\n",matches);
    if (strcmp(comm,sa_comm) == 0) done = 1;
  }
  if (matches == EOF) {
    fprintf(stderr,"No source address skew specified for custom distribution.\n");
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
	fprintf(stderr,"Level for source address skew is greater than 32.\n");
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

void sbintree::print_skew(FILE *fp) {
  
  fprintf(fp,"Level\tp1\tp2\tSkew\n");
  for (int i = 0; i < 33; i++) {
    fprintf(fp,"%d\t%.4f\t%.4f\t%.4f\n",
	    i,p1child[i],p2child[i],skew[i]);
  }
  return;
}

void sbintree::build_tree(dlist* Flist, struct filter filters[]){
  unsigned int addr = 0;
  // Create copy of list
  dlist* temp_list = new dlist;
  (*temp_list)=(Flist);
  // printf("temp_list = 0x%08x, Flist = 0x%08x\n",temp_list,Flist);
  // printf("build_tree: temp_list = "); (*temp_list).print(stdout); printf("\n");
  // printf("build_tree: Flist = "); Flist->print(stdout); printf("\n");
  // Pass filter list and filters to root node
  add_node(root,0,0,addr,temp_list,filters,0);
  delete(temp_list);
  return;
}

void sbintree::add_node(struct stnode *prnt, int lev, int dir, unsigned int addr, dlist* Flist, struct filter filters[], int CurrNest){
  int Flist_size = 0;
  /*
  printf("add_node:\n");
  printf("parent = 0x%08x\n",prnt);
  printf("level = %d\n",lev);
  printf("direction = %d\n",dir);
  printf("address = %u\n",addr);
  printf("Flist = "); (*Flist).print(stdout); printf("\n");
  */

  // Find the number of items in the list
  Flist_size = (*Flist).size();
  // printf("Flist_size = %d\n",Flist_size);

  // Increment total number of nodes
  num_stnodes++;

  // Allocate new stnode
  struct stnode *me;
  me = new struct stnode;
  me->lvl = lev;
  me->parent = prnt;
  me->child0 = NULL;
  me->child1 = NULL;
  me->wt_child0 = 0;
  me->wt_child1 = 0;

  // Set parent's child pointer
  if (lev != 0) {
    if (dir == 0) prnt->child0 = me;      
    else prnt->child1 = me;
  } else {
    root = me;
  }

  // Flag set if prefix added to this node
  int nest_flag = 0;
  int lev1_flag = 0;
  // Examine filters to "add" to this node
  struct dlist_item *filt;
  filt = (*Flist)(1);
  while (filt != NULL && Flist_size--) {
    int q = filt->key;
    filt = filt->next;
    // Remove q (first item) from list
    // printf("Removing %d from Flist\n",q);
    (*Flist)<<=1;
    // printf("filters[%d].sa_len = %d\n",q,filters[q].sa_len);
    if (filters[q].sa_len == lev) {
      // Assign filter to this node (level)
      filters[q].sa = addr;
      // Remove filter from Flist (do not append it)
      // printf("NOT appending %d back to Flist\n",q);
      // Set nest flag
      nest_flag = 1;
    } else {
      // Put q back on the list
      // printf("Appending %d back to Flist\n",q);
      (*Flist)&=q;
    }
    if (filters[q].sa_len == lev+1) lev1_flag = 1;
    // printf("filt =  %d\n",filt);
  }
  // Increment nesting if necessary
  int MyNest;
  if (nest_flag == 1) MyNest = CurrNest + 1;
  else MyNest = CurrNest;

  Flist_size = 0;
  // Find the number of items in the list
  // for (int i = 1; (*Flist)(i) != NULL; i++) Flist_size = i;
  Flist_size = (*Flist).size();
  // printf("Flist_size = %d\n\n",Flist_size);

  double temp;
  int path;
  unsigned int addr0, addr1;
  int lev1;

  // If list is empty, return
  if (Flist_size == 0) return;
  else {
    // Choose heavy path
    temp = drand48();
    if (temp < 0.5) path = 0;
    else path = 1;
    // Increment level
    lev1 = lev + 1;
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
    // If at the nesting threshold and list has more than one child,
    //   then split list (allocate all nodes with level == lev1 to one path)
    // printf("lev = %d, MyNest = %d, Nest = %d, lev1_flag = %d\n",lev,MyNest,Nest,lev1_flag);
    if ((Flist_size > 1) && (MyNest >= Nest - 1) && (lev1_flag == 1) && (lev < 31)){
      // Allocate nest_list
      dlist *nest_list = new dlist();
      dlist *other_list = new dlist();
      
      int fptr;
      //      for (dlist_item* index = (*Flist)(1); index != NULL; index = index->next){
      for (int i = Flist_size; i > 0; i--){
        fptr = (*Flist).frst();
	if (filters[fptr].sa_len == lev1) {
	  // printf("Adding %d to nest_list\n",fptr);
	  (*nest_list)&=fptr;
	} else {
	  // printf("Adding %d to other_list\n",fptr);
	  (*other_list)&=fptr;
	}
	(*Flist)<<=1;
      }
      // printf("nest_list->size() = %d, other_list->size() = %d, path = %d\n",nest_list->size(),other_list->size(),path == 0);
      // Pass lists onto children
      if (nest_list->size() > other_list->size()){
	if (path == 0){
	  me->wt_child0 = nest_list->size();
	  me->wt_child1 = other_list->size();
	  add_node(me, lev1, 0, addr0, nest_list, filters, MyNest);
	  add_node(me, lev1, 1, addr1, other_list, filters, MyNest);
	} else {
	  // path == 1
	  me->wt_child1 = nest_list->size();
	  me->wt_child0 = other_list->size();
	  add_node(me, lev1, 1, addr1, nest_list, filters, MyNest);
	  add_node(me, lev1, 0, addr0, other_list, filters, MyNest);
	}
      } else {
	// nest_list->size() <= other_list->size()
	if (path == 0){
	  me->wt_child1 = nest_list->size();
	  me->wt_child0 = other_list->size();
	  add_node(me, lev1, 1, addr1, nest_list, filters, MyNest);
	  add_node(me, lev1, 0, addr0, other_list, filters, MyNest);
	} else {
	  // path == 1
	  me->wt_child0 = nest_list->size();
	  me->wt_child1 = other_list->size();
	  add_node(me, lev1, 0, addr0, nest_list, filters, MyNest);
	  add_node(me, lev1, 1, addr1, other_list, filters, MyNest);
	}
      }	  
    }
    else {
      // Othewise, branch based on branching probability and skew  
      // Branch based on branching probability
      // (or if list size = 1, node can only have one child)
      temp = drand48();
      // printf("temp = %.4f, p1child[%d] = %.4f, Flist_size = %d\n",temp,lev,p1child[lev],Flist_size);
      if (temp < p1child[lev] || Flist_size == 1) {
	// If node has one child or list has one filter
	if (path == 0) {
	  // Adjust child weights
	  me->wt_child0 = 1;
	  me->wt_child1 = 0;
	  // Pass list to single child
	  add_node(me, lev1, path, addr0, Flist, filters, MyNest);
	} else {
	  // path == 1
	  // Adjust child weights
	  me->wt_child0 = 0;
	  me->wt_child1 = 1;
	  // Pass list to single child
	  add_node(me, lev1, path, addr1, Flist, filters, MyNest);
	}
      } else {
	// If node has two children and list has more than one filter...
	// Split list according skew
	float hvy, lite;
	hvy = Flist_size / ((float)2 - skew[lev]);
	hvy = floor(hvy);
	lite = Flist_size - hvy;
	// printf("hvy = %.1f, lite = %.1f\n",hvy,lite);
	
	// Allocate temp_list
	dlist *temp_list_lite = new dlist();
	// printf("temp_list_lite = "); (*temp_list_lite).print(stdout); printf("\n");
	
	int k;
	for (int i = (int)lite; i > 0; i--){
	  k = (*Flist).frst();
	  // printf("Adding %d to temp_list_lite\n",k);
	  (*temp_list_lite)&=k;
	  // printf("Removing %d from Flist\n",k);
	  (*Flist)<<=1;
	}
	// Pass lists onto children
	if (path == 0){
	  me->wt_child0 = (int)hvy;
	  me->wt_child1 = (int)lite;
	  add_node(me, lev1, 0, addr0, Flist, filters, MyNest);
	  add_node(me, lev1, 1, addr1, temp_list_lite, filters, MyNest);
	} else {
	  // path == 1
	  me->wt_child0 = (int)lite;
	  me->wt_child1 = (int)hvy;
	  add_node(me, lev1, 0, addr0, temp_list_lite, filters, MyNest);
	  add_node(me, lev1, 1, addr1, Flist, filters, MyNest);
	}
	delete(temp_list_lite);
      }
    }
  }
  return;
}

void sbintree::scale_skew(float scale_factor){
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
