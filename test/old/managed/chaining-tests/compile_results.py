#!/usr/bin/python

import os
import time
import sys
from string import Template

result = ['DIR', 'TEST', 'SIZE', 'Length', 'Min', 'Avg', 'Max', '1%ile', '50%ile', '99%ile', '99.9%ile', '99.99%ile', '99.999%ile', '99.9999%ile', '#pkts', 'Thruput', 'Loss']


print ' '.join(result)
for arg in sys.argv[1:]:
  row = []
  row.append(os.path.dirname(arg))
  fname = os.path.basename(os.path.splitext(arg)[0])
  parts = fname.split('-')
  row.append(parts[0])
  row.append(parts[2].replace('b',''))
  row.append(parts[3])
  f = open(arg)
  for l in f:
    if l.startswith("##"):
      parts = l.split()
      if (parts[1] == 'Loss:'):
        row.append(float(parts[2]))
      else:
        row.append(int(parts[2]))
  print ' '.join(map(str, row))
