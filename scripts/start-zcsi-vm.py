import subprocess
import sys
import os
import re
# Return a command string to start ZCSI with all DPDK cores and passed in delay
if len(sys.argv) > 1:
    delay = sys.argv[1]
else:
    delay = 0
dpdk_home=os.environ["DPDK_HOME"]
zcsi_home=os.environ["ZCSI_HOME"]
status_out = subprocess.check_output([dpdk_home + "/tools/dpdk_nic_bind.py", \
        "--status"])
status_out = status_out.split('\n')
dpdk_ports = False
ports = []
# "0000:82:00.3 'Ethernet Controller XL710 for 40GbE QSFP+' unused=i40e",
pcie_re = r"[0-9a-f]{4}:[0-9a-f]{2}:[0-9a-f]{2}\.[0-9a-f]"
for status in status_out:
    if status == "Network devices using DPDK-compatible driver":
        dpdk_ports = True
    elif status.startswith("==="):
        continue
    elif status.startswith("Network devices"):
        dpdk_ports = False
    else:
        if dpdk_ports:
            parts = status.split()
            if len(parts) > 1:
                if re.match(pcie_re, parts[0]):
                    ports.append(parts[0])
ports = map(lambda p: '-c 1 -w %s'%p, ports)
print "%s/test/target/release/zcsi-delay -m 0 %s"%(zcsi_home, ' '.join(ports))
