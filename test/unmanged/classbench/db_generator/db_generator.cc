// File: db_generator.cc
// David E. Taylor 
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Synthetic Database Generator 
// See README file for details

#include "stdinc.h"
#include "FilterList.h"
#include "PortList.h"
#include "random_db.h"
#include "custom_db.h"
#include "sys/time.h"

int main(int argc, char *argv[])
{
  char filename[1024];  
  char in_filename[1024];  

  FILE *fp_in; // input file pointer for parameters 
  FILE *fp_std; // output file pointer for standard form filter file 

  int num_filters = 0; // number of filters 
  
  int smoothness = 0; // precision of database replication 
  float addr_scope = 0; // adjustment to average address scope 
  float port_scope = 0; // adjustment to average port scope 
  
  // Check for correct number of input arguments 
  if (argc > 8 || argc <= 1){
    fprintf(stderr,"Usage: db_generator -hrb (-c <input parameter file>) <number of filters> <smoothness> <address scope> <port scope> <output filename>\n");
    fprintf(stderr,"db_generator is a synthetic filter database generator.\n");
    fprintf(stderr,"Usage: db_generator -hrb (-c <input parameter file>) <number of filters> <smoothness> <address scope> <port scope> <output filename>\n");
    fprintf(stderr,"\t -h displays help menu\n");
    fprintf(stderr,"\t -r generates a random database\n");
    fprintf(stderr,"\t -b turns on address prefix scaling with database size; note that this alters the skew distribution in the parameter file\n");
    fprintf(stderr,"\t -c generates a custom database using an input parameter file\n");
    fprintf(stderr,"\t <smoothness> is a parameter [0:64] that injects structured randomness\n");
    fprintf(stderr,"\t <address scope> is a parameter [-1.0:1.0] that adjusts the average scope of the address prefix pairs\n");
    fprintf(stderr,"\t <port scope> is a parameter [-1.0:1.0] that adjusts the average scope of the port range pairs\n");
    fprintf(stderr,"\t \t positive values increase scope (favor shorter, less specific address prefixes)\n");
    fprintf(stderr,"\t \t negative values decrease scope (favor longer, more specific address prefixes)\n");
    fprintf(stderr,"\n");
    fprintf(stderr,"Example: db_generator -bc MyParameters 10000 2 -0.5 0.1 MyFilters10k\n");
    exit(1);
  }

  int random = 0;
  int custom = 0;
  int branch = 0;
  int c = 0;
  // Check for switches 
  while (--argc > 0 && (*++argv)[0] == '-'){
    while (c = *++argv[0]){
      switch (c) {
      case 'r':
	random = 1;
	break;
      case 'c':
	custom = 1;
	break;
      case 'b':
	branch = 1;
	break;
      case 'h':
	fprintf(stderr,"db_generator is a synthetic filter database generator.\n");
	fprintf(stderr,"Usage: db_generator -hrb (-c <input parameter file>) <number of filters> <smoothness> <address scope> <port scope> <output filename>\n");
	fprintf(stderr,"\t -h displays help menu\n");
	fprintf(stderr,"\t -r generates a random database\n");
	fprintf(stderr,"\t -b turns on address prefix scaling with database size; note that this alters the skew distribution in the parameter file\n");
	fprintf(stderr,"\t -c generates a custom database using an input parameter file\n");
	fprintf(stderr,"\t <smoothness> is a parameter [0:64] that injects structured randomness\n");
	fprintf(stderr,"\t <address scope> is a parameter [-1.0:1.0] that adjusts the average scope of the address prefixe pairs\n");
	fprintf(stderr,"\t <port scope> is a parameter [-1.0:1.0] that adjusts the average scope of the port range pairs\n");
	fprintf(stderr,"\t \t positive values increase scope (favor shorter, less specific address prefixes)\n");
	fprintf(stderr,"\t \t negative values decrease scope (favor longer, more specific address prefixes)\n");
	fprintf(stderr,"\n");
	fprintf(stderr,"Example: db_generator -bc MyParameters 10000 2 -0.5 0.1 MyFilters10k\n");
	exit(1);
      default :
	printf("Illegal option %c\n",c);
	argc = 0;
	break;
      }
    }
  }
  
  if (random == 1 && argc == 2){
    num_filters = atoi(argv[0]);
    strcpy(filename,argv[1]);
    // printf("filename = %s\n",filename);
  } else if (custom == 1 && argc == 6){
    strcpy(in_filename,argv[0]);
    num_filters = atoi(argv[1]);
    smoothness = atoi(argv[2]);
    // printf("smoothness = %d\n",smoothness);
    if (smoothness < 0 || smoothness > 64) {
      fprintf(stderr,"Error smoothness must be a value in the range [0:64]\n");
      fprintf(stderr,"Usage: db_generator -hr (-c <input parameter file>) <number of filters> <smoothness> <address scope> <port scope> <output filename>\n");
      exit(1);
    }
    sscanf(argv[3],"%f",&addr_scope);
    // printf("addr_scope = %.4f\n",addr_scope);
    if (addr_scope < -1 || addr_scope > 1) {
      fprintf(stderr,"Error address scope must be a value in the range [-1:1]\n");
      fprintf(stderr,"Usage: db_generator -hr (-c <input parameter file>) <number of filters> <smoothness> <address scope> <port scope> <output filename>\n");
      exit(1);
    }
    sscanf(argv[4],"%f",&port_scope);
    // printf("addr_scope = %.4f\n",port_scope);
    if (port_scope < -1 || port_scope > 1) {
      fprintf(stderr,"Error port scope must be a value in the range [-1:1]\n");
      fprintf(stderr,"Usage: db_generator -hr (-c <input parameter file>) <number of filters> <smoothness> <address scope> <port scope> <output filename>\n");
      exit(1);
    }
    strcpy(filename,argv[5]);
    // Open seed file
    fp_in = fopen(in_filename,"r");
    if (fp_in == NULL) {fprintf(stderr,"ERROR: cannot open seed file %s\n",in_filename); exit(1);}
  } else {
    fprintf(stderr,"Usage: db_generator -hr (-c <input parameter file>) <number of filters> <smoothness> <address scope> <port scope> <output filename>\n");
    exit(1);
  }

  // Open output file for writing 
  fp_std = fopen(filename,"w");
  
  // Allocate FilterList
  FilterList *filters = new FilterList();
  int filter_cnt;

  // Seed random number generator with long int
  long seed;
  struct timeval tp;
  if (gettimeofday(&tp,NULL) != 0) {
    fprintf(stderr,"ERROR: db_generator could not get time of day to seed random number generator\n");
    exit(1);
  }
  seed = tp.tv_usec;
  srand48(seed);

  // Generate Database 
  if (random == 1) {
    // Random database generation
    filter_cnt = random_db_gen(num_filters,filters);
  }
  else if (custom == 1) {
    // printf("fp_in = %s, branch = %d\n",in_filename,branch);
    filter_cnt = custom_db_gen(num_filters,filters,fp_in,smoothness,addr_scope,port_scope,branch);
    fclose(fp_in);
  }

  // Print filters in standard format
  (*filters).print(fp_std);
  printf("Target number of filters = %d\n",num_filters);
  printf("%d filters generated in standard form\n",filter_cnt);
  printf("\tNote that number of generated filters may be less than target number\n");
  printf("\tdue to the removal of redundant filters during the generation process.\n");
  printf("\tRedundancy may be reduced by turning on address scaling (-b switch),\n");
  printf("\tincreasing the smoothness adjustment,\n");
  printf("\tor appropriately modifying the input parameter file.\n");
  
  fclose(fp_std);
  
  delete(filters);

  return 0; 
}

