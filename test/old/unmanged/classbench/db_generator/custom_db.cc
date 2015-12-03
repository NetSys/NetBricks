// File: custom_db.cc
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Generates a synthetic database from seed file and input parameters

#include "stdinc.h"
#include "FilterList.h"
#include "ProtList.h"
#include "FlagList.h"
#include "ExtraList.h"
#include "PortList.h"
#include "PrefixList.h"
#include "dlist.h"
#include "sbintree.h"
#include "dbintree.h"
#include "redundant_filter_check.h"
#include "TupleBST.h"
#include "custom_db.h"


int custom_db_gen(int num_filters, FilterList* filters, FILE* fp_in, int smoothness, float addr_scope, float port_scope, int branch){

  printf("Initializing data structures...\n");
  // Read in scale
  int scale = read_scale(fp_in);
  // printf("scale = %d\n",scale);
  float scale_factor = (float)num_filters/(float)scale;
  // printf("scale_factor = %.4f\n",scale);

  // Read protocol parameters, initialize data structure
  ProtList *protL = new ProtList();
  protL->read(fp_in);
  // protL->print(stdout);
  
  // Read flags distribution
  FlagList *flagL = new FlagList();
  flagL->read(fp_in);
  // flagL->print(stdout);

  // Read extra field distribution
  ExtraList *extraL = new ExtraList(protL->size());
  extraL->read(fp_in,scale_factor);
  // extraL->print(stdout);

  // Read port distributions, initialize four lists
  // Source ports, Arbitrary Ranges
  PortList *sparL = new PortList();
  (*sparL).read(0,fp_in);
  //(*sparL).print(stdout);

  // Source ports, Exact Ports
  PortList *spemL = new PortList();
  (*spemL).read(1,fp_in);
  //(*spemL).print(stdout);

  // Destination ports, Arbitrary Ranges
  PortList *dparL = new PortList();
  (*dparL).read(2,fp_in);
  //(*dparL).print(stdout);

  // Destination ports, Exact Ports
  PortList *dpemL = new PortList();
  (*dpemL).read(3,fp_in);
  //(*dpemL).print(stdout);

  // Read prefix length distributions, initialize
  PrefixList *prefixL = new PrefixList();
  prefixL->read(fp_in);
  prefixL->smooth(smoothness);
  // for(int i = 0; i < 25; i++) prefixL->print(i,stdout);

  printf(" \tdone\n");

  // Random number
  double p,pt,ps;

  // Temporary filter
  // struct filter temp_filter;
  struct filter *temp_filters = new struct filter[num_filters+1];
  dlist *Flist = new dlist;
  struct range temp_range;
  struct ppair temp_ppair;
  int port_type;

  printf("Creating application specifications...\n");
  
  // For all filters:
  for (int i = 1; i <= num_filters; i++){
    // Select a protocol via random number
    p = drand48();
    temp_filters[i].prot_num = protL->choose_prot((float)p);
    // printf("prot_num = %d\n",temp_filters[i].prot_num);

    // Select flag specification
    p = drand48();
    flagL->choose((float)p,temp_filters[i].prot_num,&(temp_filters[i].flags),&(temp_filters[i].flags_mask));

    // Select extra fields
    temp_filters[i].num_ext_field = extraL->size();
    if (temp_filters[i].num_ext_field > 0) {
      temp_filters[i].ext_field = new int[temp_filters[i].num_ext_field];
      for (int j = 0; j < temp_filters[i].num_ext_field; j++) temp_filters[i].ext_field[j] = 0;
      extraL->choose(temp_filters[i].prot_num,temp_filters[i].ext_field);
      // printf("temp_filters[i].ext_field[0] = %d\n",temp_filters[i].ext_field[0]);
    }

    // Select a port range pair type from protocol distribution via random number
    // p = drand48();
    p = random_scope(port_scope);
    port_type = protL->choose_ports((float)p,temp_filters[i].prot_num);
    // printf("p = %f, temp_filters[%d].prot_num = %d, port_type = %d\n",(float)p,i,temp_filters[i].prot_num,port_type);

    // Select port range values based on type and distributions
    select_ports(port_type,&temp_filters[i],sparL,spemL,dparL,dpemL);
    // printf("sp [%d:%d]\tdp [%d:%d]\n",temp_filters[i].sp[0],temp_filters[i].sp[1],temp_filters[i].dp[0],temp_filters[i].dp[1]);

    // Select total prefix length from distribution associated with port range type
    // Use pseudo-random number generator
    pt = random_scope(addr_scope);
    // printf("random_scope done\n");

    // Select source/destination prefix length from source length distribution
    // Use random number
    ps = drand48();
    // printf("ps = %.4f, pt = %.4f, prot_type = %d\n",ps,pt,port_type);
    temp_ppair = prefixL->choose_prefix(port_type,(float)ps,(float)pt);
 
    // printf("temp_ppair.slen = %d, temp_ppair.dlen = %d\n",temp_ppair.slen,temp_ppair.dlen);
    // Assign prefix lengths to filter
    temp_filters[i].sa_len = temp_ppair.slen;
    temp_filters[i].da_len = temp_ppair.dlen;

    // Add temp_filter to temp_filters
    // temp_filters[i] = temp_filter;

    // Add i to list of filters
    *Flist&=i;
  }
  printf(" \tdone\n");
  // Free up memory
  delete(protL);
  delete(sparL);
  delete(spemL);
  delete(dparL);
  delete(dpemL);
  delete(prefixL);

  /*
  printf("Creating Stree1\n");
  sbintree *Stree1 = new sbintree();
  printf("Creating Stree2\n");
  sbintree *Stree2 = new sbintree();
  printf("Creating dlist\n");
  dlist* foo = new dlist;
  printf("Deleting Stree1\n");
  delete(Stree1);
  printf("Deleting Stree2\n");
  delete(Stree2);
  printf("Deleting dlist\n");
  delete(foo);
  */
  
  // Construct addresses
  // Read in skew distributions from parameter file 
  // or generate skew distributions according to input time constant
  printf("Creating source addresses...\n");
  sbintree *Stree = new sbintree();
  // Read source address nesting
  (*Stree).read_nest(fp_in);
  // Read source address skew
  (*Stree).read_skew(fp_in);
  // (*Stree).print_skew(stdout);
  if (branch == 1 && scale_factor > 1){
    (*Stree).scale_skew(scale_factor);
    // (*Stree).print_skew(stdout);
  }
  // printf("Flist = "); (*Flist).print(stdout); printf("\n");  
  (*Stree).build_tree(Flist,temp_filters);
  delete(Stree);
  /*
  printf("Flist = "); (*Flist).print(stdout); printf("\n");
  for (int i = 1; i <= num_filters; i++)
    printf("filter[%d].sa = %u/%d\n",i,temp_filters[i].sa,temp_filters[i].sa_len);
  */
  printf(" \tdone\n");

  printf("Creating destination addresses...\n");
  dbintree *Dtree = new dbintree();
  // Read destination address nesting
  (*Dtree).read_nest(fp_in);
  (*Dtree).read_skew(fp_in);
  // (*Dtree).print_skew(stdout);
  if (branch == 1 && scale_factor > 1){
    (*Dtree).scale_skew(scale_factor);
    // (*Dtree).print_skew(stdout);
  }
  (*Dtree).read_corr(fp_in);
  // (*Dtree).print_corr(stdout);

  (*Dtree).build_tree(Flist,temp_filters);
  // printf("Flist = "); (*Flist).print(stdout); printf("\n");
  delete(Dtree);
  printf(" \tdone\n");
  
  delete(Flist);

  printf("Removing redundant filters and ordering nested filters...\n");
  int filter_cnt = remove_redundant_filters(num_filters,filters,temp_filters);
  printf(" \tdone\n");

  // Resolve conflicts, throw away filters if necessary
  // Final filter set may be smaller than target
  // printf("Resolving conflicts...\n");
  // resolve_conflicts(temp_filter);
  // printf(" \tdone\n");

  // Delete data structures
  delete(temp_filters);
  // printf("Done with custom_db\n");

  return filter_cnt;
}

// Biased random number generator
double random_scope(float scope_x){

  // Seed random number generator with long int
  double p;
  // Get random number [0,1]
  p = drand48();

  // Generate biased random number
  double pb;
  pb = p*((scope_x*p) - scope_x + 1);

  return pb;
}

// Port selection
void select_ports(int port_type, struct filter *temp_filter, PortList *sparL, PortList *spemL, PortList *dparL, PortList *dpemL){
  double p;

  struct range temp_range;

  if (port_type == 0){
    // wc_wc
    temp_filter->sp[0] = 0;
    temp_filter->sp[1] = 65535;
    temp_filter->dp[0] = 0;
    temp_filter->dp[1] = 65535;
  } else if (port_type == 1){
    // wc_hi
    temp_filter->sp[0] = 0;
    temp_filter->sp[1] = 65535;
    temp_filter->dp[0] = 1024;
    temp_filter->dp[1] = 65535;
  } else if (port_type == 2){
    // hi_wc
    temp_filter->sp[0] = 1024;
    temp_filter->sp[1] = 65535;
    temp_filter->dp[0] = 0;
    temp_filter->dp[1] = 65535;
  } else if (port_type == 3){
    // hi_hi
    temp_filter->sp[0] = 1024;
    temp_filter->sp[1] = 65535;
    temp_filter->dp[0] = 1024;
    temp_filter->dp[1] = 65535;
  } else if (port_type == 4){
    // wc_lo
    temp_filter->sp[0] = 0;
    temp_filter->sp[1] = 65535;
    temp_filter->dp[0] = 0;
    temp_filter->dp[1] = 1023;
  } else if (port_type == 5){
    // lo_wc
    temp_filter->sp[0] = 0;
    temp_filter->sp[1] = 1023;
    temp_filter->dp[0] = 0;
    temp_filter->dp[1] = 65535;
  } else if (port_type == 6){
    // hi_lo
    temp_filter->sp[0] = 1024;
    temp_filter->sp[1] = 65535;
    temp_filter->dp[0] = 0;
    temp_filter->dp[1] = 1023;
  } else if (port_type == 7){
    // lo_hi
    temp_filter->sp[0] = 0;
    temp_filter->sp[1] = 1023;
    temp_filter->dp[0] = 1024;
    temp_filter->dp[1] = 65535;
  } else if (port_type == 8){
    // lo_lo
    temp_filter->sp[0] = 0;
    temp_filter->sp[1] = 1023;
    temp_filter->dp[0] = 0;
    temp_filter->dp[1] = 1023;
  } else if (port_type == 9){
    // wc_ar
    temp_filter->sp[0] = 0;
    temp_filter->sp[1] = 65535;
    // Choose arbitrary destination port range
    p = drand48();
    temp_range = dparL->choose_port(p);
    temp_filter->dp[0] = temp_range.low;
    temp_filter->dp[1] = temp_range.high;
  } else if (port_type == 10){
    // ar_wc
    // Choose arbitrary source port range
    p = drand48();
    temp_range = sparL->choose_port(p);
    temp_filter->sp[0] = temp_range.low;
    temp_filter->sp[1] = temp_range.high;
    temp_filter->dp[0] = 0;
    temp_filter->dp[1] = 65535;
  } else if (port_type == 11){
    // hi_ar
    temp_filter->sp[0] = 1024;
    temp_filter->sp[1] = 65535;
    // Choose arbitrary destination port range
    p = drand48();
    temp_range = dparL->choose_port(p);
    temp_filter->dp[0] = temp_range.low;
    temp_filter->dp[1] = temp_range.high;
  } else if (port_type == 12){
    // ar_hi
    // Choose arbitrary source port range
    p = drand48();
    temp_range = sparL->choose_port(p);
    temp_filter->sp[0] = temp_range.low;
    temp_filter->sp[1] = temp_range.high;
    temp_filter->dp[0] = 1024;
    temp_filter->dp[1] = 65535;
  } else if (port_type == 13){
    // wc_em
    temp_filter->sp[0] = 0;
    temp_filter->sp[1] = 65535;
    // Choose exact destination port range
    p = drand48();
    temp_range = dpemL->choose_port(p);
    temp_filter->dp[0] = temp_range.low;
    temp_filter->dp[1] = temp_range.high;
  } else if (port_type == 14){
    // em_wc
    // Choose exact source port range
    p = drand48();
    temp_range = spemL->choose_port(p);
    temp_filter->sp[0] = temp_range.low;
    temp_filter->sp[1] = temp_range.high;
    temp_filter->dp[0] = 0;
    temp_filter->dp[1] = 65535;
  } else if (port_type == 15){
    // hi_em
    temp_filter->sp[0] = 1024;
    temp_filter->sp[1] = 65535;
    // Choose exact destination port range
    p = drand48();
    temp_range = dpemL->choose_port(p);
    temp_filter->dp[0] = temp_range.low;
    temp_filter->dp[1] = temp_range.high;
  } else if (port_type == 16){
    // em_hi
    // Choose exact source port range
    p = drand48();
    temp_range = spemL->choose_port(p);
    temp_filter->sp[0] = temp_range.low;
    temp_filter->sp[1] = temp_range.high;
    temp_filter->dp[0] = 1024;
    temp_filter->dp[1] = 65535;
  } else if (port_type == 17){
    // lo_ar
    temp_filter->sp[0] = 0;
    temp_filter->sp[1] = 1023;
    // Choose arbitrary destination port range
    p = drand48();
    temp_range = dparL->choose_port(p);
    temp_filter->dp[0] = temp_range.low;
    temp_filter->dp[1] = temp_range.high;
  } else if (port_type == 18){
    // ar_lo
    // Choose arbitrary source port range
    p = drand48();
    temp_range = sparL->choose_port(p);
    temp_filter->sp[0] = temp_range.low;
    temp_filter->sp[1] = temp_range.high;
    temp_filter->dp[0] = 0;
    temp_filter->dp[1] = 1023;
  } else if (port_type == 19){
    // lo_em
    temp_filter->sp[0] = 0;
    temp_filter->sp[1] = 1023;
    // Choose exact destination port range
    p = drand48();
    temp_range = dpemL->choose_port(p);
    temp_filter->dp[0] = temp_range.low;
    temp_filter->dp[1] = temp_range.high;
  } else if (port_type == 20){
    // em_lo
    // Choose exact source port range
    p = drand48();
    temp_range = spemL->choose_port(p);
    temp_filter->sp[0] = temp_range.low;
    temp_filter->sp[1] = temp_range.high;
    temp_filter->dp[0] = 0;
    temp_filter->dp[1] = 1023;
  } else if (port_type == 21){
    // ar_ar
    // Choose arbitrary source port range
    p = drand48();
    temp_range = sparL->choose_port(p);
    temp_filter->sp[0] = temp_range.low;
    temp_filter->sp[1] = temp_range.high;
    // Choose arbitrary destination port range
    p = drand48();
    temp_range = dparL->choose_port(p);
    temp_filter->dp[0] = temp_range.low;
    temp_filter->dp[1] = temp_range.high;
  } else if (port_type == 22){
    // ar_em
    // Choose arbitrary source port range
    p = drand48();
    temp_range = sparL->choose_port(p);
    temp_filter->sp[0] = temp_range.low;
    temp_filter->sp[1] = temp_range.high;
    // Choose exact destination port range
    p = drand48();
    temp_range = dpemL->choose_port(p);
    temp_filter->dp[0] = temp_range.low;
    temp_filter->dp[1] = temp_range.high;
  } else if (port_type == 23){
    // em_ar
    // Choose exact source port range
    p = drand48();
    temp_range = spemL->choose_port(p);
    temp_filter->sp[0] = temp_range.low;
    temp_filter->sp[1] = temp_range.high;
    // Choose arbitrary destination port range
    p = drand48();
    temp_range = dparL->choose_port(p);
    temp_filter->dp[0] = temp_range.low;
    temp_filter->dp[1] = temp_range.high;
  } else if (port_type == 24){
    // em_em
    // Choose exact source port range
    p = drand48();
    temp_range = spemL->choose_port(p);
    temp_filter->sp[0] = temp_range.low;
    temp_filter->sp[1] = temp_range.high;
    // Choose exact destination port range
    p = drand48();
    temp_range = dpemL->choose_port(p);
    temp_filter->dp[0] = temp_range.low;
    temp_filter->dp[1] = temp_range.high;
  } else {
    fprintf(stderr,"ERROR (select_ports): port_type %d out of range\n",port_type);
    exit(1);
  }
  return;
}

void fprint_filter(FILE *fp, struct filter *filt){
  int addr[4];
  unsigned temp;

  // Print new filter character
  fprintf(fp,"@");
  // Print source address
  addr[0] = addr[1] = addr[2] = addr[3] = 0;
  temp = 0;
  temp = filt->sa;
  addr[0] = (temp >> 24);
  addr[1] = ((temp << 8) >> 24);
  addr[2] = ((temp << 16) >> 24);
  addr[3] = ((temp << 24) >> 24);
  fprintf(fp, "%d.%d.%d.%d/%d\t",
	  addr[0], addr[1], addr[2], addr[3],
	  filt->sa_len);
  // Print destination address 
  addr[0] = addr[1] = addr[2] = addr[3] = 0;
  temp = 0;
  temp = filt->da;
  addr[0] = (temp >> 24);
  addr[1] = ((temp << 8) >> 24);
  addr[2] = ((temp << 16) >> 24);
  addr[3] = ((temp << 24) >> 24);
  fprintf(fp, "%d.%d.%d.%d/%d\t",
	  addr[0], addr[1], addr[2], addr[3],
	  filt->da_len);
  // Print source port 
  fprintf(fp, "%d : %d\t",
	  filt->sp[0], filt->sp[1]);
  // Print destination port 
  fprintf(fp, "%d : %d\t",
	  filt->dp[0], filt->dp[1]);
  // Print protocol 
  fprintf(fp, "%d",
	  filt->prot_num);    
  // Print newline 
  fprintf(fp,"\n");

  return;
}

int read_scale(FILE *fp){
  int done = 0;
  int matches = 0;
  char comm[6];
  char scale_comm[]="-scale";
  int scale = 0;

  // read in scale
  while (matches != EOF && done == 0) {
    matches = fscanf(fp,"%s",comm);
    if (strcmp(comm,scale_comm) == 0) done = 1;
  }
  if (matches == EOF) {
    fprintf(stderr,"Warning: Could not find -scale identifier.\n");
    return scale;
  }
  done = 0;
  // char scomm[500];
  // int scomm_len = 500;
  while (done == 0) {
    // Read a line of the input
    printf("Reading a line from the input file...\n");
    // fgets(scomm,scomm_len,fp);
    // Read a line of the input
    // matches = sscanf(scomm,"%d",&scale);
    matches = fscanf(fp,"%d",&scale);
    printf("Read %d\n", scale);
    if (matches == 1) done = 1;
  }
  return scale;
}

int remove_redundant_filters(int num_filters, FilterList* filters, filter* temp_filters){
  int filter_cnt = 0;
  int redundant, nest, flag;
  FiveTuple* ftuple = new FiveTuple;
  dlist* TupleListPtr;
  dlist** TupleListPtrArray;
  TupleBST* TupleTree = new TupleBST;
  // For all filters in temp_filters
  for (int i = 1; i <= num_filters; i++){
    // Determine filter tuple
    ftuple->sa_len = temp_filters[i].sa_len;
    ftuple->da_len = temp_filters[i].da_len;
    ftuple->sp_wid = temp_filters[i].sp[1] - temp_filters[i].sp[0] + 1;
    ftuple->dp_wid = temp_filters[i].dp[1] - temp_filters[i].dp[0] + 1;
    if (temp_filters[i].prot_num > 0) ftuple->prot = 1;
    else ftuple->prot = 0;
    if (temp_filters[i].flags_mask > 0) ftuple->flag = 1;
    else ftuple->flag = 0;
    // Get pointer for tuple list
    TupleListPtr = TupleTree->Insert(ftuple);
    if (TupleListPtr == Null) {fprintf(stderr,"ERROR: TupleBST::Insert returned a null pointer."); exit(1);}

    // Check for redundant filters in tuple list
    dlist_item* findex;
    int rflag = 0;
    for (findex = (*TupleListPtr)(1); findex != Null && rflag == 0; findex = findex->next){
      redundant = nest = 0;
      // Check for redundancy
      rflag = redundant_check(temp_filters[i],temp_filters[findex->key]);
    }
    // If not redundant add to tuple list
    if (rflag == 0) {(*TupleListPtr)&=i; filter_cnt++;}
  }
  // Sort tuple set pointers by specificity (most to least)
  TupleListPtrArray = TupleTree->GetTupleLists();

  // Append filters to FilterList in order of most-specific tuple to least specific tuple
  for (int i = 0; i < TupleTree->size(); i++){
    TupleListPtr = TupleListPtrArray[i];
    if (TupleListPtr == Null) {fprintf(stderr,"ERROR: TupleListPtrArray contains a null pointer."); exit(1);}
    dlist_item* findex;
    for (findex = (*TupleListPtr)(1); findex != Null; findex = findex->next){
      (*filters)&=temp_filters[findex->key];
    }
  }
  delete(ftuple);
  delete(TupleTree);
  return filter_cnt;
}
