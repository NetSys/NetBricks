// File: ExtraList.cc
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Function definitions for ExtraList class
// See ExtraList.h for class definition

#include "stdinc.h"
#include "ExtraList.h"

ExtraList::ExtraList(int P1) {
  P = P1;
  first = last = NULL;
  for (int i = 1; i <= P; i++){
    // Create header list
    struct ExtraListHeader *temp = new struct ExtraListHeader;
    temp->next = NULL;
    temp->prev = last;
    if (i == 1) {
      first = temp;
    } else {
      last->next = temp;
    }
    last = temp;
  }
}

ExtraList::~ExtraList() {
  struct ExtraListHeader *temp;
  struct ExtraListItem *tempI;
  // Delete header item list
  while (first != NULL){
    // Get first header item
    temp = first;
    // For each field
    for (int j = 0; j < N; j++){
      tempI = temp->field[j];
      // Delete list of values
      delete(tempI->value);
      delete(tempI->prob);
    }
    first = first->next;
    delete(temp->field);
    delete(temp);
  }
}

struct ExtraListHeader* ExtraList::operator()(int prot){
  struct ExtraListHeader *head;
  head = first;
  while (head != NULL){
    // Find matching protocol number
    if (head->prot_num == prot) return head;
    head = head->next;
  }
  return NULL;
}

void ExtraList::choose(int prot, int *extra){
  // Check for extra spec in list
  struct ExtraListHeader *head;
  struct ExtraListItem *item;
  head = first;
  int done;
  while (head != NULL){
    // Find matching protocol number
    if (head->prot_num == prot){
      // Choose a value for each of N fields
      for (int i = 0; i < N; i++){
	item = head->field[i];
	// Choose a random number
	double x = drand48();
	done = 0;
	// Choose value from array based on probability
	for (int j = 0; ((j < item->num)&&(done == 0)); j++){
	  if ((float)x <= item->prob[j]){
	    // printf("Choosing %d for field %d\n",item->value[j],i);
	    extra[i] = item->value[j];
	    done = 1;
	  }
	}
      }
      // All fields chosen, return.
      return;
    }
    head = head->next;
  }
  return;
};

void ExtraList::read(FILE *fp, float scale_factor){
  int done = 0;
  int matches = 0;
  char comm[6];
  char extra_comm[]="-extra";
  // read in port width/range
  while (matches != EOF && done == 0) {
    matches = fscanf(fp,"%s\n",comm);
    // printf("comm = %s\n",comm);
    if (strcmp(comm,extra_comm) == 0) done = 1;
  }
  if (matches == EOF) {
    fprintf(stderr,"Warning: Could not find proper identifier.\nNo -extra information taken from parameter file.\n");
    return;
  }
  // printf("Found identifier; done = %d\n",done);
  int prot_num;
  unsigned val[10];
  float probs[10];
  char scomm[500];
  int scomm_len = 500;

  // Read number of extra fields
  fgets(scomm,scomm_len,fp);
  // Read a line of the input
  matches = sscanf(scomm,"%d",&N);
  // printf("matches = %d, N = %d\n",matches,N);

  if (matches != 1) {
    fprintf(stderr,"Warning: Could not find number of fields in -extra information.\n");
    return;
  }
  done = 0;
  struct ExtraListHeader *head = first;
  while (done == 0 && head != NULL) {
    fgets(scomm,scomm_len,fp);
    // Read a line of the input
    matches = sscanf(scomm,"%d\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f",&prot_num,&val[0],&probs[0],&val[1],&probs[1],&val[2],&probs[2],&val[3],&probs[3],&val[4],&probs[4],&val[5],&probs[5],&val[6],&probs[6],&val[7],&probs[7],&val[8],&probs[8],&val[9],&probs[9]);
    // printf("matches = %d\n",matches);
    if (matches >= 3){
      // Check for valid specification
      matches = (matches-1)/2;
      if (matches != N) {
	fprintf(stderr,"Warning: invalid specs in -extra information.\n");
	return;
      }
      // Find list header for protocol number
      head->prot_num = prot_num;
      // Create array of pointers to ExtraListItem's
      head->field = new struct ExtraListItem*[N];
      // printf("created array of pointers to ExtraListItem's\n");
      for (int j = 0; j < N; j++) {
	// Initialize ExtraListItem
	struct ExtraListItem *item = new struct ExtraListItem;
	int tempnum = (int)ceil((double)val[j] * (double)scale_factor);
	// printf("tempnum = %d\n",tempnum);
	item->num = tempnum;
	item->value = new int[tempnum];
	item->prob = new float[tempnum];
	float temptot = 0;
	// Initialize wildcard
	item->value[0] = 0;
	item->prob[0] = 1 - probs[j];
	// Select random values and probabilities for other values
	for (int k = 1; k < tempnum; k++){
	  item->value[k] = (int)lrand48();
	  item->prob[k] = (float)drand48();
	  temptot += item->prob[k];
	}
	// Compute scaling for probabilities
	float probscale = probs[j]/temptot;
	float tempprob = item->prob[0];;
	// Create cummulative distribution for this field
	for (int k = 1; k < tempnum; k++){
	  item->prob[k] = item->prob[k] * probscale;
	  tempprob += item->prob[k];
	  if (k == tempnum-1) item->prob[k] = 1;
	  else item->prob[k] = tempprob;
	}
	// Assign field pointer to item
	head->field[j] = item;
      }
    }
    else done = 1;
    head = head->next;
  }
  return;
}

// Print the contents of the list.
void ExtraList::print(FILE* fp) {
  fprintf(fp,"-extra\n%d\n",N);
  struct ExtraListHeader *i = first;
  while (i != NULL) {
    fprintf(fp,"%d\n",i->prot_num);
    struct ExtraListItem *j;
    for (int k = 0; k < N; k++){
      j = i->field[k];
      // fprintf(fp,"%d,%.6f\t",j->num,(1 - (j->prob[0])));
      for (int m = 0; m < j->num; m++){
	fprintf(fp,"%d,%.4f\t",j->value[m],j->prob[m]);
      }
      fprintf(fp,"\n");
    }
    fprintf(fp,"\n");
    i = i->next;
  }
  return;
}

int ExtraList::size(){
  return N;
}
