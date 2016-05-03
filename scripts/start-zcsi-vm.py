import subprocess
import sys
import os
import re
# Return a command string to start ZCSI with all DPDK cores and passed in delay
# if len(sys.argv) > 1:
    # app=sys.argv[1]
# else:
    # app=
# if len(sys.argv) > 1:
    # delay = sys.argv[1]
# else:
    # delay = "0"
app = sys.argv[1]
if len(sys.argv) > 2:
    arg = sys.argv[2]
else:
    arg = "0"

dpdk_home=os.environ["DPDK_HOME"]
zcsi_home=os.environ["ZCSI_HOME"]
status_out = subprocess.check_output([dpdk_home + "/tools/dpdk_nic_bind.py", \
        "--status"])
status_out = status_out.split('\n')
dpdk_ports = False
ports = []
# "0000:82:00.3 'Ethernet Controller XL710 for 40GbE QSFP+' unused=i40e",
pcie_re = r"[0-9a-f]{4}:([0-9a-f]{2}):([0-9a-f]{2})\.([0-9a-f]*)"
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
                match = re.match(pcie_re, parts[0])
                if match:
                    port = "%s:%s.%s"%(match.group(1), match.group(2), \
                            match.group(3))
                    ports.append(port)
ports = map(lambda p: '-c 1 -w %s'%p, ports)
# print "%s/test/delay-test/target/release/zcsi-delay -m 0 %s -d %s"%(zcsi_home, \
        # ' '.join(ports), delay)
if app == "delay":
    print "%s/test/delay-test/target/release/zcsi-delay -m 0 %s -d %s"%\
            (zcsi_home, \
            ' '.join(ports), arg)
elif app == "chain":
    print "%s/test/chain-test/target/release/zcsi-chain -m 0 %s -l %s"%\
            (zcsi_home, \
            ' '.join(ports), arg)
