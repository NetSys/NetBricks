#!/usr/bin/python

import os
import time
import sys
from string import Template

result = ['TEST', 'RBW', 'IDX', 'Min', 'Avg', 'Max', '1%ile', '50%ile', '99%ile', '99.9%ile', '99.99%ile', '99.999%ile', '99.9999%ile', '#pkts', 'Thruput', 'Loss']


print ' '.join(result)
for arg in sys.argv[1:]:
  try:
    row = []
    fname = os.path.basename(os.path.splitext(arg)[0])
    parts = fname.split('-')
    row.append(parts[0])
    row.append(parts[1])
    row.append(parts[2])
    f = open(arg)
    for l in f:
      if l.startswith("##"):
        parts = l.split()
        if (parts[1] == 'Loss:'):
          row.append(float(parts[2]))
        else:
          row.append(int(parts[2]))
    if len(row) < len(result):
      continue
    print ' '.join(map(str, row))
  except:
    pass
