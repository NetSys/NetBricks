// File: redundant_filter_check.h
// David E. Taylor
// Applied Research Laboratory
// Department of Computer Science and Engineering
// Washington University in Saint Louis
// det3@arl.wustl.edu
//
// Compares two filters and returns a Boolean (1 = True, 0 = False)

int redundant_check(struct filter filt1, struct filter filt2);
int sa_prefix_match(struct filter filt1, struct filter filt2);
int da_prefix_match(struct filter filt1, struct filter filt2);
int sp_range_match(struct filter filt1, struct filter filt2);
int dp_range_match(struct filter filt1, struct filter filt2);
int prot_match(struct filter filt1, struct filter filt2);
int flag_match(struct filter filt1, struct filter filt2);
