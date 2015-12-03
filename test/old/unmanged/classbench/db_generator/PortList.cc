// File: PortList.cc
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for PortList

#include "stdinc.h"
#include "PortList.h"

PortList::PortList(int N1) {
  N = N1;
  ports = new port[N];
  first = -1;
  last = -1;
  for (int i = 0; i < N; i++) {
    ports[i].low = ports[i].high = 0;
    ports[i].prob = 0;
    ports[i].next = -1;
  }
}

PortList::~PortList() { delete ports; }

void PortList::read(int t, FILE *fp) {
  int done = 0;
  int matches = 0;
  char comm[5];
  char spar_comm[]="-spar";
  char spem_comm[]="-spem";
  char dpar_comm[]="-dpar";
  char dpem_comm[]="-dpem";

  // read in port width/range
  while (matches != EOF && done == 0) {
    matches = fscanf(fp,"%s",comm);
    // printf("comm = %s\n",comm);
    if (t == 0 && (strcmp(comm,spar_comm) == 0)) done = 1;
    else if (t == 1 && (strcmp(comm,spem_comm) == 0)) done = 1;
    else if (t == 2 && (strcmp(comm,dpar_comm) == 0)) done = 1;
    else if (t == 3 && (strcmp(comm,dpem_comm) == 0)) done = 1; 
  }
  if (matches == EOF) {
    fprintf(stderr,"Warning: Could not find proper identifier.\nNo port information taken from parameter file.\n");
    return;
  }
  done = 0;
  first = 0;
  for (int i=0; i<N && done == 0; i++){
    matches = fscanf(fp,"%f\t%d:%d",&ports[i].prob,&ports[i].low,&ports[i].high);
    // printf("ports[%d].prob = %.8f\tports[%d].low = %d\tports[%d].high = %d\n",i,ports[i].prob,i,ports[i].low,i,ports[i].high);
    // printf("matches = %d\n",matches);
    if (matches == 3) last = i;
    else done = 1;
  }
  // printf("first = %d, last = %d\n",first,last);

  float port_prob = 0;

  // Build cummulative distributions
  for (int i = 0; i <= last; i++) {
    // Cummulative portocol distribution
    ports[i].prob += port_prob;
    port_prob = ports[i].prob;
    if (i == last) ports[i].prob = 1;
  }
  return;
}

struct range PortList::choose_port(double r) {
  struct range prange;
  int port = -1;
  int done = 0;

  for (int i = 0; (i <= last && done == 0); i++) {
    if (r <= ports[i].prob) {
      prange.low = ports[i].low;
      prange.high = ports[i].high;
      done = 1;
    }
  }
  return prange;
}

void PortList::print(FILE *fp) {
  fprintf(fp,"Port Range Distribution\n");
  fprintf(fp,"Prob.\tLow:High\n");
  for (int i = 0; i <= last; i++) {
    fprintf(fp,"%.4f\t%d:%d\n",ports[i].prob,ports[i].low,ports[i].high);
  }
  return;
}
