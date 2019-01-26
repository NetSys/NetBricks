#!/usr/bin/env python
"""

"""
import sys
import os
def main(*directories):
    directory_set = set(directories)
    base_dir = os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', 'test')
    print("Searching in %s"%(base_dir))
    for (dirpath, dirnames, filenames) in os.walk(base_dir):
        for filename in filenames:
            if filename == "Cargo.toml":
                rest,dir=os.path.split(dirpath)
                root=os.path.split(rest)[1]
                test_dir=os.path.join(root, dir)
                if root == 'test' and test_dir not in directory_set:
                    print("Found Cargo.toml in %s but not in build.sh"%(test_dir))
                    sys.exit(1)
    sys.exit(0)
if __name__ == "__main__":
    if len(sys.argv) == 1:
        print("Usage: %s json"%sys.argv[0], file=sys.stderr)
        sys.exit(1)
    else:
        main(*sys.argv[1:])
