// File: random_db.cc
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Generates a synthetic database af random filters

#include "stdinc.h"
#include "FilterList.h"
#include "random_db.h"

int random_db_gen(int num_filters, FilterList* filters){
  printf("Generating random filter set\n");
  int i;
  unsigned temp1;
  unsigned temp2;
  struct filter temp_filter;

  for (i = 0; i < num_filters; i++) {
    // Source Address
    temp1 = 0;
    temp1 = mrand48();
    temp_filter.sa = temp1;

    // Source Address Length
    temp1 = 0;
    temp1 = mrand48();
    temp_filter.sa_len = (temp1 >> 27);

    // Destination Address
    temp1 = 0;
    temp1 = mrand48();
    temp_filter.da = temp1;

    // Destination Address Length
    temp1 = 0;
    temp1 = mrand48();
    temp_filter.da_len = (temp1 >> 27);

    // Source Port Ranges
    temp1 = 0;
    temp1 = mrand48();
    temp1 = temp1 >> 16;
    temp2 = 0;
    temp2 = mrand48();
    temp2 = temp2 >> 16;
    if (temp1 < temp2) {
      temp_filter.sp[0] = temp1;
      temp_filter.sp[1] = temp2;
    } else {
      temp_filter.sp[0] = temp2;
      temp_filter.sp[1] = temp1;
    }

    // Destination Port Ranges
    temp1 = 0;
    temp1 = mrand48();
    temp1 = temp1 >> 16;
    temp2 = 0;
    temp2 = mrand48();
    temp2 = temp2 >> 16;
    if (temp1 < temp2) {
      temp_filter.dp[0] = temp1;
      temp_filter.dp[1] = temp2;
    } else {
      temp_filter.dp[0] = temp2;
      temp_filter.dp[1] = temp1;
    }

    // Protocol
    temp1 = 0;
    temp1 = mrand48();
    temp1 = temp1 >> 24;
    temp_filter.prot_num = temp1;

    // Flags
    temp1 = mrand48();
    temp1 = temp1 >> 16;
    temp_filter.flags = temp1;
    temp1 = mrand48();
    temp1 = temp1 >> 16;
    temp_filter.flags_mask = temp1;

    // Extra Fields
    temp_filter.num_ext_field = 0;

    // Add filter to list
    (*filters)&=temp_filter;
  }

  return num_filters;
}

