Synthetic Header Trace Generator (trace_generator)
David E. Taylor
Applied Research Laboratory
Department of Computer Science and Engineering
Washington University in Saint Louis
det3@arl.wustl.edu

DISCLAIMER:  This code is freely available for academic, non-commercial research and educational purposes.  The author, Applied Research Laboratory,
Department of Computer Science and Engineering, and Washington University in Saint Louis are NOT liable for ANYTHING.  This code is provided
with absolutely NO GUARANTEE or WARRANTY.

This tool generates a list of packet headers to probe the input filter set.  Entries in the output header trace have the 
following format (packet header fields are unsigned integers):

<Source Address>	<Destination Address>	<Source Port>	<Destination Port>	<Protocol>	<Filter Number>

The Filter Number simply records the filter used to generate the header.  This may NOT be the best-matching (or first-matching) filter
for the packet header.  The user must independently verify the correctness of their search engine.  The tool provides control over
the size of the trace (relative to the input filter set) and the locality of reference of the trace (the "burst size" of entries).
The generation process basically proceeds as follows:
	NumberOfHeaders = 0
	Threshold = sizeof(MyFilters) * scale
	while(NumberOfHeaders < Threshold)
		Pick a random filter from MyFilters
		Pick a random header covered by the filter
			(This is done by selecting a random "corner" of the polyhedra
			 defined by the filter in d-dimensional space.)
		Append the header to the trace P times
			(where P is Pareto random variable controlled by input parameters a and b)
		NumberOfHeaders += P

Compile:
make all

Usage:
trace_generator <Pareto parameter a> <Pareto parameter b> <scale> <filter set filename>

	 Pareto parameters are used to control locality of reference.
	 Pareto cummulative density function: D(x) = 1 - (b/x)^a
	 	 Examples:
	 	 No locality of reference, a = 1, b = 0
	 	 Low locality of reference, a = 1, b = 0.0001
	 	 High locality of reference, a = 1, b = 1
	 Scale parameter limits the size of the trace
	 	 Threshold = (# of filters in filter set) * scale
	 	 Once the size of the trace exceeds the threshold, the generator terminates
	 	 Note that a large burst near the end of the process may cause the trace to be considerably
	 	 larger than the Threshold

Example:
trace_generator 1 0.1 10 MyFilters

Output:
<filter set filename>_trace
This file will contain a list of header entries in the previous defined format.
