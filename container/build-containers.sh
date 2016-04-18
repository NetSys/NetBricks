#!/bin/bash
docker build -t e2d2/zcsi:0.4 -t e2d2/zcsi:latest --no-cache \
	--cpuset-cpus="4-19" .
