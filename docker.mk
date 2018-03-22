# Docker-specific Makefile for Netbricks
# ======================================
# Expectation for docker commands is to work with hub.docker.com; so
# YOU MUST BE Docker LOGGED-IN.

IMG = netbricks
TAG ?= latest
CONTAINER = netbricks
PROJECT = williamofockham
REGISTRY_IMG_NAME = netbricks
LINUX_HEADERS = -v /lib/modules:/lib/modules -v /usr/src:/usr/src
DPDK_VER = 17.08
BASE_DIR ?= $(or $(shell pwd),~/occam/Netbricks)
MAX_CORES ?= 1

# Our Vagrant setup places MoonGen's repo @ /MoonGen
# This works off of being relative (../) to utils/Netbricks.
MOONGEN_DIR ?= $(or $(basename $(dirname $(shell pwd)))/MoonGen,\
~/williamofockham/MoonGen)

FILES_TO_MOUNT := $(foreach f,$(filter-out build libmoon,\
$(notdir $(wildcard $(MOONGEN_DIR)/*))), -v $(MOONGEN_DIR)/$(f):/opt/moongen/$(f))
BASE_MOUNT := -v $(BASE_DIR):/opt/$(CONTAINER)

MOUNTS = $(LINUX_HEADERS) \
         -v /sys/bus/pci/drivers:/sys/bus/pci/drivers \
         -v /sys/kernel/mm/hugepages:/sys/kernel/mm/hugepages \
         -v /sys/devices/system/node:/sys/devices/system/node \
         -v /sbin/modinfo:/sbin/modinfo \
         -v /bin/kmod:/bin/kmod \
         -v /sbin/lsmod:/sbin/lsmod \
         -v /mnt/huge:/mnt/huge \
         -v /dev:/dev \
         -v /var/run:/var/run

ALL_MOUNTS = $(MOUNTS) $(BASE_MOUNT) $(FILES_TO_MOUNT)

.PHONY: build build-fresh run run-tests run-reg tag push pull image image-fresh rmi rmi-registry

build:
	@docker build -t $(CONTAINER):$(TAG) $(BASE_DIR)

build-fresh:
	@docker build --no-cache -t $(CONTAINER):$(TAG) $(BASE_DIR)

run:
	@docker run --name $(CONTAINER) -it --rm --privileged \
	--cpuset-cpus="0-${MAX_CORES}" --pid='host' --network='host' \
	$(ALL_MOUNTS) $(CONTAINER):$(TAG)

run-reg:
	@docker run --name $(CONTAINER) -it --rm --privileged \
	--cpuset-cpus="0-${MAX_CORES}" --pid='host' --network='host' \
	$(ALL_MOUNTS) $(PROJECT)/$(CONTAINER):$(TAG)

run-tests:
	@docker run --name $(CONTAINER) -t --rm --privileged \
	--cpuset-cpus="0-${MAX_CORES}" --pid='host' --network='host' \
	$(MOUNTS) $(CONTAINER):$(TAG) /opt/$(CONTAINER)/build.sh test

tag:
	@docker tag $(CONTAINER) $(PROJECT)/$(CONTAINER):$(TAG)

push:
	@docker push $(PROJECT)/$(CONTAINER):$(TAG)

pull:
	@docker pull $(PROJECT)/$(CONTAINER):$(TAG)

image: build tag push

image-fresh: build-fresh tag push

rmi:
	@docker rmi $(CONTAINER):$(TAG)

rmi-registry:
	@docker rmi $(PROJECT)/$(CONTAINER):$(TAG)

vm:
	@vagrant up
