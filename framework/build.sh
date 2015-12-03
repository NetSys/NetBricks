#!/bin/bash
# FIXME: Replace this with csproj and FAKE
mcs -out:DPDK.dll -unsafe -debug- -optimize+ -platform:x64 -target:library *.cs
