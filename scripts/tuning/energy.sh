#!/bin/bash
set -x
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
sudo modprobe msr
if [ -e ${BASE_DIR}/x86_energy_perf_policy ]; then
    sudo $BASE_DIR/x86_energy_perf_policy performance # Set ourselves to performance.
else
    sudo x86_energy_perf_policy performance
fi
sudo $BASE_DIR/pmqos-static.py cpu_dma_latency=0 # Tune Linux QoS to reduce DMA latency
sudo wrmsr -a 0x620 0x3f3f # Turn off uncore frequency scaling and select max frequency
