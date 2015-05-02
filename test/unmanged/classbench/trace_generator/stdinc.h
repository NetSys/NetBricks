#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <math.h>

typedef char bit;
// const int false = 0;
// const int true = 1;

const int Null = 0;
const int BIGINT = 0x7fffffff;
const int EOS = '\0';

inline int max(int x, int y) { return x > y ? x : y; }
inline double max(double x, double y) { return x > y ? x : y; }
inline int min(int x, int y) { return x < y ? x : y; }
inline double min(double x, double y) { return x < y ? x : y; }
inline int abs(int x) { return x < 0 ? -x : x; }
inline bit isdigit(int c) { return (c >= '0') && (c <= '9'); }

inline void warning(char* p) { fprintf(stderr,"Warning:%s \n",p); }
inline void fatal(char* string) {fprintf(stderr,"Fatal:%s\n",string); exit(1); }

double pow(double,double);
double log(double);

// long random(); double exp(double),log(double);

// Return a random number in [0,1] 
inline double randfrac() { return ((double) random())/BIGINT; }

// Return a random integer in the range [lo,hi].
// Not very good if range is larger than 10**7.
inline int randint(int lo, int hi) { return lo + (random() % (hi + 1 - lo)); }

// Return a random number from an exponential distribution with mean mu 
inline double randexp(double mu) { return -mu*log(randfrac()); }

// Return a random number from a geometric distribution with mean 1/p
inline int randgeo(double p) { return int(.999999 + log(randfrac())/log(1-p)); }

// Filter database stuff

#define ADDRLEN 32 // IPv4 
#define ADDRBYTES ADDRLEN/8
#define MAXFILTERS 130000
#define MAXSTR 100
// #define NULL 0

struct filter {
  unsigned sa; // IP source address
  unsigned da; // IP destination address
  int sa_len; // IP source address mask length
  int da_len; // IP destination address mask length
  int sp[2]; // Transport source port range [low,high]
  int dp[2]; // Transport destination port range [low,high]
  int prot_num; // IP protocol field
  unsigned flags; // 16-bit flags field
  unsigned flags_mask; // 16-bit mask for flags
  int num_ext_field; // Number of extra header fields
  int *ext_field; // Pointer to array of extra header fields
};

struct header {
  unsigned sa; // IP source address
  unsigned da; // IP destination address
  int sp; // Transport source port
  int dp; // Transport destination port
  int prot_num; // IP protocol field
};
