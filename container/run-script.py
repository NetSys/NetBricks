#!/usr/bin/python
import os
import subprocess
delay = os.environ['DELAY'] 
ifaces = os.environ['IFACE']
mcore = os.environ['MCORE']
rcore = os.environ['RCORE']
loc = os.environ['DELAY_TEST_ROOT']
cmd = ['%s/zcsi-delay'%loc, '-m', mcore, '--secondary', '-n', 'rte', '-d', \
        delay] 
for iface in ifaces.strip().split():
    cmd.append('-c')
    cmd.append(rcore)
    cmd.append('-v')
    cmd.append(iface)
print "Going to run ", ' '.join(cmd)
subprocess.check_call(cmd)
#echo "Using intefaces" ${IFACE[@]}
#echo "Master core" $MCORE
#echo "Receiving core" $RCORE
