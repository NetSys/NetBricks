#!/usr/bin/python

import os
import time
import sys
from string import Template

RECEIVER_HOST = '10.0.1.3'
SENDER_HOST = '10.0.1.4'

str_map = {}
str_map['resultdir'] = 'results'
str_map['outfile'] = 'output'
str_map['rate'] = 0
str_map['app_cmd'] = '/home/keonjang/softnic/libsn/fastforward'
str_map['app'] = 'fastforward'
str_map['duration'] = 10
str_map['softniccmd'] = '/home/keonjang/softnic/softnic/softnic'
str_map['yield'] = 0

PREP_CMD = []
PREP_CMD.append('sudo pkill softnic')
PREP_CMD.append('sudo pkill $app')

SOFTNIC_CMD = []
SOFTNIC_CMD.append('-f \"sudo SCENARIO=s2v2s $softniccmd -c 1 -- -l $num -d $duration -b $rate > output\"')
SOFTNIC_CMD.append('sleep 1')

APP_CMD = []
APP_CMD.append('-f \"sudo $app_cmd -c 2 -i vport$index -o vport$index  -r 0 -e > /dev/null\"')

APP_SET = ['fastfowrard']
RATE_SET = [36000]#range(1000, 100000, 1000)#,[1, 10, 100, 1000, 10000]
NUM_SET = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]#[1, 2, 4, 8, 10, 16, 32, 64, 100, 128, 256, 512, 1000, 1024, 2048, 4096, 8192, 10000]



def print_red(str):
    print ('\033[91m' + str + '\033[0m')

def run_cmd(host, cmd):
    print 'ssh %s %s' % (host, cmd)
    r = os.system('ssh %s %s' % (host, cmd))
    if r != 0 :
        print_red('Error executing: ssh %s %s' % (host, cmd))


def run_cmd_set(host, cmdset):
    for cmd in cmdset:
        c = Template(cmd).substitute(str_map)
        run_cmd(host, c)

result = [['APP', 'RATE', 'NAME', 'Min', 'Avg', 'Max', '1%ile', '50%ile', '99%ile', '99.9%ile', '99.99%ile', '99.999%ile', '99.9999%ile', '#pkts', 'Thruput', 'Loss']]
        
def run_test(num):
    run_cmd_set('localhost', PREP_CMD);    
    run_cmd_set('localhost', SOFTNIC_CMD);
    
    for i in range(num-1, -1, -1):
        str_map['index'] = i
        run_cmd_set('localhost', APP_CMD);
        time.sleep(0.1)
    
    run_cmd_set('localhost', ['sleep 25','\"cat output | grep \'##\'\"'])

    f = open('/home/keonjang/output')
    lines = f.readlines()
    r = [str_map['app'], str_map['rate'], str_map['num']]
            
    for line in lines:
        if line[:2] != '##':
            continue
        print line

        line = line.replace('  ', ' ')
        line = line.replace('  ', ' ')
        tokens = line.split(' ')

        if (tokens[1] == 'Loss:'):
            r.append(float(tokens[2]))
        else:
            r.append(int(tokens[2]))
    result.append(r)

    f.close()
    
    print r



    
for app in APP_SET:
    for num in NUM_SET:
        for rate in RATE_SET:
            str_map['rate'] = rate * 1000000 / num
            str_map['num'] = num
            run_test(num)

result_file = open('result.txt','wt')
for a in result:
    for b in a:
        result_file.write(str(b))
        result_file.write('\t')
    result_file.write('\n')

result_file.close()
