#!/bin/bash
set -e
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
if [ -e $BASE_DIR/rustc ]; then
	$BASE_DIR/rustc "$@"
else
    (>&2 echo "WARNING: Using system rustc")
	rustc "$@"
fi
