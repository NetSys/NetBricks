// File: trace_generator.cc
// David E. Taylor 
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Synthetic Trace Generator 
// Generates a list of headers given a filter set.
// Input parameters control size of trace and locality of reference.
// Locality of reference, in this context, refers to the burst size of headers.
// See README file for details

#include "stdinc.h"
#include "db_parser.h"
#include "trace_tools.h"
#include "FilterList.h"
#include "sys/time.h"

void PrintUsage(){
  fprintf(stderr,"Usage: trace_generator <Pareto parameter a> <Pareto parameter b> <scale> <filter set filename>\n");
  fprintf(stderr,"\t Pareto parameters are used to control locality of reference.\n");
  fprintf(stderr,"\t Pareto cummulative density function: D(x) = 1 - (b/x)^a\n");
  fprintf(stderr,"\t \t Examples:\n");
  fprintf(stderr,"\t \t No locality of reference, a = 1, b = 0\n");
  fprintf(stderr,"\t \t Low locality of reference, a = 1, b = 0.0001\n");
  fprintf(stderr,"\t \t High locality of reference, a = 1, b = 1\n");
  fprintf(stderr,"\t Scale parameter limits the size of the trace\n");
  fprintf(stderr,"\t \t Threshold = (# of filters in filter set) * scale\n");
  fprintf(stderr,"\t \t Once the size of the trace exceeds the threshold, the generator terminates\n");
  fprintf(stderr,"\t \t Note that a large burst near the end of the process may cause the trace to be considerably\n");
  fprintf(stderr,"\t \t larger than the Threshold\n");
  exit(1);
}

main(int argc, char *argv[])
{
  char filename[1024];  
  char filename_headers[1024]; 

  FILE *fp_in; // input file pointer 
  FILE *fp_headers; // output file pointer for header trace

  // Check for correct number of input arguments 
  if (argc != 5){
    PrintUsage();
  }

  // Get input paramters
  float a,b;
  sscanf(argv[1],"%f",&a);
  sscanf(argv[2],"%f",&b);
  int scale = atoi(argv[3]);

  // Check input parameters
  // printf("a = %.4f\n",a);
  if (a < 0 || a > 1) {
    fprintf(stderr,"Error: Pareto parameter a must be a value in the range (0:1)\n");
    PrintUsage();
  }
  // printf("b = %.4f\n",b);
  if (b < 0 || b > 1) {
    fprintf(stderr,"Error: Pareto parameter b must be a value in the range (0:1)\n");
    PrintUsage();
  }
  // printf("scale = %d\n",scale);
  if (scale <= 0) {
    fprintf(stderr,"Error: scale must be a positive integer greater than zero\n");
    PrintUsage();
  }

  // Get output filename
  strcpy(filename,argv[4]);
  // printf("filename = %s, a = %.4f\n",filename,a);

  // Open input file for reading 
  if ((fp_in = fopen(filename,"r"))==NULL){
    fprintf(stderr,"parser: can't open %s\n", filename);
    exit(1);
  }

  // Read filters
  FilterList* filters = new FilterList();
  int d;
  d = read_filters(filters, fp_in);

  // Close input file 
  fclose(fp_in);

  // Open output file for writing 
  strcpy(filename_headers,filename);
  strcat(filename_headers,"_trace");
  fp_headers = fopen(filename_headers,"w");

  // Seed random number generator with long int
  long seed;
  struct timeval tp;
  if (gettimeofday(&tp,NULL) != 0) {
    fprintf(stderr,"ERROR: trace_generator could not get time of day to seed random number generator\n");
    exit(1);
  }
  seed = tp.tv_usec;
  srand48(seed);

  // Generate headers
  int thresh = filters->size() * scale;
  printf("Generating a trace using a threshold of %d\n",thresh);
  int size = header_gen(d,filters,fp_headers,a,b,scale);
  printf("Generated a trace containing %d headers\n",size);

  return 0; 
}

