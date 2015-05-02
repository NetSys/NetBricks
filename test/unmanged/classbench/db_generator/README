Synthetic Filter Set Generator (db_generator)
David E. Taylor
Applied Research Laboratory
Department of Computer Science and Engineering
Washington University in Saint Louis
det3@arl.wustl.edu

DISCLAIMER:  This code is freely available for academic, non-commercial research and educational purposes.  The author, Applied Research Laboratory,
Department of Computer Science and Engineering, and Washington University in Saint Louis are NOT liable for ANYTHING.  This code is provided
with absolutely NO GUARANTEE or WARRANTY.

This tool generates a set of filters that preserves real filter set characteristics while providing high-level controls for altering
the composition of filters.  Filter set characteristics are defined by an input parameter file.  The format of that file is described
below.  The high-level controls are more completely described in the technical report, but we briefly discuss them below:
	- Smoothness: this adjustment allows the user to simulate increased address aggregation by allowing the tool to generate
	              address prefixes according to a "smoothed" distribution.  The "spikes" in the prefix length distributions of
		      the parameter file are binomial distributed.  The "width" of that distribution is controlled by the smoothness
		      input parameter which may take on values 0 through 64.  A value of 0 maintains the distributions specified in
		      the parameter file.  A value of 64 models a uniform distribution.

	- Address scope: this adjustment allows the user to bias the tool to generate more or less specific address prefixes.  It does
			 not alter the address prefix length distributions, but applies a bias to the random variable used to select
			 prefix lengths.  The bias may be any real number -1 to 1.  Negative values bias the tool to generate less specific
			 prefixes (shorter prefix length); positive values bias the tool to generate more specific prefixes (longer lengths).

	- Application scope: this adjustment allows the user to bias the tool to generate more or less specific application specifications (protocol,
			     port range combination).  It does not alter the application distributions, but applies a bias to the random variable
		             used to select protocol values and port ranges.  The bias may be any real number -1 to 1.  Negative values bias the tool
	 		     to generate less specific application specifications (wildcard protocols, "wider" port ranges, and/or wildcard port ranges);
			     positive values bias the tool to generate more specific specifications (specified protocols, exact ports, etc.).

Compile:
make all

Usage:
db_generator -hrb (-c <input parameter file>) <number of filters> <smoothness> <address scope> <application scope> <output filename>

	-h displays help menu
	-r generates a random database
	-b turns on address prefix scaling with database size; note that this alters the skew distribution in the parameter file
	-c generates a custom database using an input parameter file
	<smoothness> is a parameter [0:64] that injects structured randomness
	<address scope> is a parameter [-1.0:1.0] that adjusts the average scope of the address prefixes
	<application scope> is a parameter [-1.0:1.0] that adjusts the average scope of the application specification (protocol and ports)
		negative values increase scope (favor shorter, less specific address prefixes)
		positive values decrease scope (favor longer, more specific address prefixes)

Example:
db_generator -bc MyParameters 10000 2 0.5 -0.1 MyFilters10k

Parameter file format:

-scale
<integer>
#
This entry specifies the number of filters in the seed database, used to scale addresses when -b switch is used

-prots
<protocol>	<probability>	<PPC 0 probability> <PPC 1 probability> ... <PPC 24 probability>
<protocol>	<probability>	<PPC 0 probability> <PPC 1 probability> ... <PPC 24 probability>
#
This entry specifies the distribution of protocol values.  For each protocol value, there is a port pair class (PPC) distribution.  PPC's are
defined as follows:
PPC
0	WC/WC 	both port ranges wildcard
1	WC/HI	source port wildcard, destination port [1024:65535]
2	HI/WC	source port [1024:65535], destination port wildcard
3	HI/HI	source port [1024:65535], destination port [1024:65535]
4	WC/LO	source port wildcard, destination port [0:1023]
5	LO/WC	source port [0:1023], destination port wildcard
6	HI/LO	source port [1024:65535], destination port [0:1023]
7	LO/HI	source port [0:1023], destination port [1024:65535]
8	LO/LO	source port [0:1023], destination port [0:1023]
9	WC/AR	source port wildcard, destination port arbitrary range
10	AR/WC	source port arbitrary range, destination port wildcard
11	HI/AR	source port [1024:65535], destination port arbitrary range
12	AR/HI	source port arbitrary range, destination port [1024:65535]
13	LO/AR	source port [0:1023], destination port arbitrary range
14	AR/LO	source port arbitrary range, destination port [0:1023]
15	AR/AR	source port arbitrary range, destination port arbitrary range
16	WC/EM	source port wildcard, destination port exact match
17	EM/WC	source port exact match, destination port wildcard
18	HI/EM	source port [1024:65535], destination port exact match
19	EM/HI	source port exact match, destination port [1024:65535]
20	LO/EM	source port [0:1023], destination port exact match
21	EM/LO	source port exact match, destination port [0:1023]
22	AR/EM	source port arbitrary range, destination port exact match
23	EM/AR	source port exact match, destination port exact match arbitrary range
24	EM/EM	source port exact match, destination port exact match

-flags
<protocol>	<flags>/<mask>,<probability>	<flags>/<mask>,<probability>	...
<protocol>	<flags>/<mask>,<probability>	<flags>/<mask>,<probability>	...
#
Distribution of flags specification for each protocol.  Flags/masks are specified as 4 digit hexadecimal numbers, i.e. 0x0AF1/0x0FFF

-extra
<# of extra fields>
<protocol>	<# of non-wildcard values for extra field 1>,<prob. of non-wildcard for extra field 1>  <# of non-wildcard values for extra field 2>,<prob. of non-wildcard for extra field 2> ...
<protocol>	<# of non-wildcard values for extra field 1>,<prob. of non-wildcard for extra field 1>  <# of non-wildcard values for extra field 2>,<prob. of non-wildcard for extra field 2> ...
#
Number of random extra fields to generate; useful for testing scaling to additional filter fields.

-spar
<probability>	<range>
<probability>	<range>
...
<probability>	<range>
#
Distribution of arbitrary range specifications for source port.

-spem
<probability>	<range>
<probability>	<range>
...
<probability>	<range>
#
Distribution of exact match port specifications for source port.

-dpar
<probability>	<range>
<probability>	<range>
...
<probability>	<range>
#
Distribution of arbitrary range specifications for destination port.

-dpem
<probability>	<range>
<probability>	<range>
...
<probability>	<range>
#
Distribution of exact match port specifications for destination port.

-wc_wc
<total length>,<probability>	<source length>,<probability> <source length>,<probability> ...
<total length>,<probability>	<source length>,<probability> <source length>,<probability> ...
...
<total length>,<probability>	<source length>,<probability> <source length>,<probability> ...
#
Two-dimensional address prefix length distribution for filters with PPC WC/WC.  Same format for
the remaining 24 PPC's.

-wc_lo
#
-wc_hi
#
-wc_ar
#
-wc_em
#
-lo_wc
#
-lo_lo
#
-lo_hi
#
-lo_ar
#
-lo_em
#
-hi_wc
#
-hi_lo
#
-hi_hi
#
-hi_ar
#
-hi_em
#
-ar_wc
#
-ar_lo
#
-ar_hi
#
-ar_ar
#
-ar_em
#
-em_wc
#
-em_lo
#
-em_hi
#
-em_ar
#
-em_em
#

-snest
<integer>
#
Specifies the maximum number of prefixes along any path from root to leaf in the address trie.
Provides control over prefix nesting in source address trie.

-sskew
<level>	<prob. of 1 child> <prob. 2 children> <skew for nodes w/ 2 children>
...
<level>	<prob. of 1 child> <prob. 2 children> <skew for nodes w/ 2 children>
#
Source address skew distribution.  Provides a definition of the trie structure; i.e. the amount of IP address
space covered by address prefixes.

-dnest
<integer>
#
Specifies the maximum number of prefixes along any path from root to leaf in the address trie.
Provides control over prefix nesting in destination address trie.

-dskew
<level>	<prob. of 1 child> <prob. 2 children> <skew for nodes w/ 2 children>
...
<level>	<prob. of 1 child> <prob. 2 children> <skew for nodes w/ 2 children>
#
Destination address skew distribution.  Provides a definition of the trie structure; i.e. the amount of IP address
space covered by address prefixes.


-pcorr
<level> <probability>
...
<level> <probability>
#
Probability that source and destination addresses continue to be the same at a given prefix length.

