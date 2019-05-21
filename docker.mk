# Docker-specific Makefile for Netbricks Project
# ==============================================

BASE_DIR = $(shell pwd)
SANDBOX ?= williamofockham/sandbox:nightly-2019-02-01

MOUNTS = -v /lib/modules:/lib/modules \
         -v /usr/src:/usr/src \
         -v /dev/hugepages:/dev/hugepages

.PHONY: pull-sandbox run run-lint run-tests

pull-sandbox:
	@docker pull $(SANDBOX)

run: pull-sandbox
	@docker run -it --rm --privileged --network=host \
		-w /opt \
        $(MOUNTS) \
		-v $(BASE_DIR):/opt/netbricks \
		-v $(BASE_DIR)/moongen:/opt/moongen \
		$(SANDBOX) /bin/bash

run-tests: pull-sandbox
	@docker run -it --rm --privileged --network=host \
		-w /opt/netbricks \
		$(MOUNTS) \
		-v $(BASE_DIR):/opt/netbricks \
		-v $(BASE_DIR)/moongen:/opt/moongen \
		$(SANDBOX) make test

run-lint: pull-sandbox
	@docker run -it --rm --privileged --network=host \
		-w /opt/netbricks \
		$(MOUNTS) \
		-v $(BASE_DIR):/opt/netbricks \
		-v $(BASE_DIR)/moongen:/opt/moongen \
		$(SANDBOX) make lint
