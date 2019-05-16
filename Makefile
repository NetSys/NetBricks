PORT ?= "0000:00:09.0"
CORE ?= 0
BASE_DIR = $(shell git rev-parse --show-toplevel)
POOL_SIZE ?= 512
GH=git@github.com:williamofockham

.PHONY: init build build-all build-nb build-ex native build-rel build-rel-ex \
        fmt clean run run-rel test

init:
	@mkdir -p $(BASE_DIR)/.git/hooks && ln -s -f $(BASE_DIR)/.hooks/pre-commit $(BASE_DIR)/.git/hooks/pre-commit
	-git clone $(GH)/utils.git
	-ln -s utils/Vagrantfile
	-git clone --recurse-submodules $(GH)/moongen.git

build:
	@./build.sh build

build-all:
	@./build.sh build_all

build-nb:
	@./build.sh build_fmwk

build-ex:
ifdef EXAMPLE
	@./build.sh build_example $(EXAMPLE)
else
	@./build.sh build_example
endif

native:
	@./build.sh build_native

build-rel:
	@./build.sh build_rel

build-rel-ex:
ifdef EXAMPLE
	@./build.sh build_example_rel $(EXAMPLE)
else
	@./build.sh build_example
endif

fmt:
	@./build.sh fmt

clean:
	@./build.sh clean

run:
ifdef EXAMPLE
	@./build.sh run $(EXAMPLE) -p $(PORT) -c $(CORE) --pool-size=$(POOL_SIZE)
else
	@./build.sh run
endif

run-rel:
ifdef EXAMPLE
	@./build.sh run_rel $(EXAMPLE) -p $(PORT) -c $(CORE) --pool-size=$(POOL_SIZE)
else
	@./build.sh run_rel
endif

test:
ifdef TEST
	@./build.sh build_example $(TEST)
	@./build.sh test $(TEST)
else
	@./build.sh build
	@./build.sh test
endif
