# Docker-specific Makefile for Netbricks
# ======================================
# Expectation for docker commands is to work with hub.docker.com; so
# YOU MUST BE Docker LOGGED-IN.

IMG = netbricks
TAG ?= latest
CONTAINER = netbricks
PROJECT = williamofockham
REGISTRY_IMG_NAME = netbricks
LINUX_HEADERS = -v /lib/modules:/lib/modules
DPDK_VER = 17.08
BASE_DIR ?= $(or $(shell pwd),~/occam/Netbricks)
MOUNTS = $(LINUX_HEADERS) -v /dev/hugepages:/dev/hugepages \
         -v $(BASE_DIR):/opt/$(CONTAINER)

.PHONY: build build-fresh run run-reg tag push pull image image-fresh rmi rmi-registry

build:
	@./build.sh dist_clean
	@docker build -f container/Dockerfile -t $(CONTAINER):$(TAG) \
	--build-arg dpdk_file="common_linuxapp-$(DPDK_VER).container" $(BASE_DIR)

build-fresh:
	@./build.sh dist_clean
	@docker build --no-cache -f container/Dockerfile -t $(CONTAINER):$(TAG) \
	--build-arg dpdk_file="common_linuxapp-$(DPDK_VER).container" $(BASE_DIR)

run:
	@docker run --name $(CONTAINER) -it --rm --privileged --pid='host' \
	$(MOUNTS) $(CONTAINER):$(TAG)

run-reg:
	@docker run --name $(CONTAINER) -it --rm --privileged --pid='host' \
	$(MOUNTS) $(PROJECT)/$(CONTAINER):$(TAG)

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
