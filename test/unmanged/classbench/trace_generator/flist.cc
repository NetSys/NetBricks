// File: flist.cc
// David E. Taylor 
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for data structure representing a list of filters
// containing d fields
// Each field, d, specifies a range, low[d] to high[d]
//
#include "stdinc.h"
#include "flist.h"

flist::flist(int d1, int N1) {
  df = d1; Nf = N1;
  lowf = new unsigned*[Nf];
  highf = new unsigned*[Nf];
  for (int i = 0; i < Nf; i++){
    lowf[i] = new unsigned[df];
    highf[i] = new unsigned[df];
    for (int j = 0; j < df; j++) {
      lowf[i][j] = 0; highf[i][j] = 0;
    }
  }
}

flist::~flist() {
  for (int i = 0; i < Nf; i++){
    delete(lowf[i]);
    delete(highf[i]);
  }
  delete(lowf);
  delete(highf);
}

int flist::d() { return df; }

int flist::N() { return Nf; }

// int flist::size() { return num; }

unsigned flist::low(int filt, int field) {
  if ((filt < Nf) && (field < df)) return lowf[filt][field];
  else {
    fprintf(stderr,"ERROR flist::low(%d, %d): array index out of range\n",filt,field);
    exit(1);
  }
}

unsigned flist::high(int filt, int field) {
  if ((filt < Nf) && (field < df)) return highf[filt][field];
  else {
    fprintf(stderr,"ERROR flist::high(%d, %d): array index out of range\n",filt,field);
    exit(1);
  }
}

void flist::add(int filt, int field, unsigned low, unsigned high) {
  if ((filt < Nf) && (field < df)) {
    lowf[filt][field] = low;
    highf[filt][field] = high;
  }
  else {
    fprintf(stderr,"ERROR flist::add(%d, %d, %u, %u): array index out of range\n",filt,field,low,high);
    exit(1);
  }
}

void flist::print(FILE *fp){
  for (int i = 0; i < Nf; i++) {
    for (int j = 0; j < df; j++){
      fprintf(fp,"[%u,%u]\t",lowf[i][j],highf[i][j]);
    }
    fprintf(fp,"\n");
  }
  return;
}
