// File: trace_tools.cc
// David E. Taylor 
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Functions for generating synthetic trace of headers
//
#include "stdinc.h"
#include "dlist.h"
#include "hlist.h"
#include "trace_tools.h"

// Generate headers
int header_gen(int d, FilterList* filters, FILE *fp, float a, float b, int scale){
  int num_headers = 0;
  int fsize = filters->size();
  int threshold = scale * fsize;
  // Allocate flist
  flist *filts = new flist(d, filters->size());

  // Convert filters to set of [low,high] bounds
  convert_filters(d, filters, filts);

  // Create header list
  hlist *headers = new hlist(d);

  // Allocate temporary header
  unsigned *new_hdr = new unsigned[d];

  // Generate headers
  while(num_headers < threshold){
    // Pick a random filter
    int RandFilt = randint(0,fsize-1);

    // Pick a random corner of the filter for a header
    RandomCorner(RandFilt,filts,new_hdr);

    // Select number of copies to add to header list
    // from Pareto distribution
    int Copies = MyPareto(a,b);
    // printf("RandFilt = %d, a = %.4f, b = %.4f, Copies = %d\n",RandFilt,a,b,Copies);

    // Add to header list
    for (int i = 0; i < Copies; i++) headers->add(new_hdr,RandFilt);

    // Increment number of headers
    num_headers += Copies;
  }

  // Print headers
  headers->print(fp);

  delete(new_hdr);
  delete(headers);

  return num_headers;
}

void RandomCorner(int RandFilt, flist* filts, unsigned* new_hdr){
  int d = filts->d();
  // Random number
  double p;
  for (int i = 0; i < d; i++){
    // Select random number
    p = drand48();
    if (p <0.5){
      // Choose low extreme of field
      new_hdr[i] = filts->low(RandFilt,i);
    } else {
      // Choose high extreme of field
      new_hdr[i] = filts->high(RandFilt,i);
    }
  }
  return;
}

int MyPareto(float a, float b){
  if (b == 0) return 1;

  // Random number
  double p;
  // Select random number
  p = drand48();
 
  double x = (double)b / pow((double)(1 - p),(double)(1/(double)a));
  int Num = (int)ceil(x);

  return Num;
}

void convert_filters(int d, FilterList* filters, flist *filts){
  struct FilterList_item* fptr;
  int i = 0;
  // Convert standard 5-tuple
  for (fptr = (*filters)(1); fptr != Null; fptr = fptr->next){
    int len; unsigned low, high, tempaddr; 
    if (d >= 1) {
      // Convert SA
      len = fptr->filt.sa_len;
      if (len == 0) {
	// wildcard
	low = 0;
	high = ~0;
      } else {
	tempaddr = fptr->filt.sa;      
	// Shift out masked bits
	tempaddr = (tempaddr >> (32 - len));
	// Shift back in with appended zeros
	low = (tempaddr << (32 - len));
	// Shift back in with appended ones
	high = ~((~tempaddr) << (32 - len));
      }
      // Add to list
      filts->add(i,d-1,low,high);
    }
    if (d >= 2) {
      // Convert DA
      len = fptr->filt.da_len;
      if (len == 0) {
	// wildcard
	low = 0;
	high = ~0;
      } else {
	tempaddr = fptr->filt.da;      
	// Shift out masked bits
	tempaddr = (tempaddr >> (32 - len));
	// Shift back in with appended zeros
	low = (tempaddr << (32 - len));
	// Shift back in with appended ones
	high = ~((~tempaddr) << (32 - len));
      }
      // Add to list
      filts->add(i,d-2,low,high);
    }
    if (d >= 3){
      // Convert SP
      filts->add(i,d-3,(unsigned)fptr->filt.sp[0],(unsigned)fptr->filt.sp[1]);
    }
    if (d >= 4){ 
      // Convert DP
      filts->add(i,d-4,(unsigned)fptr->filt.dp[0],(unsigned)fptr->filt.dp[1]);
    }
    if (d >= 5){
      // Convert PR
      if (fptr->filt.prot_num == 0) 
	filts->add(i,d-5,0,255);
      else
	filts->add(i,d-5,(unsigned)fptr->filt.prot_num,(unsigned)fptr->filt.prot_num);
    }
    if (d >= 6){
      // Convert Flags
      // concatenate 16-bit flags and 16-bit mask
      if (fptr->filt.flags_mask == 0){
	low = 0; high = ~0;
      } else {
	low = fptr->filt.flags;
	low = low << 16;
	low = low | fptr->filt.flags_mask;
	high = low;
      }
      filts->add(i,d-6,low,high);
    }
    for (int j = d-7; j >= 0; j--){
      // Add extra fields
      if (fptr->filt.ext_field[j] == 0){
	// Wildcard
	low = 0; high = ~0;
      } else {
	low = high = (unsigned)fptr->filt.ext_field[j];
      }
      filts->add(i,j,low,high);
    }
    i++;
  }
  return;
};


