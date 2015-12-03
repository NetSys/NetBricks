// File: db_parser.cc
// David E. Taylor 
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Parser for filter sets in standard form with optional extra fields
//
#include "stdinc.h"
#include "db_parser.h"

// Read filters from input file in "NEW standard" form
// See README file for more details
// Returns the number of fields in the first filter
int read_filters(FilterList *filters, FILE *fp_in){
  int line_matches = 0;
  int matches = 0;
  char *line_buffer = new char[500];
  char string_buffers[20][20];
  int buffer_size = 500;
  int temp_addr[4];
  unsigned temp;
  int d = 5;
  struct filter temp_filter;
  int dtemp;

  for (int i = 0; i < MAXFILTERS; i++){
    dtemp = 0;
    // Initialize filter entry
    temp_filter.sa = temp_filter.da = 0;
    temp_filter.sa_len = temp_filter.da_len = 0;
    temp_filter.sp[0] = temp_filter.sp[1] = temp_filter.dp[0] = temp_filter.dp[1] = 0;
    temp_filter.prot_num = 0;
    temp_filter.flags = temp_filter.flags_mask = 0;
    temp_filter.num_ext_field = 0;
 
    
    // Read in line
    line_buffer = fgets(line_buffer, buffer_size, fp_in);
    if (line_buffer == NULL) return d;
    
    // Parse line into strings
    line_matches = sscanf(line_buffer, "%s %s %s %s %s %s %s %s %s %s %s %s %s %s %s %s %s %s %s %s",
			  string_buffers[0], string_buffers[1], string_buffers[2], string_buffers[3], string_buffers[4],
			  string_buffers[5], string_buffers[6], string_buffers[7], string_buffers[8], string_buffers[9],
			  string_buffers[10], string_buffers[11], string_buffers[12], string_buffers[13], string_buffers[14],
			  string_buffers[15], string_buffers[16], string_buffers[17], string_buffers[18], string_buffers[19]);
    // printf("line_matches = %d\n",line_matches);
    // Scan through strings to find fields
    if (line_matches > 4) {
      // Scan string buffer 0 for decimal-dot address format
      matches = sscanf(string_buffers[0], "@%d.%d.%d.%d/%d",
		       &temp_addr[0], &temp_addr[1], &temp_addr[2], &temp_addr[3], &temp_filter.sa_len);
      // Check the number of matches
      if (matches == 5) {
	// Valid format 
	// Convert bytes to unsigned
	temp = 0;
	temp = temp_addr[0] << 24 ; // high order 8-bits
	temp = (temp | (temp_addr[1] << 16)); // next 8-bits
	temp = (temp | (temp_addr[2] << 8)); // next 8-bits
	temp = (temp | temp_addr[3]); // low order 8-bits
	temp_filter.sa = temp;
	dtemp++;
      }
      else if (matches == EOF) {
	return d;
      }
      else {
	fprintf(stderr,"parser: bad source address at filter %d\n", i);
	break;
      }
      // Scan string buffer 1 for decimal-dot address format, destination
      matches = sscanf(string_buffers[1], "%d.%d.%d.%d/%d",
		       &temp_addr[0], &temp_addr[1], &temp_addr[2], &temp_addr[3], &temp_filter.da_len);
      // Check the number of matches
      if (matches == 5) {
	// Read Destination Address 
	// Convert bytes to unsigned
	temp = 0;
	temp = temp_addr[0] << 24 ; // high order 8-bits
	temp = (temp | (temp_addr[1] << 16)); // next 8-bits
	temp = (temp | (temp_addr[2] << 8)); // next 8-bits
	temp = (temp | temp_addr[3]); // low order 8-bits
	temp_filter.da = temp;
	dtemp++;
      } else {
	fprintf(stderr,"parser: bad destination address at filter %d\n", i);      
      }
      // Scan string buffer 2 for decimal source port value, low
      matches = sscanf(string_buffers[2], "%d",&temp_filter.sp[0]);
      // Check the number of matches
      if (matches != 1) {
	fprintf(stderr,"parser: error, partial filter entry at filter %d\nNo low source port specification.\n", i);
      }
      // Check for :
      if (string_buffers[3][0] != ':') {
	fprintf(stderr,"parser: error, partial filter entry at filter %d\nNo : source port specification.\n", i);
      }
      // Scan string buffer 4 for decimal source port value, high
      matches = sscanf(string_buffers[4], "%d",&temp_filter.sp[1]);
      // Check the number of matches
      if (matches != 1) {
	fprintf(stderr,"parser: error, partial filter entry at filter %d\nNo high source port specification.\n", i);
      }
      dtemp++;
      // Scan string buffer 5 for decimal destination port value, low
      matches = sscanf(string_buffers[5],"%d",&temp_filter.dp[0]);
      // printf("matches = %d, string_buffers[5] = %s\n",matches,string_buffers[5]);
      // Check the number of matches
      if (matches != 1) {
	fprintf(stderr,"parser: error, partial filter entry at filter %d\nNo low destination port specification.\n", i);
      }
      // Check for :
      if (string_buffers[6][0] != ':') {
	fprintf(stderr,"parser: error, partial filter entry at filter %d\nNo : destination port specification.\n", i);
      }
      // Scan string buffer 7 for decimal source port value, high
      matches = sscanf(string_buffers[7], "%d",&temp_filter.dp[1]);
      // Check the number of matches
      if (matches != 1) {
	fprintf(stderr,"parser: error, partial filter entry at filter %d\nNo high destination port specification.\n", i);
      }
      dtemp++;
      int prot_mask = 0;
      // Scan string buffer 8 for protocol
      matches = sscanf(string_buffers[8], "0x%02x/0x%02x",&temp_filter.prot_num,&prot_mask);
      if (matches != 2) {
	fprintf(stderr,"parser: bad protocol spec at filter %d\n", i);      
      }
      dtemp++;
      if (line_matches >= 10){
	// Initialize flags and extra fields
	temp_filter.flags = temp_filter.flags_mask = 0;
	temp_filter.num_ext_field = 0;
	temp_filter.ext_field = NULL;
	// printf("Scanning for flags\n");
	char x1, x2;
	// Read flags if present
	matches = sscanf(string_buffers[9],"0x%04x/0x%04x",&temp_filter.flags,&temp_filter.flags_mask);
	if (matches == 2) dtemp++;
	else fprintf(stderr,"parser: bad flags spec at filter %d\n", i);
      }
      if (line_matches >= 11) {
	// printf("line_matches = %d\n",line_matches);
	// Allocate extra field array
	temp_filter.ext_field = new int[line_matches - 10];
	temp_filter.num_ext_field = line_matches - 10;
	// Search for addtional fields
	for (int j = line_matches - 11; j >= 0; j--) {
	  // Scan string buffer for decimal field value, 0 = wildcard
	  matches = sscanf(string_buffers[line_matches - j - 1],"%d",&temp_filter.ext_field[j]);
	  // Check the number of matches
	  if (matches != 1) {
	    fprintf(stderr,"parser: error: invalid extra field in filter entry %d.\n", i);
	  }
	  dtemp++;
	}
      }
      // Add filter to list
      (*filters) &= temp_filter;
      if (dtemp > d) d = dtemp;
    } else {
      // printf("line_matches = %d\n",line_matches);
      fprintf(stderr,"parser: error, invalid filter at entry %d.\n", i);
      i--;
    }
  }
  delete(line_buffer);
  return d;
}
