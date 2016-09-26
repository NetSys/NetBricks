#!/usr/bin/env python3
"""
Take the output of Cargo.toml and print target name.
"""
import sys
import json

def main(inp):
    try:
        o = json.loads(inp)
    except:
        print("Failed to interpret JSON", file=sys.stderr)
    if 'targets' in o:
        for target in o['targets']:
            if 'kind' in target and (target['kind'] == 'bin' or 'bin' in target['kind']):
                print(target['name'])
            else:
                print("No kind found")

if __name__=="__main__":
    if len(sys.argv) < 2:
        print("Usage: %s json"%sys.argv[0], file=sys.stderr)
        sys.exit(1)
    if len(sys.argv) == 2 and sys.argv[1] == '-':
        inp = sys.stdin.read()
        main(inp)
    else:
        main(' '.join(sys.argv[1:]))
