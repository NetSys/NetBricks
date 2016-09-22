#!/usr/bin/python
import os
import signal
import struct
import sys
import time

ALLOWED_INTERFACES = [ "cpu_dma_latency", "network_latency", "network_throughput" ]
def read_pmqos(name):
	filename = "/dev/%s" % name
	old = open(filename)
	old_value = struct.unpack("i", old.read())[0]
	print "PMQOS value for %s is %d"%(name, old_value)
if __name__=="__main__":
    if len(sys.argv) < 2:
        print "Must specify what to read"
        sys.exit(1)
    read = sys.argv[1]
    if read not in ALLOWED_INTERFACES:
        print "Cannot read %s"%read
        sys.exit(1)
    read_pmqos(read)
