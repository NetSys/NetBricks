// File: FlagList.cc
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for FlagList

#include "stdinc.h"
#include "FlagList.h"

FlagList::FlagList() {
  first = new struct FlagListItem*[256];
  last = new struct FlagListItem*[256];
  for (int i = 0; i < 256; i++){
    first[i] = last[i] = NULL;
  }
}

FlagList::~FlagList() {
  struct FlagListItem *temp;
  for (int i = 0; i < 256; i++){
    while (first[i] != Null) {
      temp = (*first[i]).next;
      delete(first[i]);
      first[i] = temp;
    }
  }
  delete(first);
  delete(last);
}

void FlagList::choose(float p, int prot, unsigned *flags, unsigned *flags_mask){
  // Check for flag spec in list
  struct FlagListItem *j;
  j = first[prot];
  while (j != Null){
    if (p < j->prob) {
      (*flags) = j->flags;
      (*flags_mask) = j->flags_mask;
      return;
    }
    j = j->next;
  }
  return;
};

void FlagList::read(FILE *fp){
  int done = 0;
  int matches = 0;
  char comm[6];
  char flag_comm[]="-flags";
  // read in port width/range
  while (matches != EOF && done == 0) {
    matches = fscanf(fp,"%s\n",comm);
    // printf("comm = %s\n",comm);
    if (strcmp(comm,flag_comm) == 0) done = 1;
  }
  if (matches == EOF) {
    fprintf(stderr,"Warning: Could not find proper identifier.\nNo prefix information taken from parameter file.\n");
    return;
  }
  done = 0;
  int prot_num;
  unsigned flg[10];
  unsigned flg_msk[10];
  float probs[10];
  char scomm[500];
  int scomm_len = 500;
  while (done == 0) {
    fgets(scomm,scomm_len,fp);
    // Read a line of the input
    matches = sscanf(scomm,"%d\t0x%04x/0x%04x,%f\t0x%04x/0x%04x,%f\t0x%04x/0x%04x,%f\t0x%04x/0x%04x,%f\t0x%04x/0x%04x,%f\t0x%04x/0x%04x,%f\t0x%04x/0x%04x,%f\t0x%04x/0x%04x,%f\t0x%04x/0x%04x,%f\t0x%04x/0x%04x,%f",&prot_num,&flg[0],&flg_msk[0],&probs[0],&flg[1],&flg_msk[1],&probs[1],&flg[2],&flg_msk[2],&probs[2],&flg[3],&flg_msk[3],&probs[3],&flg[4],&flg_msk[4],&probs[4],&flg[5],&flg_msk[5],&probs[5],&flg[6],&flg_msk[6],&probs[6],&flg[7],&flg_msk[7],&probs[7],&flg[8],&flg_msk[8],&probs[8],&flg[9],&flg_msk[9],&probs[9]);
    if (matches >= 4) {
      matches = (matches-1)/3;
      for (int j = 0; j < matches; j++) {
	// Append to end of list
	struct FlagListItem *temp;
	temp = new struct FlagListItem;
	temp->flags = flg[j];
	temp->flags_mask = flg_msk[j];
	temp->prob = probs[j];
	temp->prev = last[prot_num];
	temp->next = NULL;
	if (first[prot_num] == Null){
	  first[prot_num] = temp;
	} else {
	  (*last[prot_num]).next = temp;
	}
	last[prot_num] = temp;
      }
      // Create cummulative distribution for this protocol number
      struct FlagListItem *i;
      float total = 0;
      for (i = first[prot_num]; i != Null; i = i->next){
	total += i->prob;
	i->prob = total;
	if (i->next == Null) i->prob = 1;
      }
    }
    else done = 1;
  }
  return;
}

// Print the contents of the list.
void FlagList::print(FILE* fp) {
  struct FlagListItem *i;
  for (int j = 0; j < 256; j++){
    if (first[j] != Null) fprintf(fp,"%d\t",j);
    for (i = first[j]; i != Null; i = i->next){
      fprintf(fp,"%.2x/%.2x,%.8f\t",i->flags,i->flags_mask,i->prob);
    }
  }
  return;
}

