// File: PrefixList.cc
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Class definition for PrefixList

#include "stdinc.h"
#include "PrefixList.h"

PrefixList::PrefixList() {
  N = 65;
  cdist = 0;
  prefixes = new struct prefix*[25];
  for (int type = 0; type < 25; type++){
    prefixes[type] = new struct prefix[N];
    for (int i = 0; i < N; i++) {
      prefixes[type][i].prob = 0;
      for (int j = 0; j < 33; j++) prefixes[type][i].sprob[j] = 0;
    }
  }
}

PrefixList::~PrefixList() {
  for (int type = 0; type < 25; type++) delete prefixes[type];
  delete prefixes;
}

void PrefixList::read(FILE* fp){
  for (int type = 0; type < 25; type++) read_type(type,fp);
  return;
}

void PrefixList::read_type(int type, FILE *fp) {
  int done = 0;
  int matches = 0;
  char comm[6];
  char wc_wc_comm[]="-wc_wc";
  char wc_lo_comm[]="-wc_lo";
  char wc_hi_comm[]="-wc_hi";
  char wc_ar_comm[]="-wc_ar";
  char wc_em_comm[]="-wc_em";
  char lo_wc_comm[]="-lo_wc";
  char lo_lo_comm[]="-lo_lo";
  char lo_hi_comm[]="-lo_hi";
  char lo_ar_comm[]="-lo_ar";
  char lo_em_comm[]="-lo_em";
  char hi_wc_comm[]="-hi_wc";
  char hi_lo_comm[]="-hi_lo";
  char hi_hi_comm[]="-hi_hi";
  char hi_ar_comm[]="-hi_ar";
  char hi_em_comm[]="-hi_em";
  char ar_wc_comm[]="-ar_wc";
  char ar_lo_comm[]="-ar_lo";
  char ar_hi_comm[]="-ar_hi";
  char ar_ar_comm[]="-ar_ar";
  char ar_em_comm[]="-ar_em";
  char em_wc_comm[]="-em_wc";
  char em_lo_comm[]="-em_lo";
  char em_hi_comm[]="-em_hi";
  char em_ar_comm[]="-em_ar";
  char em_em_comm[]="-em_em";

  // read in port width/range
  while (matches != EOF && done == 0) {
    matches = fscanf(fp,"%s\n",comm);
    // printf("comm = %s\n",comm);
    if (type == 0 && (strcmp(comm,wc_wc_comm) == 0)) done = 1;
    else if (type == 1 && (strcmp(comm,wc_hi_comm) == 0)) done = 1;
    else if (type == 2 && (strcmp(comm,hi_wc_comm) == 0)) done = 1; 
    else if (type == 3 && (strcmp(comm,hi_hi_comm) == 0)) done = 1; 
    else if (type == 4 && (strcmp(comm,wc_lo_comm) == 0)) done = 1; 
    else if (type == 5 && (strcmp(comm,lo_wc_comm) == 0)) done = 1;
    else if (type == 6 && (strcmp(comm,hi_lo_comm) == 0)) done = 1;
    else if (type == 7 && (strcmp(comm,lo_hi_comm) == 0)) done = 1; 
    else if (type == 8 && (strcmp(comm,lo_lo_comm) == 0)) done = 1; 
    else if (type == 9 && (strcmp(comm,wc_ar_comm) == 0)) done = 1; 
    else if (type == 10 && (strcmp(comm,ar_wc_comm) == 0)) done = 1;
    else if (type == 11 && (strcmp(comm,hi_ar_comm) == 0)) done = 1;
    else if (type == 12 && (strcmp(comm,ar_hi_comm) == 0)) done = 1; 
    else if (type == 13 && (strcmp(comm,wc_em_comm) == 0)) done = 1; 
    else if (type == 14 && (strcmp(comm,em_wc_comm) == 0)) done = 1; 
    else if (type == 15 && (strcmp(comm,hi_em_comm) == 0)) done = 1;
    else if (type == 16 && (strcmp(comm,em_hi_comm) == 0)) done = 1;
    else if (type == 17 && (strcmp(comm,lo_ar_comm) == 0)) done = 1; 
    else if (type == 18 && (strcmp(comm,ar_lo_comm) == 0)) done = 1; 
    else if (type == 19 && (strcmp(comm,lo_em_comm) == 0)) done = 1; 
    else if (type == 20 && (strcmp(comm,em_lo_comm) == 0)) done = 1;
    else if (type == 21 && (strcmp(comm,ar_ar_comm) == 0)) done = 1;
    else if (type == 22 && (strcmp(comm,ar_em_comm) == 0)) done = 1; 
    else if (type == 23 && (strcmp(comm,em_ar_comm) == 0)) done = 1; 
    else if (type == 24 && (strcmp(comm,em_em_comm) == 0)) done = 1; 
  }
  if (matches == EOF) {
    fprintf(stderr,"Warning: Could not find proper identifier.\nNo prefix information taken from parameter file.\n");
    return;
  }
  // printf("Found identifier; done = %d\n",done);
  done = 0;
  int tlen = 0;
  int slen = 0;
  float prob = 0;
  int lens[34];
  float probs[34];
  char scomm[500];
  int scomm_len = 500;
  while (done == 0) {
    fgets(scomm,scomm_len,fp);
    // Read a line of the input
    matches = sscanf(scomm,"%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f\t%d,%f",&lens[0],&probs[0],&lens[1],&probs[1],&lens[2],&probs[2],&lens[3],&probs[3],&lens[4],&probs[4],&lens[5],&probs[5],&lens[6],&probs[6],&lens[7],&probs[7],&lens[8],&probs[8],&lens[9],&probs[9],&lens[10],&probs[10],&lens[11],&probs[11],&lens[12],&probs[12],&lens[13],&probs[13],&lens[14],&probs[14],&lens[15],&probs[15],&lens[16],&probs[16],&lens[17],&probs[17],&lens[18],&probs[18],&lens[19],&probs[19],&lens[20],&probs[20],&lens[21],&probs[21],&lens[22],&probs[22],&lens[23],&probs[23],&lens[24],&probs[24],&lens[25],&probs[25],&lens[26],&probs[26],&lens[27],&probs[27],&lens[28],&probs[28],&lens[29],&probs[29],&lens[30],&probs[30],&lens[31],&probs[31],&lens[32],&probs[32],&lens[33],&probs[33]);
    // printf("matches = %d, tlen = %d, prob = %.4f\n",matches,lens[0],probs[0]);
    if (matches >= 4) {
      // Assign total probability
      prefixes[type][lens[0]].prob = probs[0];
      matches = (matches - 2)/2;
      // printf("matches = %d\n",matches);
      for (int j = 1; j <= matches; j++) {
	// printf("Assigning prefixes[type][%d].sprob[%d] = %.4f\n",lens[0],lens[j],probs[j]);
	prefixes[type][lens[0]].sprob[lens[j]] = probs[j];
      }
    }
    else done = 1;
  }
  
  return;
}

void PrefixList::binomial(int k, double p[]){
  // printf("binomial: k = %d\n",k);
  double num, den;
  for (int i=0; i <= k; i++) {
    num = factorial(k);
    den = factorial(i);
    den *= factorial(k-i);
    p[i] = num/den;
    // printf("p[%d] = %.6f\t",i,p[i]);
    p[i] *= pow(0.5,k);
    // printf("p[%d] = %.8f\n",i,p[i]);
  }
  // printf("\n");
  return;
}  

void PrefixList::smooth(int s){
  for (int type = 0; type < 25; type++) smooth_type(type,s);
  return;
}

void PrefixList::smooth_type(int type, int s){
  // Adjust for smoothing parameter s
  // Symmetric Binomial "spread" in each direction = s
  int skj, sk, tk;
  double tp[129];
  double sp[129];
  double spj[129];
  struct prefix temps[65];
  int tlen, slen;
  int r, start, end;
  int delta;

  // printf("this = 0x%08x\n",this);

  for (int i=0; i<65; i++) {
    temps[i].prob = 0;
    for (int j=0; j<33; j++) {
      temps[i].sprob[j] = 0;
    }
  }

  tk = 2*s;
  binomial(tk,tp);
  // printf("Generated binomial distribution\n");
  
  int bdist;
  // Adjust probabilities of total length distribution
  for (int i = 0; i < N; i++) {
    // i = total length
    // Find spike
    // printf("prefixes[type][%d].prob = %.4f\n",i,prefixes[type][i].prob);
    if (prefixes[type][i].prob > 0) {
      // printf("i - s = %d - %d = %d\n",i,s,i-s);
      // Check for boundary conditions and set start points
      if (i - s < 0) {
	// Spreading past total length 0
	tlen = 0; bdist = s - i;
      } else {
	tlen = i - s; bdist = 0;
      }
      // printf("tlen = %d, bdist = %d\n",tlen,bdist);
      while (tlen <= 64 && bdist <= tk) {

	if (tlen < 0 || tlen > 64){
	  printf("Error 1 : tlen is out of range, tlen = %d\n",tlen);
	  exit(1);
	}

	// Add binomial fraction of spike's probability
	temps[tlen].prob += tp[bdist]*prefixes[type][i].prob;
	// printf("temps[%d].prob += (tp[%d]*prefixes[type][%d].prob = %.4f) = %.4f\n",tlen,bdist,i,tp[bdist]*prefixes[type][i].prob,temps[tlen].prob);
	// Increment pointers
	tlen++;
	bdist++;
      } // end while
      // Adjust source distributions
      // Spread source distribution spikes for total length i
      // Spread spike in source distribution for total distribution i
      skj = ((int)floor((double)s/(double)2)) * 2;
      binomial(skj,spj);
      for (int j = 0; j <= 32; j++){
	// j = source length
	// Find source spike
	if (prefixes[type][i].sprob[j] > 0){
	  // Check for boundary conditions and set start points
	  // printf("j - skj/2 = %d\n",j-skj/2);	  
	  if (j - skj/2 < 0) {
	    // Spreading past source length 0
	    slen = 0; bdist = skj/2 - j;
	  } else {
	    slen = j - skj/2; bdist = 0;
	  }
	  // printf("slen = %d, bdist = %d\n",slen,bdist);
	  while (slen <= 32 && bdist <= skj) {
	    
	    if (i < 0 || i > 64 || slen < 0 || slen > 32){
	      printf("Error 2: i or slen is out of range, i = %d, slen = %d\n",i,slen);
	      exit(1);
	    }

	    // Add binomial fraction of spike's probability
	    temps[i].sprob[slen] += spj[bdist]*prefixes[type][i].sprob[j];
	    // printf("temps[%d].sprob[%d] += (spj[%d]*prefixes[type][%d].sprob[%d] = %.4f) = %.4f\n",i,slen,bdist,i,j,spj[bdist]*prefixes[type][i].sprob[j],temps[i].sprob[slen]);
	    // Increment pointers
	    slen++;
	    bdist++;
	  } // end while

	  // Create source distributions for total distribution spikes caused by "spread"
	  // for total distributions between (i - s) and (i - 1)
	  // s is the "Manhattan" distance from original i
	  for (int m = 1; m <= s; m++) {
	    // compute total length point
	    r = i - m;
	    if (0 <= r && r <= 64) {
	      // set start, end points for source distribution
	      start = j - m;
	      end = j;
	      // compute difference in "distance"
	      delta = s - m;
	      // printf("m = %d, r = %d, start = %d, end = %d, delta = %d\n",m,r,start,end,delta);
	      while (delta >= 2) {
		start--;
		end++;
		delta -= 2;
	      }
	      // printf("start = %d, end = %d, delta = %d\n",start,end,delta);
	      sk = end - start;
	      // printf("sk = %d\n",sk);
	      binomial(sk,sp);
	      // Spread source distribution of r
	      // Check for boundary conditions and set start points
	      if (start < 0) {
		// Spreading past source length 0
		bdist = 0 - start;
		start = 0;
	      } else {
		bdist = 0;
	      }
	      // printf("start = %d, bdist = %d\n",start,bdist);
	      while (start <= 32 && bdist <= sk) {
		
		if (r < 0 || r > 64 || start < 0 || start > 32){
		  printf("Error 3: r or start is out of range, r = %d, start = %d\n",r,start);
		  exit(1);
		}
		
		// Add binomial fraction of spike's probability, weighted by total probability
		temps[r].sprob[start] += sp[bdist]*prefixes[type][i].sprob[j]*prefixes[type][i].prob;
		// printf("temps[%d].sprob[%d] += (sp[%d]*prefixes[type][%d].sprob[%d]*prefixes[type][%d].prob = %.4f) = %.4f\n",r,start,bdist,i,j,i,sp[bdist]*prefixes[type][i].sprob[j]*prefixes[type][i].prob,temps[r].sprob[start]);
		// Increment pointers
		start++; bdist++;
	      }
	    } // end if
	  } // end for (i-s) to (i-1)

	  // Create source distributions for total distribution spikes caused by "spread"
	  // for total distributions between (i + 1) and (i + s)
	  // s is the "Manhattan" distance from original i
	  for (int m = 1; m <= s; m++) {
	    // compute total length point
	    r = i + m;
	    if (0 <= r && r <= 64) {
	      // set start, end points for source distribution
	      start = j;
	      end = j + m;
	      // compute difference in "distance"
	      delta = s - m;
	      // printf("i = %d, j= %d, m = %d, r = %d, start = %d, end = %d, delta = %d\n",i,j,m,r,start,end,delta);
	      while (delta >= 2) {
		start--;
		end++;
		delta -= 2;
	      }
	      // printf("start = %d, end = %d, delta = %d\n",start,end,delta);
	      sk = end - start;
	      binomial(sk,sp);
	      // Spread source distribution of r
	      // Check for boundary conditions and set start points
	      if (start < 0) {
		// Spreading past source length 0
		bdist = 0 - start;
		start = 0;
	      } else {
		bdist = 0;
	      }
	      // printf("start = %d, bdist = %d\n",start,bdist);
	      while (start <= 32 && bdist <= sk) {
		
		if (r < 0 || r > 64 || start < 0 || start > 32){
		  printf("Error 4: r or start is out of range, start = %d, q = %d\n",r,start);
		  exit(1);
		}
		
		// Add binomial fraction of spike's probability, weighted by total probability
		temps[r].sprob[start] += sp[bdist]*prefixes[type][i].sprob[j]*prefixes[type][i].prob;
		// printf("temps[%d].sprob[%d] += (sp[%d]*prefixes[type][%d].sprob[%d]*prefixes[type][%d].prob = %.4f) = %.4f\n",r,start,bdist,i,j,i,sp[bdist]*prefixes[type][i].sprob[j]*prefixes[type][i].prob,temps[r].sprob[start]);
		// Increment pointer
		start++; bdist++;
	      } // end while
	    } // end if
	  } // end for (i+1) to (i+s)
	} // end if sprob > 0, source spike
      } // end for j : 0 to 32
    } // end if prob > 0, total spike
  } // end for i : 0 to 64

  // Normalize data structure
  float totw = 0;
  float tots = 0;
  for (int i = 0; i < N; i++) totw += temps[i].prob;
  // Check for empty distribution
  if (totw > 0) {
    for (int i = 0; i < N; i++) {
      temps[i].prob = temps[i].prob / totw;
      if (temps[i].prob > 0) {
	// Truncate source distribution
	if (i < 32) {
	  for (int j = i + 1; j <= 32; j++) temps[i].sprob[j] = 0;
	}
	else if (i > 32) {
	  for (int j = 0; j < i - 32; j++) temps[i].sprob[j] = 0; 
	}
	// Normalize source distribution
	tots = 0;
	for (int j = 0; j <= 32; j++) tots += temps[i].sprob[j]; 
	for (int j = 0; j <= 32; j++) temps[i].sprob[j] = temps[i].sprob[j] / tots; 
      }
    }
    // Apply adjustments to prefixes data structure
    for (int i = 0; i < N; i++) {
      prefixes[type][i].prob = temps[i].prob;
      if (i == N-1) prefixes[type][i].prob = 1;
      for (int j = 0; j < 33; j++) {
	prefixes[type][i].sprob[j] = temps[i].sprob[j];
	if (j == 32) prefixes[type][i].sprob[j] = 1;
      }
    }
  }
  // print(stdout);
  // printf("this = 0x%08x\n",this);
  return;
}

struct ppair PrefixList::choose_prefix(int type, float rs, float rt) {
  struct ppair pair;
  int port = -1;
  int done = 0;
  // printf("choose_prefix(%.4f,%.4f)\n",rs,rt);
  if (cdist == 0) {build_cdist(); cdist = 1;}

  for (int i = 0; (i < N && done == 0); i++) {
    // printf("rt = %.6f, prefixes[type][%d].prob = %.6f\n",rt,i,prefixes[type][i].prob);
    if (rt <= prefixes[type][i].prob) {
      for (int j = 0; (j < 33 && done == 0); j++) {
	// printf("rs = %.6f, prefixes[type][%d].sprob[%d] = %.6f\n",rs,i,j,prefixes[type][i].sprob[j]);
	if (rs <= prefixes[type][i].sprob[j]){
	  pair.slen = j;
	  pair.dlen = i - j;
	  done = 1;
	}
      }
    }
  }
  return pair;
}

void PrefixList::build_cdist() {
  for (int type = 0; type < 25; type++){
    float tp = 0;
    float sp = 0;
    
    // printf("build_cdist()\n");
    // Build cummulative distributions
    for (int i = 0; i < N; i++) {
      prefixes[type][i].prob += tp;
      tp = prefixes[type][i].prob;
      // Cummulative source distribution
      sp = 0;
      for (int j = 0; j < 33; j++){
	prefixes[type][i].sprob[j] += sp;
	sp = prefixes[type][i].sprob[j];
      } 
    }
  }
  return;
}

void PrefixList::print(int type, FILE *fp) {
  // printf("N = %d\n",N);
  for (int i = 0; i < N; i++){
    if (prefixes[type][i].prob != 0) {
      fprintf(fp,"%d,%.8f",i,prefixes[type][i].prob);
      for (int j = 0; j <= 32; j++) {
	if (prefixes[type][i].sprob[j] != 0) {
	  fprintf(fp,"\t%d,%.8f",j,prefixes[type][i].sprob[j]);
	}
      }
      fprintf(fp,"\n");
    }
  }

  return;
}

