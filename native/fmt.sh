#!/bin/bash
BASE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
for c_file in ${BASE_DIR}/*.c; do
    echo ${c_file}
   clang-format -style=file -i ${c_file}
done

for h_file in ${BASE_DIR}/include/*.h; do
    echo ${h_file}
   clang-format -style=file -i ${h_file}
done
