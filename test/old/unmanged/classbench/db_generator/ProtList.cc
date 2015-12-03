// File: ProtList.cc
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for ProtList

#include "stdinc.h"
#include "ProtList.h"

ProtList::ProtList() {
  N = 25;
  protocols = new struct protocol[N];
  first = -1;
  last = -1;
  for (int i = 0; i < N; i++) {
    protocols[i].prot_num = 0;
    protocols[i].prob = 0;
    protocols[i].pt_prob = new float[25];
    for (int j = 0; j < 25; j++) protocols[i].pt_prob[j] = 0;
  }
}

ProtList::~ProtList() {
  for (int i = 0; i < 25; i++) delete protocols[i].pt_prob;
  delete protocols;
}

void ProtList::read(FILE *fp) {
  int done = 0;
  int matches = 0;
  char comm[6];
  char prots_comm[]="-prots";

  // read in port width/range
  while (matches != EOF && done == 0) {
    matches = fscanf(fp,"%s",comm);
    // printf("comm = %s\n",comm);
    if (strcmp(comm,prots_comm) == 0) done = 1;
  }
  if (matches == EOF) {
    fprintf(stderr,"Warning: Could not find -prots identifier.\nNo protocol information taken from parameter file.\n");
    return;
  }
  done = 0;
  first = 0;
  for (int i=0; i<N && done == 0; i++){
    matches = fscanf(fp,"%d\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f\t%f",
		     &protocols[i].prot_num,&protocols[i].prob,
		     &protocols[i].pt_prob[0],&protocols[i].pt_prob[1], &protocols[i].pt_prob[2], &protocols[i].pt_prob[3], &protocols[i].pt_prob[4],
		     &protocols[i].pt_prob[5], &protocols[i].pt_prob[6], &protocols[i].pt_prob[7], &protocols[i].pt_prob[8], &protocols[i].pt_prob[9],
		     &protocols[i].pt_prob[10],&protocols[i].pt_prob[11],&protocols[i].pt_prob[12],&protocols[i].pt_prob[13],&protocols[i].pt_prob[14],
		     &protocols[i].pt_prob[15],&protocols[i].pt_prob[16],&protocols[i].pt_prob[17],&protocols[i].pt_prob[18],&protocols[i].pt_prob[19],
		     &protocols[i].pt_prob[20],&protocols[i].pt_prob[21],&protocols[i].pt_prob[22],&protocols[i].pt_prob[23],&protocols[i].pt_prob[24]);
    // printf("protocols[%d].prot_num = %d\nprotocols[%d].prob = %.4f\n",i,protocols[i].prot_num,i,protocols[i].prob);
    // printf("matches = %d\n",matches);
    if (matches == 27) last = i;
    else done = 1;
  }
  // printf("first = %d, last = %d\n",first,last);
  // print(stdout);

  float prot_prob = 0;
  float port_prob = 0;

  // Build cummulative distributions
  for (int i = 0; i <= last; i++) {
    // Cummulative protocol distribution
    if (i == last) protocols[i].prob = 1;
    else protocols[i].prob += prot_prob;
    prot_prob = protocols[i].prob;
    // printf("prot_prob = %f\n",prot_prob);
    
    // Cummulative port type distribution
    port_prob = 0;
    for (int j = 0; j < 25; j++){
      if (j == 24) protocols[i].pt_prob[j] = 1;
      else protocols[i].pt_prob[j] += port_prob;
      port_prob = protocols[i].pt_prob[j];
      // printf("\tport_prob = %f, protocols[%d].pt_prob[%d] = %f\n",port_prob,i,j,protocols[i].pt_prob[j]);
    }
    // print(stdout);
  }
  // print(stdout);
  return;
}

int ProtList::choose_prot(float r) {
  int prot = -1;
  int done = 0;

  for (int i = 0; (i <= last && done == 0); i++) {
    if (r <= protocols[i].prob) {
      prot = protocols[i].prot_num;
      done = 1;
    }
  }
  return prot;
}

int ProtList::choose_ports(float r, int prot) {
  int ptype = -1;
  int done = 0;
  int pr = -1;

  // Find protocol in list
  for (int i = 0; (i <= last && done == 0); i++) {
    if (prot == protocols[i].prot_num) {
      pr = i;
      done = 1;
    }
  }
  if (done == 0) {
    fprintf(stderr,"ERROR (ProtList::choose_ports): protocol number not found in list");
    exit(1);
  }

  done = 0;
  for (int i = 0; (i < 25 && done == 0); i++) {
    if (r <= protocols[pr].pt_prob[i]) {
      ptype = i;
      done = 1;
    }
  }
  return ptype;
}

void ProtList::print(FILE *fp) {
  printf("last = %d\n",last);
  fprintf(fp,"Protocol Distribution with Port Types\n");
  fprintf(fp,"prot\tFreq.\t");
  fprintf(fp,"wc-wc\twc-hi\thi-wc\thi-hi\twc-lo\t");
  fprintf(fp,"lo-wc\thi-lo\tlo-hi\tlo-lo\twc-ar\t");
  fprintf(fp,"ar-wc\thi-ar\tar-hi\twc-em\tem-wc\t");
  fprintf(fp,"hi-em\tem-hi\tlo-ar\tar-lo\tlo-em\t");
  fprintf(fp,"em-lo\tar-ar\tar-em\tem-ar\tem-em\n");
  for (int i = 0; i <= last; i++) {
    fprintf(fp,"%d\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\n",
	    protocols[i].prot_num,protocols[i].prob,
	    protocols[i].pt_prob[0], protocols[i].pt_prob[1], protocols[i].pt_prob[2], protocols[i].pt_prob[3], protocols[i].pt_prob[4],
	    protocols[i].pt_prob[5], protocols[i].pt_prob[6], protocols[i].pt_prob[7], protocols[i].pt_prob[8], protocols[i].pt_prob[9],
	    protocols[i].pt_prob[10],protocols[i].pt_prob[11],protocols[i].pt_prob[12],protocols[i].pt_prob[13],protocols[i].pt_prob[14],
	    protocols[i].pt_prob[15],protocols[i].pt_prob[16],protocols[i].pt_prob[17],protocols[i].pt_prob[18],protocols[i].pt_prob[19],
	    protocols[i].pt_prob[20],protocols[i].pt_prob[21],protocols[i].pt_prob[22],protocols[i].pt_prob[23],protocols[i].pt_prob[24]);
  }
  return;
}

int ProtList::size(){
  return last+1;
}

int ProtList::operator()(int i){
  return protocols[i].prot_num;
}
