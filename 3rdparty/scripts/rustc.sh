#!/bin/sh
set -e
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
if [ -e $BASE_DIR/rustc ]; then
	$BASE_DIR/rustc "$@"
else
	echo "WARNING: Using system rustc"
	rustc "$@"
fi
